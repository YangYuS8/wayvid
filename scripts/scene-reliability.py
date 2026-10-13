#!/usr/bin/env python3
"""Durable, opt-in real desktop reliability recorder.

Launches an explicitly supplied test executable in a private XDG_CONFIG_HOME.
The executable must emit NDJSON started/sample/switch/stopped events. Ordinary
Rust test-runner output is retained but is not acceptance evidence. This script
never requests suspend: a real sleep counts only with a BOOTTIME/MONOTONIC gap,
paired kernel suspend messages, unchanged state and subsequent rendered frames.
No credentials, environment dump or generated evidence belong in the repository.
"""

import argparse
import collections
import hashlib
import json
import math
import os
from pathlib import Path
import selectors
import signal
import statistics
import subprocess
import sys
import time
import tomllib

SCHEMA = 1
MAX_LINE = 1024 * 1024
SLEEP_GAP_SECONDS = 2.0
SOURCE_PREFIXES = ("crates/", "src/", "src-tauri/src/")
SOURCE_FILES = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "package.json", "pnpm-lock.yaml",
                "src-tauri/Cargo.toml", "src-tauri/tauri.conf.json", "src-tauri/build.rs",
                "scripts/scene-reliability.py", "scripts/scene-suspend-cycles.py"}


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for block in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def tree_identity(path):
    path = path.resolve(strict=True)
    if path.is_file():
        return {"path": str(path), "sha256": sha256_file(path), "bytes": path.stat().st_size}
    digest, count, total = hashlib.sha256(), 0, 0
    for file in sorted(path.rglob("*")):
        if file.is_symlink() and not file.resolve().is_relative_to(path):
            raise RuntimeError("Input tree contains a symlink outside its root")
        if not file.is_file():
            continue
        if not file.resolve().is_relative_to(path):
            raise RuntimeError("Input tree contains a file symlink outside its root")
        relative = file.relative_to(path).as_posix()
        size = file.stat().st_size
        digest.update(relative.encode() + b"\0" + str(size).encode() + b"\0")
        digest.update(bytes.fromhex(sha256_file(file)))
        count += 1
        total += size
    return {"path": str(path), "sha256": digest.hexdigest(), "files": count, "bytes": total}


def source_identity(root):
    """Include dirty tracked and untracked source; exclude Git ignored outputs."""
    root = root.resolve(strict=True)
    names = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=root).split(b"\0")
    digest, count = hashlib.sha256(), 0
    for name in sorted(set(names) - {b""}):
        decoded = os.fsdecode(name)
        if not decoded.startswith(SOURCE_PREFIXES) and decoded not in SOURCE_FILES:
            continue
        path = root / os.fsdecode(name)
        digest.update(name + b"\0")
        if path.is_symlink():
            digest.update(b"symlink\0" + os.fsencode(os.readlink(path)))
        elif path.is_file():
            digest.update(bytes.fromhex(sha256_file(path)))
        else:
            digest.update(b"missing")
        count += 1
    return {"root": str(root), "head": subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "source_sha256": digest.hexdigest(), "files": count,
        "scope": {"prefixes": SOURCE_PREFIXES, "files": sorted(SOURCE_FILES), "docs_excluded": True}}


def atomic_json(path, value):
    temporary = path.with_name(path.name + ".tmp")
    encoded = (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + "\n").encode()
    fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as file:
        file.write(encoded)
        file.flush()
        os.fsync(file.fileno())
    os.replace(temporary, path)
    directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def clock_snapshot():
    return {"monotonic": time.clock_gettime(time.CLOCK_MONOTONIC),
            "boottime": time.clock_gettime(time.CLOCK_BOOTTIME), "realtime": time.time(),
            "boot_id": Path("/proc/sys/kernel/random/boot_id").read_text().strip()}


def clock_delta(before, after):
    if before["boot_id"] != after["boot_id"]:
        raise RuntimeError("Machine reboot broke runtime continuity")
    monotonic = after["monotonic"] - before["monotonic"]
    boottime = after["boottime"] - before["boottime"]
    if monotonic < 0 or boottime < 0:
        raise RuntimeError("Machine clocks moved backwards")
    return {"active_clock_seconds": monotonic, "elapsed_seconds": boottime,
            "suspended_seconds": max(0.0, boottime - monotonic)}


