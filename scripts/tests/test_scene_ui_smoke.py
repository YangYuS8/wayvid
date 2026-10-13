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
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import Mock, patch

SPEC = importlib.util.spec_from_file_location(
    "scene_ui_smoke", Path(__file__).resolve().parents[1] / "scene-ui-smoke.py")
SMOKE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SMOKE)


def accessible_node(name="", role=None, pid=None, children=(), sensitive=True):
    node = Mock()
    node.get_name.return_value = name
    node.get_role.return_value = role
    node.get_process_id.return_value = pid
    node.get_child_count.return_value = len(children)
    node.get_child_at_index.side_effect = children.__getitem__
    node.get_state_set.return_value.contains.return_value = sensitive
    node.get_action_iface.return_value.do_action.return_value = True
    return node


def fake_atspi_modules(desktop):
    gi = ModuleType("gi")
    gi.require_version = Mock()
    repository = ModuleType("gi.repository")
    repository.Atspi = SimpleNamespace(
        Role=SimpleNamespace(FILE_CHOOSER="file-chooser", PUSH_BUTTON="push-button"),
        StateType=SimpleNamespace(SENSITIVE="sensitive"),
        get_desktop=Mock(return_value=desktop),
    )
    gi.repository = repository
    return {"gi": gi, "gi.repository": repository}


