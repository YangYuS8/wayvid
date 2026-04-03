# LWE Scene Runtime Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real desktop apply support for subscribed Wallpaper Engine `scene` items by introducing scene parsing, typed runtime targets, and a scene-capable engine path.

**Architecture:** Extend `lwe-library` with a scene manifest parser and structured scene parse failures, then replace the current desktop apply `PathBuf` assumption with a typed runtime target shared between the Tauri backend and `lwe-engine`. Preserve the current video/mpv path, but dispatch `scene` items through a sibling engine runtime path with explicit compatibility and failure reporting.

**Tech Stack:** Rust, Tauri 2, `lwe-library`, `lwe-engine`, `src-tauri`, existing workshop/catalog/compatibility services

---

## File Map

- Create: `crates/lwe-library/src/scene_manifest.rs`
  - Parse a Wallpaper Engine scene project into a typed manifest and structured parse errors.
- Modify: `crates/lwe-library/src/workshop.rs`
  - Reuse `WeProject` metadata and hook scene-manifest loading into project handling.
- Modify: `crates/lwe-library/src/lib.rs`
  - Re-export the new scene manifest types.
- Test: `crates/lwe-library/src/scene_manifest.rs` unit tests or adjacent test module
  - Verify supported scene parsing and explicit failure modes.
- Create: `crates/lwe-engine/src/scene.rs`
  - Hold the first scene runtime/session path in the engine crate.
- Modify: `crates/lwe-engine/src/lib.rs`
  - Export scene runtime types.
- Modify: `crates/lwe-engine/src/engine/command.rs`
  - Replace single-path wallpaper apply command payload with a typed runtime target.
- Modify: `crates/lwe-engine/src/engine/mod.rs`
  - Dispatch video and scene runtime targets to the correct session path.
- Modify: `crates/lwe-engine/src/engine/session.rs`
  - Support the scene runtime path alongside the existing mpv/video session logic.
- Test: `crates/lwe-engine/src/engine/command.rs` and `crates/lwe-engine/src/engine/mod.rs` tests
  - Verify command typing and dispatch behavior.
- Modify: `src-tauri/src/services/compatibility_service.rs`
  - Fold scene parse/runtime support into compatibility assessment.
- Modify: `src-tauri/src/services/library_service.rs`
  - Surface typed runtime inspection data for library items.
- Modify: `src-tauri/src/services/desktop_service.rs`
  - Replace `resolve_real_apply_path()` with typed runtime-target resolution and scene-aware engine apply.
- Modify: `src-tauri/src/results/workshop.rs`
  - Extend assessed metadata only if needed to surface scene support reasons cleanly.
- Modify: `src-tauri/src/assembly/*` and result modules only if needed
  - Preserve frontend-thin behavior while surfacing clearer support/failure states.
- Test: `src-tauri/src/services/desktop_service.rs`
  - Verify video target, scene target, and scene failure cases.
- Test: `src-tauri/src/services/compatibility_service.rs`
  - Verify supported vs unsupported scene compatibility outcomes.

## Fixture Guidance

Before implementation, identify one minimal real or trimmed `scene` fixture shape for tests. If repository-safe real fixtures are not available, create synthetic directory fixtures under test temp dirs that mimic:

- `project.json`
- a scene entry file referenced by `file`
- at least one required asset

Use temporary directories for fixture construction in unit tests unless a committed fixture directory is already available.

### Task 1: Add Scene Manifest Parsing in `lwe-library`

**Files:**
- Create: `crates/lwe-library/src/scene_manifest.rs`
- Modify: `crates/lwe-library/src/workshop.rs`
- Modify: `crates/lwe-library/src/lib.rs`
- Test: `crates/lwe-library/src/scene_manifest.rs`

- [ ] **Step 1: Write the failing scene-manifest tests**

Add tests that construct temporary project directories and assert:

```rust
#[test]
fn scene_manifest_parses_supported_scene_entry_and_assets() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("project.json"),
        r#"{ "type": "scene", "file": "scene.pkg", "title": "Forest Scene" }"#,
    )
    .unwrap();
    std::fs::write(temp.path().join("scene.pkg"), b"fake-scene").unwrap();

    let manifest = SceneManifest::load(temp.path()).unwrap();

    assert_eq!(manifest.entry_file, temp.path().join("scene.pkg"));
    assert!(manifest.required_assets.contains(&temp.path().join("scene.pkg")));
    assert!(manifest.unsupported_features.is_empty());
}

#[test]
fn scene_manifest_reports_missing_entry_file() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("project.json"),
        r#"{ "type": "scene", "file": "missing.pkg" }"#,
    )
    .unwrap();

    let error = SceneManifest::load(temp.path()).unwrap_err();

    assert!(matches!(error, SceneManifestError::MissingEntryFile { .. }));
}
```

