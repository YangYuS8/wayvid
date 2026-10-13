"""Isolated fixture/logic tests. These never establish real suspend acceptance."""

import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "scene_suspend_cycles", Path(__file__).resolve().parents[1] / "scene-suspend-cycles.py")
HELPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPER)
BOOT_ID = "71befe61-1af9-4f64-a6ae-a7b975df7b6d"


def sample():
    return {"owner_uid": 1000, "child_pid": 1234, "child_starttime_ticks": 5678,
            "boot_id": BOOT_ID, "session_id": "3", "sampled_wall_time_ns": 100_000_000_000,
            "monotonic_ns": 50_000_000_000, "boottime_ns": 60_000_000_000,
            "scene_active": True, "healthy": True, "session_locked": False, "swaps": 200}


def snapshot(success=0, fail=0, monotonic=50, boottime=60):
    return {"clocks": {"wall_ns": int(100e9 + (boottime - 60) * 1e9),
                       "monotonic_ns": int(monotonic * 1e9), "boottime_ns": int(boottime * 1e9)},
            "counters": {"success": success, "fail": fail}}


class MemoryWriter:
    def __init__(self):
        self.events = []
        self.reports = []

    def save(self, report, event=None):
        self.reports.append(copy.deepcopy(report))
        if event is not None:
            self.events.append(copy.deepcopy(event))


