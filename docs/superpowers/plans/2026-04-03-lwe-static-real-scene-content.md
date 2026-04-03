# LWE Static Real Scene Content Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the placeholder solid-color scene renderer with the first honest renderer that shows real static scene-derived content from the package-backed scene payload.

**Architecture:** Keep the package-backed `scene.pkg` runtime contract intact and add the smallest evidence-backed static-content derivation path that can produce a real renderable asset. Then wire `SceneSession` to upload and draw that content natively through the existing EGL/OpenGL path, and tighten compatibility so only scenes with a real static-content path are marked supported.

**Tech Stack:** Rust, `lwe-library`, `lwe-engine`, `apps/lwe/src-tauri`, Wayland/EGL/OpenGL, package-backed scene manifest/runtime target model

---

## File Map

- Create: `crates/lwe-library/src/scene_static_content.rs`
  - Narrow static-content derivation logic based on package-backed evidence only.
- Modify: `crates/lwe-library/src/lib.rs`
  - Re-export static-content derivation types if needed.
- Modify: `crates/lwe-library/src/scene_manifest.rs`
  - Extend the manifest only if it needs to carry a supported static-content source or its provenance.
- Modify: `crates/lwe-engine/src/scene.rs`
  - Replace `glClearColor` placeholder rendering with real static-content loading and drawing.
- Modify: `crates/lwe-engine/src/engine/mod.rs`
  - Keep first-frame success semantics aligned with the real static renderer path.
- Modify: `apps/lwe/src-tauri/src/results/workshop.rs`
  - Extend runtime assessment only if compatibility needs a structured static-content capability signal.
- Modify: `apps/lwe/src-tauri/src/services/compatibility_service.rs`
  - Mark scenes supported only when real static content can be derived.
- Modify: `apps/lwe/src-tauri/src/services/desktop_service.rs`
  - Only if desktop apply tests need tiny alignment for first-frame success semantics.
- Test: `crates/lwe-library/src/scene_static_content.rs`
  - Verify supported static-content derivation and clear unsupported cases.
- Test: `crates/lwe-engine/src/scene.rs`
  - Verify the placeholder color path is gone for supported scenes and real content is rendered.
- Test: `apps/lwe/src-tauri/src/services/compatibility_service.rs`
  - Verify compatibility matches static-content derivation availability.
- Test: `apps/lwe/src-tauri/src/services/desktop_service.rs`
  - Verify scene apply and restore success depend on the real static renderer path.

## Precondition

This plan assumes all package-backed runtime work in `.worktrees/scene-runtime-support` remains in place and green:

- `scene.pkg` is the canonical external runtime payload
- `SceneManifest` is package-backed
- compatibility/library/runtime-target/engine/desktop identity all align on `scene.pkg`

If that state drifts, restore it before starting this plan.

## Hard Boundary

Do **not** implement or imply:

- preview-image fallback
- solid-color placeholder fallback for “supported” scenes
- package entry enumeration not justified by current evidence

If deriving real static content requires general package internals, stop and write a new package-internals plan instead of guessing.

### Task 1: Define and Test the First Real Static-Content Derivation Rule

**Files:**
- Create: `crates/lwe-library/src/scene_static_content.rs`
- Modify: `crates/lwe-library/src/lib.rs`
- Modify: `crates/lwe-library/src/scene_manifest.rs` only if a tiny manifest extension is required
- Test: `crates/lwe-library/src/scene_static_content.rs`

- [ ] **Step 1: Write the failing static-content derivation tests**

Add tests that prove a scene package is only “supported for static rendering” when a real static content source can be derived from the package-backed model.

Add tests shaped like:

```rust
#[test]
fn static_scene_content_derives_real_content_source_for_supported_pkg_sample() {
    let manifest = supported_package_backed_manifest();
    let content = derive_static_scene_content(&manifest).unwrap();

    assert!(matches!(content, StaticSceneContent::ImageAsset { .. } | StaticSceneContent::PackagePayload { .. }));
}

#[test]
fn static_scene_content_rejects_scene_without_evidence_backed_content_source() {
    let manifest = package_backed_manifest_without_supported_static_content();
    let error = derive_static_scene_content(&manifest).unwrap_err();

    assert!(error.to_string().contains("static scene content"));
}
```

