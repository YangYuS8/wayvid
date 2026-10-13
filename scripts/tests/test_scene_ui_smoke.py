"""No desktop operations: native control contracts, publication and ownership safety."""

import copy
import hashlib
import importlib.util
from pathlib import Path
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "scene_ui_smoke", Path(__file__).resolve().parents[1] / "scene-ui-smoke.py")
SMOKE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SMOKE)


class SceneUiSmokeTests(unittest.TestCase):
    def property_project(self):
        return {"general": {"properties": {
            "enabled": {"type": "bool", "value": True, "text": "作者开关", "order": 10},
            "intensity": {"type": "slider", "value": 0, "min": 0, "max": 1, "step": 0.1,
                          "text": "Author slider", "order": 11},
            "tint": {"type": "color", "value": "0.7 0.5 0.25", "text": "作者颜色", "order": 0}
        }}}

    def rendered_controls(self, plan):
        controls = []
        for expected in plan["controls"]:
            kind, value = expected["kind"], expected["value"]
            actual = {"label": expected["label"], "type": {"bool": "checkbox", "slider": "number", "color": "color", "combo": "select-one"}[kind],
                      "disabled": False, "checked": value if kind == "bool" else False,
                      "value": str(value), "min": "", "max": "", "step": ""}
            if kind == "color":
                actual["value"] = "#b28040"  # f32 0.7 rounds below the exact decimal midpoint.
            if kind == "slider":
                actual.update({field: str(expected[field]) if expected[field] is not None else
                               ("any" if field == "step" else "") for field in ("min", "max", "step")})
            if kind == "combo":
                actual["options"] = copy.deepcopy(expected["options"])
            controls.append(actual)
        return controls

    def test_property_plan_uses_author_order_labels_and_a_distinct_bounded_slider(self):
        plan = SMOKE.scene_property_plan(self.property_project())
        self.assertEqual([control["name"] for control in plan["controls"]], ["tint", "enabled", "intensity"])
        self.assertEqual((plan["boolIndex"], plan["sliderIndex"]), (1, 2))
        self.assertFalse(plan["changed"]["enabled"])
        self.assertEqual(plan["changed"]["intensity"], 0.1)
        self.assertEqual(plan["controls"][1]["label"], "作者开关")
        project = self.property_project()
        project["general"]["properties"]["intensity"]["value"] = 1
        self.assertEqual(SMOKE.scene_property_plan(project)["changed"]["intensity"], 0.9)

    def test_property_plan_rejects_invalid_declarations_and_missing_editable_controls(self):
        for changes in [{"value": 1}, {"binding": "code()"}, {"type": "script"}, {"order": float("nan")}]:
            with self.subTest(changes=changes):
                project = self.property_project()
                project["general"]["properties"]["enabled"].update(changes)
                with self.assertRaises(RuntimeError):
                    SMOKE.scene_property_plan(project)

    def test_all_controls_plan_changes_every_value_and_preserves_author_bounds(self):
        plan = SMOKE.scene_property_plan(self.property_project(), all_controls=True)
        self.assertEqual(plan["changed"].keys(), plan["defaults"].keys())
        for name in plan["defaults"]:
            self.assertNotEqual(plan["changed"][name], plan["defaults"][name])
        self.assertEqual(plan["changed"]["intensity"], 0.1)
        self.assertEqual(plan["changed"]["tint"], [0.2, 0.4, 0.6])
        for changes in [{"max": 0}, {"step": 0}, {"value": float("inf")}, {"min": 2}]:
            with self.subTest(changes=changes):
                project = self.property_project()
                project["general"]["properties"]["intensity"].update(changes)
                with self.assertRaises(RuntimeError):
                    SMOKE.scene_property_plan(project)

    def test_property_values_require_exact_keys_and_boolean_types_with_rgb_float_tolerance(self):
        self.assertFalse(SMOKE.property_values_match({"enabled": 1}, {"enabled": True}))
        self.assertFalse(SMOKE.property_values_match({"enabled": True, "extra": 1}, {"enabled": True}))
        self.assertFalse(SMOKE.property_values_match({}, {"intensity": 0.1}))
        self.assertTrue(SMOKE.property_values_match([0.70000001, 0.5, 0.25], [0.7, 0.5, 0.25]))
        self.assertFalse(SMOKE.property_values_match([0.71, 0.5, 0.25], [0.7, 0.5, 0.25]))

    def test_dom_readback_matches_author_controls_bounds_and_f32_color_preview(self):
        plan = SMOKE.scene_property_plan(self.property_project())
        controls = self.rendered_controls(plan)
        self.assertTrue(SMOKE.property_controls_match(controls, plan, plan["defaults"]))
        for index, field, value in [(1, "checked", False), (1, "label", "wrong author"),
                                    (2, "value", "nan"), (2, "min", "2"), (0, "disabled", True)]:
            with self.subTest(field=field):
                invalid = copy.deepcopy(controls)
                invalid[index][field] = value
                self.assertFalse(SMOKE.property_controls_match(invalid, plan, plan["defaults"]))

    def test_native_number_input_uses_webdriver_clear_and_send_keys_without_synthetic_input(self):
        driver = SMOKE.Driver("http://unused")
        driver.session = "test"
        with patch.object(driver, "request") as request, patch.object(SMOKE.time, "sleep"):
            driver.type_number("native-number", 0.1)
        self.assertEqual(request.call_args_list[-2].args,
                         ("POST", "/session/test/element/native-number/clear", {}))
        self.assertEqual(request.call_args_list[-1].args,
                         ("POST", "/session/test/element/native-number/value", {"text": "0.1", "value": ["0", ".", "1"]}))
        scripts = [call.args[2]["script"] for call in request.call_args_list if call.args[1].endswith("execute/sync")]
        self.assertEqual(scripts, ["arguments[0].scrollIntoView({block:'center', inline:'nearest'})"])

    def test_all_controls_support_combo_without_a_boolean_or_slider(self):
        project = {"general": {"properties": {"mode": {"type": "combo", "value": "on", "text": "Audio", "options": [
            {"label": "On", "value": "on"}, {"label": "Off", "value": "off"}]}}}}
        plan = SMOKE.scene_property_plan(project, all_controls=True)
        self.assertEqual(plan["changed"], {"mode": "off"})
        self.assertIsNone(plan["boolIndex"])
        controls = self.rendered_controls(plan)
        self.assertTrue(SMOKE.property_controls_match(controls, plan, plan["defaults"]))
        controls[0]["options"][0]["value"] = "wrong"
        self.assertFalse(SMOKE.property_controls_match(controls, plan, plan["defaults"]))
        self.assertTrue(SMOKE.property_values_match({"mode": "off"}, plan["changed"]))
        for value in [0, "unknown"]:
            invalid = copy.deepcopy(project)
            invalid["general"]["properties"]["mode"]["value"] = value
            with self.assertRaises(RuntimeError):
                SMOKE.scene_property_plan(invalid, all_controls=True)

    def test_native_combo_uses_webdriver_option_click_and_refuses_missing_options(self):
        driver = SMOKE.Driver("http://unused")
        driver.session = "test"
        with patch.object(driver, "script", return_value={SMOKE.ELEMENT: "selected-option"}), patch.object(driver, "request") as request:
            driver.select_option("native-select", "off")
            self.assertEqual(request.call_args.args, ("POST", "/session/test/element/selected-option/click", {}))
        with patch.object(driver, "script", return_value=None), patch.object(driver, "request") as request:
            with self.assertRaises(RuntimeError):
                driver.select_option("native-select", "off")
            request.assert_not_called()

    def test_session_publication_requires_atomic_replacement_and_retains_unrelated_entries(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "session.toml"
            before_text = '[assignments]\nDP1="scene1"\nDP2="other"\n[scene_properties.DP2.other]\nenabled=true\n'
            after_text = before_text + '[scene_properties.DP1.scene1]\nenabled=false\nintensity=0.1\n'
            path.write_text(before_text)
            before = SMOKE.read_session_snapshot(path)
            with path.open("rb") as old_file:
                replacement = path.with_name("replacement.toml")
                replacement.write_text(after_text)
                os.replace(replacement, path)
                after = SMOKE.read_session_snapshot(path)
                audit = SMOKE.verify_session_replacement(before, after, old_file, "DP1", "scene1")
                self.assertTrue(audit["replacementInode"])
                self.assertNotEqual(audit["beforeSha256"], audit["afterSha256"])
                after["state"]["assignments"]["DP2"] = "changed"
                with self.assertRaisesRegex(RuntimeError, "unrelated"):
                    SMOKE.verify_session_replacement(before, after, old_file, "DP1", "scene1")

    def test_in_place_session_write_is_not_reported_as_atomic_replacement(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "session.toml"
            path.write_text('[assignments]\nDP1="scene1"\n')
            before = SMOKE.read_session_snapshot(path)
            with path.open("rb") as old_file:
                path.write_text('[assignments]\nDP1="scene1"\n[scene_properties.DP1.scene1]\nenabled=false\n')
                with self.assertRaisesRegex(RuntimeError, "replacing"):
                    SMOKE.verify_session_replacement(before, SMOKE.read_session_snapshot(path), old_file, "DP1", "scene1")

    def test_write_denial_restores_private_permissions_even_when_action_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory)
            session = config / "lwe"
            session.mkdir()
            with self.assertRaisesRegex(RuntimeError, "action failed"):
                with SMOKE.denied_isolated_session_writes(config):
                    self.assertEqual(stat.S_IMODE(session.stat().st_mode), 0o500)
                    raise RuntimeError("action failed")
            self.assertEqual(stat.S_IMODE(session.stat().st_mode), 0o700)
            session.rmdir()
            target = config / "unrelated"
            target.mkdir(mode=0o755)
            session.symlink_to(target, target_is_directory=True)
            with self.assertRaisesRegex(RuntimeError, "isolated"):
                with SMOKE.denied_isolated_session_writes(config):
                    self.fail("Symlinked directory must never be changed")
            self.assertEqual(stat.S_IMODE(target.stat().st_mode), 0o755)

    def test_long_occlusion_toggles_only_owned_id_and_restores_geometry(self):
        window = {"id": 71, "is_focused": True, "layout": {"window_size": [700, 900]}}
        clock = [0.0]
        def action(*args):
            window["layout"]["window_size"] = [1600, 900] if window["layout"]["window_size"] == [700, 900] else [700, 900]
        def sleep(seconds):
            clock[0] += seconds
        with patch.object(SMOKE, "niri_action", side_effect=action) as called, \
                patch.object(SMOKE.time, "monotonic", side_effect=lambda: clock[0]), \
                patch.object(SMOKE.time, "sleep", side_effect=sleep):
            result = SMOKE.fullscreen_owned_cover(lambda: copy.deepcopy(window), [1600, 900])
        self.assertEqual(called.call_args_list[0].args, ("fullscreen-window", "--id", 71))
        self.assertEqual(called.call_args_list[1].args, ("fullscreen-window", "--id", 71))
        self.assertGreaterEqual(result["coveredSeconds"], 12)
        self.assertEqual(window["layout"]["window_size"], [700, 900])
        self.assertFalse(result["systemSuspendOrLock"])
        self.assertFalse(result["frameCallbackPauseDirectlyMeasured"])

    def test_occlusion_focus_interruption_restores_only_the_owned_window(self):
        window = {"id": 71, "is_focused": True, "layout": {"window_size": [700, 900]}}
        def action(*args):
            window["layout"]["window_size"] = [1600, 900] if window["layout"]["window_size"] == [700, 900] else [700, 900]
        def interrupted(seconds):
            window["is_focused"] = False
        with patch.object(SMOKE, "niri_action", side_effect=action) as called, patch.object(SMOKE.time, "sleep", side_effect=interrupted):
            with self.assertRaisesRegex(RuntimeError, "interrupted"):
                SMOKE.fullscreen_owned_cover(lambda: copy.deepcopy(window), [1600, 900])
        self.assertEqual(called.call_count, 2)
        self.assertTrue(all(call.args[-1] == 71 for call in called.call_args_list))
        self.assertEqual(window["layout"]["window_size"], [700, 900])
    def test_library_identity_accepts_equivalent_steam_alias_only(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            steam = home / ".local/share/Steam"
            project = steam / "steamapps/workshop/content/431960/3337606455"
            project.mkdir(parents=True)
            (home / ".steam").mkdir()
            (home / ".steam/steam").symlink_to(steam, target_is_directory=True)
            identities = SMOKE.expected_library_ids(project, {"file": "scene.json"}, home)
            alias_file = home / ".steam/steam/steamapps/workshop/content/431960/3337606455/scene.json"
            self.assertIn(hashlib.sha256(str(alias_file).encode()).hexdigest()[:16], identities)
            wrong = project.parent / "3258032485/scene.json"
            self.assertNotIn(hashlib.sha256(str(wrong).encode()).hexdigest()[:16], identities)

    def test_stop_waits_for_child_after_driver_exits_and_preserves_other_group(self):
        other = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"],
                                 start_new_session=True)
        owned = subprocess.Popen([
            sys.executable, "-c",
            "import subprocess,sys; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); print(p.pid,flush=True)"
        ], start_new_session=True, stdout=subprocess.PIPE, text=True)
        try:
            child_pid = int(owned.stdout.readline())
            owned.wait(timeout=5)
            self.assertIn(child_pid, SMOKE.process_group_members(owned.pid))
            SMOKE.stop_owned(owned)
            self.assertEqual(SMOKE.process_group_members(owned.pid), [])
            self.assertIsNone(other.poll(), "The unrelated process group must remain alive")
        finally:
            SMOKE.stop_owned(owned)
            SMOKE.stop_owned(other)
            owned.stdout.close()

    def test_already_focused_workspace_does_not_toggle_back(self):
        workspaces = [{"id": 1, "idx": 1, "name": None, "is_focused": True}]
        with patch.object(SMOKE, "niri_query", return_value=workspaces), patch.object(SMOKE, "niri_action") as action:
            SMOKE.focus_workspace_id(1)
            action.assert_not_called()

    def test_focus_uses_owned_name_once_then_confirms_identity(self):
        workspaces = [{"id": 2, "idx": 2, "name": "owned-scene", "is_focused": False}]
        def focus(*args):
            workspaces[0]["is_focused"] = True
        with patch.object(SMOKE, "niri_query", return_value=workspaces), patch.object(SMOKE, "niri_action", side_effect=focus) as action:
            SMOKE.focus_workspace_id(2)
            action.assert_called_once_with("focus-workspace", "owned-scene")

    def test_opt_in_guard_prevents_all_desktop_operations(self):
        import os
        env = os.environ.copy()
        env.pop("LWE_REAL_DESKTOP_TESTS", None)
        result = subprocess.run([
            sys.executable, str(Path(SMOKE.__file__)), "--binary", "/missing", "--driver", "/missing",
            "--project", "/missing", "--output", "/missing", "--scene-properties",
            "--scene-properties-write-failure", "--long-occlusion"
        ], env=env, capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 2)
        self.assertIn("Set LWE_REAL_DESKTOP_TESTS=1", result.stderr)


if __name__ == "__main__":
    unittest.main()
