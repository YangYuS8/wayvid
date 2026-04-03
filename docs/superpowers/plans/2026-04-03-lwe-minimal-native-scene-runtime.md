# LWE Minimal Native Scene Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current scene-runtime stub with a real minimal native `SceneSession` that can render a supported subset of Wallpaper Engine scene projects and complete desktop apply or restore successfully.

**Architecture:** Keep the existing scene manifest, compatibility, runtime-target, and desktop apply wiring intact, and focus this implementation on `lwe-engine` as the primary work surface. The first scene runtime should validate a narrow supported subset during setup, create a real renderable scene session, and only report apply success after the first real rendered frame.

**Tech Stack:** Rust, `lwe-engine`, `lwe-library`, `src-tauri`, Wayland layer-shell, EGL/OpenGL

---

## File Map

- Modify: `crates/lwe-engine/src/scene.rs`
  - Replace the unsupported stub with a real minimal `SceneSession` implementation.
- Modify: `crates/lwe-engine/src/engine/session.rs`
  - Keep session lifecycle coherent for a real scene session.
- Modify: `crates/lwe-engine/src/engine/mod.rs`
  - Preserve first-frame apply-success semantics for scene sessions.
- Modify: `crates/lwe-engine/src/lib.rs`
  - Keep docs and exports aligned with the now-real scene runtime.
- Modify: `crates/lwe-library/src/scene_manifest.rs`
  - Extend manifest or validation only if the supported subset needs more structured data.
- Modify: `src-tauri/src/services/compatibility_service.rs`
  - Align “supported scene” with the actual minimal runtime subset.
- Modify: `src-tauri/src/results/workshop.rs`
  - Extend structured runtime assessment only if needed for subset support reasons.
- Modify: `src-tauri/src/services/desktop_service.rs`
  - Only if scene first-frame success/failure semantics require tiny integration updates.
- Test: `crates/lwe-engine/src/scene.rs`
  - Add setup/render tests for supported and unsupported scene runtime cases.
- Test: `crates/lwe-engine/src/engine/session.rs`
  - Verify a supported scene session can be created and an unsupported one fails at setup.
- Test: `crates/lwe-engine/src/engine/mod.rs`
  - Verify first-frame apply-success behavior remains correct for scenes.
- Test: `src-tauri/src/services/compatibility_service.rs`
  - Verify compatibility only reports supported for the real minimal subset.
- Test: `src-tauri/src/services/desktop_service.rs`
  - Verify supported scene apply and restore succeed; unsupported scenes fail before persistence success.

## Supported-Subset Rule for This Plan

This plan assumes the first runtime subset is deliberately narrow and must be enforced explicitly. The initial supported subset is:

- valid `scene` project
- resolvable scene entry file
- only local bundled assets
- no scripting requirement
- no explicitly unsupported feature markers in the scene data
- enough structured data to draw a stable output every frame

If a real local subscribed scene sample does not fit this subset, the runtime subset must be revised before claiming completion.

### Task 1: Replace the `SceneSession` Stub with a Real Minimal Runtime

**Files:**
- Modify: `crates/lwe-engine/src/scene.rs`
- Modify: `crates/lwe-engine/src/engine/session.rs`
- Test: `crates/lwe-engine/src/scene.rs`
- Test: `crates/lwe-engine/src/engine/session.rs`

- [ ] **Step 1: Write the failing scene-session setup tests**

Add tests that require a real setup-time distinction between supported and unsupported scene manifests. Use a narrow supported subset and keep the test fixture synthetic.

Add tests equivalent to:

```rust
#[test]
fn scene_session_initializes_for_minimal_supported_scene_manifest() {
    let session = SceneSession::new(
        PathBuf::from("/tmp/scene-project"),
        SceneRuntimeManifest {
            entry_file: PathBuf::from("/tmp/scene-project/scene.pkg"),
            required_assets: vec![PathBuf::from("/tmp/scene-project/scene.pkg")],
        },
        output_info(),
    )
    .unwrap();

    assert_eq!(session.output_name(), "DISPLAY-1");
}

#[test]
fn scene_session_rejects_manifest_outside_supported_subset_during_setup() {
    let error = SceneSession::new(
        PathBuf::from("/tmp/scene-project"),
        SceneRuntimeManifest {
            entry_file: PathBuf::from("/tmp/scene-project/scene.pkg"),
            required_assets: vec![],
        },
        output_info(),
    )
    .unwrap_err();

    assert!(error.to_string().contains("unsupported"));
}
```

