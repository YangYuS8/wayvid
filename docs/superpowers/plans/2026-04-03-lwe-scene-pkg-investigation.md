# LWE Scene Package Investigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Determine the real structure of Wallpaper Engine `scene.pkg` files so LWE can stop building scene manifests from guessed top-level file paths and instead use package-derived facts.

**Architecture:** Start with evidence gathering from multiple real local subscribed samples using non-destructive structural inspection. If the package layout becomes clear enough, implement the smallest possible probe or reader in `lwe-library` that can identify package type, enumerate package entries, and locate the likely logical scene entry for later manifest synthesis.

**Tech Stack:** Rust, `lwe-library`, local Steam Workshop content, shell/file inspection tools, existing scene manifest code

---

## File Map

- Modify: `crates/lwe-library/src/scene_manifest.rs`
  - Only if the investigation establishes a more accurate scene-entry model or needs a temporary probe hook.
- Create: `crates/lwe-library/src/scene_package.rs`
  - Minimal package probe or index reader, only if the investigation reaches a stable enough format conclusion.
- Modify: `crates/lwe-library/src/lib.rs`
  - Re-export probe/index types if a package probe module is added.
- Create: `docs/superpowers/research/2026-04-03-scene-pkg-samples.md`
  - Record sample-by-sample evidence: workshop id, visible files, signatures, strings, and conclusions.
- Test: `crates/lwe-library/src/scene_package.rs`
  - Only if a package probe or reader is implemented.

## Real Sample Set

Use at least these already-discovered local subscribed scene projects as the investigation pool under `~/.local/share/Steam/steamapps/workshop/content/431960/`:

- `3337606455`
- `3578699777`
- `3232289987`
- `2798622011`
- `3241857956`

At least 2 to 3 must be inspected in detail before any parser assumptions are accepted.

### Task 1: Record Real Sample Evidence and Verify the Metadata Mismatch

**Files:**
- Create: `docs/superpowers/research/2026-04-03-scene-pkg-samples.md`

- [ ] **Step 1: Write the sample-evidence template file**

Create `docs/superpowers/research/2026-04-03-scene-pkg-samples.md` with one section per investigated sample. Use this structure:

```md
# Scene Package Sample Notes

## Sample 3337606455
- project.json file field:
- top-level files:
- scene.pkg file/magic result:
- obvious strings/header clues:
- scene.json visible at top level: yes/no
- likely conclusion:
```

- [ ] **Step 2: Inspect at least 3 real local samples non-destructively**

Run commands like:

```bash
file "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"/*
file "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3578699777"/*
file "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3232289987"/*
```

And record:

- `project.json.file`
- top-level files
- whether only `scene.pkg` exists beside `project.json`

- [ ] **Step 3: Add lightweight string/signature inspection for each sample**

Use minimal non-destructive probes such as:

```bash
strings "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455/scene.pkg" | rg "scene\.json|\.png|\.jpg|\.tex|material|shader" -n
strings "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3578699777/scene.pkg" | rg "scene\.json|\.png|\.jpg|\.tex|material|shader" -n
```

If `strings` output is noisy, record that explicitly.

- [ ] **Step 4: Write the findings into the research note**

Summarize for each sample whether:

- `scene.json` seems to exist only logically
- the package appears to expose filenames or embedded metadata
- the package format already resembles a known container

- [ ] **Step 5: Commit the recorded sample evidence**

Run:

```bash
git add docs/superpowers/research/2026-04-03-scene-pkg-samples.md
git commit -m "docs: record real scene package sample evidence"
```

### Task 2: Determine Whether `scene.pkg` Matches a Known Container Pattern

**Files:**
- Modify: `docs/superpowers/research/2026-04-03-scene-pkg-samples.md`

- [ ] **Step 1: Add a failing investigation checklist to the research note**

Append a short checklist section to the research note with unanswered questions:

```md
## Open Questions
- [ ] Is scene.pkg a known archive/container format?
- [ ] Does it contain a logical scene.json entry?
- [ ] Can entries be enumerated without full extraction?
```

- [ ] **Step 2: Probe the package signature and header bytes**

Run commands like:

```bash
xxd -l 64 "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455/scene.pkg"
xxd -l 64 "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3578699777/scene.pkg"
```

Record whether the header matches any obvious known format signature.

- [ ] **Step 3: Attempt safe identification with common archive tools only if the signature suggests it**

Examples:

```bash
bsdtar -tf "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455/scene.pkg"
unzip -l "$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455/scene.pkg"
```

Only use tools that make sense for the observed signature. Record failures exactly rather than guessing.

- [ ] **Step 4: Update the research note with a format conclusion**

Document one of:

- likely known container type
- custom container with visible entry metadata
- opaque binary package with no obvious enumerator yet

- [ ] **Step 5: Commit the format-investigation update**

Run:

```bash
git add docs/superpowers/research/2026-04-03-scene-pkg-samples.md
git commit -m "docs: characterize scene package container format"
```

### Task 3: Implement a Minimal Package Probe Only If Structure Is Clear Enough

**Files:**
- Create: `crates/lwe-library/src/scene_package.rs`
- Modify: `crates/lwe-library/src/lib.rs`
- Test: `crates/lwe-library/src/scene_package.rs`