class SuspendCyclesTests(unittest.TestCase):
    def test_counts_and_waits_are_bounded(self):
        for options in [{"cycles": 0}, {"cycles": 11}, {"wake_seconds": 14},
                        {"wake_seconds": 61}, {"recovery_timeout": 0}, {"recovery_timeout": 121}]:
            with self.subTest(options=options), self.assertRaises(HELPER.AcceptanceError):
                HELPER.SuspendCycles("/unused", 1000, **options)

    def test_readiness_is_bounded_regular_and_owned_and_refuses_symlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "readiness.json"
            path.write_text(json.dumps(sample()))
            self.assertEqual(HELPER.read_readiness(path, os.getuid()), sample())
            with self.assertRaises(HELPER.AcceptanceError):
                HELPER.read_readiness(path, os.getuid() + 1)
            symlink = Path(directory) / "link"
            symlink.symlink_to(path)
            with self.assertRaises(OSError):
                HELPER.read_readiness(symlink, os.getuid())
            path.write_bytes(b"x" * (HELPER.MAX_STATE_BYTES + 1))
            with self.assertRaises(HELPER.AcceptanceError):
                HELPER.read_readiness(path, os.getuid())
            path.write_text(json.dumps(sample()))
            path.chmod(0o666)
            with self.assertRaises(HELPER.AcceptanceError):
                HELPER.read_readiness(path, os.getuid())

    def test_health_requires_fresh_unlocked_scene_and_increasing_swaps(self):
        now = {"wall_ns": 101_000_000_000, "monotonic_ns": 51_000_000_000, "boottime_ns": 61_000_000_000}
        self.assertTrue(HELPER.is_fresh_healthy(sample(), now, 49_000_000_000, 199))
        for changes in [{"scene_active": False}, {"healthy": False}, {"session_locked": True},
                        {"session_locked": None}, {"swaps": 199}, {"monotonic_ns": 40_000_000_000},
                        {"boottime_ns": 40_000_000_000}, {"sampled_wall_time_ns": 90_000_000_000},
                        {"monotonic_ns": 52_000_000_000}]:
            with self.subTest(changes=changes):
                value = dict(sample(), **changes)
                self.assertFalse(HELPER.is_fresh_healthy(value, now, 49_000_000_000, 199))
        self.assertFalse(HELPER.is_fresh_healthy(sample(), now, 50_000_000_000, 199))
        with self.assertRaises(HELPER.AcceptanceError):
            HELPER.is_fresh_healthy(dict(sample(), swaps=True), now)

    def test_live_process_identity_handles_spaces_parentheses_and_detects_reuse(self):
        value = sample()
        identity = HELPER.readiness_identity(value)
        actual = {key: identity[key] for key in ("child_pid", "child_starttime_ticks", "owner_uid")}
        with patch.object(HELPER, "read_text", return_value=BOOT_ID), patch.object(HELPER, "process_identity", return_value=actual):
            HELPER.validate_identity(value, identity)
            changed = dict(value, child_starttime_ticks=5679)
            with self.assertRaises(HELPER.AcceptanceError):
                HELPER.validate_identity(changed, identity)
        with patch.object(HELPER, "read_text", return_value=BOOT_ID), patch.object(HELPER, "process_identity", return_value=dict(actual, child_starttime_ticks=9999)):
            with self.assertRaises(HELPER.AcceptanceError):
                HELPER.validate_identity(value, identity)
        fields = ["S"] + ["0"] * 18 + ["5678"] + ["0"]
        with patch.object(HELPER, "read_text", return_value="1234 (a name ) here) " + " ".join(fields)), \
                patch.object(HELPER.Path, "stat") as status:
            status.return_value.st_uid = 1000
            self.assertEqual(HELPER.process_identity(1234), actual)

    def test_real_sleep_requires_one_kernel_success_no_failure_and_measured_gap(self):
        before = snapshot()
        after = snapshot(success=1, monotonic=51, boottime=80)
        self.assertEqual(HELPER.sleep_evidence(before, after), 19_000_000_000)
        for invalid in [snapshot(success=0, monotonic=51, boottime=80),
                        snapshot(success=2, monotonic=51, boottime=80),
                        snapshot(success=1, fail=1, monotonic=51, boottime=80),
                        snapshot(success=1, monotonic=70, boottime=80),
                        snapshot(success=1, monotonic=49, boottime=80)]:
            with self.subTest(invalid=invalid), self.assertRaises(HELPER.AcceptanceError):
                HELPER.sleep_evidence(before, invalid)

    def test_preflight_does_not_arm_or_suspend_and_refuses_existing_alarm(self):
        identity = HELPER.readiness_identity(sample())
        outputs = {"/sys/power/state": "freeze mem disk", "/alarm": "",
                   "/sys/class/rtc/rtc0/device/power/wakeup": "enabled"}
        commands = []
        def command(arguments, **_):
            commands.append(arguments)
            if arguments[0].endswith("timedatectl"):
                return "no"
            if arguments[-1] == "CanSuspend":
                return 's "yes"'
            raise AssertionError(arguments)
        with patch.object(HELPER, "ac_online", return_value=True), patch.object(HELPER, "read_text", side_effect=lambda path: outputs[str(path)]), \
                patch.object(HELPER, "run_command", side_effect=command), patch.object(HELPER, "suspend_counters", return_value={"success": 0, "fail": 0}), \
                patch.object(HELPER, "session_unlocked", return_value=True):
            HELPER.check_system(identity, Path("/alarm"))
            outputs["/alarm"] = "1234"
            with self.assertRaises(HELPER.AcceptanceError):
                HELPER.check_system(identity, Path("/alarm"))
        self.assertTrue(all("Suspend" not in args and not args[0].endswith("rtcwake") for args in commands))

    def test_arm_uses_no_sleep_and_cleanup_only_touches_unchanged_owned_alarm(self):
        runner = HELPER.SuspendCycles("/unused", 1000)
        commands = []
        with patch.object(HELPER.time, "time", return_value=1000), \
                patch.object(HELPER, "read_text", side_effect=["", "1020", "1020", ""]), \
                patch.object(HELPER, "run_command", side_effect=lambda args: commands.append(args)):
            runner.arm()
            self.assertEqual(runner.owned_alarm, "1020")
            runner.clear_owned_alarm()
        self.assertEqual([args[args.index("--mode") + 1] for args in commands], ["no", "disable"])
        self.assertIsNone(runner.owned_alarm)
        runner.owned_alarm = "1020"
        with patch.object(HELPER, "read_text", return_value="2000"), patch.object(HELPER, "run_command") as command:
            with self.assertRaises(HELPER.AcceptanceError):
                runner.clear_owned_alarm()
            command.assert_not_called()

    def test_failed_rtc_setup_never_requests_suspend_and_cleans_its_partial_alarm(self):
        runner = HELPER.SuspendCycles("/unused", 1000, cycles=1)
        writer = MemoryWriter()
        commands = []
        alarms = iter(["", "1020", "1020", ""])
        def command(args, **_):
            commands.append(args)
            if "--seconds" in args:
                raise HELPER.AcceptanceError("RTC setup failed")
        with patch.object(HELPER.os, "geteuid", return_value=0), patch.object(runner, "initial", return_value=(HELPER.readiness_identity(sample()), sample())), \
                patch.object(HELPER, "validate_identity"), patch.object(HELPER, "is_fresh_healthy", return_value=True), patch.object(HELPER, "check_system"), \
                patch.object(runner, "snapshot", return_value=snapshot()), patch.object(HELPER.time, "time", return_value=1000), \
                patch.object(HELPER, "read_text", side_effect=lambda _: next(alarms)), patch.object(HELPER, "run_command", side_effect=command):
            report = runner.execute(writer)
        self.assertFalse(report["passed"])
        self.assertEqual(report["completed_cycles"], 0)
        self.assertFalse(any("Suspend" in args for args in commands))
        self.assertIn("disable", commands[-1])
        self.assertEqual(report["cleanup_errors"], [])

    def test_cycle_waits_for_recovery_and_preserves_normal_logind_policy(self):
        runner = HELPER.SuspendCycles("/unused", 1000, cycles=2)
        writer = MemoryWriter()
        commands = []
        recovered = dict(sample(), swaps=300)
        with patch.object(HELPER.os, "geteuid", return_value=0), patch.object(runner, "initial", return_value=(HELPER.readiness_identity(sample()), sample())), \
                patch.object(HELPER, "validate_identity"), patch.object(HELPER, "is_fresh_healthy", return_value=True), patch.object(HELPER, "check_system"), \
                patch.object(runner, "snapshot", return_value=snapshot()), patch.object(runner, "arm"), patch.object(runner, "ready_for_suspend"), patch.object(runner, "clear_owned_alarm") as cleanup, \
                patch.object(runner, "wait_for_kernel_resume", return_value=snapshot(success=1, monotonic=51, boottime=80)), \
                patch.object(runner, "journals", return_value={"kernel": "PM: suspend entry; PM: suspend exit"}), \
                patch.object(runner, "wait_for_scene", side_effect=[recovered, HELPER.AcceptanceError("Unlock timed out")]) as recovery, \
                patch.object(HELPER, "run_command", side_effect=lambda args, **_: commands.append(args)):
            report = runner.execute(writer)
        self.assertFalse(report["passed"])
        self.assertEqual(report["completed_cycles"], 1)
        self.assertEqual(recovery.call_count, 2)
        self.assertEqual(cleanup.call_count, 3)
        self.assertTrue(all(args[-3:] == ["Suspend", "b", "false"] for args in commands))
        self.assertTrue(all("--allow-interactive-authorization=no" in args for args in commands))
        self.assertTrue(any(event.get("phase") == "kernel_resumed" for event in writer.events))

    def test_complete_result_requires_all_cycles_and_recovery(self):
        runner = HELPER.SuspendCycles("/unused", 1000, cycles=1)
        writer = MemoryWriter()
        with patch.object(HELPER.os, "geteuid", return_value=0), patch.object(runner, "initial", return_value=(HELPER.readiness_identity(sample()), sample())), \
                patch.object(HELPER, "validate_identity"), patch.object(HELPER, "is_fresh_healthy", return_value=True), patch.object(HELPER, "check_system"), \
                patch.object(runner, "snapshot", return_value=snapshot()), patch.object(runner, "arm"), patch.object(runner, "ready_for_suspend"), patch.object(runner, "clear_owned_alarm"), \
                patch.object(runner, "wait_for_kernel_resume", return_value=snapshot(success=1, monotonic=51, boottime=80)), \
                patch.object(runner, "journals", return_value={"kernel": "PM: suspend entry; PM: suspend exit"}), \
                patch.object(runner, "wait_for_scene", return_value=dict(sample(), swaps=300)), patch.object(HELPER, "run_command"):
            report = runner.execute(writer)
        self.assertTrue(report["passed"])
        self.assertEqual(report["phase"], "complete")
        self.assertEqual(report["completed_cycles"], report["requested_cycles"])
        self.assertEqual(report["cycles"][0]["recovered_sample"]["swaps"], 300)

    def test_kernel_journal_requires_both_pm_entry_and_exit(self):
        runner = HELPER.SuspendCycles("/unused", 1000)
        identity = HELPER.readiness_identity(sample())
        with patch.object(HELPER, "run_command", side_effect=["service", '{"MESSAGE":"PM: suspend entry (s2idle)"}']):
            with self.assertRaises(HELPER.AcceptanceError):
                runner.journals(snapshot(), snapshot(success=1, monotonic=51, boottime=80), identity)
        with patch.object(HELPER, "run_command", side_effect=["service", '{"MESSAGE":"PM: suspend entry (s2idle)"}\n{"MESSAGE":"PM: suspend exit"}']) as command:
            self.assertIn("suspend exit", runner.journals(snapshot(), snapshot(success=1, monotonic=51, boottime=80), identity)["kernel"])
        self.assertTrue(all("--boot=" + BOOT_ID in call.args[0] for call in command.call_args_list))

    def test_delayed_or_replaced_rtc_alarm_blocks_sleep_before_any_suspend_call(self):
        runner = HELPER.SuspendCycles("/unused", 1000)
        runner.owned_alarm = "1020"
        for current, now in [("2000", 1000), ("1020", 1015), ("1020", 1021)]:
            with self.subTest(current=current, now=now), patch.object(HELPER, "read_text", return_value=current), \
                    patch.object(HELPER, "run_command", return_value="t 5000000") as command, \
                    patch.object(HELPER.time, "time", return_value=now), \
                    patch.object(HELPER, "read_readiness") as readiness:
                with self.assertRaises(HELPER.AcceptanceError):
                    runner.ready_for_suspend(HELPER.readiness_identity(sample()))
                readiness.assert_not_called()
                self.assertTrue(all("Suspend" not in call.args[0] for call in command.call_args_list))

    def test_recovery_keeps_waiting_while_locked_and_requires_fresh_new_frames(self):
        runner = HELPER.SuspendCycles("/unused", 1000, recovery_timeout=5)
        after = snapshot(success=1, monotonic=49, boottime=59)
        now = {"wall_ns": 101_000_000_000, "monotonic_ns": 51_000_000_000, "boottime_ns": 61_000_000_000}
        with patch.object(HELPER.time, "monotonic", side_effect=[0, 0, 1]), patch.object(HELPER.time, "sleep") as sleep, \
                patch.object(HELPER, "read_readiness", side_effect=[dict(sample(), session_locked=True), sample()]), \
                patch.object(HELPER, "validate_identity"), patch.object(HELPER, "clocks", return_value=now), \
                patch.object(HELPER, "session_unlocked", return_value=True):
            self.assertEqual(runner.wait_for_scene(HELPER.readiness_identity(sample()), after, 199), sample())
        sleep.assert_called_once_with(0.5)

    def test_read_only_cli_does_not_create_evidence_or_invoke_authentication(self):
        with patch.object(HELPER.sys, "argv", ["helper", "--readiness", "/unused", "--owner-uid", "1000", "--check", "--authenticate-unlock"]), \
                patch.object(HELPER.SuspendCycles, "initial", return_value=(HELPER.readiness_identity(sample()), sample())), \
                patch.object(HELPER, "EvidenceWriter") as writer, patch.object(HELPER, "run_command") as command, \
                patch.object(HELPER.os, "geteuid", return_value=1000), patch("builtins.print") as output, \
                patch.object(HELPER.getpass, "getpass") as password:
            self.assertEqual(HELPER.main(), 0)
        writer.assert_not_called()
        command.assert_not_called()
        password.assert_not_called()
        report = json.loads(output.call_args.args[0])
        self.assertTrue(report["preflight_passed"])
        self.assertFalse(report["privileged_execution_available"])
        self.assertFalse(report["real_suspend_performed"])

    def test_atomic_reports_reject_existing_directory_and_remain_on_owned_dirfd(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "helper"
            writer = HELPER.EvidenceWriter(output)
            try:
                writer.save({"passed": False}, {"cycle": 1})
                self.assertEqual(json.loads((output / "report.json").read_text()), {"passed": False})
                with self.assertRaises(FileExistsError):
                    HELPER.EvidenceWriter(output)
                moved = Path(directory) / "moved"
                output.rename(moved)
                other = Path(directory) / "other"
                other.mkdir()
                output.symlink_to(other, target_is_directory=True)
                writer.save({"passed": True}, {"cycle": 2})
                self.assertFalse((other / "report.json").exists())
                self.assertEqual(json.loads((moved / "report.json").read_text()), {"passed": True})
                self.assertEqual(len((moved / "suspend-helper.jsonl").read_text().splitlines()), 2)
            finally:
                writer.close()

    def test_root_execution_does_not_inherit_caller_command_environment(self):
        with patch.object(HELPER.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "ok", "")) as run:
            self.assertEqual(HELPER.run_command(["/usr/bin/rtcwake", "--help"]), "ok")
        self.assertEqual(run.call_args.kwargs["env"], HELPER.COMMAND_ENV)
        self.assertFalse(run.call_args.kwargs.get("shell", False))

    def test_authenticated_input_maps_printable_ascii_and_zeros_ephemeral_secret(self):
        self.assertEqual(set(HELPER.ASCII_KEYS), set(range(32, 127)))
        secret = bytearray(b"Fixture1!")
        HELPER.validate_password(secret)
        for invalid in [bytearray(), bytearray(b"x" * 129), bytearray(b"a\nb"), bytearray(b"\xc3\xa9")]:
            with self.subTest(size=len(invalid)), self.assertRaises(HELPER.AcceptanceError):
                HELPER.validate_password(invalid)
        unlocker = HELPER.AuthenticatedUnlock.__new__(HELPER.AuthenticatedUnlock)
        unlocker.secret = secret
        unlocker.close()
        self.assertEqual(secret, bytearray(9))

    def test_every_private_key_requires_agreeing_locked_sources(self):
        keyboard = HELPER.UnlockKeyboard()
        events = []
        checks = []
        def trusted():
            checks.append(True)
            return True
        with patch.object(keyboard, "event", side_effect=lambda code, value: events.append((code, value))), patch.object(HELPER.time, "sleep"):
            keyboard.type_password(bytearray(b"a1"), trusted)
        self.assertEqual(events, [(1, 1), (1, 0), (30, 1), (30, 0), (2, 1), (2, 0), (28, 1), (28, 0)])
        self.assertEqual(len(checks), 4)  # Escape, each private byte, Enter.
        with patch.object(keyboard, "event") as event:
            with self.assertRaises(HELPER.AcceptanceError):
                keyboard.tap(2, lambda: False)
        event.assert_not_called()

    def test_keyboard_discovery_uses_the_event_child_and_always_destroys_device(self):
        keyboard = HELPER.UnlockKeyboard()
        def ioctl(_descriptor, request, argument=None, *_):
            if request == 0x8100552c:
                argument[:7] = b"input7\0"
            return 0
        with patch.object(HELPER.os, "open", return_value=42), patch.object(HELPER.os, "close") as close, \
                patch.object(HELPER.fcntl, "ioctl", side_effect=ioctl) as control, patch.object(HELPER.time, "sleep"), \
                patch.object(HELPER.Path, "glob", return_value=[Path("/sys/devices/virtual/input/input7/event5")]), \
                patch.object(HELPER, "run_command", return_value="ID_INPUT_KEYBOARD=1") as query:
            with self.assertRaises(HELPER.AcceptanceError):
                with keyboard:
                    raise HELPER.AcceptanceError("fixture interruption")
        self.assertIn("--path=/sys/devices/virtual/input/input7/event5", query.call_args.args[0])
        enabled_keys = {call.args[2] for call in control.call_args_list if call.args[1] == 0x40045565}
        self.assertTrue(set(range(1, 32)).issubset(enabled_keys))
        self.assertTrue(any(call.args[1] == 0x5502 for call in control.call_args_list))
        close.assert_called_once_with(42)
        self.assertIsNone(keyboard.descriptor)

    def unlocker_fixture(self):
        unlocker = HELPER.AuthenticatedUnlock.__new__(HELPER.AuthenticatedUnlock)
        unlocker.secret = bytearray(b"Fixture1")
        unlocker.owner_uid = 1000
        return unlocker

    def test_lock_queries_fail_closed_on_unknown_or_disagreeing_sources(self):
        unlocker = self.unlocker_fixture()
        identity = HELPER.readiness_identity(sample())
        for state, hint in [({"locked": None}, False), ({"locked": True}, True), ({"locked": False}, False)]:
            with self.subTest(state=state, hint=hint), patch.object(unlocker, "user_command", return_value=json.dumps(state)), \
                    patch.object(HELPER, "session_unlocked", return_value=hint), patch.object(HELPER.time, "sleep"):
                with self.assertRaises(HELPER.AcceptanceError):
                    unlocker.locked(identity)
        with patch.object(unlocker, "user_command", return_value='{"locked":true}'), patch.object(HELPER, "session_unlocked", return_value=False):
            self.assertTrue(unlocker.locked(identity))

    def test_normal_pam_input_occurs_once_and_auth_failure_stops_without_retry(self):
        unlocker = self.unlocker_fixture()
        identity = HELPER.readiness_identity(sample())
        with patch.object(unlocker, "locked", return_value=True), patch.object(unlocker, "verify_layout"), \
                patch.object(HELPER, "UnlockKeyboard") as keyboard, \
                patch.object(HELPER, "run_command", return_value="pam_unix(login:auth): authentication failure") as command:
            with self.assertRaisesRegex(HELPER.AcceptanceError, "no retry"):
                unlocker.after_resume(identity)
        keyboard.return_value.__enter__.return_value.type_password.assert_called_once()
        self.assertFalse(any("Unlock" in argument or "unlock-session" in argument for call in command.call_args_list for argument in call.args[0]))
        self.assertFalse(any("Fixture1" in argument for call in command.call_args_list for argument in call.args[0]))

    def test_normal_unlock_requires_both_sources_false_and_skips_an_already_unlocked_session(self):
        unlocker = self.unlocker_fixture()
        identity = HELPER.readiness_identity(sample())
        with patch.object(unlocker, "locked", side_effect=[True, True, False]), patch.object(unlocker, "verify_layout"), \
                patch.object(HELPER, "UnlockKeyboard") as keyboard, patch.object(HELPER, "run_command") as command:
            result = unlocker.after_resume(identity)
        self.assertEqual(result, {"authenticated_input": True, "normal_pam_unlock": True, "attempts": 1})
        keyboard.return_value.__enter__.return_value.type_password.assert_called_once()
        command.assert_not_called()
        with patch.object(unlocker, "locked", return_value=False), patch.object(HELPER, "UnlockKeyboard") as keyboard:
            self.assertFalse(unlocker.after_resume(identity)["authenticated_input"])
        keyboard.assert_not_called()

    def test_authenticated_launch_refuses_non_echo_terminal_before_getpass(self):
        with patch.object(HELPER.sys, "argv", ["helper", "--readiness", "/unused", "--owner-uid", "1000",
                                               "--output-dir", "/unused/output", "--authenticate-unlock"]), \
                patch.object(HELPER.os, "geteuid", return_value=0), patch.object(HELPER.sys.stdin, "isatty", return_value=False), \
                patch.object(HELPER.getpass, "getpass") as password, patch.object(HELPER, "EvidenceWriter") as writer, patch("builtins.print"):
            self.assertEqual(HELPER.main(), 1)
        password.assert_not_called()
        writer.assert_not_called()


if __name__ == "__main__":
    unittest.main()
