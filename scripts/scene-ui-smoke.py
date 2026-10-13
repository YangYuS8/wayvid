#!/usr/bin/env python3
"""Opt-in real Tauri/Wayland Scene acceptance using native WebDriver.

Uses rendered DOM actions and real IPC; never injects IPC mocks or business
state. HOME is retained and XDG_CONFIG_HOME is isolated. Screenshot content
requires independent visual review; automatic success verifies the UI contract.
"""

import argparse
import base64
import copy
from contextlib import contextmanager
import hashlib
import http.client
import json
import math
import os
from pathlib import Path
import shutil
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.error
import urllib.request

ELEMENT = "element-6066-11e4-a52e-4f735466cecf"
BLOCKED_REASONS = {
    "unsupported_properties": "Unsupported or invalid user property bindings",
    "unsupported_hdr": "HDR or global bloom",
    "unsupported_particles": "Particle objects",
}
SCENE_UNSUPPORTED = "This Scene wallpaper cannot play with the current resources and features."


def progress(message):
    print(message, file=sys.stderr, flush=True)


def free_ports():
    with socket.socket() as first, socket.socket() as second:
        first.bind(("127.0.0.1", 0))
        second.bind(("127.0.0.1", 0))
        return first.getsockname()[1], second.getsockname()[1]


def wait_for(description, check, timeout=60):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        try:
            result = check()
            if result:
                return result
        except (urllib.error.URLError, http.client.HTTPException, RuntimeError) as error:
            last = error
        time.sleep(0.2)
    raise RuntimeError(f"Timed out: {description}; last error: {last}")


def niri_query(kind):
    return json.loads(subprocess.check_output(["niri", "msg", "-j", kind], timeout=10))


def niri_action(*args):
    subprocess.run(["niri", "msg", "action", *map(str, args)], check=True, timeout=10,
                   stdout=subprocess.DEVNULL)



def focus_workspace_id(workspace_id):
    # niri's workspace-auto-back-and-forth toggles away when asked to focus
    # the already focused workspace. Identity checks make this idempotent.
    workspace = next((w for w in niri_query("workspaces") if w["id"] == workspace_id), None)
    if workspace is None:
        raise RuntimeError("Requested workspace disappeared")
    if not workspace["is_focused"]:
        niri_action("focus-workspace", workspace["name"] or workspace["idx"])
    wait_for("workspace focus arrived", lambda: any(w["id"] == workspace_id and w["is_focused"] for w in niri_query("workspaces")), 5)


def expected_library_ids(project_dir, project, home):
    """Match the existing source-path hash, retaining equivalent Steam aliases."""
    canonical = project_dir.resolve()
    directories = {project_dir.absolute(), canonical}
    steam_roots = [home / ".local/share/Steam", home / ".steam/steam", home / ".steam/root"]
    for root in steam_roots:
        try:
            relative = canonical.relative_to(root.resolve())
        except ValueError:
            continue
        candidate = root / relative
        if candidate.resolve() == canonical:
            directories.add(candidate)
    primary = project.get("file")
    return {hashlib.sha256(str(directory / primary if primary else directory).encode()).hexdigest()[:16]
            for directory in directories}


def scene_property_plan(project, all_controls=False):
    """Use author declarations and order, never application or IPC internals."""
    declarations = project.get("general", {}).get("properties", {})
    if not isinstance(declarations, dict) or len(declarations) > 256:
        raise RuntimeError("Scene property acceptance requires bounded author declarations")
    controls = []
    for name, declaration in declarations.items():
        if not isinstance(declaration, dict) or not name or any(
                field.lower() in {"binding", "expr", "expression", "user"} for field in declaration):
            raise RuntimeError("Scene property acceptance requires literal author controls")
        kind, value = declaration.get("type"), declaration.get("value")
        label, order = declaration.get("text", name), declaration.get("order", 0)
        if not isinstance(label, str) or not finite_number(order):
            raise RuntimeError("Invalid author property label or order")
        control = {"name": name, "label": label, "kind": kind, "value": value, "order": order}
        if kind == "bool":
            if type(value) is not bool:
                raise RuntimeError("Author boolean property requires a boolean default")
        elif kind == "slider":
            minimum, maximum, step = (declaration.get(field) for field in ("min", "max", "step"))
            if not finite_number(value) or any(number is not None and not finite_number(number)
                    for number in (minimum, maximum, step)) or (step is not None and step <= 0) or \
                    (minimum is not None and value < minimum) or (maximum is not None and value > maximum):
                raise RuntimeError("Invalid author slider value or bounds")
            control.update(min=minimum, max=maximum, step=step)
        elif kind == "color":
            try:
                rgb = [float(channel) for channel in value.split()] if isinstance(value, str) else []
            except ValueError as error:
                raise RuntimeError("Invalid author RGB default") from error
            if len(rgb) != 3 or any(not finite_number(channel) or not 0 <= channel <= 1 for channel in rgb):
                raise RuntimeError("Invalid author RGB default")
            # The runtime exposes normalized f32 RGB, and the native color
            # input rounds those values to an 8-bit preview without editing.
            control["value"] = [struct.unpack("<f", struct.pack("<f", channel))[0] for channel in rgb]
        elif kind == "combo":
            options = declaration.get("options")
            if not isinstance(value, str) or not isinstance(options, list) or not 1 <= len(options) <= 64:
                raise RuntimeError("Invalid author combo declaration")
            if any(not isinstance(option, dict) or not isinstance(option.get("label"), str)
                   or not isinstance(option.get("value"), str) or not option["value"]
                   or len(option["value"].encode()) > 256 for option in options):
                raise RuntimeError("Invalid author combo option")
            values = [option["value"] for option in options]
            if len(set(values)) != len(values) or value not in values:
                raise RuntimeError("Invalid author combo default or duplicate option")
            control["options"] = options
        else:
            raise RuntimeError("Scene property acceptance requires supported author controls")
        controls.append(control)
    controls.sort(key=lambda control: (control["order"], control["name"]))
    boolean = next((control for control in controls if control["kind"] == "bool"), None)
    slider = next((control for control in controls if control["kind"] == "slider" and
                   changed_slider_value(control) is not None), None)
    if not all_controls and (boolean is None or slider is None):
        raise RuntimeError("--scene-properties requires an author boolean and a changeable slider")
    defaults = {control["name"]: control["value"] for control in controls}
    changed = copy.deepcopy(defaults)
    if boolean is not None:
        changed[boolean["name"]] = not boolean["value"]
    if slider is not None:
        changed[slider["name"]] = changed_slider_value(slider)
    if all_controls:
        for control in controls:
            if control["kind"] == "bool":
                changed[control["name"]] = not control["value"]
            elif control["kind"] == "slider":
                replacement = changed_slider_value(control)
                if replacement is None:
                    raise RuntimeError("Every slider must be changeable for all-control acceptance")
                changed[control["name"]] = replacement
            elif control["kind"] == "combo":
                alternate = next((option["value"] for option in control["options"] if option["value"] != control["value"]), None)
                if alternate is None:
                    raise RuntimeError("Every combo must be changeable for all-control acceptance")
                changed[control["name"]] = alternate
            else:
                changed[control["name"]] = [channel / 255 for channel in (51, 102, 153)]
                if property_values_match(changed[control["name"]], control["value"]):
                    changed[control["name"]] = [channel / 255 for channel in (153, 102, 51)]
    return {"controls": controls, "defaults": defaults, "changed": changed,
            "boolIndex": controls.index(boolean) if boolean is not None else None,
            "sliderIndex": controls.index(slider) if slider is not None else None}


