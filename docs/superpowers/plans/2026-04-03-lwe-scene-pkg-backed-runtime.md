# LWE Scene Package-Backed Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Shift scene runtime support to use `scene.pkg` as the authoritative external payload for supported scene projects, aligning manifest, runtime target identity, apply barriers, and status semantics with the investigated package-backed model.

**Architecture:** Keep the current narrow `PKGV00xx` signature/version evidence and package-backed manifest fallback, and push that model consistently through `lwe-library`, `lwe-engine`, and the Tauri backend. The runtime should stop depending on guessed top-level `scene.json` paths and treat `scene.pkg` as the external scene payload, while still refusing to invent internal package enumeration semantics.

**Tech Stack:** Rust, `lwe-library`, `lwe-engine`, `src-tauri`, Wayland/EGL runtime path, existing typed `RuntimeTarget`

---

## File Map

- Modify: `crates/lwe-library/src/scene_manifest.rs`
  - Preserve package-backed provenance and make `scene.pkg` the authoritative runtime payload path.
- Modify: `crates/lwe-library/src/lib.rs`
  - Re-export any adjusted manifest/provenance types if needed.
- Modify: `src-tauri/src/results/workshop.rs`
  - Keep runtime assessment/result types aligned with package-backed scene payloads.
- Modify: `src-tauri/src/services/compatibility_service.rs`
  - Align compatibility with package-backed acceptance rules instead of guessed entry-file rules.
- Modify: `src-tauri/src/services/library_service.rs`
  - Build `RuntimeTarget::Scene` from package-backed manifest facts.
- Modify: `crates/lwe-engine/src/scene.rs`
  - Make scene runtime identity package-backed rather than guessed-entry-backed.
- Modify: `crates/lwe-engine/src/engine/mod.rs`
  - Keep apply barrier/status identity aligned with the package-backed scene target.
- Modify: `crates/lwe-engine/src/engine/command.rs`
  - Only if status or target metadata names need alignment.
- Modify: `src-tauri/src/services/desktop_service.rs`
  - Keep apply barrier identity aligned with the package-backed scene target.
- Test: `crates/lwe-library/src/scene_manifest.rs`
  - Verify package-backed manifest provenance and failure semantics.
- Test: `src-tauri/src/services/library_service.rs`
  - Verify `RuntimeTarget::Scene` is package-backed.
- Test: `src-tauri/src/services/compatibility_service.rs`
  - Verify compatibility reflects package-backed acceptance/failure.
- Test: `crates/lwe-engine/src/scene.rs`
  - Verify scene runtime identity uses the package payload path.
- Test: `crates/lwe-engine/src/engine/mod.rs`
  - Verify apply barrier and status use the package-backed identity.
- Test: `src-tauri/src/services/desktop_service.rs`
  - Verify apply barrier identity and restore behavior stay aligned with the package-backed target.

### Task 1: Make the Manifest and Runtime Target Explicitly Package-Backed

**Files:**
- Modify: `crates/lwe-library/src/scene_manifest.rs`
- Modify: `src-tauri/src/services/library_service.rs`
- Test: `crates/lwe-library/src/scene_manifest.rs`
- Test: `src-tauri/src/services/library_service.rs`

- [ ] **Step 1: Write the failing package-backed manifest and runtime-target tests**

Add tests that require the package payload path, not the declared logical `scene.json`, to drive runtime identity.

Add tests shaped like:

```rust
#[test]
fn scene_manifest_marks_scene_pkg_as_the_runtime_payload_when_scene_json_is_only_logical() {
    let manifest = SceneManifest::load(sample_scene_project_dir()).unwrap();
    assert_eq!(manifest.entry_source, SceneEntrySource::InferredScenePackage);
    assert_eq!(manifest.entry_file.file_name().unwrap(), "scene.pkg");
}

#[test]
fn library_service_builds_runtime_target_scene_from_package_backed_manifest() {
    let target = LibraryService::runtime_target_from_refresh(&refresh_with_supported_scene(), "scene-42").unwrap();
    assert!(matches!(target, RuntimeTarget::Scene { manifest, .. } if manifest.entry_file.ends_with("scene.pkg")));
}
```

