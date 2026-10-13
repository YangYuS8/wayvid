#!/usr/bin/env python3
"""Privileged, opt-in real suspend acceptance for an owned Scene endurance run.

`--check` is read-only and never asks for authentication. Running cycles needs
root because the RTC alarm is privileged. This helper preserves normal logind
inhibitors and screen locking; the user must unlock after each wake. Unit tests
of this file are fixture tests and do not establish real desktop acceptance.
"""

import argparse
import fcntl
import getpass
import json
import math
import os
from pathlib import Path
import re
import pwd
import resource
import signal
import stat
import struct
import subprocess
import sys
import time
import tomllib
import warnings


LOGIN = ["org.freedesktop.login1", "/org/freedesktop/login1",
         "org.freedesktop.login1.Manager"]
COMMAND_ENV = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"}
MAX_STATE_BYTES = 64 * 1024
MAX_FRESH_NS = 10_000_000_000
MIN_SLEEP_NS = 2_000_000_000


class AcceptanceError(RuntimeError):
    pass


def require(condition, message):
    if not condition:
        raise AcceptanceError(message)


def integer(value, label, minimum=0):
    require(type(value) is int and value >= minimum, f"Invalid {label}")
    return value


def clocks():
    return {"wall_ns": time.time_ns(), "monotonic_ns": time.monotonic_ns(),
            "boottime_ns": time.clock_gettime_ns(time.CLOCK_BOOTTIME)}


def run_command(arguments, timeout=15):
    result = subprocess.run(arguments, check=False, timeout=timeout,
                            capture_output=True, text=True, env=COMMAND_ENV)
    require(result.returncode == 0,
            f"{Path(arguments[0]).name} failed ({result.returncode}): {result.stderr[:1000].strip()}")
    return result.stdout.strip()


def read_text(path, limit=4096):
    with open(path, encoding="utf-8") as stream:
        value = stream.read(limit + 1)
    require(len(value) <= limit, f"Oversized system field: {path}")
    return value.strip()


def process_identity(pid):
    integer(pid, "child PID", 1)
    status = read_text(Path("/proc") / str(pid) / "stat")
    # comm can contain spaces and closing parentheses. Remaining field 3 is
    # state, so field 22 (starttime) is token 19 after the final closing paren.
    fields = status[status.rfind(")") + 2:].split()
    require(len(fields) > 19, "Invalid child process stat")
    require(fields[0] not in {"Z", "X", "x"}, "Owned child process exited")
    return {"child_pid": pid, "child_starttime_ticks": int(fields[19]),
            "owner_uid": (Path("/proc") / str(pid)).stat().st_uid}


def read_readiness(path, owner_uid):
    # A privileged reader must not follow a user-supplied final symlink or read
    # root-owned files. Atomic replacement by the harness is supported.
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    try:
        info = os.fstat(descriptor)
        require(stat.S_ISREG(info.st_mode), "Readiness must be a regular file")
        require(info.st_uid == owner_uid, "Readiness owner does not match the harness user")
        require(not info.st_mode & 0o022, "Readiness must not be writable by other users")
        require(info.st_size <= MAX_STATE_BYTES, "Oversized readiness file")
        with os.fdopen(descriptor, encoding="utf-8", closefd=False) as stream:
            payload = stream.read(MAX_STATE_BYTES + 1)
        require(len(payload.encode()) <= MAX_STATE_BYTES, "Oversized readiness file")
        value = json.loads(payload)
    finally:
        os.close(descriptor)
    require(isinstance(value, dict), "Readiness must be an object")
    return value


