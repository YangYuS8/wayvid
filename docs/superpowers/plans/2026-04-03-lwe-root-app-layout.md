# LWE Root App Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the single active LWE app from `apps/lwe` to the repository root and remove the redundant wrapper layer.

**Architecture:** Promote the frontend app files and `src-tauri/` directly into the repository root, update the Rust workspace and app-relative dependency/config paths, and then sweep all active scripts/docs/tooling off the old `apps/lwe` path. Leave the retained Rust crates under `crates/` untouched.

**Tech Stack:** pnpm, SvelteKit, Vite, Tauri 2, Rust workspace, Makefile, docs/specs/plans

---

## File Map

- Move: `apps/lwe/src` -> `src`
- Move: `apps/lwe/static` -> `static` (if present)
- Move: `apps/lwe/src-tauri` -> `src-tauri`
- Move: `apps/lwe/package.json` -> `package.json`
- Move: `apps/lwe/pnpm-lock.yaml` -> `pnpm-lock.yaml`
- Move: `apps/lwe/vite.config.ts` -> `vite.config.ts`
- Move: `apps/lwe/svelte.config.js` -> `svelte.config.js`
- Move: `apps/lwe/tsconfig.json` -> `tsconfig.json`
- Move: `apps/lwe/tailwind.config.ts` -> `tailwind.config.ts`
- Move: `apps/lwe/postcss.config.cjs` -> `postcss.config.cjs`
- Move: `apps/lwe/components.json` -> `components.json`
- Modify: `Cargo.toml`
  - Change workspace member from `apps/lwe/src-tauri` to `src-tauri`.
- Modify: `src-tauri/Cargo.toml`
  - Update `lwe-engine` / `lwe-library` relative paths.
- Modify: `Makefile`
  - Remove `apps/lwe` path prefixes.
- Modify: root docs and active plan/spec files that still reference `apps/lwe`
- Modify: any scripts, VS Code tasks, or GitHub workflows that hardcode `apps/lwe`
- Delete: `apps/lwe/` after all active files are moved and references updated

### Task 1: Move the Active App to the Root and Restore Basic Build/Test Commands

**Files:**
- Move: `apps/lwe/*` app files to root
- Move: `apps/lwe/src-tauri` to `src-tauri`
- Modify: `Cargo.toml`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Write the failing path-baseline checks**

Before moving files, record the old-path assumptions by checking these commands fail from the root without `apps/lwe` indirection:

```bash
test -f package.json
test -d src-tauri
pnpm check
```

Expected now: missing root app files / failing root-level frontend command.

- [ ] **Step 2: Run the checks to verify the current root layout is not yet app-root-ready**

Run:

```bash
test -f package.json && test -d src-tauri && pnpm check
```

Expected: FAIL because the app still lives under `apps/lwe`.

- [ ] **Step 3: Move the app files to the root and update workspace paths**

Move the entire active app payload out of `apps/lwe` into the root, then update:

```toml
# Cargo.toml
members = [
  "crates/lwe-core",
  "crates/lwe-engine",
  "crates/lwe-library",
  "src-tauri",
]
```

and:

```toml
# src-tauri/Cargo.toml
lwe-engine = { path = "../crates/lwe-engine" }
lwe-library = { path = "../crates/lwe-library" }
```

Do not refactor application code during the move.

- [ ] **Step 4: Run the root-level frontend and backend smoke checks**

Run:

```bash
pnpm install
pnpm check
cargo check -p lwe-app-shell
```

Expected: PASS, or fail only on remaining old-path references to be cleaned in later tasks.

- [ ] **Step 5: Commit the structural root move**

Run:

```bash
git add Cargo.toml src-tauri package.json pnpm-lock.yaml vite.config.ts svelte.config.js tsconfig.json tailwind.config.ts postcss.config.cjs components.json src static
git commit -m "refactor: move the LWE app to the repository root"
```

### Task 2: Update Build Scripts and Dev Workflow to the New Root Layout

**Files:**
- Modify: `Makefile`
- Modify: any path-sensitive root scripts under `scripts/`
- Modify: `src-tauri/tauri.conf.json` only if path assumptions changed

- [ ] **Step 1: Write the failing command-path expectations**

Add or update lightweight verification expectations for the new root commands, for example by asserting the Makefile no longer contains `apps/lwe`:

```bash
rg "apps/lwe" Makefile scripts src-tauri/tauri.conf.json
```

Expected: FAIL because stale path references still exist.

- [ ] **Step 2: Run the stale-path search to verify it fails**

Run:

```bash
rg "apps/lwe|pnpm --dir apps/lwe|cd apps/lwe" Makefile scripts src-tauri/tauri.conf.json
```

Expected: matches found.

- [ ] **Step 3: Update scripts/workflow commands to root-relative app commands**

Update `Makefile` to the new command model:

```make
install:
	pnpm install

dev:
	cargo tauri dev

test:
	cargo test -p lwe-app-shell
	pnpm test

check:
	cargo test -p lwe-app-shell
	pnpm exec svelte-kit sync
	pnpm check
```

Apply the same root-relative update to any touched scripts or Tauri config assumptions.

- [ ] **Step 4: Run the new root command checks**

Run:

```bash
make install
make check
```

Expected: PASS.

- [ ] **Step 5: Commit the script/workflow updates**

Run:

```bash
git add Makefile scripts src-tauri/tauri.conf.json
git commit -m "chore: update app workflows to the root layout"
```

### Task 3: Sweep Active Docs and Tooling Off the Old Path

**Files:**
- Modify: `README.md`
- Modify: active docs under `docs/`
- Modify: active superpowers specs/plans that still describe live `apps/lwe` paths
- Modify: `.vscode/` tasks/settings if they reference `apps/lwe`
- Modify: `.github/` workflows if they reference `apps/lwe`

- [ ] **Step 1: Find the active stale path references**

Run:

```bash
rg "apps/lwe|apps/lwe/src-tauri|pnpm --dir apps/lwe|cd apps/lwe" README.md docs .vscode .github
```

Expected: matches found in active docs/tooling.

- [ ] **Step 2: Run the search and confirm stale references exist**

Run the command above and use the results as the update list.

- [ ] **Step 3: Update active docs/tooling to root-relative paths**

Examples of expected path updates:

- `apps/lwe/src-tauri` -> `src-tauri`
- `apps/lwe/src/...` -> `src/...`
- `pnpm --dir apps/lwe test` -> `pnpm test`

Historical archived docs may remain unchanged if they are clearly archival and not teaching the live layout.

- [ ] **Step 4: Re-run the active stale-path search**

Run:

```bash
rg "apps/lwe|apps/lwe/src-tauri|pnpm --dir apps/lwe|cd apps/lwe" README.md docs .vscode .github
```

Expected: no matches in active/live docs/tooling.

- [ ] **Step 5: Commit the doc/tooling sweep**

Run:

```bash
git add README.md docs .vscode .github
git commit -m "docs: update active paths for the root app layout"
```

### Task 4: Remove the Wrapper Layer and Verify Nothing Depends on It

**Files:**
- Delete: `apps/lwe/` remaining contents
- Delete: `apps/` if empty and no longer needed

- [ ] **Step 1: Verify no active references to the old wrapper remain**

Run:

```bash
rg "apps/lwe|apps/lwe/src-tauri" .
```

Expected: only historical/archive references remain, or no relevant live references remain.

- [ ] **Step 2: Remove the now-redundant wrapper directory**

Delete `apps/lwe` after confirming all active files were moved and references updated. If `apps/` becomes empty, remove it too.

- [ ] **Step 3: Run full migration verification from the new root layout**

Run:

```bash
pnpm test
pnpm check
cargo test -p lwe-app-shell
```

Expected: PASS.

- [ ] **Step 4: Launch the dev workflow once from the new root layout**

Run:

```bash
cargo tauri dev
```

Expected: the app starts without needing `apps/lwe` prefixes.

- [ ] **Step 5: Commit the wrapper removal**

Run:

```bash
git add -A
git commit -m "refactor: remove the apps/lwe wrapper layer"
```

## Plan Self-Review

- Spec coverage check: the plan covers moving the app files, updating Rust workspace paths, fixing scripts/tooling, sweeping active docs, removing `apps/lwe`, and verifying all live workflows from the new root.
- Placeholder scan: no `TODO`, `TBD`, or vague “clean up references later” steps remain.
- Type consistency check: the plan consistently treats the root as the new sole app root and `src-tauri` as the direct workspace member.