- [ ] **Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test -p lwe-library scene_manifest -- --nocapture
cargo test -p lwe-app-shell library_service -- --nocapture
```

Expected: FAIL if any path still depends on the logical `scene.json` assumption or hides package-backed provenance.

- [ ] **Step 3: Implement the minimal package-backed manifest/runtime-target alignment**

Keep `SceneManifest` honest about provenance, but ensure the runtime-facing path is clearly the package payload path.

Do not add package entry enumeration. The only accepted runtime payload should be the validated `scene.pkg` path already justified by the investigation.

- [ ] **Step 4: Re-run the focused tests**

Run:

```bash
cargo test -p lwe-library scene_manifest -- --nocapture
cargo test -p lwe-app-shell library_service -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the manifest/runtime-target package-backed shift**

Run:

```bash
git add crates/lwe-library/src/scene_manifest.rs src-tauri/src/services/library_service.rs
git commit -m "feat: make scene runtime targets package-backed"
```

### Task 2: Align Compatibility with the Package-Backed Acceptance Model

**Files:**
- Modify: `src-tauri/src/results/workshop.rs`
- Modify: `src-tauri/src/services/compatibility_service.rs`
- Test: `src-tauri/src/services/compatibility_service.rs`

- [ ] **Step 1: Write the failing compatibility tests**

Add tests proving compatibility is driven by package-backed acceptance rules rather than by guessed logical-entry availability:

```rust
#[test]
fn compatibility_service_marks_package_backed_scene_as_supported_when_pkg_signature_is_known() {
    let assessed = CompatibilityService::assess_catalog_entry(package_backed_scene_entry());
    assert_eq!(assessed.compatibility.reason, CompatibilityReason::ReadyForLibrary);
}

#[test]
fn compatibility_service_rejects_scene_when_scene_pkg_signature_is_unknown() {
    let assessed = CompatibilityService::assess_catalog_entry(unsupported_scene_pkg_entry());
    assert_ne!(assessed.compatibility.level, CompatibilityLevel::FullySupported);
}
```

- [ ] **Step 2: Run the focused compatibility tests to verify failure**

Run:

```bash
cargo test -p lwe-app-shell compatibility_service -- --nocapture
```

Expected: FAIL if compatibility still over- or under-claims support relative to the package-backed model.

- [ ] **Step 3: Implement the minimal compatibility alignment**

Keep the compatibility model grounded in what the package-backed manifest can honestly prove:

- known supported package-backed payload
- missing package payload
- unsupported package signature
- package-backed payload present but not acceptable for current runtime subset

- [ ] **Step 4: Re-run the focused compatibility tests**

Run:

```bash
cargo test -p lwe-app-shell compatibility_service -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the compatibility alignment**

Run:

```bash
git add src-tauri/src/results/workshop.rs src-tauri/src/services/compatibility_service.rs
git commit -m "feat: align scene compatibility with package-backed runtime"
```

### Task 3: Make Engine Scene Identity and Status Package-Backed

**Files:**
- Modify: `crates/lwe-engine/src/scene.rs`
- Modify: `crates/lwe-engine/src/engine/mod.rs`
- Modify: `crates/lwe-engine/src/engine/command.rs` only if needed for status naming clarity
- Test: `crates/lwe-engine/src/scene.rs`
- Test: `crates/lwe-engine/src/engine/mod.rs`

- [ ] **Step 1: Write the failing engine identity tests**

Add tests requiring scene runtime identity and apply barrier identity to use the package path:

```rust
#[test]
fn scene_runtime_identity_uses_scene_pkg_path() {
    let session = supported_scene_session_with_pkg_path();
    assert_eq!(session.target_path(), Some("/tmp/scene-project/scene.pkg"));
}