def kernel_suspend_evidence(before, after):
    """Clock gaps alone are insufficient; read paired kernel events for the gap."""
    try:
        result = subprocess.run(["journalctl", "--quiet", "-k", "-b", "-o", "json",
                                 "--since", f"@{before['realtime'] - 1:.6f}",
                                 "--until", f"@{after['realtime'] + 1:.6f}"],
                                capture_output=True, timeout=10, check=True)
        entries = []
        for line in result.stdout.splitlines():
            message = json.loads(line)
            text = message.get("MESSAGE", "")
            if "PM: suspend entry" in text or "PM: suspend exit" in text:
                entries.append({"message": text, "realtime_us": message.get("__REALTIME_TIMESTAMP"),
                                "monotonic_us": message.get("__MONOTONIC_TIMESTAMP")})
        entering = next((index for index, entry in enumerate(entries) if "PM: suspend entry" in entry["message"]), None)
        paired = entering is not None and any("PM: suspend exit" in entry["message"] for entry in entries[entering + 1:])
        return {"paired": paired, "entries": entries, "available": True}
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        return {"paired": False, "available": False, "error": type(error).__name__}


def kib_fields(path):
    values = {}
    for line in path.read_text().splitlines():
        fields = line.split()
        if len(fields) == 3 and fields[2] == "kB" and fields[1].isdigit():
            values[fields[0].rstrip(":")] = int(fields[1]) * 1024
    return values


def proc_identity(pid):
    # starttime distinguishes an owned child from a later PID reuse.
    text = Path(f"/proc/{pid}/stat").read_text()
    fields = text[text.rfind(")") + 2:].split()
    return {"pid": pid, "start_ticks": int(fields[19]),
            "cpu_ticks": int(fields[11]) + int(fields[12])}


def process_sample(pid):
    proc = Path(f"/proc/{pid}")
    result = proc_identity(pid)
    result["fds"] = len(list((proc / "fd").iterdir()))
    result["threads"] = len(list((proc / "task").iterdir()))
    result["status_bytes"] = kib_fields(proc / "status")
    try:
        result["smaps_bytes"] = kib_fields(proc / "smaps_rollup")
    except OSError as error:
        result["smaps_error"] = type(error).__name__
    drm = {}
    for file in (proc / "fdinfo").iterdir():
        try:
            lines = file.read_text().splitlines()
        except OSError:
            continue
        fields = {line.split(":", 1)[0]: line.split(":", 1)[1].strip() for line in lines if ":" in line}
        if "drm-client-id" in fields:
            key = "/".join(fields.get(name, "unknown") for name in ("drm-pdev", "drm-driver", "drm-client-id"))
            drm[key] = {name: value for name, value in fields.items() if name.startswith(("drm-memory-", "drm-engine-"))}
    result["drm_clients"] = drm
    for name in ("oom_score", "oom_score_adj"):
        try:
            result[name] = int((proc / name).read_text())
        except OSError:
            pass
    gpu = {}
    for file in Path("/sys/class/drm").glob("card*/device/gpu_busy_percent"):
        try:
            gpu[file.parent.parent.name] = int(file.read_text())
        except (OSError, ValueError):
            pass
    result["global_gpu_busy_percent"] = gpu  # Whole GPU, not this process or power.
    return result


def pulse_sample(pid):
    try:
        result = subprocess.run(["pactl", "--format=json", "list", "source-outputs"],
                                capture_output=True, check=True, timeout=5)
        streams = json.loads(result.stdout)
        owned = [{"index": stream.get("index"), "source": stream.get("source"), "corked": stream.get("corked")}
                 for stream in streams if str(stream.get("properties", {}).get("application.process.id")) == str(pid)]
        return {"available": True, "owned_source_outputs": owned, "count": len(owned)}
    except (OSError, subprocess.SubprocessError, ValueError, TypeError) as error:
        return {"available": False, "error": type(error).__name__}


def session_sample(config):
    path = config / "lwe" / "session.toml"
    if not path.exists():
        return {"exists": False}
    contents = path.read_bytes()
    state = tomllib.loads(contents.decode())
    return {"exists": True, "sha256": hashlib.sha256(contents).hexdigest(),
            "bytes": len(contents), "assignments": state.get("assignments", {}),
            "scene_properties_sha256": hashlib.sha256(json.dumps(
                state.get("scene_properties", {}), sort_keys=True).encode()).hexdigest()}


def session_locked():
    try:
        result = subprocess.run(["loginctl", "show-session", os.environ.get("XDG_SESSION_ID", "self"),
                                 "--property=LockedHint", "--value"],
                                capture_output=True, text=True, check=True, timeout=5)
        return {"yes": True, "no": False}.get(result.stdout.strip())
    except (OSError, subprocess.SubprocessError):
        return None