- [ ] **Step 2: Run the parser tests to verify they fail**

Run:

```bash
cargo test -p lwe-library scene_manifest -- --nocapture
```

Expected: FAIL because `SceneManifest` and/or `SceneManifestError` do not exist yet.

- [ ] **Step 3: Implement the minimal scene manifest parser**

In `crates/lwe-library/src/scene_manifest.rs`, add a small parser surface like:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneManifest {
    pub project_dir: PathBuf,
    pub entry_file: PathBuf,
    pub required_assets: Vec<PathBuf>,
    pub cover_image: Option<PathBuf>,
    pub unsupported_features: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SceneManifestError {
    #[error("scene entry file is missing: {path}")]
    MissingEntryFile { path: PathBuf },
    #[error("project is not a scene project")]
    NotSceneProject,
    #[error("failed to load project metadata: {0}")]
    ProjectLoad(String),
}
```

Implement `SceneManifest::load(project_dir: &Path)` by reusing `WeProject::load`, validating `type == scene`, resolving `file`, checking it exists, and seeding `required_assets` with the entry file. Do not overbuild full recursive asset graph parsing in the first green step.

- [ ] **Step 4: Re-run the parser tests and the adjacent workshop tests**

Run:

```bash
cargo test -p lwe-library scene_manifest -- --nocapture
cargo test -p lwe-library workshop -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the library scene-manifest foundation**

Run:

```bash
git add crates/lwe-library/src/scene_manifest.rs crates/lwe-library/src/workshop.rs crates/lwe-library/src/lib.rs
git commit -m "feat: add scene manifest parsing"
```

### Task 2: Add Typed Runtime Targets to the Engine Boundary

**Files:**
- Create: `crates/lwe-engine/src/scene.rs`
- Modify: `crates/lwe-engine/src/lib.rs`
- Modify: `crates/lwe-engine/src/engine/command.rs`
- Modify: `crates/lwe-engine/src/engine/mod.rs`
- Modify: `crates/lwe-engine/src/engine/session.rs`
- Test: engine command/dispatch tests in `crates/lwe-engine/src/engine/*`

- [ ] **Step 1: Write the failing engine command typing tests**

Add tests that require a typed runtime target instead of a raw path:

```rust
#[test]
fn engine_command_can_carry_video_and_scene_runtime_targets() {
    let video = RuntimeTarget::Video {
        path: PathBuf::from("/tmp/video.mp4"),
    };
    let scene = RuntimeTarget::Scene {
        project_dir: PathBuf::from("/tmp/scene-project"),
        manifest: SceneRuntimeManifest {
            entry_file: PathBuf::from("/tmp/scene-project/scene.pkg"),
            required_assets: vec![PathBuf::from("/tmp/scene-project/scene.pkg")],
        },
    };

    let video_cmd = EngineCommand::ApplyWallpaper {
        target: video,
        output: Some("DISPLAY-1".to_string()),
    };
    let scene_cmd = EngineCommand::ApplyWallpaper {
        target: scene,
        output: Some("DISPLAY-1".to_string()),
    };

    assert!(matches!(video_cmd, EngineCommand::ApplyWallpaper { .. }));
    assert!(matches!(scene_cmd, EngineCommand::ApplyWallpaper { .. }));
}
```

- [ ] **Step 2: Run the focused engine tests to verify they fail**

Run:

```bash
cargo test -p lwe-engine engine_command -- --nocapture
```

Expected: FAIL because `RuntimeTarget` and scene runtime payloads do not exist yet.

- [ ] **Step 3: Implement the typed runtime target and minimal scene session path**

In `crates/lwe-engine/src/engine/command.rs`, replace the current path-only command payload with a typed target:

```rust
#[derive(Debug, Clone)]
pub enum RuntimeTarget {
    Video { path: PathBuf },
    Scene {
        project_dir: PathBuf,
        manifest: SceneRuntimeManifest,
    },
}

pub enum EngineCommand {
    ApplyWallpaper {
        target: RuntimeTarget,
        output: Option<String>,
    },
    // ...
}
```

In `crates/lwe-engine/src/scene.rs`, add the smallest scene runtime/session surface that can be initialized and fail explicitly if not supported yet. The first green step can be minimal as long as it is a distinct runtime path and not an mpv path masquerading as scene support.

- [ ] **Step 4: Re-run focused engine tests**

Run:

```bash
cargo test -p lwe-engine engine_command -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the engine runtime-target boundary**

Run:

```bash
git add crates/lwe-engine/src/scene.rs crates/lwe-engine/src/lib.rs crates/lwe-engine/src/engine/command.rs crates/lwe-engine/src/engine/mod.rs crates/lwe-engine/src/engine/session.rs
git commit -m "feat: add typed engine runtime targets"
```

### Task 3: Integrate Scene Parsing into Compatibility and Library Inspection

**Files:**
- Modify: `src-tauri/src/services/compatibility_service.rs`
- Modify: `src-tauri/src/services/library_service.rs`
- Modify: `src-tauri/src/results/workshop.rs` if structured metadata needs extension
- Test: `src-tauri/src/services/compatibility_service.rs`
- Test: `src-tauri/src/services/library_service.rs`

- [ ] **Step 1: Write the failing compatibility and library inspection tests**

Add tests for two concrete cases:

```rust
#[test]
fn compatibility_service_marks_supported_scene_with_valid_manifest_as_library_ready() {
    let entry = synced_scene_entry_with_realistic_project_dir();

    let assessed = CompatibilityService::assess_catalog_entry(entry);

    assert!(CompatibilityService::supports_library_projection(&assessed));
}

#[test]
fn library_service_can_build_scene_runtime_target_for_supported_scene() {
    let projection = projection_with_supported_scene();

    let target = LibraryService::runtime_target_in_projection(&projection, "scene-42").unwrap();

    assert!(matches!(target, RuntimeTarget::Scene { .. }));
}
```

- [ ] **Step 2: Run the focused backend tests to verify they fail**

Run:

```bash
cargo test -p lwe-shell compatibility_service -- --nocapture
cargo test -p lwe-shell library_service -- --nocapture
```

Expected: FAIL because scene runtime-target inspection does not exist yet.

- [ ] **Step 3: Implement minimal compatibility/runtime-target integration**

Add a small library-facing runtime-target resolver in `LibraryService`, for example:

```rust
pub fn runtime_target(item_id: &str) -> Result<RuntimeTarget, String> {
    let projection = Self::load_projection()?;
    Self::runtime_target_in_projection(&projection, item_id)
}
```

For `scene` items, load the scene manifest from `lwe-library` and convert supported manifests into `RuntimeTarget::Scene`. Unsupported or broken manifests should return clear reasons rather than silently dropping back to video.

- [ ] **Step 4: Re-run the focused backend tests**

Run:

```bash
cargo test -p lwe-shell compatibility_service -- --nocapture
cargo test -p lwe-shell library_service -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the compatibility/library integration**

Run:

```bash
git add src-tauri/src/services/compatibility_service.rs src-tauri/src/services/library_service.rs src-tauri/src/results/workshop.rs
git commit -m "feat: assess scene runtime support"
```

### Task 4: Replace Desktop Apply Path Resolution with Runtime-Target Dispatch

**Files:**
- Modify: `src-tauri/src/services/desktop_service.rs`
- Modify: `crates/lwe-engine/src/engine/command.rs` if command payload names need final alignment
- Test: `src-tauri/src/services/desktop_service.rs`

- [ ] **Step 1: Write the failing desktop-apply tests for video and scene targets**

Add tests for:

```rust
#[test]
fn desktop_apply_flow_resolves_video_item_into_video_runtime_target() {
    let target = DesktopService::resolve_runtime_target_from_entry(&supported_video_entry()).unwrap();
    assert!(matches!(target, RuntimeTarget::Video { .. }));
}

#[test]
fn desktop_apply_flow_resolves_scene_item_into_scene_runtime_target() {
    let target = DesktopService::resolve_runtime_target_from_entry(&supported_scene_entry()).unwrap();
    assert!(matches!(target, RuntimeTarget::Scene { .. }));
}

#[test]
fn desktop_apply_flow_reports_scene_manifest_failure_clearly() {
    let error = DesktopService::resolve_runtime_target_from_entry(&broken_scene_entry()).unwrap_err();
    assert!(error.contains("scene"));
}
```

- [ ] **Step 2: Run the focused desktop-service tests to verify they fail**

Run:

```bash
cargo test -p lwe-shell desktop_apply_flow -- --nocapture
```

Expected: FAIL because desktop apply is still path-only and video-only.

- [ ] **Step 3: Implement minimal runtime-target dispatch in `desktop_service.rs`**

Refactor `resolve_real_apply_path()` into a typed runtime-target resolver, then dispatch that target when sending the engine command:

```rust
fn resolve_real_runtime_target(item_id: &str) -> Result<RuntimeTarget, String> {
    LibraryService::runtime_target(item_id)
}

backend.handle.send(EngineCommand::ApplyWallpaper {
    target,
    output: Some(monitor.backend_output_id.clone()),
})
```

Do not preserve the old `WorkshopProjectType::Video` hard gate.

- [ ] **Step 4: Re-run the focused desktop-service tests**

Run:

```bash
cargo test -p lwe-shell desktop_apply_flow -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the desktop runtime-target apply path**

Run:

```bash
git add src-tauri/src/services/desktop_service.rs crates/lwe-engine/src/engine/command.rs
git commit -m "feat: dispatch scene runtime targets for desktop apply"
```

### Task 5: Validate Startup Restore and Real-World Scene Apply Paths

**Files:**
- Modify: `src-tauri/src/services/desktop_service.rs`
- Modify: tests in `src-tauri/src/services/desktop_service.rs`
- Modify: docs only if a small manual verification note is needed

- [ ] **Step 1: Write the failing restore/regression test for scene assignments**

Add a restore-path test proving a saved scene assignment is re-applied through the new typed runtime-target flow:

```rust
#[test]
fn desktop_apply_flow_startup_restore_reapplies_supported_scene_assignment() {
    let page = desktop_page_with_restorable_scene_assignment();
    let mut restored = Vec::new();

    DesktopService::restore_saved_assignments_with(&page, |monitor, item_id| {
        restored.push((monitor.id.clone(), item_id.to_string()));
        Ok(())
    });

    assert_eq!(restored, vec![("DISPLAY-1".to_string(), "scene-7".to_string())]);
}
```

- [ ] **Step 2: Run the focused restore tests to verify they fail if the scene path is not wired**

Run:

```bash
cargo test -p lwe-shell desktop_apply_flow_startup_restore -- --nocapture
```

Expected: FAIL until the typed scene path is actually used end-to-end.

- [ ] **Step 3: Implement the minimal final restore wiring and manual verification notes**

Ensure startup restore goes through the same runtime-target resolver used by direct apply. Do not add a separate scene-only restore path.

If helpful, add a short comment or doc note in the test module describing the expected local manual verification sample source (subscribed Workshop scene content under Steam).

- [ ] **Step 4: Run full backend verification and targeted local runtime checks**

Run:

```bash
cargo test -p lwe-library -- --nocapture
cargo test -p lwe-engine -- --nocapture
cargo test -p lwe-shell desktop_apply_flow -- --nocapture
```

Then perform a local manual check with a real subscribed scene project already on this machine:

1. open Library and confirm a real `scene` item appears supported
2. apply it to a monitor
3. confirm Desktop reflects the assignment
4. restart the app and confirm the assignment restores

- [ ] **Step 5: Commit the completed scene runtime support path**

Run:

```bash
git add crates/lwe-library/src/scene_manifest.rs crates/lwe-library/src/workshop.rs crates/lwe-library/src/lib.rs crates/lwe-engine/src/scene.rs crates/lwe-engine/src/lib.rs crates/lwe-engine/src/engine/command.rs crates/lwe-engine/src/engine/mod.rs crates/lwe-engine/src/engine/session.rs src-tauri/src/services/compatibility_service.rs src-tauri/src/services/library_service.rs src-tauri/src/services/desktop_service.rs src-tauri/src/results/workshop.rs
git commit -m "feat: add scene runtime desktop support"
```

## Plan Self-Review

- Spec coverage check: the plan covers scene parsing, typed runtime targets, engine/runtime dispatch, compatibility integration, desktop apply integration, restore behavior, and real-device verification.
- Placeholder scan: no `TODO`, `TBD`, or content-free “handle errors later” steps remain.
- Type consistency check: the plan consistently uses a `RuntimeTarget` boundary, `SceneManifest` parser output, and the existing package names `lwe-library`, `lwe-engine`, and `lwe-shell`.