#[test]
fn engine_apply_barrier_uses_package_backed_scene_identity() {
    let target = supported_scene_runtime_target();
    let plan = apply_dispatch_plan(&target);
    assert_eq!(plan.pending_apply_path.as_deref(), Some(Path::new("/tmp/scene-project/scene.pkg")));
}
```

- [ ] **Step 2: Run the focused engine tests to verify failure**

Run:

```bash
cargo test -p lwe-engine scene -- --nocapture
cargo test -p lwe-engine engine_command_dispatch -- --nocapture
```

Expected: FAIL if any engine identity path still depends on a guessed logical entry.

- [ ] **Step 3: Implement the minimal package-backed engine identity alignment**

The engine should treat the package payload path as the authoritative scene identity for:

- `target_path()`
- apply barrier identity
- any scene-specific status identity exposed through engine events/state

Do not assume package entry enumeration.

- [ ] **Step 4: Re-run the focused engine tests**

Run:

```bash
cargo test -p lwe-engine scene -- --nocapture
cargo test -p lwe-engine engine_command_dispatch -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the engine identity alignment**

Run:

```bash
git add crates/lwe-engine/src/scene.rs crates/lwe-engine/src/engine/mod.rs crates/lwe-engine/src/engine/command.rs
git commit -m "feat: align scene engine identity with package payload"
```

### Task 4: Align Desktop Apply and Restore with the Package-Backed Scene Identity

**Files:**
- Modify: `src-tauri/src/services/desktop_service.rs`
- Test: `src-tauri/src/services/desktop_service.rs`

- [ ] **Step 1: Write the failing desktop apply/restore identity tests**

Add tests that require the desktop apply barrier and restore path to match the package-backed scene identity exactly:

```rust
#[test]
fn desktop_apply_flow_uses_scene_pkg_as_apply_barrier_identity() {
    let target = supported_scene_runtime_target();
    let barrier = DesktopService::apply_barrier_path(&target).unwrap();
    assert_eq!(barrier, PathBuf::from("/tmp/scene-project/scene.pkg"));
}

#[test]
fn desktop_apply_flow_startup_restore_reuses_package_backed_scene_identity() {
    // assert restore target resolution and apply barrier stay package-backed
}
```

- [ ] **Step 2: Run the focused desktop tests to verify failure**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: FAIL if any apply or restore path still diverges from package-backed identity.

- [ ] **Step 3: Implement the minimal desktop alignment**

Keep `desktop_service` strictly aligned to the package-backed target identity already established upstream.

Do not add package-internal parsing here.

- [ ] **Step 4: Re-run the focused desktop tests**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the desktop identity alignment**

Run:

```bash
git add src-tauri/src/services/desktop_service.rs
git commit -m "feat: align scene desktop apply with package identity"
```

### Task 5: Verify the Package-Backed Runtime Contract End-to-End and Stop Before Inventing Enumeration

**Files:**
- Modify only if tiny fixture or note updates are needed

- [ ] **Step 1: Run the full relevant automated verification**

Run:

```bash
cargo test -p lwe-library -- --nocapture
cargo test -p lwe-engine -- --nocapture
cargo test -p lwe-app-shell -- --nocapture
```

Expected: PASS.

- [ ] **Step 2: Check a real local subscribed scene sample against the package-backed runtime contract**

Using one real sample under `~/.local/share/Steam/steamapps/workshop/content/431960/`, verify that:

1. the manifest resolves it through `scene.pkg`
2. compatibility reflects the package-backed acceptance or rejection honestly
3. runtime target identity is package-backed

- [ ] **Step 3: Explicitly confirm what is still out of scope**

Record or report that the implementation still does **not** claim:

- package entry enumeration
- internal `scene.json` extraction
- full package structural parsing

- [ ] **Step 4: Stop if the runtime still needs package internals the investigation does not justify**

If a further renderer/runtime step requires true package entry enumeration, stop and create a new design/plan rather than guessing internal structure.

- [ ] **Step 5: Commit the package-backed runtime contract work**

Run:

```bash
git add .
git commit -m "feat: shift scene runtime to package-backed inputs"
```

## Plan Self-Review

- Spec coverage check: the plan covers package-backed manifest modeling, compatibility alignment, engine identity, desktop apply/restore identity, and an explicit stop condition before invented enumeration.
- Placeholder scan: no `TODO`, `TBD`, or vague “figure this out later” steps remain.
- Type consistency check: the plan consistently treats `scene.pkg` as the authoritative external payload while keeping package internals deliberately out of scope.