- [ ] **Step 2: Run the focused engine tests to verify they fail**

Run:

```bash
cargo test -p lwe-engine engine_command_scene_targets_fail_explicitly_during_runtime_setup -- --nocapture
```

Expected: FAIL because `SceneSession::new(...)` is still the unconditional unsupported stub.

- [ ] **Step 3: Implement the minimal real `SceneSession`**

In `crates/lwe-engine/src/scene.rs`, replace the unsupported stub with a real setup-validating session type.

Keep the first real implementation deliberately small. For example, the session may store:

```rust
pub struct SceneSession {
    project_dir: PathBuf,
    manifest: SceneRuntimeManifest,
    output_info: OutputInfo,
    initialized: bool,
}
```

The setup path should:

- validate the supported subset
- reject unsupported manifests with a clear setup-time error
- return a live session for supported manifests

Do not leave render-time `not implemented yet` stubs in the success path.

- [ ] **Step 4: Re-run the focused scene-session tests**

Run:

```bash
cargo test -p lwe-engine scene_session -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the first real scene-session implementation**

Run:

```bash
git add crates/lwe-engine/src/scene.rs crates/lwe-engine/src/engine/session.rs
git commit -m "feat: add minimal native scene session"
```

### Task 2: Make the Minimal Scene Runtime Render a Real First Frame

**Files:**
- Modify: `crates/lwe-engine/src/scene.rs`
- Modify: `crates/lwe-engine/src/engine/mod.rs`
- Test: `crates/lwe-engine/src/scene.rs`
- Test: `crates/lwe-engine/src/engine/mod.rs`

- [ ] **Step 1: Write the failing first-frame render tests**

Add tests that require a supported scene session to produce a successful first-frame path rather than only “setup succeeded.” For the first pass, you may test the scene runtime at the session layer with a minimal deterministic draw path.

Add tests shaped like:

```rust
#[test]
fn scene_session_renders_a_first_frame_without_runtime_errors() {
    let mut session = supported_scene_session();
    let rendered = session.render_frame().unwrap();
    assert!(rendered);
}
```

And in engine-mod-level tests, require the apply-success barrier to use the first rendered frame for scene targets.

- [ ] **Step 2: Run the focused engine tests to verify they fail**

Run:

```bash
cargo test -p lwe-engine engine_command_dispatch -- --nocapture
```

Expected: FAIL because scene targets do not yet complete a real first-frame success path.

- [ ] **Step 3: Implement the minimal real render path**

In `crates/lwe-engine/src/scene.rs`, add the smallest real draw path for the supported subset. Keep it native and explicit.

The implementation should:

- use the existing EGL/GL path
- draw a stable frame for supported scene data
- return success once a real frame is produced
- avoid render-loop permanent error spam

If the first supported subset is effectively static, that is acceptable as long as it is rendered by the scene runtime path rather than a disguised video/image fallback.

- [ ] **Step 4: Re-run the focused engine tests**

Run:

```bash
cargo test -p lwe-engine engine_command_dispatch -- --nocapture
cargo test -p lwe-engine -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the first-frame scene runtime path**

Run:

```bash
git add crates/lwe-engine/src/scene.rs crates/lwe-engine/src/engine/mod.rs crates/lwe-engine/src/lib.rs
git commit -m "feat: render a first native scene frame"
```

### Task 3: Align Compatibility with the Actual Supported Scene Subset

**Files:**
- Modify: `crates/lwe-library/src/scene_manifest.rs` only if subset flags must be added
- Modify: `src-tauri/src/results/workshop.rs`
- Modify: `src-tauri/src/services/compatibility_service.rs`
- Test: `src-tauri/src/services/compatibility_service.rs`

- [ ] **Step 1: Write the failing compatibility-subset tests**

Add tests proving compatibility only marks scenes as supported when they fit the actual runtime subset, for example:

```rust
#[test]
fn compatibility_service_only_marks_minimal_supported_scene_subset_as_ready() {
    let assessed = CompatibilityService::assess_catalog_entry(minimal_supported_scene_entry());
    assert_eq!(assessed.compatibility.level, CompatibilityLevel::FullySupported);
}

#[test]
fn compatibility_service_rejects_scene_with_unsupported_runtime_feature() {
    let assessed = CompatibilityService::assess_catalog_entry(scene_entry_with_unsupported_feature());
    assert_ne!(assessed.compatibility.level, CompatibilityLevel::FullySupported);
}
```

- [ ] **Step 2: Run the focused compatibility tests to verify they fail**

Run:

```bash
cargo test -p lwe-app-shell compatibility_service -- --nocapture
```

Expected: FAIL until compatibility is aligned with the actual supported subset.

- [ ] **Step 3: Implement the minimal shared subset validation**

If the runtime subset needs one or two explicit manifest-level flags, add them in the smallest possible way and reuse them from compatibility. Do not duplicate scene-feature rules in multiple places without sharing a helper or structured assessment.

- [ ] **Step 4: Re-run the focused compatibility tests**

Run:

```bash
cargo test -p lwe-app-shell compatibility_service -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the compatibility alignment**

Run:

```bash
git add crates/lwe-library/src/scene_manifest.rs src-tauri/src/results/workshop.rs src-tauri/src/services/compatibility_service.rs
git commit -m "feat: align scene compatibility with runtime subset"
```

### Task 4: Prove Desktop Apply and Restore Use Real Scene First-Frame Success

**Files:**
- Modify: `src-tauri/src/services/desktop_service.rs`
- Test: `src-tauri/src/services/desktop_service.rs`

- [ ] **Step 1: Write the failing desktop scene apply and restore tests**

Add tests that require:

```rust
#[test]
fn desktop_apply_flow_persists_scene_assignment_only_after_first_real_scene_frame() {
    // assert success barrier uses a real scene frame, not command acceptance
}

#[test]
fn desktop_apply_flow_startup_restore_reapplies_supported_scene_assignment_successfully() {
    // assert restore completes via the same runtime-target path
}

#[test]
fn desktop_apply_flow_does_not_persist_unsupported_scene_as_success() {
    // assert unsupported setup failure prevents persistence success
}
```

- [ ] **Step 2: Run the focused desktop tests to verify they fail**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: FAIL until scene first-frame success semantics are honored end-to-end.

- [ ] **Step 3: Implement the minimal desktop/apply success alignment**

Update `desktop_service` only if the now-real scene runtime needs small success-barrier or error-path adjustments. Do not redesign the already-finished runtime-target model.

- [ ] **Step 4: Re-run the focused desktop tests**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the desktop scene success semantics**

Run:

```bash
git add src-tauri/src/services/desktop_service.rs
git commit -m "feat: complete scene apply and restore success flow"
```

### Task 5: Validate Against a Real Subscribed Scene Project

**Files:**
- Modify only if small test notes or fixture support is truly needed
- Primary work here is verification

- [ ] **Step 1: Identify one real subscribed scene sample on this machine that fits the initial supported subset**

Check the local Steam Workshop content under the Wallpaper Engine app id and confirm at least one scene project qualifies for the supported subset.

- [ ] **Step 2: Run the full automated verification set**

Run:

```bash
cargo test -p lwe-library -- --nocapture
cargo test -p lwe-engine -- --nocapture
cargo test -p lwe-app-shell -- --nocapture
```

Expected: PASS.

- [ ] **Step 3: Perform the real local manual verification**

Using a real subscribed scene sample already on this machine:

1. confirm compatibility shows it as supported
2. apply it to a monitor
3. confirm the desktop visibly changes
4. restart the app and confirm restore succeeds

- [ ] **Step 4: If no real subscribed sample fits the supported subset, stop and document the gap clearly**

Do not claim completion if the real local validation fails because the supported subset is too narrow.

- [ ] **Step 5: Commit the validated minimal native scene runtime**

Run:

```bash
git add .
git commit -m "feat: add minimal native scene runtime"
```

## Plan Self-Review

- Spec coverage check: this plan covers replacing the scene stub, real first-frame rendering, compatibility alignment with the actual subset, desktop apply/restore success semantics, and required real local verification.
- Placeholder scan: no `TODO`, `TBD`, or content-free “handle later” steps remain.
- Type consistency check: the plan keeps `SceneSession`, `SceneRuntimeManifest`, `RuntimeTarget::Scene`, and the existing desktop runtime-target model aligned throughout.