- [ ] **Step 1: Write the failing package-probe tests**

Only proceed with this task if Tasks 1 and 2 produced enough evidence for a stable probe boundary.

Add tests such as:

```rust
#[test]
fn scene_package_probe_detects_known_package_signature() {
    let bytes = sample_pkg_bytes();
    let probe = ScenePackageProbe::from_bytes(&bytes).unwrap();
    assert_eq!(probe.format_name, "...");
}

#[test]
fn scene_package_probe_reports_unknown_signature_cleanly() {
    let error = ScenePackageProbe::from_bytes(b"not-a-scene-pkg").unwrap_err();
    assert!(error.to_string().contains("unknown"));
}
```

- [ ] **Step 2: Run the focused package-probe tests to verify they fail**

Run:

```bash
cargo test -p lwe-library scene_package -- --nocapture
```

Expected: FAIL because the probe module does not exist yet.

- [ ] **Step 3: Implement the smallest honest probe or indexer**

Only implement what the evidence supports. Examples of acceptable small outputs:

```rust
pub struct ScenePackageProbe {
    pub format_name: String,
    pub likely_entries: Vec<String>,
}
```

or

```rust
pub struct ScenePackageIndex {
    pub entries: Vec<String>,
    pub logical_scene_entry: Option<String>,
}
```

Do not invent unsupported parsing claims.

- [ ] **Step 4: Re-run the focused package-probe tests**

Run:

```bash
cargo test -p lwe-library scene_package -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the package probe**

Run:

```bash
git add crates/lwe-library/src/scene_package.rs crates/lwe-library/src/lib.rs
git commit -m "feat: add scene package probe"
```

### Task 4: Revise the Scene Manifest Assumption Based on Package Facts

**Files:**
- Modify: `crates/lwe-library/src/scene_manifest.rs`
- Modify: `crates/lwe-library/src/lib.rs` if exports change
- Test: `crates/lwe-library/src/scene_manifest.rs`

- [ ] **Step 1: Write the failing manifest test that encodes the new package-derived entry model**

Once the package investigation identifies a better real-entry model, add a failing test that proves `SceneManifest` no longer assumes a top-level filesystem file from `project.json.file`.

Example shape:

```rust
#[test]
fn scene_manifest_uses_package_derived_entry_model_when_top_level_scene_json_is_absent() {
    let manifest = SceneManifest::load(sample_scene_project_dir()).unwrap();
    assert_eq!(manifest.entry_file, expected_package_derived_entry_path());
}
```

If the final model should not use a plain `entry_file` at all, update the test and types accordingly.

- [ ] **Step 2: Run the focused manifest tests to verify they fail**

Run:

```bash
cargo test -p lwe-library scene_manifest -- --nocapture
```

Expected: FAIL until the old guessed entry-file assumption is replaced.

- [ ] **Step 3: Implement the minimal manifest revision**

Update `SceneManifest` only as far as the investigation supports. Acceptable outcomes include:

- replacing `entry_file` with package-derived entry metadata
- adding package/index-derived fields while keeping backward-compatible names temporarily

Do not overspecify unsupported package features.

- [ ] **Step 4: Re-run the focused manifest tests**

Run:

```bash
cargo test -p lwe-library scene_manifest -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the manifest revision**

Run:

```bash
git add crates/lwe-library/src/scene_manifest.rs crates/lwe-library/src/lib.rs
git commit -m "feat: base scene manifest on package facts"
```

### Task 5: Produce the Investigation Conclusion and Next-Step Decision

**Files:**
- Modify: `docs/superpowers/research/2026-04-03-scene-pkg-samples.md`

- [ ] **Step 1: Add a final conclusions section to the research note**

Summarize:

- what `scene.pkg` most likely is
- whether `scene.json` exists logically inside it
- whether a minimal probe or indexer was viable
- what should change next in `SceneManifest` and the native runtime plan

- [ ] **Step 2: Run the relevant automated checks for any implemented probe or manifest changes**

Run only what was touched, for example:

```bash
cargo test -p lwe-library -- --nocapture
```

Expected: PASS.

- [ ] **Step 3: Confirm the decision checkpoint explicitly**

End the note with one of these next-step decisions:

- implement a minimal `scene.pkg` reader/indexer next
- revise `SceneManifest` around package-derived entries next
- or document that the package format is still too opaque and why

- [ ] **Step 4: Commit the final investigation summary**

Run:

```bash
git add docs/superpowers/research/2026-04-03-scene-pkg-samples.md
git commit -m "docs: conclude scene package investigation"
```

- [ ] **Step 5: Stop and request the next implementation plan based on the investigation result**

Do not continue back into runtime implementation without first updating the design or plan to reflect what the package investigation actually discovered.

## Plan Self-Review

- Spec coverage check: the plan covers multi-sample evidence gathering, non-destructive package probing, optional minimal package-reader work only if justified, manifest-assumption revision, and an explicit decision checkpoint.
- Placeholder scan: no `TODO`, `TBD`, or unsupported “handle later” steps remain.
- Type consistency check: the plan consistently treats `SceneManifest` as the consumer of package-derived facts and keeps any `ScenePackageProbe`/`ScenePackageIndex` role investigative rather than prematurely general-purpose.