def finite_number(value):
    return type(value) in (int, float) and math.isfinite(value)


def changed_slider_value(control):
    value, minimum, maximum = (control[field] for field in ("value", "min", "max"))
    step = control["step"] or (max((maximum - minimum) / 10, 1e-12)
                              if minimum is not None and maximum is not None else 1)
    candidates = [value + step, value - step, minimum, maximum]
    for candidate in candidates:
        if finite_number(candidate) and candidate != value and \
                (minimum is None or candidate >= minimum) and (maximum is None or candidate <= maximum):
            return candidate
    return None


def property_values_match(actual, expected):
    if isinstance(expected, dict):
        return isinstance(actual, dict) and actual.keys() == expected.keys() and all(
            property_values_match(actual[name], value) for name, value in expected.items())
    if type(expected) is bool:
        return type(actual) is bool and actual == expected
    if isinstance(expected, str):
        return isinstance(actual, str) and actual == expected
    if isinstance(expected, list):
        return isinstance(actual, list) and len(actual) == len(expected) and all(
            finite_number(a) and math.isclose(a, b, rel_tol=1e-6, abs_tol=1e-7)
            for a, b in zip(actual, expected))
    return finite_number(actual) and math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-12)


def read_session_snapshot(path):
    with path.open("rb") as file:
        identity = os.fstat(file.fileno())
        contents = file.read()
    return {"state": tomllib.loads(contents.decode()), "contents": contents,
            "identity": (identity.st_dev, identity.st_ino)}


def unrelated_session_state(state, monitor_id, item_id):
    result = copy.deepcopy(state)
    assignments = result.setdefault("assignments", {})
    assignments.pop(monitor_id, None)
    properties = result.setdefault("scene_properties", {})
    items = properties.get(monitor_id, {})
    items.pop(item_id, None)
    if not items:
        properties.pop(monitor_id, None)
    return result


def verify_session_replacement(before, after, old_file, monitor_id, item_id):
    """Evidence of complete replacement; not a claim about every transient read."""
    old_file.seek(0)
    if before["identity"] == after["identity"] or old_file.read() != before["contents"]:
        raise RuntimeError("Scene choices were not published by replacing the previous session inode")
    if unrelated_session_state(before["state"], monitor_id, item_id) != \
            unrelated_session_state(after["state"], monitor_id, item_id):
        raise RuntimeError("Scene property publication changed unrelated session entries")
    if after["state"].get("assignments", {}).get(monitor_id) != item_id:
        raise RuntimeError("Scene property publication did not retain the selected assignment")
    return {"replacementInode": True, "oldOpenFileUnchanged": True, "unrelatedEntriesPreserved": True,
            "beforeSha256": hashlib.sha256(before["contents"]).hexdigest(),
            "afterSha256": hashlib.sha256(after["contents"]).hexdigest()}


@contextmanager
def denied_isolated_session_writes(config):
    directory = config / "lwe"
    if directory.is_symlink() or directory.resolve() != config.resolve() / "lwe":
        raise RuntimeError("Write failure probe requires the owned isolated session directory")
    directory.chmod(0o500)
    try:
        yield
    finally:
        directory.chmod(0o700)


def property_controls_match(controls, plan, values):
    if len(controls) != len(plan["controls"]):
        return False
    for actual, expected in zip(controls, plan["controls"]):
        kind, value = expected["kind"], values[expected["name"]]
        if actual["label"] != expected["label"] or actual["type"] != {
                "bool": "checkbox", "slider": "number", "color": "color", "combo": "select-one"}[kind] or actual["disabled"]:
            return False
        if kind == "bool" and actual["checked"] is not value:
            return False
        if kind == "combo" and (actual["value"] != value or actual.get("options") != expected["options"]):
            return False
        if kind == "slider":
            try:
                number = float(actual["value"])
            except ValueError:
                return False
            if not property_values_match(number, value):
                return False
            for field in ("min", "max", "step"):
                declared = expected[field]
                rendered = actual[field]
                if declared is None:
                    if rendered != ("any" if field == "step" else ""):
                        return False
                else:
                    try:
                        if not property_values_match(float(rendered), declared):
                            return False
                    except ValueError:
                        return False
        if kind == "color" and actual["value"].lower() != "#" + "".join(
                f"{math.floor(channel * 255 + 0.5):02x}" for channel in value):
            return False
    return True