def readiness_identity(value):
    session_id = value.get("session_id")
    require(isinstance(session_id, str) and re.fullmatch(r"[A-Za-z0-9_-]{1,32}", session_id),
            "Invalid harness logind session ID")
    boot_id = value.get("boot_id")
    require(isinstance(boot_id, str) and re.fullmatch(r"[0-9a-f-]{36}", boot_id),
            "Invalid harness boot ID")
    return {"owner_uid": integer(value.get("owner_uid"), "owner UID", 1),
            "child_pid": integer(value.get("child_pid"), "child PID", 1),
            "child_starttime_ticks": integer(value.get("child_starttime_ticks"), "child start time", 1),
            "boot_id": boot_id, "session_id": session_id}


def validate_identity(value, identity):
    require(readiness_identity(value) == identity, "Harness identity changed")
    require(read_text("/proc/sys/kernel/random/boot_id") == identity["boot_id"], "Boot identity changed")
    actual = process_identity(identity["child_pid"])
    require(all(actual[key] == identity[key] for key in actual), "Owned child PID was replaced or exited")


def is_fresh_healthy(value, now, after_monotonic_ns=None, previous_swaps=None):
    sampled = integer(value.get("monotonic_ns"), "sample monotonic time")
    wall = integer(value.get("sampled_wall_time_ns"), "sample wall time")
    boot = integer(value.get("boottime_ns"), "sample boot time")
    swaps = integer(value.get("swaps"), "completed swaps")
    fresh = 0 <= now["monotonic_ns"] - sampled <= MAX_FRESH_NS
    fresh = fresh and abs(now["wall_ns"] - wall) <= MAX_FRESH_NS
    fresh = fresh and 0 <= now["boottime_ns"] - boot <= MAX_FRESH_NS
    return (fresh and value.get("scene_active") is True and value.get("healthy") is True
            and value.get("session_locked") is False
            and (after_monotonic_ns is None or sampled > after_monotonic_ns)
            and (previous_swaps is None or swaps > previous_swaps))


def session_unlocked(identity):
    result = run_command(["/usr/bin/loginctl", "show-session", identity["session_id"],
                          "--property=LockedHint", "--value"])
    require(result in {"yes", "no"}, "Cannot establish the session lock state")
    return result == "no"


def ac_online(power_root=Path("/sys/class/power_supply")):
    mains = [entry for entry in power_root.iterdir()
             if (entry / "type").is_file() and read_text(entry / "type") == "Mains"]
    return any(read_text(entry / "online") == "1" for entry in mains)


def suspend_counters():
    return {name: int(read_text(Path("/sys/power/suspend_stats") / name))
            for name in ("success", "fail")}


def check_system(identity, wakealarm):
    require(ac_online(), "AC must remain connected for unattended acceptance")
    require("mem" in read_text("/sys/power/state").split(), "Kernel mem suspend is unavailable")
    require(read_text(wakealarm) == "", "Existing RTC alarm will not be overwritten")
    require(read_text("/sys/class/rtc/rtc0/device/power/wakeup") == "enabled", "RTC wakeup is disabled")
    require(run_command(["/usr/bin/timedatectl", "show", "--property=LocalRTC", "--value"]) == "no",
            "This helper requires an RTC in UTC")
    require(run_command(["/usr/bin/busctl", "--system", "--allow-interactive-authorization=no",
                         "call", *LOGIN, "CanSuspend"]) == 's "yes"', "Logind cannot suspend without authentication")
    suspend_counters()  # Require actual kernel counters before writing the RTC.
    require(session_unlocked(identity), "Unlock the normal desktop session before starting a cycle")


def sleep_evidence(before, after):
    require(after["counters"]["success"] == before["counters"]["success"] + 1,
            "Expected exactly one successful kernel suspend cycle")
    require(after["counters"]["fail"] == before["counters"]["fail"], "Kernel suspend failure count increased")
    monotonic_delta = after["clocks"]["monotonic_ns"] - before["clocks"]["monotonic_ns"]
    boot_delta = after["clocks"]["boottime_ns"] - before["clocks"]["boottime_ns"]
    require(monotonic_delta >= 0 and boot_delta >= 0, "System clocks moved backwards")
    slept = boot_delta - monotonic_delta
    require(slept >= MIN_SLEEP_NS, "No measured real suspend interval of at least two seconds")
    return slept