def readiness(event, clock, owned, locked=None, soak_ready=False):
    swaps = event.get("metrics", {}).get("swaps")
    return {"schema": SCHEMA, "owner_uid": os.getuid(), "child_pid": owned["pid"],
            "child_starttime_ticks": owned["start_ticks"], "boot_id": clock["boot_id"],
            "sampled_wall_time_ns": round(clock["realtime"] * 1e9),
            "monotonic_ns": round(clock["monotonic"] * 1e9), "boottime_ns": round(clock["boottime"] * 1e9),
            "scene_active": soak_ready and event.get("phase") == "scene" and event.get("finalized") is True and
                            event.get("healthy") is True and event.get("paused") is not True and finite_positive(swaps),
            "swaps": swaps, "healthy": event.get("healthy") is True, "session_locked": locked,
            "session_id": os.environ.get("XDG_SESSION_ID")}


def probe_event(line):
    # libtest can print 'test name ... ' immediately before the first JSON line.
    start = line.find(b"{")
    if start < 0:
        return None
    try:
        event = json.loads(line[start:])
    except (ValueError, UnicodeDecodeError):
        return None
    return event if isinstance(event, dict) and isinstance(event.get("event"), str) else None


def finite_positive(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def histogram_p95_ms(counts):
    if not isinstance(counts, list) or len(counts) != 32 or any(type(count) is not int or count < 0 for count in counts):
        return None
    total = sum(counts)
    if total == 0:
        return None
    threshold, cumulative = math.ceil(total * 0.95), 0
    for index, count in enumerate(counts):
        cumulative += count
        if cumulative >= threshold:
            # Histogram reports an upper bound; overflow cannot pass a finite budget.
            return float(index + 1) if index < 30 else (1000 / 30 if index == 30 else None)
    return None


class Acceptance:
    """Bounded summary; full samples are streamed to evidence.jsonl on disk."""

    def __init__(self, targets):
        self.targets = targets
        self.active_seconds = 0.0
        self.cycles = 0
        self.suspends = 0
        self.samples = 0
        self.failures = []
        self.backend = None
        self.last = None
        self.pending_cycle = None
        self.seen_cycles = set()
        self.pending_sleep = None
        self.shutdown_joined = False
        self.worst_p95_ms = None
        self.memory = collections.deque(maxlen=120)
        self.warm_memory = []
        self.maxima = {}
        self.started_clock = None
        self.primary_item_id = None
        self.soak_started = False
        self.timing_bucket_totals = [0] * 32
        self.scene_swaps = 0
        self.min_interval_fps = None
        self.max_interval_fps = None
        self.max_cpu_percent = None
        self.fps_limit = None
        self.presentation_seconds = 0.0
        self.presentation_swaps = 0
        self.starved_scene_seconds = 0.0
        self.cap_violations = 0

    def fail(self, message):
        if message not in self.failures:
            self.failures.append(message)

    def observe(self, event, clock, external=None, sleep_evidence=None):
        external = external or {}
        kind = event["event"]
        if kind == "started":
            if self.started_clock is not None:
                self.fail("Probe restarted its backend during acceptance")
            self.started_clock = clock
            self.backend = event.get("backend_identity", event.get("backend_generation"))
            cap = event.get("fps_limit")
            if type(cap) is not int or not 1 <= cap <= 240:
                self.fail("Probe did not declare a valid presentation FPS cap")
            else:
                self.fps_limit = cap
            return
        if kind == "error" or event.get("healthy") is False:
            self.fail("Probe reported an unhealthy runtime")
        backend = event.get("backend_identity", event.get("backend_generation"))
        if backend is not None and self.backend is not None and backend != self.backend:
            self.fail("Backend identity changed during acceptance")
        if kind == "soak_started":
            if self.soak_started:
                self.fail("Primary Scene soak restarted during acceptance")
            self.soak_started = True
            self.last = None
            return
        if kind == "switch":
            self.switch(event)
            self.last = None  # Switching/CPU preparation earns no Scene soak time.
            return
        if kind == "stopped":
            self.shutdown_joined = event.get("shutdown_joined") is True
            if not self.shutdown_joined:
                self.fail("Owned backend shutdown was not joined")
            return
        if kind not in {"sample", "snapshot"}:
            return
        self.samples += 1
        metrics = event.get("metrics", {})
        scene = event.get("phase") == "scene" and event.get("finalized") is True and event.get("healthy") is True and event.get("paused") is not True and external.get("session_locked") is not True
        if scene and self.primary_item_id is None:
            self.primary_item_id = event.get("item_id")
        scene = scene and self.soak_started and event.get("item_id") == self.primary_item_id and self.cycles >= self.targets["cycles"]
        swaps = metrics.get("swaps")
        if not finite_positive(swaps):
            self.fail("Runtime sample lacks a valid completed-swap counter")
            scene = False
        p95 = metrics.get("render_p95_ms", metrics.get("frame_p95_ms"))
        if finite_positive(p95):
            self.worst_p95_ms = max(self.worst_p95_ms or 0.0, p95)
        if self.last is not None:
            previous, previous_clock, previous_external, previous_scene = self.last
            delta = clock_delta(previous_clock, clock)
            previous_swaps = previous.get("metrics", {}).get("swaps", 0)
            increasing = scene and previous_scene and swaps > previous_swaps
            generation_equal = event.get("renderer_generation") == previous.get("renderer_generation")
            if scene and previous_scene and not generation_equal:
                self.fail("Primary Scene renderer generation changed during soak")
                increasing = False
            if increasing:
                buckets = metrics.get("frame_duration_buckets")
                before_buckets = previous.get("metrics", {}).get("frame_duration_buckets")
                if isinstance(buckets, list) and isinstance(before_buckets, list) and len(buckets) == len(before_buckets) == 32 and \
                        all(type(a) is int and type(b) is int and a >= b >= 0 for a, b in zip(buckets, before_buckets)):
                    differences = [a - b for a, b in zip(buckets, before_buckets)]
                    self.timing_bucket_totals = [a + b for a, b in zip(self.timing_bucket_totals, differences)]
                    interval_p95 = histogram_p95_ms(differences)
                    if finite_positive(interval_p95):
                        self.worst_p95_ms = max(self.worst_p95_ms or 0.0, interval_p95)
            if scene and previous_scene and swaps < previous_swaps:
                self.fail("Scene counters reset without a finalized switch")
            if delta["suspended_seconds"] >= SLEEP_GAP_SECONDS:
                self.pending_sleep = {"clock": clock, "elapsed_seconds": delta["suspended_seconds"],
                                      "kernel": sleep_evidence or {"paired": False},
                                      "session_before": previous_external.get("session"),
                                      "session_after": external.get("session"),
                                      "recovery_start": clock["monotonic"] if scene else None}
                # Include neither sleep nor the first uncertain recovery interval.
            elif increasing and delta["active_clock_seconds"] <= self.targets["max_sample_gap_seconds"]:
                frame_count = swaps - previous_swaps
                if delta["active_clock_seconds"] > 0:
                    fps = frame_count / delta["active_clock_seconds"]
                    self.presentation_swaps += frame_count
                    self.presentation_seconds += delta["active_clock_seconds"]
                    if self.fps_limit is not None and frame_count > self.fps_limit * delta["active_clock_seconds"] + 2:
                        self.cap_violations += 1
                        self.fail("Scene presentation exceeded its declared FPS cap")
                    if self.fps_limit is not None and fps >= self.fps_limit * 0.7:
                        self.active_seconds += delta["active_clock_seconds"]
                        self.scene_swaps += frame_count
                    else:
                        self.starved_scene_seconds += delta["active_clock_seconds"]
                    self.min_interval_fps = min(self.min_interval_fps if self.min_interval_fps is not None else fps, fps)
                    self.max_interval_fps = max(self.max_interval_fps or 0.0, fps)
                    before_cpu = previous_external.get("process", {}).get("cpu_ticks")
                    after_cpu = external.get("process", {}).get("cpu_ticks")
                    if type(before_cpu) is int and type(after_cpu) is int and after_cpu >= before_cpu:
                        cpu = 100 * (after_cpu - before_cpu) / os.sysconf("SC_CLK_TCK") / delta["active_clock_seconds"]
                        self.max_cpu_percent = max(self.max_cpu_percent or 0.0, cpu)
            elif previous_scene and scene:
                self.fail("Scene failed to present within the bounded sampling interval")
            if self.pending_sleep is not None and scene and self.pending_sleep["recovery_start"] is None:
                # A lock can intentionally withhold frames. Bound recovery from
                # the first unlocked Scene sample, not time spent awaiting the user.
                self.pending_sleep["recovery_start"] = clock["monotonic"]
            if self.pending_sleep is not None and increasing and delta["suspended_seconds"] < SLEEP_GAP_SECONDS:
                sleep = self.pending_sleep
                recovery = clock["monotonic"] - sleep["recovery_start"]
                session_ok = sleep["session_before"] == sleep["session_after"] == external.get("session")
                if sleep["kernel"].get("paired") and session_ok and recovery <= self.targets["recovery_seconds"]:
                    self.suspends += 1
                else:
                    self.fail("Suspend recovery lacked paired kernel/state/frame evidence")
                self.pending_sleep = None
        pulse = external.get("pulse", {})
        if pulse.get("available") and pulse.get("count", 0) > 1:
            self.fail("Owned process accumulated duplicate audio capture streams")
        resources = event.get("resources") or {}
        proc = external.get("process", {})
        smaps = proc.get("smaps_bytes", {})
        memory = {"rss_bytes": smaps.get("Rss", proc.get("status_bytes", {}).get("VmRSS")),
                  "pss_bytes": smaps.get("Pss"), "rust_live_bytes": resources.get("rust_live_bytes"),
                  "fds": proc.get("fds"), "threads": proc.get("threads")}
        for name, value in memory.items():
            if finite_positive(value):
                self.maxima[name] = max(value, self.maxima.get(name, 0))
        # Compare the same finalized Scene state after initial allocation warm-up.
        if scene and self.cycles >= min(10, self.targets["cycles"]) and self.active_seconds >= min(60, self.targets["active_seconds"]):
            self.memory.append(memory)
            if len(self.warm_memory) < 12:
                self.warm_memory.append(memory)
        self.last = (event, clock, external, scene)

    def switch(self, event):
        cycle = event.get("cycle")
        if type(cycle) is not int or cycle < 1 or event.get("finalized") is not True:
            self.fail("Switch lacked a valid finalized cycle")
            return
        leg = (event.get("from"), event.get("to"))
        if leg == ("scene", "video") and cycle not in self.seen_cycles and self.pending_cycle is None:
            self.pending_cycle = cycle
        elif leg == ("video", "scene") and self.pending_cycle == cycle and cycle not in self.seen_cycles:
            self.seen_cycles.add(cycle)
            self.pending_cycle = None
            self.cycles += 1
        else:
            self.fail("Switch sequence repeated, skipped or did not complete a round trip")

    def summary(self):
        growth = {}
        for name in self.maxima:
            first = [sample[name] for sample in self.warm_memory if finite_positive(sample.get(name))]
            last = [sample[name] for sample in list(self.memory)[-12:] if finite_positive(sample.get(name))]
            if first and last:
                growth[name] = statistics.median(last) - statistics.median(first)
        limits = self.targets["memory_growth_limits"]
        memory_pass = bool(limits) and len(self.warm_memory) >= 12 and len(self.memory) >= 24 and all(
            name in growth and growth[name] <= limit for name, limit in limits.items())
        aggregate_p95 = histogram_p95_ms(self.timing_bucket_totals)
        measured_p95 = aggregate_p95 if sum(self.timing_bucket_totals) else self.worst_p95_ms
        timing_pass = measured_p95 is not None and measured_p95 <= self.targets["render_p95_ms"]
        aggregate_fps = self.presentation_swaps / self.presentation_seconds if self.presentation_seconds else None
        criteria = {"active_scene_soak": self.active_seconds >= self.targets["active_seconds"],
                    "full_finalized_round_trips": self.cycles >= self.targets["cycles"],
                    "real_suspend_resume": self.suspends >= self.targets["suspends"],
                    "memory_growth_within_explicit_budget": memory_pass,
                    "render_p95_within_budget": timing_pass,
                    "scene_presentation_rate": aggregate_fps is not None and self.fps_limit is not None and aggregate_fps >= self.fps_limit * 0.7,
                    "scene_presentation_cap": self.fps_limit is not None and self.presentation_swaps > 0 and self.cap_violations == 0,
                    "shutdown_joined": self.shutdown_joined, "runtime_healthy": not self.failures}
        return {"passed": all(criteria.values()), "criteria": criteria,
                "completed_components": [name for name, passed in criteria.items() if passed],
                "incomplete_components": [name for name, passed in criteria.items() if not passed],
                "active_scene_seconds": self.active_seconds,
                "full_round_trips": self.cycles, "real_suspend_resume_cycles": self.suspends,
                "samples": self.samples, "worst_render_p95_ms": self.worst_p95_ms,
                "aggregate_render_p95_upper_bound_ms": aggregate_p95,
                "soak_frame_duration_buckets": self.timing_bucket_totals,
                "scene_completed_swaps": self.scene_swaps,
                "scene_swaps_per_active_second": self.scene_swaps / self.active_seconds if self.active_seconds else None,
                "min_interval_swaps_per_second": self.min_interval_fps,
                "max_interval_swaps_per_second": self.max_interval_fps,
                "max_scene_cpu_percent_one_core": self.max_cpu_percent,
                "fps_limit": self.fps_limit, "aggregate_scene_presentation_fps": aggregate_fps,
                "starved_scene_seconds_excluded": self.starved_scene_seconds,
                "fps_cap_violations": self.cap_violations,
                "memory_growth_bytes_or_counts": growth, "resource_maxima": self.maxima,
                "failures": self.failures, "pending_suspend": self.pending_sleep is not None}


def stop_owned(process):
    if process.poll() is not None:
        return
    # Only the newly created process group is signalled; no existing desktop PID.
    os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=10)


