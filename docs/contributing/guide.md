# Contributor guide

## Development setup

Install the core toolchain:

- Node.js 24, matching CI;
- pnpm 11.1.3, pinned in `package.json`;
- Rust 1.99.0, selected by `rust-toolchain.toml`; the minimum supported Rust version is declared separately in `Cargo.toml`;
- platform dependencies required by Tauri 2 on your distribution.

The Cargo Tauri CLI version is pinned in `.tauri-cli-version` (2.12.1). Use the shared installer to verify or install that exact version rather than an unbounded latest CLI.

Install dependencies:

```bash
pnpm install --frozen-lockfile
pnpm exec svelte-kit sync
./scripts/ensure-tauri-cli.sh
export PATH="$PWD/target/tauri-cli/bin:$PATH"
```

The installer keeps the CLI separate from your existing Cargo tools in `target/tauri-cli` by default. To use another location, set `CARGO_INSTALL_ROOT` when running the installer and add that directory's `bin/` to `PATH` before running `pnpm tauri:dev` or `cargo tauri build`.

Run frontend checks:

```bash
pnpm check
pnpm test
pnpm build
pnpm docs:build
```

Run Rust checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
```

Build documentation locally:

```bash
pnpm docs:build
```

Preview documentation locally:

```bash
pnpm docs:preview
```

## Before changing product behavior

Use the maintained documentation in this site as the policy source for product changes.

Every significant change should explain how it supports the Linux dynamic wallpaper platform rather than the retired wayvid product story. It should also state first-release non-goals when scope could be misunderstood.

Design work should identify its area:

- product foundation;
- Workshop loop;
- compatibility;
- runtime;
- application shell.

When content types or import paths are affected, call out user-facing compatibility implications.

## User-facing text and i18n

LWE targets English and Simplified Chinese user-facing surfaces.

When changing user-facing text:

1. update both languages;
2. check layout impact for longer Chinese or English strings;
3. include i18n verification in the task or pull request notes;
4. keep terminology consistent with the documentation.

The documentation site mirrors this rule: every maintained page under the English root must have a Simplified Chinese counterpart under `docs/zh/`.

## Documentation rules

- Keep all maintained documentation under `docs/`.
- Prefer task-oriented pages over archive dumps.
- Delete or merge obsolete planning documents after their useful information is represented in maintained pages.
- Do not reintroduce the deleted `openspec/` tree; project guidance now lives in this documentation site.
- Keep root README files concise and link to the published documentation for details.
- Do not document unsupported runtime behavior as supported.
- If a feature only works on a verified environment, name that environment.

## Testing expectations

Before submitting a change, run the smallest meaningful set of checks. For broad maintenance changes, use:

```bash
pnpm check
pnpm test
pnpm build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
pnpm docs:build
```

Quality Check also runs the lightweight packaging regressions:

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

Frontend tests include both server rendering and real component interactions in a simulated DOM. `pnpm test` runs both projects. For a focused run, use `pnpm test --project server` or `pnpm test --project interaction`; interaction fixtures mock IPC while clicking the mounted components. Cover business failures (`ActionOutcome.ok=false`), rejected IPC, and successful actions whose subsequent refresh fails. Keep existing content available for retries and distinguish refreshing data from repeating a successful action.

Tauri commands run synchronous service work through `commands/background.rs`. Desktop-dependent commands wait for background startup restore, and writes stay serialized in the service layer. Session and settings files use private temporary files and atomic replacement; persistence tests must prove that partial-write or replacement failures retain the previous file. Use fake API keys and local HTTP fixtures for network error redaction tests.

For documentation-only changes, `pnpm docs:build` and `git diff --check` are the required validation.

## Packaging validation

Both release workflows read `.tauri-cli-version`, include it in the CLI cache key, and verify `cargo-tauri` after restoring the cache. A cached binary with a different version must be replaced and checked before building.

AppImages are built on Ubuntu 22.04 to keep their glibc requirements compatible with the AppImage catalog test host. Native deb/rpm packages use Ubuntu 24.04 to retain the libmpv.so.2 ABI used by Arch packages. Release CLI and Rust caches are separated by distribution; static metadata validation alone does not prove that an AppImage can launch on the oldest supported system. Quality CI runs checksum-verified actionlint against every workflow.

AppImage validation requires Python 3, `unsquashfs` (from squashfs-tools), `file`, and `desktop-file-validate` (from desktop-file-utils):

```bash
python3 scripts/validate-appimage.py target/release/bundle/appimage/*.AppImage
# Validate an already extracted AppDir instead:
python3 scripts/validate-appimage.py --appdir path/to/LWE.AppDir
```

The validator checks executable `AppRun`, one valid root desktop entry, its icon, and a readable PNG `.DirIcon`. Relevant symbolic links must be relative, resolve inside the AppDir, and have existing targets; ordinary icon files are also accepted. It inspects packages without launching the application or repairing failed packages. Every AppImage selected for upload must pass this gate before release artifacts are published. Check the selected version and remove stale build artifacts from the upload selection.

The regression suite covers valid packages and missing, broken, absolute, or escaping icon links. Passing fixtures does not replace validation of the built release artifact or real desktop acceptance.

## GitHub to CNB mirror

GitHub `main` and its release tags are the authoritative source and build origin. CNB `Nesoriel/lwe` is a one-way code and release mirror. Set the CNB repository's `auto_trigger=false`; `.cnb.yml` keeps only `api_trigger_docs` for EdgeOne documentation deployment. CNB must not repeat Quality Check or Rust/package builds.

The `Sync CNB` workflow mirrors code on pushes to GitHub `main`. Both release workflows call it as a reusable job with `release_tag` and `expected_source_sha` after publishing the GitHub release, rather than depending on a release event generated by `GITHUB_TOKEN`. `workflow_dispatch` can backfill one `release_tag`, or explicitly request the documentation API trigger with `deploy_docs=true`.

CNB writes share a non-canceling GitHub queue (`queue: max`). CI strictly checks that queue declaration and applies an exact exception only for the fixed actionlint version's unsupported queue-field diagnostic; other workflows retain full lint checks. CNB's server SHA-256 and size are verified after upload; legacy assets without SHA-256 are downloaded for verification.

Configure the GitHub repository secret `CNB_TOKEN` with a long-lived CNB personal access token restricted to `Nesoriel/lwe`. Required scopes are:

- `repo-code:rw`;
- `repo-release:rw`;
- `repo-cnb-trigger:rw`;
- `repo-manage:r`.

The local OAuth token lasts eight hours and is unsuitable for persistent CI. An administrator performs the one-time CNB repository setup locally; CI does not receive repository-management write access. Keep token values out of source, logs, and release notes.

Preserve the GitHub tag object and verify its source commit against `expected_source_sha`; never rewrite a tag or force-update CNB `main`, which accepts only fast-forward updates. Copy the public GitHub release assets unchanged, verify the SHA-256 of the `.deb`, `.rpm`, and `.AppImage` on both sides, and retain CNB attachments permanently with `ttl=0`. A rerun fills missing content, skips identical existing assets, and rejects different content under the same name. Any failed sync or verification must remain a failed run.

Source changes alone do not prove that the mirror or EdgeOne deployment is live. After merging and running the workflow, read back the CNB commit, tag object, release assets and hashes; report the documentation trigger/deployment result separately.

## Release smoke checklist

Before publishing a stable tag or merging to `main` for a prerelease, verify the release path that action will exercise:

1. Run the required CI-equivalent checks locally when the change is broad: `pnpm check`, `pnpm test`, `pnpm build`, `pnpm docs:build`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace`, `cargo test --workspace`, and the packaging regressions above.
2. Confirm the GitHub Actions quality workflow is passing on the commit to be tagged.
3. Confirm release workflows build the expected Linux artifacts (`.deb`, `.rpm`, and `.AppImage`) and pass AppImage validation before upload. After publishing, download the public AppImage, record its SHA-256, and validate it again.
4. Confirm package-channel metadata matches the release channel: AUR stable (`lwe`) for stable tags and AUR git (`lwe-git`) for development/prerelease paths.
5. For runtime-affecting release candidates, run the real desktop runtime acceptance checklist below on a verified Wayland + `niri` session before describing the runtime path as supported.
6. Smoke the fresh install path when possible: install a package artifact, open and move/resize the window, restore it from the tray, open Settings and generate diagnostics, review Workshop/Library state, apply a compatible video wallpaper, clear it, restart LWE, and confirm saved assignment behavior is visible. A Tauri upgrade that changes Wayland startup also needs this check on `niri`.
7. Read back the release and package-channel results. For an AppImage directory repair, request upstream retesting only after the new stable AppImage is publicly available and verified; confirm the upstream log selected the new version.

Package install success only proves the application starts from that package. It does not guarantee runtime support on an unverified compositor, GPU/EGL stack, monitor layout, or wallpaper type.

## Real desktop runtime acceptance

Runtime changes must be validated on a real supported desktop session before they are described as supported. Video on Wayland with `niri` is the main verified target. Experimental Scene acceptance is limited to the sample and environment recorded below.

Session restore uses the LWE session file at `$XDG_CONFIG_HOME/lwe/session.toml`, or `$HOME/.config/lwe/session.toml` when `XDG_CONFIG_HOME` is unset. Use an isolated `XDG_CONFIG_HOME` for acceptance runs and inspect that test session file when checking restore behavior. Do not delete existing user settings or assignments as routine test setup.

Use this checklist for runtime changes:

1. discover at least one active monitor in LWE;
2. apply a compatible video wallpaper from Library to one monitor;
3. confirm the desktop visibly changes;
4. if multiple monitors are present, apply a wallpaper to a second monitor and then clear only one monitor;
5. confirm clearing one monitor does not stop wallpapers on other monitors;
6. restart LWE and confirm saved assignments are restored or that restore failures are visible in Desktop;
7. clear all assignments and confirm the saved session no longer restores them;
8. apply, clear, and reapply a video at least three times in the same process to check EGL resource reuse.

If a runtime step fails, keep the terminal log line that names the failing stage: backend start, output discovery, first-frame apply, per-monitor clear, or startup restore. These messages are the supported way to distinguish missing video assets, output mismatches, Wayland layer-shell/EGL failures, and backend timeouts.

Real desktop tests in the Rust test suite are opt-in because they depend on the active compositor, monitor layout, GPU/EGL stack, Steam Workshop content, and local video assets. Run them explicitly on a verified machine:

```bash
lwe_test_config="$(mktemp -d /tmp/lwe-real-desktop.XXXXXX)" && \
XDG_CONFIG_HOME="$lwe_test_config" LWE_REAL_DESKTOP_TESTS=1 \
  cargo test --locked -p lwe-shell --lib \
  services::desktop_service::tests::desktop_apply_flow_reapplies_video_after_clear_on_same_backend \
  -- --exact --nocapture --test-threads=1
```

For the experimental native 2D Scene subset, keep at least one locally validated Scene and one runnable video in Library, then use a separate test config directory:

```bash
lwe_scene_test_config="$(mktemp -d /tmp/lwe-scene-desktop.XXXXXX)" && \
XDG_CONFIG_HOME="$lwe_scene_test_config" LWE_REAL_DESKTOP_TESTS=1 \
  cargo test --locked -p lwe-shell --lib \
  services::desktop_service::tests::desktop_apply_flow_scene_video_switch_clear_and_restore \
  -- --exact --nocapture --test-threads=1
```

This test defaults to three apply/clear cycles and checks Scene → Video → Scene switches in the same backend, rollback after an isolated session-save failure, cancellation after the commit command has been processed, saved Scene assignment restore after backend shutdown/join/restart, and final clear. `LWE_REAL_DESKTOP_SCENE_CYCLES` accepts 1–1,000 cycles; use 100 to repeat the recorded mixed-source stress run, with the same isolated config directory:

```bash
XDG_CONFIG_HOME="$lwe_scene_test_config" LWE_REAL_DESKTOP_TESTS=1 \
LWE_REAL_DESKTOP_SCENE_CYCLES=100 \
  cargo test --locked -p lwe-shell --lib \
  services::desktop_service::tests::desktop_apply_flow_scene_video_switch_clear_and_restore \
  -- --exact --nocapture --test-threads=1
```

Keep `HOME` available for Steam discovery and inspect the isolated session file when checking persistence. Also confirm the wallpaper is visible and its supported effects animate. Increasing the cycle count is a procedure, not a recorded pass. The test covers only the current 2D subset; it does not establish full Scene compatibility. Independent output rendering and clearing require actual multiple-monitor hardware: a single-monitor run cannot cover those cases.

### Scene inventory

The read-only inventory example accepts one project directory or a directory containing up to 64 packaged projects. Replace the paths with the installed Steam library and keep reports local. It does not need the desktop opt-in flag:

```bash
lwe_scene_projects="$HOME/.local/share/Steam/steamapps/workshop/content/431960"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
lwe_scene_inventory_report="$(mktemp /tmp/lwe-scene-inventory.XXXXXX.json)"
cargo run --locked -p lwe-scene --example inventory -- \
  "$lwe_scene_projects" "$lwe_scene_assets" > "$lwe_scene_inventory_report"
```

`SceneInventory::collect` follows modeled references and scans packaged JSON/TEX/shaders plus project metadata. Loose files are followed by references without recursive directory enumeration. The report distinguishes reachable and packaged-only declarations, `user`/`value` bindings and JS, HDR and bloom, resource origins, framing, and structural unknowns. It omits script/shader/text contents and private values, including asset-looking strings in unknown/text subtrees. It never marks content runnable or supported. Budget exhaustion and invalid/missing resources produce explicit partial coverage; dynamic/script-created references remain unknown.

Default per-traversal limits include 8,192 resources, 32 MiB of JSON, 16 MiB of shader sources, and 16,384 mips plus 16,384 animation frames; document, depth/node, observation, and report-string limits also apply. The framing reader consumes the shared remaining mip/frame budgets before loops and allocations, skips media payloads, and performs no decoding. PKG indexing, range reads, and hashing use the same open descriptor; metadata checks detect ordinary in-place changes and keep a replaced pathname from redirecting the hash. This is a conservative consistency check, not an immutable filesystem snapshot. A complete bounded traversal does not prove complete runtime semantics.

### Scene UI and runtime measurement

Run the runtime modes of these opt-in tools only for desktop acceptance on a suitable Wayland + `niri` session: they apply and clear real wallpapers. Keep `HOME` for Steam discovery and use the locally acquired project/assets paths. The tools do not substitute for original Wallpaper Engine visual A/B or multiple-monitor/endurance acceptance.

`scene_probe` starts its own engine backend, uses a 30 FPS engine limit, and measures animated, paused, and resumed phases under frequent `GetStatus` requests. It emits JSON with draw/swap/callback counters, elapsed time, and logical Scene payload; it does not save settings/session files or test the Tauri UI. Replace `eDP-1` with the actual output:

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
lwe_scene_probe_report="$(mktemp /tmp/lwe-scene-probe.XXXXXX.json)"
LWE_REAL_DESKTOP_TESTS=1 cargo run --locked -p lwe-engine --example scene_probe -- \
  "$lwe_scene_project" "$lwe_scene_assets" eDP-1 60 \
  > "$lwe_scene_probe_report"
```

Add `--static /absolute/path/to/an-authored-static-project` to test a static first frame while paused and a separate static idle phase. The static project must pass preparation and contain no animated effects. Without it, the report has no static-phase evidence. The probe checks the frame cap under command traffic, settled paused/static zero activity, clear, and shutdown/join; its deliberate status traffic is not a normal idle CPU/power baseline. The reported logical RGBA texture/FBO bytes exclude actual driver VRAM, EGL buffers, and video resources.

Add `--properties /absolute/path/to/choices.json` to prepare and measure a project with author-property overrides. The JSON must be an object mapping declared names to typed values and is limited to 64 KiB. Preparation uses the same validation as LWE; the probe does not modify the original project or persist those choices to settings/session files.

`scene_resources` separates CPU preparation, Scene rendering, video rendering, mixed-source switching, and repeated backend restart. It requires `LWE_REAL_DESKTOP_TESTS=1` even for its `cpu` mode. Runtime modes require exactly one physical output. Each clear is confirmed by an empty active-session/metrics status before sampling; backend shutdown is joined. It streams one JSON line per snapshot instead of retaining a growing report, and writes no settings/session files:

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
lwe_scene_video="/absolute/path/to/local/video.mp4"
lwe_scene_resource_report="$(mktemp /tmp/lwe-scene-resources.XXXXXX.jsonl)"
LWE_REAL_DESKTOP_TESTS=1 cargo run --locked -p lwe-engine --example scene_resources -- \
  "$lwe_scene_project" "$lwe_scene_assets" "$lwe_scene_video" mixed 30 \
  > "$lwe_scene_resource_report"
```

Replace `mixed` with `cpu`, `scene`, `video`, or `restart`; the cycle count is 1–200. `cpu` prepares/drops the Scene without starting a desktop backend; `restart` applies/clears the Scene and shuts down/joins a backend every cycle. The other modes retain one backend across cycles.

The JSON records Rust requested live/peak bytes, glibc used/free/mmap blocks, `/proc/self/smaps` RSS categories, available DRM fdinfo memory fields, FDs, and threads. Rust counts exclude direct native allocations, and its logical realloc peak excludes a native allocator's temporary old/new buffers. glibc free blocks are retained allocator space, not live wallpaper assets; mmap and driver allocations need separate interpretation. Sampling itself allocates temporary `/proc`/JSON buffers and has a fixed warm-up cost. These process snapshots are not total VRAM, frame timing, power, or proof that every native allocation is leak-free. A fresh child process with `MALLOC_ARENA_MAX=2` is a controlled allocator diagnostic only; it is not a production setting or accepted memory fix.

`scripts/scene-ui-smoke.py` uses native Tauri WebDriver DOM actions and real IPC. It selects the Library item and monitor with native pointer clicks, applies, reads Desktop feedback and the isolated session, checks titlebar close-to-hide without stopping the wallpaper, restarts its owned application process, confirms restore, clears, and optionally checks a blocked item. It writes UI/desktop captures and `report.json`. The harness currently requires exactly one output in its target compositor, a built shell, `tauri-driver` and `WebKitWebDriver` available on `PATH`, `grim`, an unnamed empty workspace on the focused output, and acquired Scene content. A nested compositor's output must be reported as nested, not as physical-monitor acceptance. UI monitor labels may differ from backend output names; the report records both. For a debug shell, run `pnpm dev` in a separate terminal and build `cargo build --locked -p lwe-shell` first:

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"
lwe_scene_blocked_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/2454418355"
lwe_scene_driver="$(command -v tauri-driver)"
lwe_scene_ui_report="$(mktemp -d /tmp/lwe-scene-ui-report.XXXXXX)"
LWE_REAL_DESKTOP_TESTS=1 python3 scripts/scene-ui-smoke.py \
  --binary "$PWD/target/debug/lwe-shell" --driver "$lwe_scene_driver" \
  --project "$lwe_scene_project" --blocked-project "$lwe_scene_blocked_project" \
  --blocked-reason unsupported_hdr \
  --output "$lwe_scene_ui_report"
```

The script creates and removes its own isolated `XDG_CONFIG_HOME`, preserves `HOME`, temporarily names its acceptance workspace, and restores the prior focused workspace when it finishes. Wallpaper capture verifies that this workspace is focused and unobstructed. `--output` is a report directory, not a monitor name. Review the screenshots as well as its business-state assertions; a visible Running label alone is insufficient. Missing drivers, a suitable empty workspace, or desktop prerequisites block acceptance. These tools do not establish physical battery transitions, measured VRAM, power, a 24-hour soak, or independent multiple-output behavior.

Add `--playback-settings` for native Settings actions: save 15 FPS with pause-on-battery, change to 30 FPS while the wallpaper runs, and verify saved preferences after the owned-process restart. The example explicitly checks `unsupported_hdr` on `2454418355`; reassess the blocked item/reason before rerunning. `3659880761` now passes original-package preparation and is no longer the current bloom-blocked example. Saved UI values do not measure cadence or physical battery transitions; the separate desktop-service measurement below covers engine cadence.

For the native property/failure/fullscreen acceptance, use the default `3659880761` project and add `--scene-properties --scene-properties-write-failure --long-occlusion` (optionally `--playback-settings`). `--scene-properties` needs authored bool and changeable slider controls; it saves edits, verifies session/control restoration after restart, and resets every author default. The write-failure flag requires a non-root user and changes permissions only inside the harness-owned isolated config, then verifies rollback/draft retention and retries. `--long-occlusion` uses only the owned Tauri fullscreen window for at least twelve seconds; it does not suspend/lock the system or measure callback pause. Review the actual wallpaper, editor, rollback and recovery captures; automated business assertions alone are insufficient.

For a project with many controls, add `--all-scene-properties`; it enables the property flow and edits every declared control. Booleans and sliders use native WebDriver pointer/keyboard actions. Color is changed by dispatching the actual rendered color input's input event, then using the normal Save/IPC flow; this does not test the system color picker. The original `2903458386` run used all twenty controls together with `--scene-properties-write-failure --playback-settings --long-occlusion`.

For a project with exactly one resource-image control, add `--scene-image /absolute/path/to/static.png` together with `--all-scene-properties` and put `wtype` in the test PATH. The harness cancels and selects through the owned native file picker, deletes only its copied picker source, and verifies durable content-token restore and author-image reset. Both config and data directories are isolated; `HOME` remains available for Steam discovery. For a complete effect graph, repeat `--keep-scene-property NAME` for author choices that must remain enabled while other controls are edited. Inverting every boolean can disable all effects and is insufficient render acceptance. These flags do not inject IPC or bypass candidate/save gates.

Where native chooser keys do not activate GTK buttons, add `--native-dialog-accessibility`. It requires Python GI/AT-SPI and the GTK accessibility bridge. The harness automatically sets `GTK_MODULES=gail:atk-bridge`, `NO_AT_BRIDGE=0` and `LC_ALL=C.UTF-8` only in its owned child environment. `wtype` still enters the copied source path; the alternative mode invokes the real Cancel/Open button's AT-SPI action after checking the owned chooser window, PID and focus. It does not invoke business IPC or replace the native file picker.

Add `--narrow-image-layout` with `--scene-image --playback-settings` to capture the real image controls at a 380-pixel native viewport in English and Simplified Chinese. The harness switches language through Settings, checks the image control and its buttons for overflow, then restores the native window. This checks the image editor's layout, not the whole Library's narrow-window layout.

### Miku extension (2026-10-13) {#miku-extension-2026-10-13}

The shared bounded path now implements empty-default image properties, instance materials, genericimage4/sprite frame selection, the original motionblur accumulation/copy/combine graph and CRT composite alias, required SceneScript frame/audio/property forms and builtin Atami CFF text. Author-hidden layers remain prepared; explicit choices determine visibility. The default graph preserves eleven declarations, ten render layers, five effects and twenty-five passes, including four global bloom passes.

The unchanged original `3420277613` passed complete CPU preparation and a software-EGL full-graph oracle on the final production source. Both eight-frame atlases were selected and visibly rendered under controlled positive audio through the actual original VM. Three independent histories, exact paused output, a same-project candidate, frozen rollback pixels, thaw and final cleanup passed. The retained CPU charge was 255,616,293 bytes and its reservation peak was 525,312,699 bytes (500.98 MiB); the accepted old/candidate GPU union was 525,624,984 bytes (501.28 MiB). Both peaks are below their separate 536,870,912-byte limits. Freezing moved 79,833,600 bytes of private history into sealed CPU snapshots; keeping the old history on GPU would require 605,458,584 bytes and is correctly rejected. CPU charges and shared uploads returned to zero at cleanup. These production allocation/reservation counters exclude test-only readback vectors, allocator overhead, RSS and measured VRAM.

The final-source CPU checks separately passed all five original script modules and the original Atami font, including shared font ownership and bounded text rasterization. The 2.5D Space software-EGL regression also passed all twenty-six pixel comparisons. Those checks use controlled supplied spectra; they do not establish live capture or desktop acceptance.

Separate CPU preparation/worker checks passed the original Glitch OFF (`0`) and POOF (`2`) branches with Framerate enabled and the default effects retained. OFF prepares eleven layers/five effects/twenty-six passes and shows Static50; POOF prepares ten/five/twenty-five and intentionally hides the character. Hidden script-owned layers remain hidden after callbacks. This is CPU option evidence, not native pixels for those branches. The package's scheme-color control has no explicit object or script binding; its UI/persistence checks do not establish an implicit original-engine color effect.

The separate `original-miku-private-capture.json` passed on AMD Radeon/radeonsi with surfaceless EGL. Owned PCM played into a private silent Pulse sink, then entered production default capture and the accepted sixteen/thirty-two/sixty-four-band spectra. Thirteen accepted frame snapshots matched the original sixteen/sixty-four-band VM alpha responses while the complete default graph and three histories rendered. A separate causal probe kept input, geometry and time fixed: captured audio changed 1,468 pixels through the unchanged original SimpleBars shader and its thirty-two-band stereo uniforms. Pause disconnected capture and preserved the accepted pixels exactly; resume reconnected, and cleanup removed capture/shared uploads. This is real private capture plus hardware offscreen rendering, separately scoped from native desktop presentation.

The final native engine-only probe used the unchanged default graph at 1920×1080 in nested niri on AMD 680M. With a 30 FPS cap, thirty animated seconds completed 136 swaps (4.53 FPS); thirty paused seconds completed zero draw/swap/callback operations, and three resumed seconds completed thirteen swaps (4.33 FPS). Clear removed the output metrics and backend shutdown joined. All effects stayed enabled and authored resource resolutions were preserved. **This run did not reach 30 FPS and does not establish daily-use readiness.** Its 500,230,580-byte current GPU plan is a logical union; overlapping diagnostic pool/private views must not be summed. The private final manifest records shell SHA-256 `09857b830cc0d2147c0ffb7ec32aaff0cbac6928d8a094a461d4a40b3de5355d` and probe SHA-256 `42ee6e3d9e04f5f20357107e52ad7e2ff72a42c4ac0a4d84dabf1460701fe29a`.

The actual native layout check passed both languages at a 380-pixel viewport, with a 230-pixel image control and no overflow inside the control or its buttons. Language changed through the real Settings flow; source stayed unchanged during the run, no DOM styling was injected, and owned-window/workspace cleanup passed. The existing catalog-toolbar overflow remains outside this image-editor check.

On this machine's Mesa 26.2.4/llvmpipe LLVM 23.1.1, a separate tiny scalar-array/tint fixture produced incorrect pixels despite correct uniform readback. Independent pure-GL reproduction confirmed the software-driver issue; forty-two equivalent AMD Radeon/radeonsi cases and fresh-program A→B→A checks passed. Keep the failing software comparison explicit; do not weaken its pixel expectation or infer original-engine parity from a source/math check.

The final `native-full-effects-final3/report.json` passed all seventeen native steps in the same nested environment, with no mocked IPC or cleanup errors. It included all fourteen value controls, kept Glitch ON and the full-effect booleans enabled, and changed Framerate, scheme color, all six CRT sliders and the background image. Two real GTK chooser cancel/select rounds used the owned accessibility buttons. The copied picker source was deleted after import; the content token survived atomic save and restart. Permission-denied saves retained the previous session bytes/inode, editable draft and previous Scene Running state; retry succeeded. Settings changed the 15 → 30 FPS limits without replacing the application or assignment. Titlebar close hid the same process, its whole owned group joined before restart, and a new process restored the image and choices. Reset removed overrides and restored every author default; twelve seconds of owned fullscreen coverage recovered, then target clear removed the assignment. The run also passed both 380-pixel image layouts and restored the original named workspace. Fullscreen coverage does not prove callback pausing or system suspend.

Custom image contracts and remaining limits are documented in [Scene images, timed frames and history](./scene-miku-runtime.md). Physical suspend/resume is skipped by explicit user instruction. Physical multi-monitor, installed-package, Windows A/B and Miku endurance acceptance remain unverified.

### Recorded experimental Scene acceptance

Development acceptance on 2026-10-09 used Wayland with `niri`, AMD Radeon 680M (`1002:1681`, `amdgpu`), and one `eDP-1` output at 2160×1440 with scale 1.25. Scene sample `3337606455` uses the default zero-X/Y camera, one image layer, and Shine: five effect passes, six total passes. A full-resolution capture confirmed visible Shine rendering.

The final compatibility gate admitted only that Scene sample from the local catalog. `desktop_apply_flow_scene_video_switch_clear_and_restore` passed three Scene ↔ Video/apply/clear cycles, isolated session-save failure rollback, cancellation after commit processing, and saved Scene restore after the backend was shut down and joined. The existing three-cycle video reapply test also passed. Tests used isolated `XDG_CONFIG_HOME`; the original workspace presentation was restored and the user's original config was not changed.

At this 2026-10-09 acceptance point, `3258032485` was blocked as `unsupported_camera` because its non-zero camera X/Y semantics were unconfirmed. Earlier captures were research and did not establish support. This historical record covers one accepted Scene sample on one monitor, not complete Scene compatibility; independent multi-monitor behavior was not tested. The Scene implementation is independent Rust code without an Open Wallpaper Engine runtime dependency.

### 2026-10-10 development record

The new limited camera branch is based on REA evidence from the installed original `wallpaper64.exe`: its orthographic loader resets saved eye/center/up when the active camera-path vector is empty, and the runtime camera update consumes those reset values. LWE admits only explicit fixed dimensions, parallel forward-facing saved vectors, default up, disabled automatic projection, and absent/empty paths; other camera modes remain blocked. This supports the restricted implementation used by `3258032485`, not a claim of original-engine visual equivalence. Original visual A/B has not been executed.

| Area | Implemented evidence/tooling | Acceptance status for this build |
| --- | --- | --- |
| P0 inventory | Bounded declarations/framing API, same-descriptor PKG fingerprint, privacy and pre-loop aggregate budget regressions | Final `scene-inventory-p0.json`: 11 projects, 2,695,798 bytes, all 11 PKG SHA-256 values match the baseline. Reachable declarations include 31 JS locations in five projects, 20 puppet locations in five, and hierarchy in three. This does not assert runtime eligibility. |
| P1 runtime | Per-output deadlines, settled static/paused idle behavior, counters, 512 MiB logical RGBA/FBO accounting including retained rollback, one/two scratch targets; EGL lifetime cleanup and single initial video load | `scene-probe-shine.json` passed the 30 FPS completed-swap cap under frequent status requests and separate 60-second paused/static idle windows. Resource follow-up is recorded below. Settings FPS/battery persistence/live integration, the separate 15 → 30 FPS desktop-service measurement, and the finite native Settings/restart flow passed. Display timestamps, GPU p95, physical battery/AC transitions, VRAM/power/endurance remain unverified. |
| P0 real UI | Native pointer monitor selection → Apply → Desktop → titlebar close/hide → owned-process restart/restore → target Clear; isolated session and unobstructed captures | `3337606455`, `3258032485` and the original-default `3659880761` passed native-pointer apply/hide/new-process restore/clear and capture review. Earlier reports retained the then-blocked 365 explanation. The default 365 run also passed Settings 15 → 30 FPS/restart preferences; the separate property/failure/reset/fullscreen flow passed as recorded below. Cleanup and original-workspace restoration passed. |
| P2 camera | Authored positive/blocked fixtures and original-binary control-flow/constants evidence | Original-engine visual A/B and daily-use parity are unverified. |
| P2 properties | Exact bool/slider/color binding defaults and validated per-project/output edits; atomic assignment/value persistence with rollback | Unknown expressions/types/shapes or invalid declarations remain blocked. Native bool/slider edit, real write-failure rollback/retry, atomic inode replacement, new-process control restore and full-default reset passed. CPU/persistence tests remain distinct from UI evidence. |

The preceding development snapshot passed 18 resource-crate tests and clippy; the independent privacy/budget/fingerprint review had no remaining blockers. Rust fmt, workspace clippy, check, and tests passed. Engine tests passed 49 with one GPU fixture ignored in the ordinary run. That fixture was then explicitly run under surfaceless EGL and passed; the ordinary ignored result is not its GPU evidence. The final inventory confirms shared pre-loop budgets of 16,384 mips and 16,384 animation frames, and checks the indexed descriptor stamp after hashing. After the CSS fix, frontend type checking reported zero errors/warnings, all 113 tests across 25 files passed, and the production build passed. The Python suite passed 88 tests, including five UI-harness safety regressions. These counts belong to that snapshot; the subsequent settings/property/resource changes need their own final checks. Documentation build results are recorded separately. The 100-cycle result is recorded below.

The 2026-10-10 `3337606455` UI run passed on the recorded Wayland + `niri`, AMD 680M single `eDP-1` machine (2160×1440, scale 1.25). Native pointer selection verified Library identity `d6d309c547f3c764` and monitor identity `niri:BOE:0x0893:eDP-1` (backend `eDP-1`, UI label `BOE 0x0893`). Apply persisted the assignment; the matching Desktop card reported Running. Normal titlebar Close hid the window while retaining the same application PID, wallpaper, and session. The test then terminated its owned process group, launched a new application process, confirmed saved restore, and cleared the target. `3659880761` remained browsable with Apply disabled and its property-binding explanation visible. Applied/restored/cleared background captures were reviewed as focused and unobstructed.

Native pointer testing exposed a CSS scope defect: `body > div` applied application-root sizing to the monitor menu's portal wrapper and placed the menu outside the viewport. The application root now has ID `lwe-app`, and those styles target `body > #lwe-app`; native pointer selection passed after the fix. The acceptance uses native WebDriver clicks, without synthetic DOM input.

Final timed captures of both Shine and Shake showed pixel changes in the background region outside static desktop components (ROI x=100–1700, y=400–1400). Both UI reports completed visual review, with separate visual-review and animation-difference records. This shows changing pixels, not original-engine visual parity. Normal tray Show/Quit was not tested; owned-process termination is the test's restart step. The final `3258032485` UI rerun also passed: Library identity `18bab41e6ab0c721`, the same native monitor selection, same-PID close-to-hide, owned-process exit before new-process restore, target clear, and the then-current blocked-property explanation for `3659880761`. Its report has no cleanup errors and restores original workspace ID 1. The completed-swap/idle probe and 100-cycle lifecycle stress passed as recorded below; the follow-up resource diagnostics do not yet establish daily-use RSS stability.

The final `scene-probe-shine.json` records an engine-only probe on `eDP-1` with a 30 FPS limit; it writes no session/settings and does not test UI. The animated window processed about 475.93 status requests per second. Its stable logical Scene RGBA/FBO payload was 51,871,748 bytes (49.47 MiB), not measured VRAM.

| Phase | Window | Completed swaps | Other observations |
| --- | --- | --- | --- |
| Animated Shine | 60.000055 seconds | 1,773; about 29.54997/s | 1,773 draws under frequent command wakeups. |
| Paused, after settling | 60.000080 seconds | 0 | Render attempts, draws, empty frames, and callback requests/receipts all zero. |
| Resumed | 3 seconds | 89; about 29.66595/s | Animation resumed within the configured cap. |
| Authored static candidate while paused | First frame | 1 | Exactly one draw and swap to accept the candidate. |
| Static playing, after settling | 60 seconds | 0 | Render attempts, draws, swaps, and callback requests/receipts zero; logical payload 1,032 bytes. |
| Clear and shutdown | End of probe | — | Output metrics removed; backend shutdown joined successfully. |

These are engine completed-swap/callback counters and bounded idle observations. They establish deadline enforcement under command traffic and the absence of a sustained paused/static render loop during these windows. They do not measure compositor display timestamps, p95 GPU render time, normal idle CPU/power, VRAM, 24-hour endurance, or multiple outputs. The completed 100-cycle lifecycle result below does not establish flat memory or daily-use readiness.

The final `scene-stress-100.json` and log record 100 alternating Shake/Shine cycles on the same backend, each Scene → Video → Scene → Clear. The Rust test passed in 424.37 seconds; the wrapper took 430.392 seconds. Isolated save-failure rollback, late commit acknowledgement with deadline cancellation, backend shutdown/join, saved Scene restore, and final clear all passed. The user's original config remained unchanged; owned application processes were removed and workspace 1 restored.

Whole-test-process RSS, including prepare/cache/engine/libmpv, peaked at 832,548,864 bytes (793.98 MiB) and ended at 714,219,520 bytes (681.13 MiB). Its four time-window medians rose from 562.84 → 620.44 → 641.48 → 664.35 MiB. This is neither application-only memory nor VRAM; attribution was unresolved at that acceptance point. P1 daily-use resource stability has not passed. FD maxima were 22 in each window and 14 at the end; the initial count of 4 is not a steady-state baseline. No FD growth was observed across those windows, which does not establish long-term stability.

The resource follow-up added explicit context/display destruction and idempotent surface cleanup, including initialization failures; the display/library owner remains alive until its last context/surface is released. Video construction now loads its initial source once instead of issuing two consecutive `loadfile replace` commands. These fixes address concrete lifetime/redundant-work paths without claiming they explain all of the preceding RSS rise.

Private `resources-{cpu,scene,video,mixed,restart}.jsonl` and `resources-summary.json` record separate engine-only subprocesses on the same single-output 680M machine; all exited successfully and preserved the user's original config. The following values compare the first and last release/clear snapshots, not cold startup:

| Mode | Cycles | Rust requested live bytes after release/clear | Additional observation |
| --- | --- | --- | --- |
| CPU prepare/drop | 30 | 2,177 → 2,177 | No desktop backend; decoded Scene data was released each cycle. |
| Scene apply/clear | 30 | 49,259 → 51,539 | Backend remains running; this small change is not a proof of a steady state. |
| Video apply/clear | 30 | 49,259 → 49,259 | RSS still rose despite fixed Rust live bytes. |
| Scene → Video → Scene → Clear | 50 | 49,637 → 63,029 | glibc free blocks: 166,546,944 → 654,284,608 bytes; RSS: 290,889,728 → 614,535,168 bytes. glibc used blocks changed from 15,622,656 to 16,472,256 bytes. |
| Backend restart | 20 | 2,229 after every shutdown/join | Every stopped snapshot has four FDs and one thread. |
| Mixed, fresh child with `MALLOC_ARENA_MAX=2` | 100 | 49,260 → 51,846 | glibc free blocks: 134,691,232 → 198,131,456 bytes; clear snapshots retain 12 FDs. Final clear has 19 threads. |

These counters support substantial allocator-retained free space as a contributor to the video/mixed process memory trend; they do not establish that every native allocation is correct or that RSS reaches a stable bound. System-allocator Rust counts and glibc block counts measure different scopes, and the probe omits shell catalog/cache/UI work. P1 remains open.

The controlled arena comparison in private `resources-mixed-arena2.jsonl`/log passed 100 cycles in 174.26 seconds. After final shutdown/join, Rust live bytes were 2,228, with four FDs, one thread, and no DRM clients. Its final clear RSS was 226,689,024 bytes with another 29,519,872 bytes swapped out; file-page residency also changed. The observations are consistent with the arena policy influencing retained free blocks, but the default run used 50 cycles and the controlled run 100 under different memory pressure. They are not a production performance comparison or proof of an RSS benefit. Only the diagnostic child received this environment variable; no product allocator tuning or forced trimming is adopted.

The follow-up also implements persisted FPS/battery controls and live engine policy updates. Saved settings remain authoritative if a live update fails, and the UI exposes the runtime issue. The FPS option caps paced rendering/swaps; battery pause preserves the user's manual pause intent. The opt-in `desktop_settings_live_updates_preserve_assignments_and_backend` test passed in 22.05 seconds with isolated config, recording the following in private `settings-live.log`:

| Saved limit | Measurement window | Completed swaps/rate | Status requests |
| --- | --- | --- | --- |
| 15 FPS | 10.0003 seconds | 147; 14.6996/s | 4,589 |
| 30 FPS, updated live | 10.0004 seconds | 296; 29.5987/s | 4,758 |

Both samples confirmed the same backend, unchanged assignment bytes, and saved preference readback. Pause-on-battery was saved and active in the engine policy; the machine reported `on_battery=false` and `power_paused=false`. No physical battery/AC transition was tested. These are completed-swap counts from the real service path, not compositor presentation timestamps or Settings UI actions.

The separate native UI run in private `ui-settings-verified/report.json` passed, with `visual-review.json` recording completed review. Native English Settings actions saved 15 FPS and pause-on-battery, changed the limit to 30 FPS while the wallpaper ran, and verified 30 FPS plus the battery preference after a new owned process restored the wallpaper. Chinese Settings layout was also reviewed. Apply, same-PID close-to-hide, complete owned-process-group exit before restart, saved restore, target clear, and the then-visible `3659880761` bloom blocker all passed. Cleanup errors were empty, the isolated config was removed, and original workspace ID 1 was restored. The preceding `ui-settings-final` automated result had poorly framed 15/30 FPS captures and is not the new visual acceptance record. This finite workflow does not establish physical battery transitions, screen lock/suspend recovery, power, 24-hour endurance, or full P1 readiness. These are development-checkout results; installed-package acceptance remains outstanding.

### Mixed Scene and per-output properties: 2026-10-10

The independent Rust CPU/GPU path now implements the original `3659880761` package's active particle tree, WaterRipple/Shake/Iris effects, LDR bloom and authored single-entry FLAC loop layers. Preparation succeeds with seven render layers, five effects and 21 passes. Its default is six visible particle roots plus all five children, with worst-case particles/vertices/work of 3,965/26,790/30,980. The seventh root, Reactive, is literal `visible:false`; preserving that author-hidden layer is correct default playback. Enabling its audio analysis remains blocked. No authored active particle/effect/bloom layer was disabled to pass preparation.

Particles execute bounded emitters, initializers, movement/drag/gravity, alpha/size oscillation, angular movement, turbulence, sprite/rope rendering and spawn/follow children against SceneClock. Unknown behaviors fail explicitly. OWE source is only a behavior reference; the Rust runtime has independent state structures, RNG and gradient/curl field. Native noise, child lifecycle/transforms, rope sampling and complete-frame visual equivalence remain unverified.

REA confirms the LDR four-pass input chain, RGBA8 color targets, scaled dimensions rounded down with a two-pixel minimum, and texel size from the full output dimensions. It also confirms that a single-entry loop has no random min/max interval and uses squared layer gain. These are static native findings, separate from visual or audio acceptance.

The final finite engine-only probe, private `runtime/365-engine-final-60s.json`, ran on the recorded Wayland + `niri`, AMD 680M, single `eDP-1` machine. It passed with shutdown joined and output metrics removed after clear, writes no session/settings, and performs no Tauri UI actions:

| Phase | Observation |
| --- | --- |
| Default mixed Scene, 30 FPS cap | 1,677 completed swaps in 60 seconds, about 27.95/s; about 466 status requests/s and zero empty frames. |
| Settled pause | 60 seconds and 28,000 status requests with zero render attempts, draws, swaps, empty frames and callbacks. |
| Resume | 85 completed swaps in three seconds, about 28.33/s. |
| Logical GPU payload | 134,862,328 bytes for RGBA textures/FBOs and particle VBO; driver/EGL/video overhead is excluded. |

A separate private Pulse sink/PCM probe observed zero PCM in candidate, paused, muted and dropped phases, and activated RMS 2,894.53. Its owned sink was cleaned up and pre-existing sinks/default sink/source remained unchanged. This does not prove physical-speaker output, every loop boundary or the complete property/audio rollback transaction.

The separate native default UI flow in private `runtime/ui-365-default-final/report.json` passed with `mockIpc=false`, isolated config and manual capture review. Native Library/monitor Apply reached Desktop Running; titlebar close hid the same PID while wallpaper/session remained; the entire owned process group exited before a new Tauri process restored the Scene; target Clear restored the baseline. Native Settings saved 15 FPS and battery pause, changed to 30 FPS without changing PID or assignment bytes, then read back 30 FPS and the battery preference after the new-process restore. English/Chinese captures were reviewed for full composition, particle change across two frames, restored visibility, clear baseline and label layout. Cleanup errors were empty and workspace 1 was restored. No original-default active feature was disabled. This passes the default UI workflow; the separate property and finite fullscreen run is recorded below. Daily-use acceptance remains outstanding.

The finite `runtime/365-resources-mixed-20.jsonl` and summary record 20 cycles of `3659880761` → video `2848041076` → `3659880761` → clear. Every clear had empty active status/metrics; shutdown joined. Clear FDs stayed 12 and Rust live bytes stayed 48,811. First/last clear process RSS was 257,609,728 → 287,825,920 bytes, with glibc used blocks 14,897,040 → 15,433,312 bytes. After shutdown: four FDs, one thread, 2,227 Rust live bytes, RSS 191,705,088 bytes and 250,144,480 glibc free bytes. This supports finite resource return and no FD accumulation in this run; substantial allocator/driver RSS retention prevents a memory-stability or all-native-allocations-leak-free claim. Power and daily-use endurance remain unverified.

The final implementation snapshot passed workspace fmt/clippy/check/tests and 98 Python tests. After the error-state Reset follow-up, frontend type checking reported zero errors/warnings, 25 component regressions and all 148 tests across 26 files passed, and the production build passed. The new error-state recovery remains component evidence, distinct from the recorded native normal Save/Reset flow. Three ignored GPU fixtures explicitly passed in a separate surfaceless EGL run: authored image/GL state, bloom halo/state, and installed original-asset bloom/WaterRipple/Iris/generic-particle/padded-orientation/cutout pixels. This is separate from CPU preparation and native desktop/UI acceptance; it does not establish an original-engine frame A/B. Documentation build is the final integration check after these records are complete.

Validated bool/slider/color edits are scoped to project and output and saved in `session.toml` alongside assignments. The Save and apply transaction is first-frame/commit acceptance → one atomic assignment/value save → Finalize and audio activation. Preparation, candidate acceptance or save failure before atomic publication retains the prior renderer and choices. Once saved, the durable assignment remains authoritative; deferred finalization cleanup cannot undo it. Ordinary save/clear retains other monitor assignments and every unrelated choice; clearing the final assignment does not delete saved properties. Missing files mean empty state; invalid/oversized state prevents overwrite and diagnostics omit property values. The separate native property run below passed. If an author-schema change invalidates shape-valid saved choices, Reset defaults is also available after property-load failure with no snapshot; it retries `{}` through default preparation and removes only that monitor/item selection on successful application/save. Normal Save requires validated definitions/values. Unsupported defaults or unreadable/corrupt session state remain explicit failures. This new error-state recovery has component-regression validation; the recorded native run covers normal Save/Reset, not a separate author-schema-change scenario.

The native property/failure/fullscreen run, private `runtime/ui-365-properties-final/report.json`, passed with real checkbox/number input and manual capture review. The author property `hdr` changed true → false to disable **LDR bloom**, not enable HDR rendering; `newproperty2` changed 0 → 0.1 for the authored audio-layer gain. Real Save failed with PermissionDenied after only the isolated `config/lwe` directory was made 0500: session bytes/inode stayed unchanged, the draft remained editable and the previous Scene stayed Running and visible. Restoring 0700 and retrying the same choices succeeded: the complete TOML was atomically replaced with a new inode, while an already-open descriptor retained the prior values. A new Tauri process restored the saved choices in real controls. Reset defaults removed overrides, restored every author value including bloom=true and the original session byte hash, and retained the assignment.

The owned Tauri window then covered the whole logical 1728×1152 output for 12.000 seconds. Reveal retained Running and two captures showed evolving particles. Callback/occlusion-pause was not directly measured; this does not establish system suspend or lock recovery. All owned process groups stopped, isolated config was removed, workspace 1 was restored and cleanup errors were empty. The passed finite property/fullscreen flow does not establish original-engine visual parity, 24-hour endurance, actual suspend/lock, physical battery transitions, power/VRAM stability or installed-package acceptance.

An original-engine reference attempt used an isolated engine copy and Wine prefix. The reference UI failed with `c0000005` before any usable capture; owned cleanup had no errors and source hashes stayed unchanged. This is not an original-engine A/B result. The read-only 11-project CPU rerun is saved privately as `scene-prepare-corpus-2026-10-10-mixed.json` (binary/inventory SHA-256, package hashes, timings, exit codes and raw first-blocker results). It passes `3258032485`, `3337606455` and `3659880761`; the eight first blockers are five SceneScript, `2798622011` camera, `2454418355` HDR and `3485147659` layer-count bound. The latter reports a null feature code plus the exact layer-count validation error. All eleven packages match P0. The existing compiled example was reused without a build, backend/GL launch or settings/session write. This 3/11 CPU result is not a Workshop compatibility rate or native UI acceptance. Keep finite CPU/GPU/PCM evidence and the passed default native UI workflow separate from outstanding original-engine visual comparison, 24-hour soak, ten actual suspend/resume cycles, physical battery/AC transitions, power/VRAM measurement, independent multi-monitor and installed-package acceptance. Daily-use readiness is not established.

Keep the 2026-10-09 acceptance and earlier property/bloom-blocked UI captures as historical evidence. The new original-package mixed-Scene implementation and finite probes do not supersede their workflow boundaries or establish daily-use support.

The three separate GPU pixel/GL-state fixtures can be repeated without applying a desktop wallpaper. Original-asset coverage requires the local default 365 project and installed assets:

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3659880761"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
for lwe_scene_gpu_test in \
  scene::renderer::tests::offscreen_pixels_and_gl_state_are_preserved \
  scene::postprocess::tests::offscreen_bloom_halo_threshold_tint_and_state \
  scene::postprocess::tests::offscreen_original_bloom_and_effect_shader_assets
do
  EGL_PLATFORM=surfaceless LWE_SCENE_TEST_PROJECT="$lwe_scene_project" \
    LWE_SCENE_TEST_ASSETS="$lwe_scene_assets" \
    cargo test --locked -p lwe-engine --lib "$lwe_scene_gpu_test" \
    -- --ignored --exact --test-threads=1 --nocapture
done
```

These fixtures validate their pixel cases and GL-state restoration on the available EGL/GPU context. They do not compare captured original-engine frames or establish Wayland/multiple-output lifecycle behavior.

### Space 2026-10-10

The original Workshop [2.5D Space (`2903458386`)](https://steamcommunity.com/sharedfiles/filedetails/?id=2903458386) passes shared CPU preparation and real LWE application without rewriting its package or removing layers/effects: ten image layers, eleven effects and twenty-four passes. Eight Scroll instances, the audio Pulse/Chromatic passes and the four-pass project bloom remain in authored order. General bloom is disabled in this project; that does not disable its custom bloom. Project resources retain priority over installed builtins.

Fourteen unchanged scalar scripts passed seventy independent numeric cases across default, minimum, maximum, zero and mixed property snapshots. Oxc checks the module AST before bounded QuickJS evaluation on the CPU worker; no script callback runs on the renderer. Twelve original shader programs linked as GL 130. Real desktop-output monitor analysis separately passed stereo bass/treble, pause/resume, silence and cleanup checks. Its sixteen normalized bins per channel do not establish Windows frequency-cutoff or visual equivalence.

Private `space-runtime/gl-acceptance.json` passed all twenty-six GPU pixel comparisons: setting each of eight image layers to zero opacity changed more than a million pixels; six opacity groups, Chromatic/Pulse/Bloom switches, layer order and background color changed the complete scene as expected. Original-source composition modes 0/30/31 passed their pixel checks. With a nonzero spectrum, paused forced redraw and resize back to the original size retained identical pixels, and resume accepted a new spectrum. These EGL checks establish the authored paths and retained input; they do not compare a Windows engine screenshot. The separate production monitor-to-desktop flow also passed: with Global Speed set to zero, quiet frames and the return to silence were identical; an owned low-volume stereo tone changed the wallpaper, pause retained that audible frame through tone changes, and resume responded again. Pixel comparisons used the 2160×1090 wallpaper region below existing desktop widgets. The user's audio routes, configuration, original assets and workspace were preserved.

The real native Tauri flow in private `runtime/space-ui-delivery/report.json` passed on the recorded single-output Wayland + `niri`/AMD 680M machine: Library/monitor Apply reached Running, all twenty author controls (sixteen sliders, three booleans and one color) changed and saved, isolated PermissionDenied preserved the prior assignment/choices and draft, and retry atomically replaced the complete session. A new owned application process restored saved controls and wallpaper; Reset restored every author default. Settings 15 → 30 FPS, twelve-second owned fullscreen cover/reveal, target clear and owned cleanup also passed. Color used the rendered input event rather than the system picker. This finite UI flow does not measure physical suspend/lock or occlusion callback policy.

The final engine-only probe used a 30 FPS cap and frequent status requests: twelve seconds measured about 27.25 completed swaps/s, twelve seconds of settled pause had zero render attempts/draws/swaps/empty frames/callbacks, and resume measured about 27.33/s. Shared scratch targets by layer dimensions plus named/full-frame targets accounted for 202,180,100 logical GPU bytes. The configured cap is an upper bound, not guaranteed 30 FPS. Driver/EGL/video overhead and measured VRAM are excluded. Windows A/B, 24-hour endurance, ten real suspend/resume cycles, hardware battery transitions, measured power, independent multiple monitors and installed-package acceptance remain open.

### Yakuza extension 2026-10-12

This batch adds a bounded dynamic SceneScript worker alongside the existing pure scalar path. The accepted lifecycle is module initialization, `init`, initial declared-property delivery and repeated `update`; callbacks can use limited `Vec3`/`copy`, declared controls and layer `origin`/`scale`/`angles`/`color`/`alpha`/`visible`. An `init` return can update its bound property. `createLayer(path)` is limited to safe image-model JSON resources during initialization, with normal shared preparation/decoding; the rendering thread does not run a JavaScript VM. Authored object handles are reserved before assigning created handles. Validated worker state precedes first-frame ACK, paused forced redraws keep the accepted state, and the reversible apply/save/finalize boundary still owns failures and rollback. This subset does not implement config/handle overloads, arbitrary clones, destruction, scene queries, Vec2, engine time, cursor/media events or general interaction. Official [init](https://docs.wallpaperengine.io/en/scene/scenescript/reference/event/init.html), [update](https://docs.wallpaperengine.io/en/scene/scenescript/reference/event/update.html) and [scene APIs](https://docs.wallpaperengine.io/en/scene/scenescript/reference/class/IScene.html) describe a wider engine contract than LWE currently accepts.

Oxc validates source and AST before QuickJS, with at most 32 bindings, 64 KiB per source/256 KiB total, AST depth 48, 128 total authored/created handles, 8 MiB VM memory, 256 KiB stack and 100 ms initialization/10 ms frame execution. Prototype/global escapes, unvalidated computed writes, imports, unknown APIs and unsupported outputs fail closed. Runtime exceptions, deadline exhaustion, worker exit and cancellation must preserve the last durable assignment. CPU tests cover these finite rules; they do not replace GPU or native application acceptance.

`registerAudioBuffers(64)` is accepted only at module global scope; registration inside `init`, `update` or property callbacks is rejected. Its `average` is an actual fixed-length `Float32Array` refreshed in place from a genuine 64-band transform of the captured desktop output. The sixteen-bin stereo shader uniforms remain separate; expanding them would not produce 64 independent bins. LWE currently accepts only the average channel at this resolution, with silence on unavailable input and retained accepted input/state during pause or rollback. The official [audio buffers](https://docs.wallpaperengine.io/en/scene/scenescript/reference/class/AudioBuffers.html) expose additional channels and resolutions; Windows cutoff, level normalization and picture parity are unverified.

Combo declarations now preserve author labels and nonempty unique string option values, require the default and every override to be declared members, expose an accessible labelled select and persist strings in the same per-project/per-output choice snapshot. Exact `{name, condition}` combo equality bindings produce bool visibility. A mixed script/user wrapper extracts the script and validates its remaining user condition. Scalar shader mappings do not consume combo strings. The original Yakuza choices are `av`/`0` for `av`, four `pic` variants (`Majima_1`, `Kiryu_1`, `YakuzaDragon`, `Majima_2`) and `schemecolor`; invalid declarations/selections cannot silently substitute defaults.

Single-image `TEXS0002` particle atlases retain validated rectangular UV frames and sequence/random-frame selection, with supported adjacent-frame interpolation in sequence mode. Timed `TEXS0003`, multi-image atlases and use as ordinary image/effect textures remain gated. Added planar particle components include turbulent velocity, random alpha/angular velocity, angular movement, alpha/size oscillation and turbulence; finite simulation/geometry/work budgets still apply. Native retained framing/component evidence supports specific fields, while noise/random behavior and complete original-engine particle A/B remain unverified.

Yakuza's authored `perspective:true` bar is admitted only in the coplanar fixed-orthographic subset: origin Z=0, no X/Y rotation and no camera movement/parallax. Retained original-binary evidence supports canonical camera/projection behavior, but does not establish the per-layer perspective branch's extra mapping. Treat planar equivalence as a mathematical inference and retain the original-engine visual parity gap.

The shared `genericimage2` card adapter now preserves the legacy VERSION-absent Brightness/Alpha interface as well as explicit VERSION 1/2 Color4, without adding a renderer API or relaxing unknown-input/combo guards. The final GPU cases below verify original-card color/alpha/visibility; original-engine visual parity remains unverified. The final adapter also preserves the author's object color/alpha when VERSION is absent; this fixes the original owner bar's black tint without removing its handle. The earlier Space VERSION 2 record remains unchanged.

**Recorded finite acceptance:** private `yakuza/cpu-prepare-report.json` passes all eight original-package combinations: four `pic` backgrounds × `av`/`0`. Private `yakuza/gpu-report-final.json` passes five full-scene native offscreen OpenGL cases: each background with bars enabled, plus default background with bars disabled. These retain 64 bar handles, 63 initialization-created layers, original 4×4 bar material/texture, 66–67 render layers and 67–68 passes. All twenty-five pixel comparisons and fifteen worker destruction/recreation cycles passed, including original-card color, zero-alpha and hidden-layer assertions. Injected 64-band audio changed 8,070 isolated bar pixels in each enabled case and zero in the disabled case; twenty paused forced redraws and resize-back preserved the accepted frame. Injected-input GPU evidence does not establish the actual capture path.

Private `yakuza/ui-run-2/report.json` passes the final frozen build through the real `mockIpc=false` Tauri Library/Desktop flow on single `eDP-1`, Wayland + `niri`/AMD 680M. Private `yakuza/visual-review-final.json` records review of the full default and Kiryu scenes, restored controls and Chinese Settings layout. The author color label remains its original localization token; this does not establish complete translation of author labels. Native controls saved `av=0`, `pic=Kiryu_1` and color, while a real isolated PermissionDenied preserved session bytes/inode, editable draft and the prior Running renderer. Retry atomically saved the complete choices; a new process restored them, Reset removed overrides and restored all defaults while retaining the assignment. Settings 15 → 30 FPS, same-process close/hide, twelve-second owned fullscreen recovery and target clear passed; cleanup errors were empty and workspace 1 was restored. Color used the rendered input event, not the system picker. Private `yakuza/native-temporal-pixels-final.json` compares exact RGB8 pixels below desktop widgets (y≥300): default paired frames changed 1,297 pixels and paired frames after fullscreen recovery changed 1,848. This confirms temporal Scene-area changes, without assigning every changed pixel to a particular component. Fullscreen recovery does not measure callback occlusion pause or physical suspend.

Private `yakuza/gpu-report-final-live-audio.json` separately passes real PulseAudio/PipeWire PCM capture from an owned silent null-sink monitor → genuine 64-band FFT average → unchanged original SceneScript in LWE's worker → native offscreen bar GPU pixels. The renderer is AMD Radeon Graphics (`radeonsi`, `rembrandt`, ACO), OpenGL 4.6/Mesa 26.2.4. Playback changed 1,285 bar pixels; after pause/disconnect and resume/reconnect, responsive bars changed 690 pixels. Pause retained the displayed pixels exactly, and stopping playback returned exactly to the silent bar frame; initial and released 64-element buffers were zero. The captured source was verified as the owned sink monitor, capture and script workers released, the owned sink unloaded, and default sink/source remained unchanged. This proves the real capture/FFT/script/GPU chain under isolated routing. It uses offscreen pixels, excludes speaker output and default-device capture, and is separate from the passed visible desktop UI flow; it does not certify the user's current music/video route or Windows FFT/visual parity.

The final source/binary identity is retained privately in `yakuza/final-source-identity.json`; all 125 recorded Rust source/manifest/lock files match the frozen tree. Final workspace fmt/clippy/check, 466 ordinary Rust tests and the shell build passed; earlier frontend 154 tests and Python 144 regressions also passed. These source checks remain separate from runtime acceptance.

The final `yakuza/space-regression-gpu-report-final.json` additionally passes all twenty-six 2.5D Space GPU comparisons after this extension. The unchanged project retains ten layers, eleven effects, twenty-four passes, twenty controls and 202,180,100 logical GPU bytes. Original opacity/effect/order/composition, injected stereo spectrum and paused redraw/resize retention remain covered. This is a new deterministic offscreen GPU regression, preserving the older Space acceptance record; it does not add a new desktop/audio-capture run or Windows parity.

**Remaining acceptance:** Preserve original file hashes and private evidence, and keep earlier blockers as dated snapshots. Physical suspend/resume was explicitly deferred by the user because it interrupts Codex; this batch neither invokes it nor requires it for completion. The old 24-hour job was gracefully ended using verified owned PIDs to release the monitor, with its evidence retained; it is an interrupted run, not passed endurance. Physical multi-monitor, installed-package, Windows A/B, hardware battery/power, advanced scripts, bones and HDR remain separately unverified or unsupported. These finite results do not establish daily-use readiness.

## Reporting issues

Useful reports include:

- package type (`lwe`, `lwe-git`, `.deb`, `.rpm`, or `.AppImage`);
- distribution and version;
- session type and compositor/desktop environment;
- monitor layout;
- wallpaper type if known (`video`, `scene`, or `web`);
- compatibility status shown by LWE;
- copyable diagnostics from Settings, with the Steam Web API key redacted;
- terminal logs when available.

### Durable Scene reliability acceptance

Physical suspend/resume is explicitly deferred from the current batch at the user's request because sleep interrupts Codex. The following non-suspend recorder example uses zero required sleep cycles and cannot certify suspend recovery. The full future recovery target and helper remain documented separately.

Use `scripts/scene-reliability.py` with an immutable copy of the compiled `lwe-shell` library-test executable and its exact ignored test `services::desktop_service::reliability_tests::desktop_scene_daily_use_reliability`. Build the executable with `cargo test --locked -p lwe-shell --lib --no-run --message-format=json`; select the `compiler-artifact.executable` from that output. Preserve the full dirty-source fingerprint and original input files during the run. The recorder isolates `XDG_CONFIG_HOME`; retain `HOME` for Steam discovery.

Run with `LWE_REAL_DESKTOP_TESTS=1`, `--hours 24 --cycles 100 --suspend-cycles 0 --sample-seconds 5 --memory-growth-mib 64`, the actual `--binary`, `--project`, `--assets`, `--video`, and a new private `--output` directory outside Git. Set `--scene-ids 2903458386,3659880761,3337606455,3258032485` for the fixed cohort, and append `-- services::desktop_service::reliability_tests::desktop_scene_daily_use_reliability --ignored --exact --nocapture --test-threads=1`. The first project is the continuous primary; the others participate in finalized round trips. The recorder never requests sleep.

For a future task that includes physical recovery acceptance, after `suspend-readiness.json` reports a fresh healthy primary Scene, use `scripts/scene-suspend-cycles.py --check --readiness PATH` for read-only preflight. Privileged RTC cycles require an authorized local invocation with `--readiness PATH --output-dir NEW_PRIVATE_HELPER_DIRECTORY --cycles 10`. The helper preserves normal logind inhibitors and locking and waits for normal authenticated unlock. A root invocation must not overwrite an existing RTC alarm. Credentials must never appear in argv, environment, files, reports, or automation prompts.

A completed recorder's `passed=true` establishes only its recorded targets; a run with `--suspend-cycles 0` cannot establish physical recovery. Future recovery acceptance also needs real helper evidence. `report.json`, `manifest.json`, and append-only `evidence.jsonl` distinguish active presentation, complete switches, actual kernel sleep, resource growth, timing and cleanup. A running job, mock/unit test, wall-clock day, restart after failure, or source rebuild is not a passed 24-hour run.

`--finish-after-soak` optionally requests normal clear/shutdown/join once effective Scene presentation and finalized round trips reach their targets, even if a separately requested suspend target is incomplete. Such a partial result remains `passed=false` with recorder exit code 1, `status=completed`, a successful child exit, explicit completed/incomplete components and `stop_reason=soak_and_switch_targets_reached_suspend_incomplete`; it is not a runtime failure requiring a restart. This policy is part of the immutable manifest and cannot be changed with `--resume`. Interrupted reports record the actual signal. Fresh complete acceptance can pass; prior continuity segments remain separate and cannot be appended into an uninterrupted pass.