class EvidenceWriter:
    """Own a new root directory and write through its stable descriptor.

    The parent harness directory can be user-owned. Final symlinks and existing
    destinations are refused, and atomic writes cannot follow a swapped path.
    Reports contain acceptance evidence and are readable by the harness user.
    """

    def __init__(self, directory, owner_uid=None):
        directory = Path(directory)
        require(directory.is_absolute(), "Output directory must be absolute")
        parent = os.open(directory.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
        try:
            require(os.fstat(parent).st_uid == (os.getuid() if owner_uid is None else owner_uid),
                    "Evidence parent must belong to the harness user")
            os.mkdir(directory.name, 0o755, dir_fd=parent)  # Never reuse an existing destination.
            self.descriptor = os.open(directory.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC,
                                      dir_fd=parent)
        finally:
            os.close(parent)
        os.fchmod(self.descriptor, 0o755)
        info = os.fstat(self.descriptor)
        require(info.st_uid == os.geteuid() and not info.st_mode & 0o022, "Output directory is not exclusively owned")
        self.events = []
        self.sequence = 0

    def atomic(self, name, value):
        self.sequence += 1
        temporary = f".{name}.{os.getpid()}.{self.sequence}"
        descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                             0o644, dir_fd=self.descriptor)
        try:
            os.fchmod(descriptor, 0o644)
            with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
                stream.write(value)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temporary, name, src_dir_fd=self.descriptor, dst_dir_fd=self.descriptor)
            os.fsync(self.descriptor)
        finally:
            try:
                os.unlink(temporary, dir_fd=self.descriptor)
            except FileNotFoundError:
                pass

    def save(self, report, event=None):
        if event is not None:
            self.events.append(event)
            self.atomic("suspend-helper.jsonl", "".join(json.dumps(item, ensure_ascii=False) + "\n" for item in self.events))
        self.atomic("report.json", json.dumps(report, ensure_ascii=False, indent=2) + "\n")

    def close(self):
        os.close(self.descriptor)


# Linux evdev codes with the explicitly verified US layout. Password bytes are
# never passed to a child process, written to a file, or included in evidence.
ASCII_KEYS = {}
for row, first in (("1234567890", 2), ("qwertyuiop", 16), ("asdfghjkl", 30), ("zxcvbnm", 44)):
    ASCII_KEYS.update({ord(char): (first + index, False) for index, char in enumerate(row)})
ASCII_KEYS.update({char - 32: (code, True) for char, (code, _) in list(ASCII_KEYS.items()) if 97 <= char <= 122})
ASCII_KEYS.update({ord(char): (code, False) for char, code in [("-", 12), ("=", 13), ("[", 26), ("]", 27),
                                                         (";", 39), ("'", 40), ("`", 41), ("\\", 43),
                                                         (",", 51), (".", 52), ("/", 53), (" ", 57)]})
ASCII_KEYS.update({ord(char): (code, True) for char, code in zip("!@#$%^&*()_+{}:\"~|<>?", [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 26, 27, 39, 40, 41, 43, 51, 52, 53])})


def validate_password(secret):
    require(isinstance(secret, bytearray) and 1 <= len(secret) <= 128
            and all(byte in ASCII_KEYS for byte in secret), "Unlock requires 1–128 printable ASCII characters")