def fullscreen_owned_cover(owned_window, output_size, duration=12, capture=None):
    """Toggle only the positively identified owned window, with a bounded cover."""
    before = owned_window()
    window_id, original_size = before["id"], list(before["layout"]["window_size"])
    if not before["is_focused"]:
        raise RuntimeError("Only the focused owned window can begin fullscreen coverage")

    def covered():
        current = owned_window()
        size = current["layout"]["window_size"]
        return current if current["id"] == window_id and current["is_focused"] and all(
            actual >= expected - 1 for actual, expected in zip(size, output_size)) else None

    niri_action("fullscreen-window", "--id", window_id)
    try:
        fullscreen = wait_for("owned fullscreen covers the output", covered, 5)
        if capture:
            capture()
        started = time.monotonic()
        checks = 0
        while time.monotonic() - started < duration:
            if not covered():
                raise RuntimeError("Owned fullscreen coverage or focus was interrupted")
            checks += 1
            time.sleep(min(1, max(0, duration - (time.monotonic() - started))))
        elapsed = time.monotonic() - started
    finally:
        if owned_window()["id"] != window_id:
            raise RuntimeError("Owned fullscreen identity changed; no unrelated window was toggled")
        niri_action("fullscreen-window", "--id", window_id)
    wait_for("owned window left fullscreen", lambda: all(abs(a - b) <= 1 for a, b in zip(
        owned_window()["layout"]["window_size"], original_size)), 5)
    return {"windowId": window_id, "coveredSeconds": elapsed, "coverageSamples": checks,
            "outputLogicalSize": output_size, "coveredWindowSize": fullscreen["layout"]["window_size"],
            "scope": "owned Tauri fullscreen coverage", "systemSuspendOrLock": False,
            "frameCallbackPauseDirectlyMeasured": False}


def process_group_members(group):
    """Only live processes in the newly created group's matching session."""
    members = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdecimal():
            continue
        try:
            # comm may contain spaces or parentheses; fields after its final ')' are stable.
            fields = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            if fields[0] != "Z" and int(fields[2]) == group and int(fields[3]) == group:
                members.append(int(entry.name))
        except (OSError, ValueError, IndexError):
            continue
    return members


def stop_owned(process):
    if process is None:
        return
    group = process.pid  # Popen(start_new_session=True), never an existing desktop group.
    for sig, timeout in [(signal.SIGTERM, 10), (signal.SIGKILL, 5)]:
        if process_group_members(group):
            try:
                os.killpg(group, sig)
            except ProcessLookupError:
                pass
        deadline = time.monotonic() + timeout
        while process_group_members(group) and time.monotonic() < deadline:
            process.poll()  # Reap our driver without treating its exit as the group's exit.
            time.sleep(0.1)
        if not process_group_members(group):
            process.wait(timeout=2)
            return
    raise RuntimeError(f"Owned process group {group} did not terminate")


