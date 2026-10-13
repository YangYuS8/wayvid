"""Recorder protocol, clock and evidence regressions; never operate a desktop."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "scene_reliability", Path(__file__).resolve().parents[1] / "scene-reliability.py")
RELIABILITY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RELIABILITY)


def clock(active, sleep=0, wall=None, boot="fixture"):
    return {"monotonic": active, "boottime": active + sleep,
            "realtime": active + sleep if wall is None else wall, "boot_id": boot}


def targets(**changes):
    result = {"active_seconds": 120, "cycles": 0, "suspends": 0,
              "max_sample_gap_seconds": 30, "recovery_seconds": 30,
              "render_p95_ms": 1000 / 30, "memory_growth_limits": {}}
    result.update(changes)
    return result


def scene(swaps, **changes):
    result = {"event": "sample", "phase": "scene", "finalized": True, "healthy": True,
              "backend_generation": 1, "renderer_generation": 8, "item_id": "space",
              "metrics": {"swaps": swaps, "frame_duration_buckets": [swaps] + [0] * 31},
              "resources": {"rust_live_bytes": 1024}}
    result.update(changes)
    return result


def external(rss=100 * 1024 ** 2, locked=False):
    return {"session": {"exists": True, "sha256": "fixed", "assignments": {"eDP1": "space"}},
            "session_locked": locked, "pulse": {"available": True, "count": 1},
            "process": {"smaps_bytes": {"Rss": rss, "Pss": rss}, "fds": 12, "threads": 4}}


def acceptance(fps_limit=1, **changes):
    result = RELIABILITY.Acceptance(targets(**changes))
    result.observe({"event": "started", "backend_generation": 1, "fps_limit": fps_limit}, clock(0))
    result.observe({"event": "soak_started"}, clock(0))
    return result


class SceneReliabilityTests(unittest.TestCase):
    def test_wall_clock_adjustment_is_not_suspend_or_active_time(self):
        result = RELIABILITY.clock_delta(clock(5, wall=10), clock(15, wall=90000))
        self.assertEqual(result["active_clock_seconds"], 10)
        self.assertEqual(result["suspended_seconds"], 0)
        self.assertEqual(RELIABILITY.clock_delta(clock(5), clock(15, sleep=40))["suspended_seconds"], 40)
        with self.assertRaisesRegex(RuntimeError, "reboot"):
            RELIABILITY.clock_delta(clock(5), clock(15, boot="another"))

    def test_only_active_finalized_primary_frames_after_soak_start_are_credited(self):
        result = RELIABILITY.Acceptance(targets())
        result.observe({"event": "started", "backend_generation": 1, "fps_limit": 1}, clock(0))
        result.observe(scene(10), clock(0), external())
        result.observe(scene(20), clock(10), external())
        self.assertEqual(result.active_seconds, 0)
        result.observe({"event": "soak_started"}, clock(10))
        result.observe(scene(20), clock(10), external())
        result.observe(scene(30), clock(20), external())
        result.observe(scene(40, item_id="another"), clock(30), external())
        result.observe(scene(50, paused=True), clock(40), external())
        self.assertEqual(result.active_seconds, 10)

    def test_clock_gap_does_not_earn_soak_and_requires_kernel_pair_state_and_frames(self):
        result = acceptance(suspends=1)
        result.observe(scene(10), clock(0), external())
        result.observe(scene(20), clock(10, sleep=60), external(), {"paired": True})
        self.assertEqual((result.active_seconds, result.suspends), (0, 0))
        result.observe(scene(30), clock(20, sleep=60), external())
        self.assertEqual((result.active_seconds, result.suspends), (10, 1))
        self.assertFalse(result.failures)

    def test_gap_without_kernel_markers_never_counts_as_real_suspend(self):
        result = acceptance(suspends=1)
        result.observe(scene(10), clock(0), external())
        result.observe(scene(20), clock(10, sleep=60), external(), {"paired": False})
        result.observe(scene(30), clock(20, sleep=60), external())
        self.assertEqual(result.suspends, 0)
        self.assertTrue(result.failures)

    def test_changed_session_after_suspend_fails_recovery(self):
        result = acceptance(suspends=1)
        result.observe(scene(10), clock(0), external())
        changed = external()
        changed["session"]["sha256"] = "changed"
        result.observe(scene(20), clock(10, sleep=60), changed, {"paired": True})
        result.observe(scene(30), clock(20, sleep=60), changed)
        self.assertEqual(result.suspends, 0)
        self.assertTrue(result.failures)

    def test_normal_user_unlock_wait_is_not_a_render_recovery_timeout(self):
        result = acceptance(suspends=1)
        result.observe(scene(10), clock(0), external())
        result.observe(scene(10), clock(10, sleep=60), external(locked=True), {"paired": True})
        result.observe(scene(10), clock(600, sleep=60), external(locked=True))
        result.observe(scene(20), clock(610, sleep=60), external())
        result.observe(scene(30), clock(620, sleep=60), external())
        self.assertEqual(result.suspends, 1)
        self.assertFalse(result.failures)

    def test_stale_frames_counter_reset_and_backend_replacement_fail(self):
        for event in (scene(10), scene(5), scene(20, backend_generation=2), scene(20, renderer_generation=9)):
            with self.subTest(event=event):
                result = acceptance()
                result.observe(scene(10), clock(0), external())
                result.observe(event, clock(10), external())
                self.assertTrue(result.failures)

    def test_round_trip_requires_both_finalized_legs_without_repetition(self):
        result = acceptance(cycles=1)
        result.observe({"event": "switch", "cycle": 1, "from": "scene", "to": "video", "finalized": True}, clock(1))
        self.assertEqual(result.cycles, 0)
        result.observe({"event": "switch", "cycle": 1, "from": "video", "to": "scene", "finalized": True}, clock(2))
        self.assertEqual(result.cycles, 1)
        result.observe({"event": "switch", "cycle": 1, "from": "scene", "to": "video", "finalized": True}, clock(3))
        self.assertEqual(result.cycles, 1)
        self.assertTrue(result.failures)

    def test_unfinalized_switches_and_duplicate_capture_fail(self):
        result = acceptance()
        result.observe({"event": "switch", "cycle": 1, "from": "scene", "to": "video", "finalized": False}, clock(1))
        duplicated = external()
        duplicated["pulse"]["count"] = 2
        result.observe(scene(10), clock(2), duplicated)
        self.assertEqual(result.cycles, 0)
        self.assertEqual(len(result.failures), 2)

    def test_p95_uses_only_same_generation_delta_histogram(self):
        result = acceptance()
        before = scene(100)
        before["metrics"]["frame_duration_buckets"] = [0] * 31 + [100]  # Old video/candidate samples.
        after = scene(200)
        after["metrics"]["frame_duration_buckets"] = [100] + [0] * 30 + [100]
        result.observe(before, clock(0), external())
        result.observe(after, clock(10), external())
        self.assertEqual(result.summary()["aggregate_render_p95_upper_bound_ms"], 1)
        self.assertTrue(result.summary()["criteria"]["render_p95_within_budget"])
        self.assertIsNone(RELIABILITY.histogram_p95_ms([0] * 31 + [1]))
        self.assertIsNone(RELIABILITY.histogram_p95_ms([False] * 32))

    def test_memory_needs_explicit_budget_and_disjoint_warm_tail_windows(self):
        limits = {"rss_bytes": 64 * 1024 ** 2, "pss_bytes": 64 * 1024 ** 2, "fds": 2, "threads": 2}
        result = acceptance(memory_growth_limits=limits)
        for index in range(40):
            result.observe(scene(index * 5 + 1), clock(index * 5), external())
        self.assertTrue(result.summary()["criteria"]["memory_growth_within_explicit_budget"])
        result.observe(scene(201), clock(200), external(rss=500 * 1024 ** 2))
        # One outlier is recorded in the peak but does not invent retained growth.
        self.assertEqual(result.summary()["resource_maxima"]["rss_bytes"], 500 * 1024 ** 2)
        for index in range(42, 55):
            result.observe(scene(index * 5 + 1), clock(index * 5), external(rss=500 * 1024 ** 2))
        self.assertFalse(result.summary()["criteria"]["memory_growth_within_explicit_budget"])
        self.assertFalse(acceptance().summary()["criteria"]["memory_growth_within_explicit_budget"])

    def test_complete_finite_fixture_cannot_claim_24_hours(self):
        result = acceptance(active_seconds=86400)
        result.observe(scene(10), clock(0), external())
        result.observe(scene(20), clock(10), external())
        result.observe({"event": "stopped", "shutdown_joined": True}, clock(11))
        self.assertFalse(result.summary()["passed"])
        self.assertFalse(result.summary()["criteria"]["active_scene_soak"])

    def test_fast_draws_cannot_bypass_declared_presentation_cap(self):
        result = acceptance(fps_limit=30)
        result.observe(scene(100), clock(0), external())
        result.observe(scene(403), clock(10), external())  # 303 swaps exceeds 300 + two-frame tolerance.
        self.assertFalse(result.summary()["criteria"]["scene_presentation_cap"])
        self.assertIn("Scene presentation exceeded its declared FPS cap", result.failures)
        tolerant = acceptance(fps_limit=30)
        tolerant.observe(scene(100), clock(0), external())
        tolerant.observe(scene(402), clock(10), external())
        self.assertTrue(tolerant.summary()["criteria"]["scene_presentation_cap"])

    def test_sparse_updates_are_not_a_24_hour_rendering_acceptance(self):
        result = acceptance(fps_limit=30, active_seconds=10)
        result.observe(scene(100), clock(0), external())
        result.observe(scene(101), clock(10), external())
        summary = result.summary()
        self.assertEqual(summary["active_scene_seconds"], 0)
        self.assertEqual(summary["starved_scene_seconds_excluded"], 10)
        self.assertFalse(summary["criteria"]["scene_presentation_rate"])
        self.assertFalse(summary["criteria"]["active_scene_soak"])
        healthy = acceptance(fps_limit=30, active_seconds=10)
        healthy.observe(scene(100), clock(0), external())
        healthy.observe(scene(370), clock(10), external())  # Recorded machine normally presents about 27 FPS.
        self.assertTrue(healthy.summary()["criteria"]["scene_presentation_rate"])
        self.assertEqual(healthy.active_seconds, 10)

    def test_30_fps_histogram_budget_preserves_exact_frame_period(self):
        result = acceptance(fps_limit=30)
        before = scene(100)
        before["metrics"]["frame_duration_buckets"] = [0] * 32
        after = scene(400)
        after["metrics"]["frame_duration_buckets"] = [0] * 30 + [300, 0]
        result.observe(before, clock(0), external())
        result.observe(after, clock(10), external())
        self.assertEqual(result.summary()["aggregate_render_p95_upper_bound_ms"], 1000 / 30)
        self.assertTrue(result.summary()["criteria"]["render_p95_within_budget"])

    def test_libtest_prefix_is_parsed_but_ordinary_output_is_not_evidence(self):
        self.assertEqual(RELIABILITY.probe_event(b'test host ... {"event":"started"}')["event"], "started")
        for line in (b"test host ... ok", b'{"other":"value"}', b'{"event":"sample"'):
            self.assertIsNone(RELIABILITY.probe_event(line))

    def test_atomic_report_keeps_prior_file_if_temporary_path_is_symlink(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            RELIABILITY.atomic_json(path, {"passed": False})
            prior = path.read_bytes()
            target = Path(directory) / "unrelated"
            target.write_text("retain")
            path.with_name(path.name + ".tmp").symlink_to(target)
            with self.assertRaises(OSError):
                RELIABILITY.atomic_json(path, {"passed": True})
            self.assertEqual(path.read_bytes(), prior)
            self.assertEqual(target.read_text(), "retain")

    def test_identity_includes_dirty_runtime_and_untracked_code_but_excludes_docs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            (root / "crates").mkdir()
            code = root / "crates" / "runtime.rs"
            code.write_text("original")
            (root / "docs").mkdir()
            doc = root / "docs" / "guide.md"
            doc.write_text("original")
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                            "commit", "-qm", "fixture"], cwd=root, check=True)
            original = RELIABILITY.source_identity(root)
            doc.write_text("updated")
            self.assertEqual(original["source_sha256"], RELIABILITY.source_identity(root)["source_sha256"])
            code.write_text("changed")
            changed = RELIABILITY.source_identity(root)
            self.assertNotEqual(original["source_sha256"], changed["source_sha256"])
            (root / "crates" / "new.rs").write_text("untracked")
            self.assertNotEqual(changed["source_sha256"], RELIABILITY.source_identity(root)["source_sha256"])

    def test_tree_identity_tracks_inputs_and_rejects_external_symlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "inputs"
            root.mkdir()
            (root / "project.json").write_text("first")
            first = RELIABILITY.tree_identity(root)
            (root / "project.json").write_text("second")
            self.assertNotEqual(first["sha256"], RELIABILITY.tree_identity(root)["sha256"])
            outside = Path(directory) / "private"
            outside.write_text("untouched")
            (root / "external").symlink_to(outside)
            with self.assertRaisesRegex(RuntimeError, "symlink"):
                RELIABILITY.tree_identity(root)

    def test_proc_sampling_uses_starttime_identity_and_available_memory(self):
        result = RELIABILITY.process_sample(os.getpid())
        self.assertEqual(result["pid"], os.getpid())
        self.assertGreater(result["start_ticks"], 0)
        self.assertGreater(result["threads"], 0)
        self.assertGreater(result["status_bytes"]["VmRSS"], 0)


if __name__ == "__main__":
    unittest.main()