class UnlockKeyboard:
    """Temporary kernel keyboard, destroyed on every exit path."""

    def __init__(self):
        self.descriptor = None
        self.created = False
        self.name = f"lwe-scene-suspend-auth-{os.getpid()}".encode()
        self.held = set()

    def __enter__(self):
        self.descriptor = os.open("/dev/uinput", os.O_WRONLY | os.O_NONBLOCK | os.O_CLOEXEC)
        try:
            fcntl.ioctl(self.descriptor, 0x40045564, 1)  # UI_SET_EVBIT(EV_KEY)
            # Include complete keyboard rows so udev identifies a keyboard.
            # udev's keyboard heuristic requires every key from ESC through S
            # (the low 31 nonzero bits), including TAB and LEFTCTRL even though
            # authentication never presses those two keys.
            for code in sorted({code for code, _ in ASCII_KEYS.values()} | {1, 14, 15, 28, 29, 42}):
                fcntl.ioctl(self.descriptor, 0x40045565, code)  # UI_SET_KEYBIT
            setup = struct.pack("HHHH80sI", 3, 1, 1, 1, self.name, 0)
            fcntl.ioctl(self.descriptor, 0x40000000 | (len(setup) << 16) | 0x5503, setup)
            fcntl.ioctl(self.descriptor, 0x5501)  # UI_DEV_CREATE
            self.created = True
            system_name = bytearray(256)
            fcntl.ioctl(self.descriptor, 0x80000000 | (256 << 16) | 0x552c, system_name, True)
            path = Path("/sys/devices/virtual/input") / system_name.split(b"\0", 1)[0].decode("ascii")
            require(path.name.startswith("input") and re.fullmatch(r"input[0-9]+", path.name), "Cannot identify the owned keyboard")
            deadline = time.monotonic() + 8
            while time.monotonic() < deadline:
                events = list(path.glob("event*"))
                if len(events) == 1:
                    properties = run_command(["/usr/bin/udevadm", "info", "--query=property", "--path=" + str(events[0])])
                    if "ID_INPUT_KEYBOARD=1" in properties:
                        break
                time.sleep(0.2)
            else:
                raise AcceptanceError("The owned keyboard was not discovered as a keyboard")
            # Allow the compositor to consume udev discovery and the lock
            # transition before any private input. No events are sent here.
            time.sleep(3)
            return self
        except BaseException:
            self.close()
            raise

    def event(self, code, value):
        require(self.descriptor is not None, "Unlock keyboard is unavailable")
        data = struct.pack("llHHi", 0, 0, 1, code, value) + struct.pack("llHHi", 0, 0, 0, 0, 0)
        require(os.write(self.descriptor, data) == len(data), "Unlock keyboard write failed")
        if value:
            self.held.add(code)
        else:
            self.held.discard(code)

    def tap(self, code, trusted_lock):
        require(trusted_lock(), "Exclusive Noctalia lock input could not be established")
        self.event(code, 1)
        try:
            time.sleep(0.035)
        finally:
            self.event(code, 0)
        time.sleep(0.035)

    def type_password(self, secret, trusted_lock):
        self.tap(1, trusted_lock)  # Normal Escape clears the shared password field.
        for byte in secret:
            code, shifted = ASCII_KEYS[byte]
            if shifted:
                require(trusted_lock(), "Exclusive Noctalia lock input could not be established")
                self.event(42, 1)
            try:
                self.tap(code, trusted_lock)
            finally:
                if shifted:
                    self.event(42, 0)
        self.tap(28, trusted_lock)  # Normal Enter invokes Noctalia's PAM path once.

    def close(self):
        if self.descriptor is not None:
            try:
                for code in list(self.held):
                    self.event(code, 0)
            finally:
                try:
                    if self.created:
                        fcntl.ioctl(self.descriptor, 0x5502)  # UI_DEV_DESTROY
                finally:
                    os.close(self.descriptor)
                    self.descriptor = None
                    self.created = False

    def __exit__(self, _kind, _value, _traceback):
        self.close()


