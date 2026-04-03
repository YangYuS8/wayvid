# LWE AppImage Bundler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a stable AppImage packaging path for LWE using the Tauri v2 official bundler and the repository-root `logo.svg` as the source branding asset.

**Architecture:** Generate a proper Tauri icon set from `logo.svg`, wire Linux/AppImage bundle settings into `src-tauri/tauri.conf.json`, and verify the build through `cargo tauri build --bundles appimage`. Keep the scope to a stable official bundler path and avoid custom Linux packaging scripts.

**Tech Stack:** Tauri v2 bundler, pnpm, SvelteKit, Rust, Linux AppImage packaging, root-level app layout

---

## File Map

- Modify: `src-tauri/tauri.conf.json`
  - Add bundle/AppImage configuration and icon references.
- Create or update: `src-tauri/icons/*`
  - Generated icon assets consumed by Tauri bundler.
- Possibly create: `scripts/generate-icons.sh` or equivalent small helper only if needed
  - Only if generating the required icon assets reproducibly needs a tracked helper.
- Modify: `README.md` and/or packaging docs
  - Document the AppImage build command and any required host prerequisites.
- Test/Verify: AppImage output under Tauri bundle output directory

### Task 1: Establish a Reproducible Tauri Icon Asset Set from `logo.svg`

**Files:**
- Create/update: `src-tauri/icons/*`
- Optionally create: `scripts/generate-icons.sh`

- [ ] **Step 1: Write the failing icon-precondition checks**

Verify the expected Tauri icon inputs are not yet in place:

```bash
test -f src-tauri/icons/128x128.png
test -f src-tauri/icons/icon.icns
test -f src-tauri/icons/icon.ico
```

Expected now: at least one of these fails.

- [ ] **Step 2: Run the checks to confirm the icon set is incomplete**

Run:

```bash
test -f src-tauri/icons/128x128.png && test -f src-tauri/icons/icon.icns && test -f src-tauri/icons/icon.ico
```

Expected: FAIL.

- [ ] **Step 3: Generate the Tauri icon set from `logo.svg`**

Use the Tauri icon-generation path appropriate for the current toolchain. If a reproducible helper script is needed, keep it tiny and root-relative.

The result should populate `src-tauri/icons/` with the icon assets needed by the bundler.

- [ ] **Step 4: Re-run the icon-precondition checks**

Run:

```bash
test -f src-tauri/icons/128x128.png && test -f src-tauri/icons/icon.icns && test -f src-tauri/icons/icon.ico
```

Expected: PASS.

- [ ] **Step 5: Commit the icon asset setup**

Run:

```bash
git add src-tauri/icons scripts/generate-icons.sh
git commit -m "chore: generate Tauri bundle icons from logo.svg"
```

### Task 2: Configure Tauri Bundler for Linux AppImage

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Write the failing bundle-config expectation**

Add or use a simple grep-based expectation that AppImage bundling is not yet explicitly configured:

```bash
rg '"bundle"|"targets"|"appimage"|"icon"' src-tauri/tauri.conf.json
```

Expected: missing or incomplete Linux/AppImage bundle configuration.

- [ ] **Step 2: Run the check to verify the bundle config is incomplete**

Run the command above and confirm AppImage-specific bundler settings are absent.

- [ ] **Step 3: Add the minimal official bundler configuration**

Update `src-tauri/tauri.conf.json` with bundle settings shaped like:

```json
"bundle": {
  "active": true,
  "targets": ["appimage"],
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ]
}
```

Keep the config minimal and official.

- [ ] **Step 4: Re-run the bundle-config expectation**

Run:

```bash
rg '"bundle"|"targets"|"appimage"|"icon"' src-tauri/tauri.conf.json
```

Expected: AppImage bundle settings are now present.

- [ ] **Step 5: Commit the Tauri bundler configuration**

Run:

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: configure Tauri AppImage bundling"
```

### Task 3: Verify AppImage Build and Classify Any External Blockers

**Files:**
- Modify: `README.md` and/or packaging docs only if build prerequisites need documentation

- [ ] **Step 1: Run the AppImage build command**

Run:

```bash
cargo tauri build --bundles appimage
```

Expected: either PASS with an AppImage artifact, or fail with a clearly classifiable external packaging dependency issue.

- [ ] **Step 2: Verify artifact output if the build succeeds**

Check for the AppImage artifact in the expected output directory, for example under:

```bash
test -d src-tauri/target/release/bundle/appimage || test -d target/release/bundle/appimage
```

If the artifact path differs, record the actual location.

- [ ] **Step 3: If the build fails, document the blocker honestly**

Classify the failure as one of:

- Tauri configuration error
- icon/asset generation error
- missing host packaging dependency

Document the specific blocker in the README or an active packaging note only if the issue is external and reproducible.

- [ ] **Step 4: Run the standard root verification after packaging changes**

Run:

```bash
pnpm exec svelte-kit sync && pnpm check && pnpm test
cargo test -p lwe-shell
```

Expected: PASS.

- [ ] **Step 5: Commit the packaging phase**

Run:

```bash
git add README.md docs src-tauri/tauri.conf.json src-tauri/icons scripts/generate-icons.sh
git commit -m "feat: add official AppImage bundling"
```

## Plan Self-Review

- Spec coverage check: the plan covers icon generation, Tauri AppImage bundle config, artifact build verification, and blocker classification.
- Placeholder scan: no `TODO`, `TBD`, or vague “fix packaging later” steps remain.
- Type consistency check: the plan consistently uses the root app layout, `src-tauri/tauri.conf.json`, and the official Tauri bundler path.