- [ ] **Step 2: Run the focused derivation tests to verify they fail**

Run:

```bash
cargo test -p lwe-library scene_static_content -- --nocapture
```

Expected: FAIL because the static-content derivation module does not exist yet.

- [ ] **Step 3: Implement the minimal static-content derivation module**

In `crates/lwe-library/src/scene_static_content.rs`, add the smallest honest derivation API the current evidence supports.

Example shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticSceneContent {
    ImageAsset { path: PathBuf },
}

pub fn derive_static_scene_content(manifest: &SceneManifest) -> Result<StaticSceneContent> {
    // Evidence-backed derivation only.
}
```

Do not add a fake fallback to preview images or colors.

- [ ] **Step 4: Re-run the focused derivation tests**

Run:

```bash
cargo test -p lwe-library scene_static_content -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the static-content derivation foundation**

Run:

```bash
git add crates/lwe-library/src/scene_static_content.rs crates/lwe-library/src/lib.rs crates/lwe-library/src/scene_manifest.rs
git commit -m "feat: derive static scene content from package payloads"
```

### Task 2: Replace the Placeholder Color Renderer with Real Static Content Rendering

**Files:**
- Modify: `crates/lwe-engine/src/scene.rs`
- Test: `crates/lwe-engine/src/scene.rs`

- [ ] **Step 1: Write the failing scene-renderer tests**

Add tests that explicitly reject the placeholder path and require real static content rendering for supported scenes.

Add tests shaped like:

```rust
#[test]
fn scene_session_supported_static_content_no_longer_uses_placeholder_clear_color_path() {
    let mut session = supported_static_scene_session();
    let rendered = session.render_frame().unwrap();
    assert!(rendered);
    assert!(session.first_frame_rendered);
    assert!(session.uses_real_static_content());
}

#[test]
fn scene_session_fails_before_first_frame_when_static_content_cannot_be_derived() {
    let error = SceneSession::new(project_dir(), unsupported_static_manifest(), output_info()).unwrap_err();
    assert!(error.to_string().contains("static scene content"));
}
```

- [ ] **Step 2: Run the focused engine tests to verify they fail**

Run:

```bash
cargo test -p lwe-engine scene -- --nocapture
```

Expected: FAIL because `SceneSession` still uses `stable_frame_color(...)` and does not know about real static content.

- [ ] **Step 3: Implement the minimal real static renderer**

Update `crates/lwe-engine/src/scene.rs` so `SceneSession`:

- resolves static content during setup or first render using the new derivation layer
- uploads that content into GPU resources
- draws the real content instead of clearing to a synthetic color
- reports first-frame success only after that real content is drawn

Do not add animation and do not use preview assets.

- [ ] **Step 4: Re-run the focused engine tests**

Run:

```bash
cargo test -p lwe-engine scene -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the real static scene renderer**

Run:

```bash
git add crates/lwe-engine/src/scene.rs
git commit -m "feat: render real static scene content"
```

### Task 3: Tighten Compatibility to the Static-Content Subset

**Files:**
- Modify: `apps/lwe/src-tauri/src/results/workshop.rs`
- Modify: `apps/lwe/src-tauri/src/services/compatibility_service.rs`
- Test: `apps/lwe/src-tauri/src/services/compatibility_service.rs`

- [ ] **Step 1: Write the failing compatibility tests**

Add tests that require compatibility to say “supported” only when real static content can be derived, not merely when `scene.pkg` is package-backed.

Example shape:

```rust
#[test]
fn compatibility_service_marks_scene_supported_only_when_static_content_is_derivable() {
    let assessed = CompatibilityService::assess_catalog_entry(scene_entry_with_supported_static_content());
    assert_eq!(assessed.compatibility.reason, CompatibilityReason::ReadyForLibrary);
}