class AuthenticatedUnlock:
    """Deliver one normal PAM attempt per real resume, without force-unlock."""

    def __init__(self, owner_uid, secret):
        validate_password(secret)
        self.secret = secret
        self.owner_uid = owner_uid
        account = pwd.getpwuid(owner_uid)
        self.group = account.pw_gid
        self.environment = {**COMMAND_ENV, "HOME": account.pw_dir, "USER": account.pw_name,
                            "LOGNAME": account.pw_name, "XDG_RUNTIME_DIR": f"/run/user/{owner_uid}"}
        sockets = list(Path(self.environment["XDG_RUNTIME_DIR"]).glob("niri.*.sock"))
        require(len(sockets) == 1 and sockets[0].stat().st_uid == owner_uid
                and stat.S_ISSOCK(sockets[0].stat().st_mode), "Cannot identify one owned niri session")
        self.environment["NIRI_SOCKET"] = str(sockets[0])

    def user_command(self, arguments):
        result = subprocess.run(arguments, capture_output=True, text=True, timeout=10,
                                user=self.owner_uid, group=self.group, extra_groups=[], env=self.environment)
        require(result.returncode == 0, "Normal lock authentication capability query failed")
        return result.stdout

    def locked(self, identity):
        for _ in range(3):
            state = json.loads(self.user_command(["/usr/bin/noctalia", "msg", "status"]))
            require(type(state.get("locked")) is bool, "Noctalia lock status is unavailable")
            hint = not session_unlocked(identity)
            if hint == state["locked"]:
                return hint
            # Unlock and its logind hint are sequential, so two read-only
            # queries can straddle the normal transition. Never input then.
            time.sleep(0.1)
        raise AcceptanceError("Normal lock sources disagree")

    def preflight(self):
        require("5.2.1" in self.user_command(["/usr/bin/noctalia", "--version"]), "Authenticated input is bounded to the inspected Noctalia 5.2.1")
        configuration = tomllib.loads(self.user_command(["/usr/bin/noctalia", "config", "export"]))
        # The inspected defaults are Return/Enter and Escape. Custom chords
        # cannot prove field clearing, so never send a password in that case.
        require(not configuration.get("keybinds"), "Custom Noctalia keybindings are not supported for authenticated input")
        self.verify_layout()
        require(os.access("/dev/uinput", os.W_OK), "Temporary keyboard access is unavailable")

    def verify_layout(self):
        layouts = json.loads(self.user_command(["/usr/bin/niri", "msg", "-j", "keyboard-layouts"]))
        require(layouts.get("names") == ["English (US)"] and layouts.get("current_idx") == 0,
                "Authenticated input requires the inspected single US keyboard layout")
        require(all(read_text(path) == "0" for path in Path("/sys/class/leds").glob("*::capslock/brightness")),
                "Disable Caps Lock before normal authenticated input")

    def after_resume(self, identity):
        if not self.locked(identity):
            return {"authenticated_input": False, "already_unlocked": True}
        self.verify_layout()
        started = time.time_ns()
        with UnlockKeyboard() as keyboard:
            def trusted_lock():
                return self.locked(identity)
            require(trusted_lock(), "Normal locked input is unavailable")
            keyboard.type_password(self.secret, trusted_lock)
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if not self.locked(identity):
                    return {"authenticated_input": True, "normal_pam_unlock": True, "attempts": 1}
                failures = run_command(["/usr/bin/journalctl", "--boot=" + identity["boot_id"],
                                        "--since=@" + str(started // 1_000_000_000), "--no-pager", "--lines=30",
                                        "--grep=authentication failure|pam_faillock.*authfail", "_UID=" + str(self.owner_uid)])
                require(not failures or failures.startswith("-- No entries --"), "Normal PAM unlock failed; no retry will be made")
                time.sleep(0.5)
            raise AcceptanceError("Normal PAM unlock did not complete; no retry will be made")

    def close(self):
        for index in range(len(self.secret)):
            self.secret[index] = 0


class SuspendCycles:
    def __init__(self, readiness, owner_uid, cycles=10, wake_seconds=20, recovery_timeout=120, unlocker=None):
        require(1 <= cycles <= 10, "Cycle count must be between 1 and 10")
        require(15 <= wake_seconds <= 60, "RTC wake delay must be between 15 and 60 seconds")
        require(1 <= recovery_timeout <= 120, "Recovery timeout must be between 1 and 120 seconds")
        self.readiness = Path(readiness)
        self.owner_uid = integer(owner_uid, "owner UID", 1)
        self.cycles = cycles
        self.wake_seconds = wake_seconds
        self.recovery_timeout = recovery_timeout
        self.wakealarm = Path("/sys/class/rtc/rtc0/wakealarm")
        self.owned_alarm = None
        self.unlocker = unlocker

    def initial(self):
        value = read_readiness(self.readiness, self.owner_uid)
        identity = readiness_identity(value)
        require(identity["owner_uid"] == self.owner_uid, "Harness UID does not match invocation")
        validate_identity(value, identity)
        require(is_fresh_healthy(value, clocks()), "Harness needs a fresh, healthy, unlocked Scene sample")
        check_system(identity, self.wakealarm)
        return identity, value

    def snapshot(self):
        return {"clocks": clocks(), "counters": suspend_counters()}

    def arm(self):
        require(read_text(self.wakealarm) == "", "Existing RTC alarm will not be overwritten")
        before = time.time()
        try:
            run_command(["/usr/bin/rtcwake", "--device", "/dev/rtc0", "--utc", "--mode", "no",
                         "--seconds", str(self.wake_seconds)])
        finally:
            alarm = read_text(self.wakealarm)
            # Even a failed utility may have armed the RTC. Only an alarm in
            # this invocation's requested time window can be claimed/cleaned.
            if alarm.isdecimal() and math.floor(before) + self.wake_seconds - 1 <= int(alarm) <= math.ceil(before) + self.wake_seconds + 2:
                self.owned_alarm = alarm
        require(self.owned_alarm is not None, "RTC utility did not establish the expected wake alarm")
        require(int(self.owned_alarm) > time.time(), "RTC wake alarm expired before suspend")

    def clear_owned_alarm(self):
        if self.owned_alarm is None:
            return
        current = read_text(self.wakealarm)
        if current == "":
            self.owned_alarm = None
            return
        require(current == self.owned_alarm, "RTC alarm changed externally; it was left untouched")
        run_command(["/usr/bin/rtcwake", "--device", "/dev/rtc0", "--utc", "--mode", "disable"])
        require(read_text(self.wakealarm) == "", "Owned RTC alarm did not clear")
        self.owned_alarm = None

    def ready_for_suspend(self, identity):
        # Report fsync or other work can outlast the wake alarm. Never sleep
        # after an expired, externally replaced, or insufficient alarm.
        require(self.owned_alarm is not None and read_text(self.wakealarm) == self.owned_alarm,
                "Owned RTC alarm changed before suspend")
        delay = run_command(["/usr/bin/busctl", "--system", "--allow-interactive-authorization=no",
                             "get-property", *LOGIN, "InhibitDelayMaxUSec"])
        require(re.fullmatch(r"t [0-9]+", delay) is not None, "Cannot establish logind's maximum sleep delay")
        remaining = int(self.owned_alarm) - time.time()
        require(remaining >= int(delay.split()[1]) / 1_000_000 + 5,
                "RTC alarm has insufficient time remaining for logind and real suspend")
        value = read_readiness(self.readiness, self.owner_uid)
        validate_identity(value, identity)
        require(is_fresh_healthy(value, clocks()) and session_unlocked(identity),
                "Owned Scene or normal session changed before suspend")
        require(read_text(self.wakealarm) == self.owned_alarm
                and int(self.owned_alarm) - time.time() >= int(delay.split()[1]) / 1_000_000 + 5,
                "Owned RTC alarm expired or changed during final checks")

    def wait_for_kernel_resume(self, before):
        deadline = time.clock_gettime(time.CLOCK_BOOTTIME) + self.wake_seconds + 45
        while time.clock_gettime(time.CLOCK_BOOTTIME) < deadline:
            after = self.snapshot()
            require(after["counters"]["fail"] == before["counters"]["fail"], "Kernel suspend failed")
            if after["counters"]["success"] != before["counters"]["success"]:
                sleep_evidence(before, after)
                return after
            time.sleep(0.2)
        raise AcceptanceError("No successful kernel suspend/resume before the deadline")

    def wait_for_scene(self, identity, after, previous_swaps):
        deadline = time.monotonic() + self.recovery_timeout
        while time.monotonic() < deadline:
            value = read_readiness(self.readiness, self.owner_uid)
            validate_identity(value, identity)
            if (is_fresh_healthy(value, clocks(), after["clocks"]["monotonic_ns"], previous_swaps)
                    and session_unlocked(identity)):
                return value
            time.sleep(0.5)
        raise AcceptanceError("No fresh, unlocked healthy Scene with advancing swaps within recovery timeout")

    def journals(self, before, after, identity):
        bounds = ["--boot=" + identity["boot_id"], "--since=@" + str(before["clocks"]["wall_ns"] // 1_000_000_000),
                  "--until=@" + str(after["clocks"]["wall_ns"] // 1_000_000_000 + 1),
                  "--no-pager", "--output=json", "--lines=100"]
        service = run_command(["/usr/bin/journalctl", *bounds, "--unit=systemd-suspend.service"])
        kernel = run_command(["/usr/bin/journalctl", *bounds, "--kernel", "--grep=PM: suspend"])
        require('PM: suspend entry' in kernel and 'PM: suspend exit' in kernel,
                "Kernel journal lacks a paired PM suspend entry/exit")
        return {"service": service[:64 * 1024], "kernel": kernel[:64 * 1024]}

    def execute(self, writer):
        require(os.geteuid() == 0, "Real cycles require root RTC access; use --check for a read-only preflight")
        report = {"schema_version": 1, "passed": False, "real_suspend": True,
                  "requested_cycles": self.cycles, "completed_cycles": 0,
                  "created": clocks(), "phase": "preflight", "cycles": [], "cleanup_errors": []}
        writer.save(report)
        try:
            identity, baseline = self.initial()
            report["identity"] = identity
            for number in range(1, self.cycles + 1):
                # Each cycle starts only after the previous wallpaper recovery.
                validate_identity(baseline, identity)
                require(is_fresh_healthy(baseline, clocks()), "Harness sample became stale before suspend")
                check_system(identity, self.wakealarm)
                before = self.snapshot()
                report["phase"] = "arming_rtc"
                writer.save(report, {"cycle": number, "phase": "before", **before, "swaps": baseline["swaps"]})
                self.arm()  # A failure here must never invoke Suspend.
                report["phase"] = "suspending"
                writer.save(report, {"cycle": number, "phase": "rtc_armed", "alarm": self.owned_alarm})
                self.ready_for_suspend(identity)
                run_command(["/usr/bin/busctl", "--system", "--allow-interactive-authorization=no", "--timeout=90",
                             "call", *LOGIN, "Suspend", "b", "false"], timeout=95)
                after = self.wait_for_kernel_resume(before)
                journal = self.journals(before, after, identity)
                self.clear_owned_alarm()
                report["phase"] = "waiting_for_unlock"
                writer.save(report, {"cycle": number, "phase": "kernel_resumed", **after,
                                     "suspended_ns": sleep_evidence(before, after), "journal": journal})
                if self.unlocker is not None:
                    unlock = self.unlocker.after_resume(identity)
                    writer.save(report, {"cycle": number, "phase": "normal_unlock", **unlock})
                baseline = self.wait_for_scene(identity, after, baseline["swaps"])
                record = {"cycle": number, "before": before, "after": after,
                          "suspended_ns": sleep_evidence(before, after), "journal": journal,
                          "recovered_sample": baseline, "passed": True}
                report["cycles"].append(record)
                report["completed_cycles"] = number
                report["phase"] = "recovered"
                writer.save(report, record)
            report["passed"] = True
            report["phase"] = "complete"
        except (AcceptanceError, OSError, ValueError, subprocess.SubprocessError, KeyboardInterrupt) as error:
            report["error"] = str(error) or type(error).__name__
            report["phase"] = "failed"
        finally:
            try:
                self.clear_owned_alarm()
            except (AcceptanceError, OSError, subprocess.SubprocessError) as error:
                report["cleanup_errors"].append(str(error))
                report["passed"] = False
                report["phase"] = "failed"
            report["finished"] = clocks()
            writer.save(report)
        return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--readiness", required=True, type=Path, help="Owned endurance harness suspend-readiness.json")
    parser.add_argument("--owner-uid", type=int, default=int(os.environ.get("SUDO_UID", os.getuid())))
    parser.add_argument("--output-dir", type=Path, help="New root-owned evidence directory (must not already exist)")
    parser.add_argument("--cycles", type=int, default=10)
    parser.add_argument("--wake-seconds", type=int, default=20)
    parser.add_argument("--recovery-timeout", type=int, default=120, help="Maximum wait for normal unlock and healthy Scene")
    parser.add_argument("--check", action="store_true", help="Read-only preflight; never arms RTC or requests sleep")
    parser.add_argument("--authenticate-unlock", action="store_true", help="Read a password once without echo; use normal Noctalia PAM input after resume")
    arguments = parser.parse_args()
    unlocker = None
    secret = None
    try:
        runner = SuspendCycles(arguments.readiness, arguments.owner_uid, arguments.cycles,
                               arguments.wake_seconds, arguments.recovery_timeout)
        if arguments.check:
            identity, _ = runner.initial()
            result = {"preflight_passed": True, "privileged_execution_available": os.geteuid() == 0,
                      "real_suspend_performed": False, "identity": identity,
                      "cycles": runner.cycles, "normal_unlock_required_after_each_cycle": True}
            print(json.dumps(result, indent=2))
            return 0
        require(os.geteuid() == 0, "Real cycles require root RTC access; --check is read-only")
        require(arguments.output_dir is not None, "--output-dir is required for real cycles")
        if arguments.authenticate_unlock:
            require(sys.stdin.isatty(), "Authenticated unlock requires a non-echo interactive terminal")
            resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
            try:
                with warnings.catch_warnings():
                    warnings.simplefilter("error", getpass.GetPassWarning)
                    entered = getpass.getpass("Normal Noctalia unlock password (kept in memory for this run): ")
            except getpass.GetPassWarning:
                raise AcceptanceError("A terminal with echo disabled is required for authentication") from None
            try:
                secret = bytearray(entered.encode("ascii"))
            except UnicodeEncodeError:
                raise AcceptanceError("Authenticated unlock requires printable ASCII") from None
            finally:
                entered = None
            unlocker = AuthenticatedUnlock(arguments.owner_uid, secret)
            unlocker.preflight()
            runner.unlocker = unlocker
        def interrupted(_signum, _frame):
            raise KeyboardInterrupt("Suspend acceptance interrupted")
        signal.signal(signal.SIGTERM, interrupted)
        writer = EvidenceWriter(arguments.output_dir, arguments.owner_uid)
        try:
            report = runner.execute(writer)
        finally:
            writer.close()
        print(json.dumps({"passed": report["passed"], "completed_cycles": report["completed_cycles"],
                          "report": str(arguments.output_dir / "report.json")}, indent=2))
        return 0 if report["passed"] else 1
    except (AcceptanceError, OSError, ValueError, subprocess.SubprocessError) as error:
        print(json.dumps({"passed": False, "real_suspend_performed": False, "error": str(error)}), file=sys.stderr)
        return 1
    finally:
        if unlocker is not None:
            unlocker.close()
        elif secret is not None:
            for index in range(len(secret)):
                secret[index] = 0


if __name__ == "__main__":
    sys.exit(main())