def requested_stop_reason(summary, finish_after_soak=False):
    criteria = summary["criteria"]
    if not criteria["active_scene_soak"] or not criteria["full_finalized_round_trips"]:
        return None
    if criteria["real_suspend_resume"]:
        return "all_runtime_targets_reached"
    if finish_after_soak:
        return "soak_and_switch_targets_reached_suspend_incomplete"
    return None


def interruption_telemetry(signum):
    return {"received_signal": signum,
            "received_signal_name": signal.Signals(signum).name if signum is not None else None}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--video", type=Path, required=True)
    parser.add_argument("--scene-ids", help="Ordered sibling Workshop IDs; first must be --project directory name")
    parser.add_argument("--source-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--hours", type=float, default=24)
    parser.add_argument("--max-hours", type=float, default=168, help="Bounded wall-clock lifetime while waiting for real suspend evidence")
    parser.add_argument("--cycles", type=int, default=100, help="Full Scene→Video→Scene round trips")
    parser.add_argument("--suspend-cycles", type=int, default=10)
    parser.add_argument("--sample-seconds", type=float, default=10)
    parser.add_argument("--recovery-seconds", type=float, default=30)
    parser.add_argument("--render-p95-ms", type=float, default=1000 / 30)
    parser.add_argument("--memory-growth-mib", type=float, help="Explicit post-warm-up RSS/PSS growth budget; no default pass claim")
    parser.add_argument("--fd-growth", type=int, default=2)
    parser.add_argument("--thread-growth", type=int, default=2)
    parser.add_argument("--resume", action="store_true", help="Retain earlier failures; begin a new continuity segment")
    parser.add_argument("--finish-after-soak", action="store_true",
                        help="Finish/join after the soak and switch targets; missing suspend evidence still fails combined acceptance")
    parser.add_argument("probe_arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if os.environ.get("LWE_REAL_DESKTOP_TESTS") != "1":
        parser.error("Set LWE_REAL_DESKTOP_TESTS=1 to authorize real wallpaper application")
    numbers = (args.hours, args.max_hours, args.sample_seconds, args.recovery_seconds, args.render_p95_ms,
               args.fd_growth, args.thread_growth)
    if any(not finite_positive(value) for value in numbers) or args.hours <= 0 or args.hours > args.max_hours or args.max_hours > 168 or args.sample_seconds < 1 or \
            not 0 <= args.cycles <= 10000 or not 0 <= args.suspend_cycles <= 1000 or \
            (args.memory_growth_mib is not None and not finite_positive(args.memory_growth_mib)):
        parser.error("Invalid bounded target, interval or budget")
    binary = args.binary.resolve(strict=True)
    if not binary.is_file() or not os.access(binary, os.X_OK):
        parser.error("--binary must be an executable file")
    arguments = args.probe_arguments[1:] if args.probe_arguments[:1] == ["--"] else args.probe_arguments
    output = args.output.absolute()
    if output.resolve().is_relative_to(args.source_root.resolve()):
        parser.error("Evidence must be outside the source tree")
    output.mkdir(mode=0o700, parents=True, exist_ok=True)
    if output.is_symlink():
        parser.error("Evidence directory must not be a symlink")
    config = output / "config"
    config.mkdir(mode=0o700, exist_ok=True)
    scene_ids = args.scene_ids.split(",") if args.scene_ids else [args.project.name]
    if not scene_ids or len(scene_ids) > 20 or any(not item.isdecimal() or len(item) > 20 for item in scene_ids) or \
            len(set(scene_ids)) != len(scene_ids) or scene_ids[0] != args.project.name:
        parser.error("Scene IDs must be unique decimal Workshop IDs, beginning with --project basename")
    scene_projects = [args.project.parent / item for item in scene_ids]
    limits = {}
    if args.memory_growth_mib is not None:
        limits = {"rss_bytes": args.memory_growth_mib * 1024 ** 2, "pss_bytes": args.memory_growth_mib * 1024 ** 2,
                  "fds": args.fd_growth, "threads": args.thread_growth}
    targets = {"active_seconds": args.hours * 3600, "cycles": args.cycles, "suspends": args.suspend_cycles,
               "max_sample_gap_seconds": max(args.sample_seconds * 3, 30),
               "max_elapsed_seconds": args.max_hours * 3600,
               "recovery_seconds": args.recovery_seconds, "render_p95_ms": args.render_p95_ms,
               "memory_growth_limits": limits, "finish_after_soak": args.finish_after_soak}
    identity = {"schema": SCHEMA, "binary": tree_identity(binary), "source": source_identity(args.source_root),
                "project": tree_identity(args.project), "scene_projects": [tree_identity(path) for path in scene_projects],
                "assets": tree_identity(args.assets), "video": tree_identity(args.video),
                "argument_sha256": hashlib.sha256(json.dumps(arguments).encode()).hexdigest(), "targets": targets}
    manifest_path = output / "manifest.json"
    report_path = output / "report.json"
    if manifest_path.exists():
        if not args.resume or json.loads(manifest_path.read_text()) != identity:
            parser.error("Existing evidence requires --resume and identical source/build/inputs/targets")
        previous = json.loads(report_path.read_text())
        child = previous.get("owned_child")
        if child:
            try:
                if proc_identity(child["pid"])["start_ticks"] == child["start_ticks"]:
                    parser.error("Previous owned child still exists; do not launch a second backend")
            except FileNotFoundError:
                pass
        previous_segments = previous.get("segments", [])
    else:
        if args.resume:
            parser.error("--resume requires existing evidence")
        if any(output.iterdir()) and list(output.iterdir()) != [config]:
            parser.error("New evidence output must be empty")
        atomic_json(manifest_path, identity)
        previous_segments = []
    acceptance = Acceptance(targets)
    report = {"schema": SCHEMA, "status": "starting", "passed": False, "scope": "Real isolated host/runtime acceptance; one uninterrupted backend per segment",
              "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
              "segments": list(previous_segments), "identity_manifest": str(manifest_path), "suspend_requested_by_harness": False,
              "received_signal": None, "received_signal_name": None, "stop_reason": None,
              "proof_scope": "combined_daily_use_targets"}
    if previous_segments:
        report["prior_continuity_failures"] = "Earlier segments are retained; no uninterrupted combined acceptance claim"
    atomic_json(report_path, report)
    environment = os.environ.copy()
    environment["XDG_CONFIG_HOME"] = str(config)
    environment["LWE_RELIABILITY_SOAK_SECONDS"] = str(math.ceil(targets["active_seconds"]))
    environment["LWE_RELIABILITY_SWITCH_CYCLES"] = str(args.cycles)
    environment["LWE_RELIABILITY_SAMPLE_SECONDS"] = str(math.ceil(args.sample_seconds))
    environment["LWE_RELIABILITY_OUTPUT"] = str(output)
    environment["LWE_RELIABILITY_VIDEO_FILE"] = str(args.video.resolve(strict=True))
    environment["LWE_RELIABILITY_SCENE_IDS"] = ",".join(scene_ids)
    stop_file = config / "soak-stop"
    if stop_file.exists():
        if not args.resume or stop_file.read_bytes() != b"stop\n":
            parser.error("Existing stop marker is not owned resumable evidence")
        stop_file.unlink()
    environment["LWE_RELIABILITY_STOP_FILE"] = str(stop_file)
    environment["LWE_RELIABILITY_MAX_SECONDS"] = str(math.ceil(targets["max_elapsed_seconds"]))
    interrupted = [None]
    def interrupt(signum, frame):
        interrupted[0] = signum
    saved_signals = {signum: signal.signal(signum, interrupt) for signum in (signal.SIGINT, signal.SIGTERM)}
    process = None
    event_count = 0
    last_clock = clock_snapshot()
    last_event_clock = last_clock
    segment = {"started_clock": last_clock, "status": "running"}
    selector = selectors.DefaultSelector()
    try:
        with (output / "probe-stderr.log").open("ab", buffering=0) as stderr, \
                (output / "probe-stdout.log").open("ab", buffering=0) as stdout, \
                (output / "evidence.jsonl").open("ab", buffering=0) as evidence:
            process = subprocess.Popen([str(binary), *arguments], stdout=subprocess.PIPE, stderr=stderr,
                                       env=environment, cwd=args.source_root, start_new_session=True)
            owned = proc_identity(process.pid)
            report["owned_child"] = {key: owned[key] for key in ("pid", "start_ticks")}
            report["recorder_pid"] = os.getpid()
            report["status"] = "running"
            atomic_json(report_path, report)
            print(f"Owned test PID {process.pid}; durable report: {report_path}", flush=True)
            os.set_blocking(process.stdout.fileno(), False)
            selector.register(process.stdout, selectors.EVENT_READ)
            pending = b""
            while selector.get_map() and not interrupted[0]:
                for key, mask in selector.select(timeout=1):
                    block = os.read(key.fileobj.fileno(), 65536)
                    if not block:
                        selector.unregister(key.fileobj)
                        continue
                    stdout.write(block)
                    pending += block
                    if len(pending) > MAX_LINE and b"\n" not in pending:
                        raise RuntimeError("Probe emitted an unbounded output line")
                    while b"\n" in pending:
                        line, pending = pending.split(b"\n", 1)
                        if len(line) > MAX_LINE:
                            raise RuntimeError("Probe emitted an unbounded output line")
                        event = probe_event(line)
                        if event is None:
                            continue
                        now = clock_snapshot()
                        external = {}
                        if event["event"] in {"sample", "snapshot"}:
                            external = {"process": process_sample(process.pid), "pulse": pulse_sample(process.pid),
                                        "session": session_sample(config), "session_locked": session_locked()}
                            if external["process"]["start_ticks"] != owned["start_ticks"]:
                                raise RuntimeError("Owned child PID identity changed")
                            soak_ready = acceptance.soak_started and acceptance.cycles >= targets["cycles"] and event.get("item_id") == acceptance.primary_item_id
                            atomic_json(output / "suspend-readiness.json", readiness(event, now, owned, external["session_locked"], soak_ready))
                        sleep = None
                        if clock_delta(last_event_clock, now)["suspended_seconds"] >= SLEEP_GAP_SECONDS:
                            sleep = kernel_suspend_evidence(last_event_clock, now)
                        acceptance.observe(event, now, external, sleep)
                        record = {"sequence": event_count, "clock": now, "probe": event, "external": external}
                        if sleep is not None:
                            record["suspend_evidence"] = sleep
                        evidence.write((json.dumps(record, allow_nan=False) + "\n").encode())
                        os.fsync(evidence.fileno())
                        event_count += 1
                        last_event_clock = now
                        report["current"] = acceptance.summary()
                        report["last_clock"] = now
                        atomic_json(report_path, report)
                        fatal = [failure for failure in acceptance.failures if failure !=
                                 "Suspend recovery lacked paired kernel/state/frame evidence"]
                        if fatal:
                            raise RuntimeError(f"Runtime evidence failed: {fatal[0]}")
                        stop_reason = requested_stop_reason(report["current"], args.finish_after_soak)
                        if stop_reason is not None and not stop_file.exists():
                            report["stop_reason"] = stop_reason
                            if stop_reason == "soak_and_switch_targets_reached_suspend_incomplete":
                                report["proof_scope"] = "scene_soak_and_switches_only_suspend_incomplete"
                            fd = os.open(stop_file, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
                            with os.fdopen(fd, "wb") as marker:
                                marker.write(b"stop\n")
                                marker.flush()
                                os.fsync(marker.fileno())
                            atomic_json(report_path, report)
                now = clock_snapshot()
                # Exclude real sleep from this watchdog too; recovery gets its own window.
                gap = clock_delta(last_clock, now)
                last_clock = now
                if now["boottime"] - segment["started_clock"]["boottime"] > targets["max_elapsed_seconds"]:
                    raise RuntimeError("Bounded acceptance lifetime expired with incomplete targets")
                if gap["suspended_seconds"] >= SLEEP_GAP_SECONDS:
                    continue
                if now["monotonic"] - last_event_clock["monotonic"] > targets["max_sample_gap_seconds"] + targets["recovery_seconds"]:
                    raise RuntimeError("Owned probe stopped emitting bounded heartbeat evidence")
            if interrupted[0]:
                acceptance.fail("Recorder interrupted; uninterrupted acceptance was not completed")
                report["status"] = "interrupted"
                stop_owned(process)
            else:
                code = process.wait(timeout=15)
                report["child_exit_code"] = code
                if code != 0:
                    acceptance.fail("Owned test executable exited unsuccessfully")
                if pending.strip():
                    acceptance.fail("Probe ended with an incomplete final output line")
                report["status"] = "completed"
            if source_identity(args.source_root)["source_sha256"] != identity["source"]["source_sha256"] or tree_identity(binary) != identity["binary"]:
                acceptance.fail("Source or test executable changed during the acceptance run")
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError) as error:
        acceptance.fail(f"Recorder failure: {type(error).__name__}: {error}")
        report["status"] = "failed"
    finally:
        selector.close()
        if process is not None:
            try:
                stop_owned(process)
            except (OSError, subprocess.SubprocessError) as error:
                acceptance.fail(f"Owned process cleanup failed: {type(error).__name__}")
            if process.stdout is not None:
                process.stdout.close()
        for signum, handler in saved_signals.items():
            signal.signal(signum, handler)
        segment["ended_clock"] = clock_snapshot()
        segment["status"] = report["status"]
        report.update(interruption_telemetry(interrupted[0]))
        segment.update(interruption_telemetry(interrupted[0]))
        segment["stop_reason"] = report["stop_reason"]
        segment["acceptance"] = acceptance.summary()
        report["segments"].append(segment)
        report["current"] = acceptance.summary()
        report["completed_components"] = report["current"]["completed_components"]
        report["passed"] = report["current"]["passed"] and report["status"] == "completed" and not previous_segments
        atomic_json(report_path, report)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