class Driver:
    def __init__(self, endpoint):
        self.endpoint = endpoint
        self.session = None
        self.before_click = None

    def request(self, method, path, payload=None):
        request = urllib.request.Request(
            self.endpoint + path, data=None if payload is None else json.dumps(payload).encode(),
            headers={"Content-Type": "application/json"}, method=method)
        try:
            with urllib.request.build_opener(urllib.request.ProxyHandler({})).open(request, timeout=15) as response:
                value = json.load(response)["value"]
        except urllib.error.HTTPError as error:
            detail = json.load(error).get("value", {})
            raise RuntimeError(f"WebDriver {method} {path}: {detail.get('error')}") from error
        if isinstance(value, dict) and "error" in value:
            raise RuntimeError(f"WebDriver {method} {path}: {value['error']}")
        return value

    def open(self, binary):
        value = self.request("POST", "/session", {"capabilities": {"alwaysMatch": {
            "tauri:options": {"application": str(binary)}}}})
        self.session = value["sessionId"]

    def script(self, script, *args):
        return self.request("POST", f"/session/{self.session}/execute/sync", {
            "script": script, "args": list(args)})

    def find(self, selector):
        return self.request("POST", f"/session/{self.session}/element", {
            "using": "css selector", "value": selector})[ELEMENT]

    def click(self, selector):
        for attempt in range(3):
            try:
                self.click_element(self.find(selector))
                return
            except RuntimeError as error:
                if 'stale element reference' not in str(error) or attempt == 2:
                    raise
                time.sleep(0.2)

    def click_element(self, element, activate=True):
        if activate and self.before_click:
            self.before_click()
        self.script("arguments[0].scrollIntoView({block:'center', inline:'nearest'})", {ELEMENT: element})
        time.sleep(0.15)
        self.request("POST", f"/session/{self.session}/element/{element}/click", {})

    def screenshot(self, path):
        data = self.request("GET", f"/session/{self.session}/screenshot")
        path.write_bytes(base64.b64decode(data))

    def type_number(self, element, number):
        self.click_element(element)
        self.request("POST", f"/session/{self.session}/element/{element}/clear", {})
        text = str(number)
        self.request("POST", f"/session/{self.session}/element/{element}/value", {
            "text": text, "value": list(text)})

    def select_option(self, element, value):
        if self.before_click:
            self.before_click()
        option = self.script("""const select=arguments[0];if(select.tagName!=='SELECT'||select.disabled)throw Error('Unavailable combo');
            return [...select.options].find(option=>option.value===arguments[1]);""", {ELEMENT: element}, value)
        if not option:
            raise RuntimeError("Author combo option is unavailable")
        # WebDriver's option click follows the browser's select interaction.
        self.request("POST", f"/session/{self.session}/element/{option[ELEMENT]}/click", {})

    def diagnostics(self, path):
        # DOM controls and geometry only: no asset/script content, env or IPC payloads.
        data = self.script("""return {
          url: location.pathname, active: document.activeElement?.outerHTML?.slice(0, 700),
          controls: [...document.querySelectorAll('button, [role=option], input[name=libraryMonitor], [data-detail-section=scene-properties] input, [data-detail-section=scene-properties] select')].map(e => {
            const r=e.getBoundingClientRect(); return {tag:e.tagName, text:e.textContent.trim().slice(0,160),
            role:e.getAttribute('role'), slot:e.dataset.slot, aria:e.getAttribute('aria-label'),
            value:e.value, disabled:e.disabled, hidden:e.getAttribute('aria-hidden'),
            expanded:e.getAttribute('aria-expanded'), rect:{x:r.x,y:r.y,width:r.width,height:r.height}};
          })};""")
        path.write_text(json.dumps(data, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--driver", type=Path, required=True)
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument("--blocked-project", type=Path)
    parser.add_argument("--blocked-reason", choices=BLOCKED_REASONS, default="unsupported_hdr")
    parser.add_argument("--playback-settings", action="store_true",
                        help="Save 15 FPS/battery preferences, then change the running Scene to 30 FPS and verify restart persistence")
    parser.add_argument("--scene-properties", action="store_true",
                        help="Edit author bool/slider using native controls, verify atomic session/restart readback, and reset full defaults")
    parser.add_argument("--all-scene-properties", action="store_true",
                        help="Edit every author control; color uses the rendered input's real input event, without IPC injection")
    parser.add_argument("--scene-properties-write-failure", action="store_true",
                        help="Also deny writes only in isolated config/lwe, verify real Save rollback and retained draft, then retry the same choices")
    parser.add_argument("--long-occlusion", action="store_true",
                        help="Cover the output with only the owned Tauri fullscreen window for at least 12 seconds, then verify recovery")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.scene_properties = args.scene_properties or args.all_scene_properties
    if os.environ.get("LWE_REAL_DESKTOP_TESTS") != "1":
        parser.error("Set LWE_REAL_DESKTOP_TESTS=1: this applies and clears real wallpapers")
    if args.scene_properties_write_failure and (not args.scene_properties or os.geteuid() == 0):
        parser.error("--scene-properties-write-failure requires --scene-properties and a non-root user")
    if not os.environ.get("WAYLAND_DISPLAY") or not all(shutil.which(tool) for tool in ["niri", "grim"]):
        parser.error("Requires the verified Wayland + niri desktop and grim")
    outputs = niri_query("outputs")
    if len(outputs) != 1:
        parser.error("This acceptance harness currently requires exactly one monitor")
    for path in [args.binary, args.driver]:
        if not path.is_file() or not os.access(path, os.X_OK):
            parser.error(f"Executable unavailable: {path}")
    args.binary, args.driver = args.binary.resolve(), args.driver.resolve()
    project = json.loads((args.project / "project.json").read_text())
    if not args.project.name.isdecimal() or project["type"].lower() != "scene":
        parser.error("--project must identify a locally acquired Workshop Scene")
    properties_plan = scene_property_plan(project, args.all_scene_properties) if args.scene_properties else None
    output = args.output.resolve()
    output.mkdir(parents=True, mode=0o700, exist_ok=True)
    original_workspaces = niri_query("workspaces")
    focused = next(w for w in original_workspaces if w["is_focused"])
    empty = next((w for w in original_workspaces if w["output"] == focused["output"]
                  and w["active_window_id"] is None and w["name"] is None), None)
    if empty is None:
        parser.error("An unnamed empty workspace on the focused output is required")
    config = Path(tempfile.mkdtemp(prefix="lwe-scene-ui-"))
    env = os.environ.copy()
    env["XDG_CONFIG_HOME"] = str(config)
    report = {"projectId": args.project.name, "originalWorkspaceId": focused["id"], "configIsolated": True, "mockIpc": False,
              "visualReview": "required", "steps": [], "cleanupErrors": [], "passed": False}
    process, driver, log_file = None, None, None
    workspace_name = "lwe-scene-ui-" + config.name
    workspace_named = False

    def owned_window():
        windows = niri_query("windows")
        for window in windows:
            pid = window.get("pid")
            try:
                if pid and process is not None and os.getpgid(pid) == process.pid \
                        and Path(f"/proc/{pid}/exe").resolve() == args.binary:
                    return window
            except ProcessLookupError:
                continue
        raise RuntimeError("Owned Tauri window unavailable")

    def activate():
        window = owned_window()
        # Re-focusing an already focused window can dismiss an open Bits UI menu.
        if not window["is_focused"]:
            niri_action("focus-window", "--id", window["id"])
            time.sleep(0.15)

    def focus_owned():
        current = next(w for w in niri_query("workspaces") if w["name"] == workspace_name)
        focus_workspace_id(current["id"])

    def start():
        nonlocal process, driver
        port, native_port = free_ports()
        process = subprocess.Popen([str(args.driver), "--port", str(port), "--native-port", str(native_port)],
                                   env=env, stdout=log_file, stderr=subprocess.STDOUT, start_new_session=True)
        report.setdefault("ownedGroups", []).append(process.pid)
        driver = Driver(f"http://127.0.0.1:{port}")
        wait_for("native WebDriver startup", lambda: driver.request("GET", "/status"), 15)
        driver.open(args.binary)
        wait_for("real Tauri IPC bridge", lambda: driver.script("return !!window.__TAURI_INTERNALS__"))
        wait_for("hydrated navigation", lambda: driver.script("return document.readyState === 'complete' && !!document.querySelector('a[href=\"/library\"]')"))
        window = wait_for("owned Tauri window", owned_window, 15)
        niri_action("move-window-to-workspace", "--window-id", window["id"], workspace_name)
        focus_owned()
        # The moved tiled column inherits its prior width. Normalize only this
        # owned window's otherwise empty workspace so repeated runs exercise
        # the same desktop layout, regardless of the user's focused column.
        activate()
        niri_action("set-column-width", "100%")
        wait_for("owned desktop window width", lambda: owned_window()["layout"]["window_size"][0] >= outputs[focused["output"]]["logical"]["width"] * 0.9, 5)
        driver.before_click = activate
        progress(f"Started owned group {process.pid}")

    def library(project_dir, metadata):
        title = metadata["title"]
        expected = expected_library_ids(project_dir, metadata, Path(os.environ["HOME"]))
        driver.click('a[href="/library"]')
        wait_for("Library catalog", lambda: driver.script("return document.querySelector('button[aria-label^=\"Select Library item \"]')"))
        seen_pages = set()
        while True:
            labels = driver.script("return [...document.querySelectorAll('button[aria-label^=\"Select Library item \"]')].map(b=>b.getAttribute('aria-label'))")
            signature = tuple(labels)
            if signature in seen_pages:
                raise RuntimeError("Library pagination did not advance")
            seen_pages.add(signature)
            cards = driver.script("return [...document.querySelectorAll('button[aria-label]')].filter(b=>b.getAttribute('aria-label')===arguments[0])", "Select Library item " + title)
            for card in cards:
                driver.click_element(card[ELEMENT])
                wait_for("Scene detail", lambda: driver.script("return document.querySelector('[data-detail-section=header] h2')?.textContent===arguments[0] && document.querySelector('[data-detail-layout][data-item-id]')?.dataset.itemId", title))
                actual = driver.script("return document.querySelector('[data-detail-layout][data-item-id]').dataset.itemId")
                if actual in expected:
                    progress(f"Selected verified Workshop {project_dir.name} / Library {actual}")
                    return actual
            next_button = driver.script("return [...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Next'&&!b.disabled)")
            if not next_button:
                raise RuntimeError("The requested Workshop Scene identity is missing from Library")
            driver.click_element(next_button[ELEMENT])
            wait_for("next Library page", lambda: driver.script("return JSON.stringify([...document.querySelectorAll('button[aria-label^=\"Select Library item \"]')].map(b=>b.getAttribute('aria-label'))) !== arguments[0]", json.dumps(labels, separators=(",", ":"))))

    def saved_assignment():
        path = config / "lwe/session.toml"
        return tomllib.loads(path.read_text()) if path.exists() else {}

    def saved_settings():
        path = config / "lwe/settings.toml"
        return tomllib.loads(path.read_text()) if path.exists() else {}

    def property_controls():
        return driver.script("""return [...document.querySelectorAll('[data-detail-section=scene-properties] fieldset input, [data-detail-section=scene-properties] fieldset select')].map(e=>({
          element:e, type:e.type, label:e.closest('label').querySelector('span').textContent,
          checked:e.checked, value:e.value, disabled:e.disabled, min:e.min, max:e.max, step:e.step,
          options:e.tagName==='SELECT'?[...e.options].map(o=>({label:o.textContent,value:o.value})):null
        }));""")

    def open_properties(expected):
        library(args.project, project)
        select_monitor()
        toggle = wait_for("enabled Scene property entry", lambda: driver.script(
            "return document.querySelector('[data-detail-section=scene-properties] button[aria-label=\"Scene properties\"]:not(:disabled)')"))
        if driver.script("return arguments[0].getAttribute('aria-expanded')", toggle) != "true":
            driver.click_element(toggle[ELEMENT])
        wait_for("native author property values", lambda: property_controls_match(
            property_controls(), properties_plan, expected))
        return property_controls()

    def capture_properties(name):
        driver.script("document.querySelector('[data-detail-section=scene-properties]').scrollIntoView({block:'start'})")
        time.sleep(0.3)
        driver.screenshot(output / name)

    def edit_properties():
        controls = open_properties(properties_plan["defaults"])
        indices = range(len(controls)) if args.all_scene_properties else [properties_plan["boolIndex"], properties_plan["sliderIndex"]]
        for index in indices:
            control = properties_plan["controls"][index]
            element = controls[index]["element"]
            value = properties_plan["changed"][control["name"]]
            if control["kind"] == "bool":
                driver.click_element(element[ELEMENT])
            elif control["kind"] == "slider":
                driver.type_number(element[ELEMENT], value)
            elif control["kind"] == "combo":
                driver.select_option(element[ELEMENT], value)
            else:
                # WebDriver cannot type into WebKit's native color dialog.
                # Drive the actual rendered control and its frontend handler;
                # mutation/publication still use the ordinary real IPC path.
                color = "#" + "".join(f"{round(channel * 255):02x}" for channel in value)
                driver.script("""const e=arguments[0];if(e.type!=='color'||e.disabled)throw Error('Unavailable color control');
                    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(e,arguments[1]);
                    e.dispatchEvent(new Event('input',{bubbles:true}));""", element, color)
        wait_for("edited author control DOM values", lambda: property_controls_match(
            property_controls(), properties_plan, properties_plan["changed"]))

    def verify_failed_property_write():
        path = config / "lwe/session.toml"
        before = read_session_snapshot(path)
        with denied_isolated_session_writes(config):
            control = wait_for("enabled native Save for write failure", lambda: driver.script(
                "return [...document.querySelectorAll('[data-detail-section=scene-properties] button')].find(b=>b.textContent.trim()==='Save and apply'&&!b.disabled)"))
            driver.click_element(control[ELEMENT])
            failure = wait_for("real atomic write failure feedback", lambda: driver.script("""const p=document.querySelector('[data-detail-section=scene-properties]');
              return p&&p.getAttribute('aria-busy')!=='true'&&[...p.querySelectorAll('[role=alert]')].map(e=>e.textContent).find(t=>t.includes('atomically save desktop assignments'));
            """))
        after = read_session_snapshot(path)
        if after["contents"] != before["contents"] or after["identity"] != before["identity"]:
            raise RuntimeError("Failed atomic write changed the previous session bytes or inode")
        wait_for("retained editable draft after real write failure", lambda: property_controls_match(
            property_controls(), properties_plan, properties_plan["changed"]))
        capture_properties("scene-properties-write-failed.png")
        driver.click('a[href="/desktop"]')
        wait_for("previous Scene Running after failed property persistence", lambda: desktop_running(project["title"]))
        driver.script("arguments[0].scrollIntoView({block:'start'})", desktop_card())
        time.sleep(0.2)
        driver.screenshot(output / "desktop-properties-rollback.png")
        capture_wallpaper("wallpaper-properties-rollback.png")
        report["steps"].append("real isolated atomic write failure preserved session bytes/inode and editable draft; Desktop reported previous Scene Running")
        # Navigation deliberately unmounts the panel. Re-enter the same choices
        # after the Desktop review rather than claiming page-persistent drafts.
        edit_properties()
        return {"failure": failure, "sessionBytesUnchanged": True, "sessionInodeUnchanged": True,
                "draftRetainedBeforeNavigation": True, "previousSceneRunning": True,
                "isolatedDirectoryPermissionsRestored": "0700",
                "retryMethod": "same native choices re-entered after Desktop review"}

    def publish_properties(reset=False):
        path = config / "lwe/session.toml"
        monitor_id = report["selectedMonitor"]
        with path.open("rb") as old_file:
            before = read_session_snapshot(path)
            identity = os.fstat(old_file.fileno())
            if (identity.st_dev, identity.st_ino) != before["identity"]:
                raise RuntimeError("Session changed before native property action")
            label = "Reset defaults" if reset else "Save and apply"
            control = wait_for("enabled native property action", lambda: driver.script(
                "return [...document.querySelectorAll('[data-detail-section=scene-properties] button')].find(b=>b.textContent.trim()===arguments[0]&&!b.disabled)", label))
            driver.click_element(control[ELEMENT])

            def completed():
                status = driver.script("""const p=document.querySelector('[data-detail-section=scene-properties]');return {
                  busy:!p||p.getAttribute('aria-busy')==='true',
                  enabled:!!p&&[...p.querySelectorAll('button')].some(b=>b.textContent.trim()==='Save and apply'&&!b.disabled),
                  errors:p?[...p.querySelectorAll('[role=alert]')].map(e=>e.textContent.trim()):[]
                };""")
                if not status["busy"] and status["errors"]:
                    return {"failure": " • ".join(status["errors"])}
                snapshot = read_session_snapshot(path)
                state = snapshot["state"]
                items = state.get("scene_properties", {}).get(monitor_id, {})
                choices_match = library_id not in items if reset else property_values_match(
                    items.get(library_id), properties_plan["changed"])
                if choices_match and state.get("assignments", {}).get(monitor_id) == library_id and \
                        not status["busy"] and status["enabled"]:
                    return snapshot
                return None

            after = wait_for("complete property assignment/choices publication and UI success", completed)
            if "failure" in after:
                raise RuntimeError(f"Native property action failed: {after['failure']}")
            audit = verify_session_replacement(before, after, old_file, monitor_id, library_id)
        expected = properties_plan["defaults"] if reset else properties_plan["changed"]
        wait_for("reloaded native property values", lambda: property_controls_match(
            property_controls(), properties_plan, expected))
        return after, audit

    def verify_preferences(fps):
        wait_for("persisted playback preferences", lambda: saved_settings().get("fps_limit") == fps
                 and saved_settings().get("pause_on_battery") is True)
        wait_for("Settings preferences feedback", lambda: driver.script("return [...document.querySelectorAll('p')].some(e=>e.textContent.trim()===arguments[0]) && [...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='Edit settings'&&!b.disabled)", f"Wallpaper frame rate: {fps} FPS"))
        if driver.script("return !!document.querySelector('[role=alert]')"):
            raise RuntimeError("Playback preferences returned a business/runtime failure")
        driver.script("[...document.querySelectorAll('p')].find(e=>e.textContent.trim()===arguments[0]).scrollIntoView({block:'center'})", f"Wallpaper frame rate: {fps} FPS")
        time.sleep(0.3)  # Capture painted review values after the save completes.

    def set_preferences(fps):
        driver.click('a[href="/settings"]')
        edit = wait_for("Settings edit control", lambda: driver.script("return [...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Edit settings'&&!b.disabled)"))
        driver.click_element(edit[ELEMENT])
        wait_for("FPS selector", lambda: driver.script("return document.querySelector('button[aria-label=\"Wallpaper frame rate\"]')"))
        driver.click('button[aria-label="Wallpaper frame rate"]')
        option = wait_for("requested FPS option", lambda: driver.script("return [...document.querySelectorAll('[role=option]')].find(e=>e.textContent.trim()===arguments[0])", f"{fps} FPS"))
        driver.click_element(option[ELEMENT])
        wait_for("FPS menu closed", lambda: not driver.script("return !!document.querySelector('[role=option]')"))
        if not driver.script("return document.querySelector('input[aria-label=\"Pause on battery power\"]').checked"):
            driver.click('input[aria-label="Pause on battery power"]')
        save = wait_for("enabled preference save", lambda: driver.script("return [...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Save changes'&&!b.disabled)"))
        driver.click_element(save[ELEMENT])
        verify_preferences(fps)
        driver.screenshot(output / f"settings-{fps}-fps.png")
        report["steps"].append(f"native Settings controls saved {fps} FPS and battery pause with successful runtime feedback")

    def desktop_card():
        return driver.script("return [...document.querySelectorAll('[aria-label^=\"Desktop monitor \"]')].find(e=>e.textContent.includes(arguments[0]))", focused["output"])

    def desktop_running(title):
        card = desktop_card()
        return card and driver.script("return arguments[0].textContent.includes(arguments[1])&&[...arguments[0].querySelectorAll('[data-slot=badge]')].some(b=>b.textContent.trim()==='Running')", card, title)

    def capture_background(name, current):
        time.sleep(1)  # Allow wallpaper presentation and workspace animations to settle.
        for attempt in range(3):
            focus_owned()
            time.sleep(1)
            before = next(w for w in niri_query("workspaces") if w["is_focused"])
            if any(w["workspace_id"] == current["id"] for w in niri_query("windows")):
                raise RuntimeError("Capture workspace is obstructed")
            if before["id"] != current["id"]:
                continue
            subprocess.run(["grim", "-o", current["output"], str(output / name)], check=True, timeout=10)
            after = next(w for w in niri_query("workspaces") if w["is_focused"])
            if after["id"] == current["id"]:
                report.setdefault("captures", []).append({"file": name, "workspaceId": current["id"],
                    "output": current["output"], "unobstructed": True, "focusedBeforeAndAfter": True})
                return
        (output / name).unlink(missing_ok=True)
        raise RuntimeError("Capture workspace lost focus; no screenshot accepted")

    def capture_wallpaper(name):
        window = owned_window()
        workspaces = niri_query("workspaces")
        current = next(w for w in workspaces if w["name"] == workspace_name)
        if window["workspace_id"] != current["id"]:
            raise RuntimeError("Owned window left the acceptance workspace")
        temporary = next(w for w in workspaces if w["output"] == current["output"]
                         and w["active_window_id"] is None and w["id"] != current["id"])
        niri_action("move-window-to-workspace", "--window-id", window["id"], "--focus", "false", temporary["idx"])
        try:
            capture_background(name, current)
        finally:
            niri_action("move-window-to-workspace", "--window-id", window["id"], "--focus", "false", workspace_name)
            focus_owned()

    def select_monitor():
        trigger = driver.find('[data-detail-section=actions] button[aria-label="Apply target monitor"]')
        driver.click_element(trigger)
        option = wait_for("monitor option", lambda: driver.script("return document.querySelector('[data-slot=select-item]')"))
        time.sleep(0.2)  # Floating UI places the portal after opening.
        geometry = driver.script("const e=arguments[0],r=e.getBoundingClientRect();return {label:e.textContent.trim(),value:e.dataset.value,x:r.x,y:r.y,width:r.width,height:r.height,viewportHeight:innerHeight,disabled:e.getAttribute('aria-disabled')}", option)
        if geometry["width"] <= 0 or geometry["height"] <= 0 or geometry["disabled"] == "true":
            raise RuntimeError("Monitor option is not rendered and enabled")
        if not geometry["value"] or not geometry["value"].endswith(":" + focused["output"]):
            raise RuntimeError("Monitor option does not identify the actual desktop output")
        if geometry["y"] < 0 or geometry["y"] + geometry["height"] > geometry["viewportHeight"]:
            raise RuntimeError("Monitor option is outside the native pointer viewport")
        # The portal option is already positioned. Native click must work for
        # this acceptance; synthetic events must not hide a product defect.
        driver.request("POST", f"/session/{driver.session}/element/{option[ELEMENT]}/click", {})
        method = "native WebDriver click"
        wait_for("monitor selection completed", lambda: not driver.script("return !!document.querySelector('[data-slot=select-item]')"), 5)
        selected = driver.script("return document.querySelector('input[name=libraryMonitor]')?.value")
        if selected != geometry["value"]:
            raise RuntimeError(f"Selected monitor identity mismatch: {selected!r}")
        report.update(selectedMonitor=selected, selectedOutput=focused["output"],
                      selectedMonitorLabel=geometry["label"], monitorSelectionMethod=method,
                      monitorOptionGeometry=geometry)
        progress(f"Selected monitor {selected}: {method}")

    def cleanup(description, action):
        try:
            action()
        except Exception as error:
            report["cleanupErrors"].append(f"{description}: {error}")
            report["passed"] = False

    try:
        (config / "lwe").mkdir()
        (config / "lwe/settings.toml").write_text('language = "en"\n')
        log_file = (output / "webdriver.log").open("w")
        niri_action("set-workspace-name", "--workspace", empty["idx"], workspace_name)
        workspace_named = True
        focus_owned()
        start()
        if args.playback_settings:
            set_preferences(15)
        library_id = library(args.project, project)
        report["libraryItemId"] = library_id
        wait_for("enabled Scene Apply", lambda: driver.script("return [...document.querySelectorAll('[data-detail-section=actions] button')].some(b=>b.textContent.trim()==='Apply'&&!b.disabled)"))
        select_monitor()
        apply_button = driver.script("return [...document.querySelectorAll('[data-detail-section=actions] button')].find(b=>b.textContent.trim()==='Apply')")
        driver.click_element(apply_button[ELEMENT])
        wait_for("persisted Scene assignment", lambda: saved_assignment().get("assignments", {}).get(report["selectedMonitor"]) == library_id)
        wait_for("completed apply feedback", lambda: driver.script("return ![...document.querySelectorAll('[data-detail-section=actions] button')].some(b=>b.textContent.includes('Applying'))"))
        if driver.script("return !!document.querySelector('[data-detail-section=actions] [role=alert]')"):
            raise RuntimeError("Apply returned a business failure")
        driver.script("document.querySelector('[data-detail-section=actions]').scrollIntoView({block:'start'})")
        time.sleep(0.2)
        driver.screenshot(output / "library-applied.png")
        report["steps"].append("verified Library item and explicit monitor Apply persisted")
        driver.click('a[href="/desktop"]')
        wait_for("Desktop assignment feedback", lambda: desktop_running(project["title"]))
        driver.script("arguments[0].scrollIntoView({block:'start'})", desktop_card())
        time.sleep(0.2)
        driver.screenshot(output / "desktop-playing.png")
        capture_wallpaper("wallpaper-applied.png")
        capture_wallpaper("wallpaper-applied-later.png")
        report["steps"].append("selected Desktop monitor displayed Scene and Running")
        property_publication = None
        if args.scene_properties:
            edit_properties()
            edited_controls = properties_plan["controls"] if args.all_scene_properties else [properties_plan["controls"][properties_plan[index]] for index in ("boolIndex", "sliderIndex")]
            failed_publication = verify_failed_property_write() if args.scene_properties_write_failure else None
            capture_properties("scene-properties-edited.png")
            property_publication, audit = publish_properties()
            report["properties"] = {"inputMethod": "native WebDriver checkbox, number and select-option interactions; rendered color input event" if args.all_scene_properties else "native WebDriver checkbox click and number clear/send keys",
                "changes": [{"name": control["name"], "label": control["label"], "kind": control["kind"],
                    "before": control["value"], "after": properties_plan["changed"][control["name"]]}
                    for control in edited_controls], "savedPublication": audit}
            if failed_publication:
                report["properties"]["failedPublication"] = failed_publication
                report["steps"].append("native Save retried the same choices successfully after isolated directory permissions were restored")
            capture_properties("scene-properties-applied.png")
            report["steps"].append("native author control Save and apply published complete choices and assignment by inode replacement")
            driver.click('a[href="/desktop"]')
            wait_for("Scene Running after native property edits", lambda: desktop_running(project["title"]))
            capture_wallpaper("wallpaper-properties-applied.png")
        if args.playback_settings:
            assignment_before = (config / "lwe/session.toml").read_bytes()
            old_pid = owned_window()["pid"]
            set_preferences(30)
            if (config / "lwe/session.toml").read_bytes() != assignment_before or owned_window()["pid"] != old_pid:
                raise RuntimeError("Live settings update changed the assignment or restarted the application")
            driver.click('a[href="/desktop"]')
            wait_for("Scene still Running after native preference update", lambda: desktop_running(project["title"]))
            report["steps"].append("live Settings update preserved the application PID, assignment bytes and Desktop Running state")
        progress("Apply and captures completed; verifying close-to-tray then restart")
        old_app_pid = owned_window()["pid"]
        driver.click('button[aria-label="Close"]')
        wait_for("same app window hidden on normal close", lambda: not any(w.get("pid") == old_app_pid for w in niri_query("windows")), 10)
        if old_app_pid not in process_group_members(process.pid):
            raise RuntimeError("Closing the window terminated its application")
        if saved_assignment().get("assignments", {}).get(report["selectedMonitor"]) != library_id:
            raise RuntimeError("Closing the window changed its wallpaper assignment")
        focus_owned()
        time.sleep(1)
        if not desktop_running(project["title"]):
            raise RuntimeError("Hidden application's wallpaper lost Running state")
        capture_background("wallpaper-window-hidden.png", next(w for w in niri_query("workspaces") if w["name"] == workspace_name))
        report["steps"].append("normal titlebar close hid the same application PID while wallpaper and session remained")
        stop_owned(process)
        process = None
        report["steps"].append("entire owned application process group exited before restart")
        start()
        if args.playback_settings:
            driver.click('a[href="/settings"]')
            verify_preferences(30)
            driver.screenshot(output / "settings-restored.png")
            report["steps"].append("new application process read back 30 FPS and battery pause preferences")
        driver.click('a[href="/desktop"]')
        wait_for("real startup restore", lambda: desktop_running(project["title"]))
        driver.script("arguments[0].scrollIntoView({block:'start'})", desktop_card())
        time.sleep(0.2)
        driver.screenshot(output / "desktop-restored.png")
        capture_wallpaper("wallpaper-restored.png")
        report["steps"].append("new Tauri process restored isolated saved Scene")
        if args.scene_properties:
            restored = read_session_snapshot(config / "lwe/session.toml")
            if restored["contents"] != property_publication["contents"]:
                raise RuntimeError("Restart changed the persisted property assignment/choices bytes")
            open_properties(properties_plan["changed"])
            capture_properties("scene-properties-restored.png")
            report["properties"]["restartRetainedChoices"] = True
            report["steps"].append("new application process restored saved choices in real author property controls")
            _, audit = publish_properties(reset=True)
            report["properties"].update(resetPublication=audit, resetFullAuthorDefaults=True)
            capture_properties("scene-properties-defaults.png")
            driver.click('a[href="/desktop"]')
            wait_for("full default Scene Running after Reset defaults", lambda: desktop_running(project["title"]))
            capture_wallpaper("wallpaper-properties-defaults.png")
            report["steps"].append("native Reset defaults removed item overrides, retained assignment and reloaded every author default with Scene Running")
        if args.long_occlusion:
            logical = outputs[focused["output"]]["logical"]
            activate()
            report["occlusion"] = fullscreen_owned_cover(owned_window, [logical["width"], logical["height"]],
                capture=lambda: driver.screenshot(output / "owned-fullscreen-cover.png"))
            wait_for("Scene Running after owned fullscreen occlusion", lambda: desktop_running(project["title"]))
            capture_wallpaper("wallpaper-after-occlusion.png")
            capture_wallpaper("wallpaper-after-occlusion-later.png")
            report["steps"].append("owned Tauri window covered the output for at least 12 seconds, then Scene remained Running with two recovery captures")
        card = desktop_card()
        clear_button = driver.script("return [...arguments[0].querySelectorAll('button')].find(b=>b.getAttribute('aria-label')?.startsWith('Clear wallpaper from ')&&!b.disabled)", card)
        if not clear_button:
            raise RuntimeError("Target monitor clear control is unavailable")
        driver.click_element(clear_button[ELEMENT])
        wait_for("cleared saved assignment", lambda: report["selectedMonitor"] not in saved_assignment().get("assignments", {}))
        wait_for("cleared Desktop feedback", lambda: not desktop_running(project["title"]))
        capture_wallpaper("wallpaper-cleared.png")
        report["steps"].append("target monitor clear removed saved assignment and Running state")
        if args.blocked_project:
            blocked = json.loads((args.blocked_project / "project.json").read_text())
            report["blockedLibraryItemId"] = library(args.blocked_project, blocked)
            wait_for("blocked Scene explanation", lambda: driver.script("""return [...document.querySelectorAll('[data-detail-section=actions] button')].some(b=>b.textContent.trim()==='Apply'&&b.disabled)
              && [...document.querySelectorAll('[data-detail-section=actions] [role=status]')].some(e=>e.textContent.includes(arguments[0]))
              && document.querySelector('[data-detail-section=compatibility]')?.textContent.includes(arguments[1]);""", SCENE_UNSUPPORTED, BLOCKED_REASONS[args.blocked_reason]))
            driver.script("document.querySelector('[data-detail-section=actions]').scrollIntoView({block:'start'})")
            time.sleep(0.2)
            driver.screenshot(output / "blocked-scene-actions.png")
            driver.script("document.querySelector('[data-detail-section=compatibility]').scrollIntoView({block:'start'})")
            time.sleep(0.2)
            driver.screenshot(output / "blocked-scene.png")
            report["blockedReason"] = args.blocked_reason
            report["steps"].append("verified blocked Scene remained browsable with the expected capability explanation")
        if args.playback_settings:
            driver.click('a[href="/settings"]')
            edit = wait_for("Settings edit for Chinese layout", lambda: driver.script("return [...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Edit settings'&&!b.disabled)"))
            driver.click_element(edit[ELEMENT])
            driver.click('button[aria-label="Language"]')
            option = wait_for("Simplified Chinese option", lambda: driver.script("return [...document.querySelectorAll('[role=option]')].find(e=>e.textContent.trim()==='Simplified Chinese')"))
            driver.click_element(option[ELEMENT])
            save = wait_for("language save", lambda: driver.script("return [...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Save changes'&&!b.disabled)"))
            driver.click_element(save[ELEMENT])
            wait_for("Chinese playback settings review", lambda: driver.script("return [...document.querySelectorAll('p')].some(e=>e.textContent.trim()==='壁纸帧率: 30 FPS') && [...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='编辑设置'&&!b.disabled)"))
            driver.script("[...document.querySelectorAll('p')].find(e=>e.textContent.trim()==='壁纸帧率: 30 FPS').scrollIntoView({block:'center'})")
            time.sleep(0.3)
            driver.screenshot(output / "settings-zh.png")
            report["steps"].append("native language control rendered Chinese playback preferences for layout review")
        report["passed"] = True
        progress("Automatic UI/persistence assertions completed; cleaning up")
    except BaseException as error:
        report["error"] = f"{type(error).__name__}: {error}"
        if driver and driver.session:
            cleanup("failure DOM diagnostics", lambda: driver.diagnostics(output / "failure-dom.json"))
            cleanup("failure screenshot", lambda: driver.screenshot(output / "failure.png"))
    finally:
        cleanup("owned process group shutdown", lambda: stop_owned(process))
        if log_file:
            cleanup("close driver log", log_file.close)
        if workspace_named:
            cleanup("remove owned workspace name", lambda: niri_action("unset-workspace-name", workspace_name))

        def restore_focus():
            original = next((w for w in niri_query("workspaces") if w["id"] == focused["id"]), None)
            if original is None:
                raise RuntimeError("Original focused workspace disappeared")
            focus_workspace_id(original["id"])
            report["restoredWorkspaceId"] = focused["id"]
        cleanup("restore original focused workspace", restore_focus)
        cleanup("remove isolated configuration", lambda: shutil.rmtree(config))
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, ensure_ascii=False), flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