class SceneUiSmokeTests(unittest.TestCase):
    def test_all_control_acceptance_can_keep_active_author_effects(self):
        project = self.property_project()
        project["general"]["properties"]["mode"] = {
            "type": "combo", "value": "on", "options": [
                {"label": "On", "value": "on"}, {"label": "Off", "value": "off"}]}
        plan = SMOKE.scene_property_plan(project, True, ["enabled", "mode"])
        self.assertTrue(plan["changed"]["enabled"])
        self.assertEqual(plan["changed"]["mode"], "on")
        self.assertNotEqual(plan["changed"]["intensity"], plan["defaults"]["intensity"])
        with self.assertRaisesRegex(RuntimeError, "no declared"):
            SMOKE.scene_property_plan(project, True, ["missing"])
        with self.assertRaisesRegex(RuntimeError, "all-control"):
            SMOKE.scene_property_plan(project, False, ["enabled"])

    def test_native_picker_retry_recreates_only_owned_copy_and_rejects_changed_fixture(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original, selected = root / "original.png", root / "owned-selected.png"
            original.write_bytes(b"stable image fixture")
            token = SMOKE.private_image_copy(original, selected)
            selected.unlink()
            self.assertEqual(SMOKE.private_image_copy(original, selected, token), token)
            selected.unlink()
            self.assertEqual(original.read_bytes(), b"stable image fixture")
            original.write_bytes(b"changed image fixture")
            with self.assertRaisesRegex(RuntimeError, "changed before retry"):
                SMOKE.private_image_copy(original, selected, token)
            self.assertFalse(selected.exists())
            self.assertEqual(original.read_bytes(), b"changed image fixture")

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
            actual = {"label": expected["label"], "type": {"bool": "checkbox", "slider": "number", "color": "color", "combo": "select-one", "texture": "texture"}[kind],
                      "disabled": False, "checked": value if kind == "bool" else False,
                      "value": str(value), "min": "", "max": "", "step": ""}
            if kind == "color":
                actual["value"] = "#b28040"  # f32 0.7 rounds below the exact decimal midpoint.
            if kind == "slider":
                actual.update({field: str(expected[field]) if expected[field] is not None else
                               ("any" if field == "step" else "") for field in ("min", "max", "step")})
            if kind == "combo":
                actual["options"] = copy.deepcopy(expected["options"])
            if kind == "texture":
                actual["selected"] = value is not None
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

    def test_author_groups_are_not_saved_controls_and_image_default_is_absent(self):
        project = self.property_project()
        project["general"]["properties"].update({
            "header": {"type": "group", "text": "Images", "value": "", "order": 12},
            "background": {"type": "scenetexture", "text": "Author background", "value": "", "order": 13}
        })
        plan = SMOKE.scene_property_plan(project, all_controls=True)
        self.assertNotIn("header", plan["defaults"])
        self.assertIsNone(plan["defaults"]["background"])
        self.assertEqual(plan["controls"][-1]["kind"], "texture")
        controls = self.rendered_controls(plan)
        self.assertTrue(SMOKE.property_controls_match(controls, plan, plan["defaults"]))
        controls[-1]["selected"] = True
        self.assertFalse(SMOKE.property_controls_match(controls, plan, plan["defaults"]))
        self.assertTrue(SMOKE.property_values_match(None, None))
        project["general"]["properties"]["background"]["value"] = "file:///other.png"
        with self.assertRaises(RuntimeError):
            SMOKE.scene_property_plan(project, all_controls=True)

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

    def test_native_image_keys_require_a_focused_owned_dialog(self):
        window = {"id": 11, "pid": 42, "is_focused": True}
        with patch.object(SMOKE.shutil, "which", return_value="/owned/wtype"), \
                patch.object(SMOKE, "niri_query", return_value=[window]), \
                patch.object(SMOKE.os, "getpgid", return_value=99), \
                patch.object(SMOKE.subprocess, "run") as run, patch.object(SMOKE.time, "sleep"):
            selected = SMOKE.native_image_dialog_input(10, 99, Path("/owned/image.png"))
            self.assertFalse(selected["cancelled"])
            self.assertEqual(run.call_args_list[0].args[0], ["/owned/wtype", "-s", "500", "-M", "ctrl", "-k", "l", "-m", "ctrl", "-s", "100", "/owned/image.png"])
            self.assertEqual(run.call_args_list[1].args[0], ["/owned/wtype", "-s", "500", "-k", "Return"])
            SMOKE.native_image_dialog_input(10, 99)
            self.assertEqual(run.call_args.args[0], ["/owned/wtype", "-s", "500", "-k", "Escape"])
        with patch.object(SMOKE.shutil, "which", return_value="/owned/wtype"), \
                patch.object(SMOKE, "wait_for", side_effect=RuntimeError("no owned dialog")), \
                patch.object(SMOKE.subprocess, "run") as run:
            with self.assertRaises(RuntimeError):
                SMOKE.native_image_dialog_input(10, 99, Path("/owned/image.png"))
            run.assert_not_called()

    def test_accessible_cancel_activates_the_owned_gtk_button_without_keyboard_input(self):
        cancel = accessible_node("Cancel", "push-button")
        chooser = accessible_node("Open File", "file-chooser", children=[cancel])
        owned = accessible_node(pid=42, children=[chooser])
        foreign = accessible_node(pid=43)
        modules = fake_atspi_modules(accessible_node(children=[foreign, owned]))
        window = {"id": 11, "pid": 42, "title": "Open File", "is_focused": True}
        with patch.dict(sys.modules, modules), \
                patch.object(SMOKE.shutil, "which", return_value="/owned/wtype"), \
                patch.object(SMOKE, "niri_query", return_value=[window]), \
                patch.object(SMOKE.os, "getpgid", return_value=99), \
                patch.object(SMOKE.subprocess, "run") as run:
            result = SMOKE.native_image_dialog_input(10, 99, accessibility=True)
        modules["gi"].require_version.assert_called_once_with("Atspi", "2.0")
        cancel.get_state_set.return_value.contains.assert_called_once_with("sensitive")
        cancel.get_action_iface.return_value.do_action.assert_called_once_with(0)
        foreign.get_child_count.assert_not_called()
        run.assert_not_called()
        self.assertEqual(result, {"windowId": 11, "cancelled": True,
                                  "activation": "GTK accessibility button"})

    def test_accessible_dialog_identity_and_button_state_fail_closed(self):
        for failure in ("foreign PID", "ambiguous chooser", "wrong title", "ambiguous button", "insensitive button"):
            with self.subTest(failure=failure):
                cancel = accessible_node("Cancel", "push-button", sensitive=failure != "insensitive button")
                other_cancel = accessible_node("Cancel", "push-button")
                children = [cancel, other_cancel] if failure == "ambiguous button" else [cancel]
                chooser = accessible_node("Other" if failure == "wrong title" else "Open File",
                                          "file-chooser", children=children)
                choosers = [chooser, chooser] if failure == "ambiguous chooser" else [chooser]
                app = accessible_node(pid=43 if failure == "foreign PID" else 42, children=choosers)
                modules = fake_atspi_modules(accessible_node(children=[app]))
                window = {"id": 11, "pid": 42, "title": "Open File", "is_focused": True}
                with patch.dict(sys.modules, modules), \
                        patch.object(SMOKE.shutil, "which", return_value="/owned/wtype"), \
                        patch.object(SMOKE, "niri_query", return_value=[window]), \
                        patch.object(SMOKE.os, "getpgid", return_value=99), \
                        patch.object(SMOKE.subprocess, "run") as run:
                    with self.assertRaisesRegex(RuntimeError, "Owned GTK"):
                        SMOKE.native_image_dialog_input(10, 99, accessibility=True)
                cancel.get_action_iface.return_value.do_action.assert_not_called()
                other_cancel.get_action_iface.return_value.do_action.assert_not_called()
                run.assert_not_called()

    def test_accessible_cancel_rechecks_focus_after_tree_lookup_before_action(self):
        cancel = accessible_node("Cancel", "push-button")
        chooser = accessible_node("Open File", "file-chooser", children=[cancel])
        desktop = accessible_node(children=[accessible_node(pid=42, children=[chooser])])
        modules = fake_atspi_modules(desktop)
        window = {"id": 11, "pid": 42, "title": "Open File", "is_focused": True}

        def read_desktop(_):
            window["is_focused"] = False
            return desktop

        modules["gi.repository"].Atspi.get_desktop.side_effect = read_desktop
        with patch.dict(sys.modules, modules), \
                patch.object(SMOKE.shutil, "which", return_value="/owned/wtype"), \
                patch.object(SMOKE, "niri_query", return_value=[window]), \
                patch.object(SMOKE.os, "getpgid", return_value=99), \
                patch.object(SMOKE.subprocess, "run") as run:
            with self.assertRaisesRegex(RuntimeError, "button identity"):
                SMOKE.native_image_dialog_input(10, 99, accessibility=True)
        cancel.get_action_iface.return_value.do_action.assert_not_called()
        run.assert_not_called()

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