#[test]
fn compatibility_service_rejects_package_backed_scene_without_supported_static_content() {
    let assessed = CompatibilityService::assess_catalog_entry(scene_entry_without_supported_static_content());
    assert_ne!(assessed.compatibility.level, CompatibilityLevel::FullySupported);
}
```

- [ ] **Step 2: Run the focused compatibility tests to verify they fail**

Run:

```bash
cargo test -p lwe-app-shell compatibility_service -- --nocapture
```

Expected: FAIL until compatibility is narrowed to the static-content subset.

- [ ] **Step 3: Implement the minimal compatibility alignment**

Make compatibility depend on the same static-content derivation rule as the renderer, without duplicating unsupported package internals.

- [ ] **Step 4: Re-run the focused compatibility tests**

Run:

```bash
cargo test -p lwe-app-shell compatibility_service -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the compatibility narrowing**

Run:

```bash
git add apps/lwe/src-tauri/src/results/workshop.rs apps/lwe/src-tauri/src/services/compatibility_service.rs
git commit -m "feat: gate scene support on static content derivation"
```

### Task 4: Verify Desktop Apply and Restore Depend on the Real Static Renderer Path

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/desktop_service.rs`
- Test: `apps/lwe/src-tauri/src/services/desktop_service.rs`

- [ ] **Step 1: Write the failing desktop tests**

Add tests that require scene apply success to depend on the real static renderer path rather than the old placeholder path.

Example shape:

```rust
#[test]
fn desktop_apply_flow_reports_success_only_after_real_static_scene_frame() {
    // assert success path is tied to renderer-backed frame success
}

#[test]
fn desktop_apply_flow_restore_reuses_real_static_scene_content_path() {
    // assert restore uses the same static scene content path and succeeds
}
```

- [ ] **Step 2: Run the focused desktop tests to verify they fail**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: FAIL until the desktop-side tests and semantics reflect the real static renderer path.

- [ ] **Step 3: Implement the minimal desktop alignment**

Only make desktop-side changes if needed to align the success barrier or restore expectations with the new renderer semantics.

- [ ] **Step 4: Re-run the focused desktop tests**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the desktop renderer-path alignment**

Run:

```bash
git add apps/lwe/src-tauri/src/services/desktop_service.rs
git commit -m "feat: align scene desktop flow with static renderer"
```

### Task 5: Real-Sample Validation and Stop Condition

**Files:**
- Modify only if tiny test/fixture/documentation notes are needed

- [ ] **Step 1: Run the relevant automated verification set**

Run:

```bash
cargo test -p lwe-library -- --nocapture
cargo test -p lwe-engine -- --nocapture
cargo test -p lwe-app-shell -- --nocapture
```

Expected: PASS.

- [ ] **Step 2: Validate against one real local subscribed scene sample**

Use a real sample already observed in the local workshop directory. Verify:

1. it appears in Library
2. apply succeeds
3. desktop shows real static content
4. the content is not a solid color
5. the content is not the preview image
6. restore succeeds

- [ ] **Step 3: Explicitly confirm out-of-scope boundaries still hold**

Document or report that the implementation still does not claim:

- animation support
- preview-image fallback
- generalized package entry enumeration

- [ ] **Step 4: Stop if real static content still cannot be derived honestly**

If a real static-content path requires invented package internals, stop and create a new package-internals design instead of guessing.

- [ ] **Step 5: Commit the real static scene-content phase**

Run:

```bash
git add .
git commit -m "feat: render real static scene content"
```

## Plan Self-Review

- Spec coverage check: the plan covers static-content derivation, replacement of the placeholder renderer, compatibility tightening, desktop success semantics, and real-sample validation.
- Placeholder scan: no `TODO`, `TBD`, or unsupported “figure this out later” steps remain.
- Type consistency check: the plan consistently preserves the package-backed runtime contract and treats static scene content as a renderer input derived from that contract, not from preview images or invented package enumeration.
