# Product Foundation and Repository Reset Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reset the current `wayvid` repository into a clean foundation for the new Linux dynamic wallpaper platform, with updated OpenSpec metadata, rewritten product documentation, a clear retention/removal boundary for legacy assets, and a minimal validated shell for follow-on feature plans.

**Architecture:** This plan treats the current repo as a migration source rather than a trusted foundation. It first rewrites product metadata and documentation around the new platform definition, then removes stale product framing and codifies which technical assets are retained for future implementation. It intentionally does not implement the Workshop runtime or compatibility engine yet; those become follow-on plans once the repo's identity and boundaries are corrected.

**Tech Stack:** Rust workspace, Cargo, Markdown docs, OpenSpec, YAML config, Git

---

## Scope Note

The approved design spans several independent subsystems: repository reset, naming/branding, Workshop loop, compatibility model, runtime architecture, and first-release UI. That is too broad for one safe implementation plan. This file is therefore the first plan in a series and focuses on **repository reset and product foundation** only.

Follow-on plans should cover at least:

1. Workshop discovery/acquisition/sync loop
2. Compatibility model and content-type analysis
3. Runtime architecture for `video + scene`
4. First-release application shell and navigation

## File Map

### Files to create

- `docs/superpowers/plans/2026-03-27-product-foundation-reset.md` - this implementation plan
- `docs/archive/legacy-wayvid-summary.md` - compact archive note describing what was intentionally retired
- `docs/product/overview.md` - new top-level product narrative for the replacement platform
- `docs/product/roadmap.md` - first-release scope and deferred scope summary
- `docs/product/repository-reset.md` - retained vs discarded asset inventory

### Files to modify

- `README.md` - replace old product framing with the new platform story and repo-reset status
- `openspec/config.yaml` - define the new OpenSpec project context and artifact rules
- `docs/README.md` - replace old mdBook-centric guidance with the new documentation structure and reset policy
- `Cargo.toml` - add a workspace comment block that marks the repo as in transition and identifies retained crates as migration candidates, without renaming crates yet
- `.gitignore` - add archive/build entries if needed for the new docs structure

### Files to delete or move

- Legacy product docs under `docs/src/**` that still describe the old `wayvid` product story
- Legacy translation artifacts under `docs/po/**` if they only serve the retired mdBook documentation tree
- Old generated/built documentation paths under `docs/book/**` if present in the working tree

Deletion should happen only after the replacement docs above exist.

### Files to inspect while implementing

- `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`
- `openspec/changes/**`
- `crates/wayvid-engine/**`
- `crates/wayvid-library/**`
- `crates/lwe-core/**`

These are the primary sources for deciding what is migrated vs retired.

## Task 1: Rewrite OpenSpec Project Identity

**Files:**
- Modify: `openspec/config.yaml`
- Test: verify with `openspec-cn list --json`

- [ ] **Step 1: Replace `openspec/config.yaml` with a new project context**

Write this exact file content:

```yaml
schema: spec-driven

context: |
  Product: A new Linux dynamic wallpaper platform replacing the current wayvid direction.
  Primary audience: Wallpaper Engine migration users on Linux.
  Product shape: Desktop application first, not a daemon-first system.
  Core differentiator: In-app Workshop discovery, acquisition orchestration, synchronization awareness, import, compatibility visibility, and runtime support.
  First release focus: video + scene support, with web recognized but deferred for strong runtime support.
  Internationalization: Chinese and English must both be supported in first-release product surfaces.
  Repository policy: Treat existing code as migration candidates, not a trusted foundation. Prefer retaining proven low-level playback and import knowledge while aggressively removing stale product framing.

rules:
  proposal:
    - State how the change supports the Linux dynamic wallpaper platform rather than the legacy wayvid product story.
    - Include explicit non-goals for first-release scope.
  design:
    - Identify whether the change belongs to product foundation, Workshop loop, compatibility, runtime, or application shell.
    - Call out user-facing compatibility implications whenever content types or import paths are affected.
  tasks:
    - Break work into independently verifiable slices.
    - Explicitly mark any task that deletes legacy assets.
    - Include verification steps for i18n impact when user-facing text changes.
```

- [ ] **Step 2: Run a quick OpenSpec parse check**

Run: `openspec-cn list --json`
Expected: valid JSON output; no YAML parse errors.

- [ ] **Step 3: Verify the retained OpenSpec context is internally consistent**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
config = Path('openspec/config.yaml').read_text()
assert 'Wallpaper Engine migration users' in config
assert 'video + scene' in config
assert not (Path('openspec') / 'project.md').exists()
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 4: Commit**

```bash
git add openspec/config.yaml
git commit -m "docs(openspec): redefine project context for platform reset"
```

## Task 2: Replace Top-Level Product Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/README.md`
- Create: `docs/product/overview.md`
- Create: `docs/product/roadmap.md`
- Test: content verification via `python3`

- [ ] **Step 1: Rewrite `README.md` to explain the reset**

Replace the root README with a concise document that includes:

```md
# wayvid (Repository Reset in Progress)

This repository is being reset into a new Linux dynamic wallpaper platform focused on Wallpaper Engine migration users.

## Current Status

- The old `wayvid` product direction is being retired.
- The repository is keeping only high-value technical assets from the previous codebase.
- The new product blueprint is defined in `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`.

## Product Direction

The new product aims to provide:

- in-app Workshop browsing and acquisition orchestration
- compatibility visibility for Workshop content
- first-release focus on `video` and `scene`
- a polished Linux desktop application experience
- Chinese and English user-facing support

## What Remains Valuable

- low-level playback/runtime knowledge
- Workshop parsing/import knowledge
- selected shared types and Linux integration code

## What Is Changing

- product framing
- docs structure
- future application architecture
- first-release scope definition

## Planning Documents

- Product blueprint: `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`
- Repository reset plan: `docs/superpowers/plans/2026-03-27-product-foundation-reset.md`
```

- [ ] **Step 2: Replace `docs/README.md` with a documentation reset guide**

Write this structure:

```md
# Documentation Reset

This directory is being rebuilt around the new Linux dynamic wallpaper platform.

## Rules

- Do not add new docs under the retired `wayvid` product framing.
- Prefer focused product docs under `docs/product/`.
- Archive or remove stale docs after replacement material exists.

## Primary Docs

- `docs/product/overview.md`
- `docs/product/roadmap.md`
- `docs/product/repository-reset.md`
- `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`

## Language Policy

First-release product surfaces must support Chinese and English.

Documentation translation strategy can evolve separately, but product-facing wording must be written with localization in mind from the start.
```

- [ ] **Step 3: Create `docs/product/overview.md`**

Write a product overview with these sections:

```md
# Product Overview

## Mission

Build a Linux dynamic wallpaper platform that serves Wallpaper Engine migration users with a polished desktop application experience.

## First-Release Focus

- Workshop-centered discovery and acquisition loop
- compatibility visibility
- `video` and `scene` runtime focus
- library-first daily use
- bilingual user-facing support

## Non-Goals for the First Release

- full `web` runtime parity
- creator tools
- cloud/community systems
- advanced automation rules
```

- [ ] **Step 4: Create `docs/product/roadmap.md`**

Write a roadmap with three explicit phases:

```md
# Product Roadmap

## Phase 1: Repository Reset

- rewrite product metadata
- replace legacy docs
- identify retained technical assets

## Phase 2: Workshop and Compatibility Foundation

- Workshop browsing/acquisition orchestration
- content-type recognition
- compatibility levels

## Phase 3: First-Release Application

- library-first application shell
- desktop integration
- `video + scene` runtime support
- Chinese/English product surfaces
```

- [ ] **Step 5: Verify top-level docs reference the same product identity**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
paths = [
    Path('README.md'),
    Path('docs/README.md'),
    Path('docs/product/overview.md'),
    Path('docs/product/roadmap.md'),
]
needles = ['Linux dynamic wallpaper platform', 'Wallpaper Engine', 'video']
for p in paths:
    text = p.read_text()
    for needle in needles:
        assert needle in text, (p, needle)
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 6: Commit**

```bash
git add README.md docs/README.md docs/product/overview.md docs/product/roadmap.md
git commit -m "docs: replace legacy product docs with reset narrative"
```

## Task 3: Inventory Retained and Discarded Assets

**Files:**
- Create: `docs/product/repository-reset.md`
- Create: `docs/archive/legacy-wayvid-summary.md`
- Modify: `Cargo.toml`
- Test: verify references with `python3`

- [ ] **Step 1: Create `docs/product/repository-reset.md`**

Write this content structure:

```md
# Repository Reset Inventory

## Retain as Migration Candidates

- `crates/wayvid-engine/` - low-level playback/runtime knowledge
- `crates/wayvid-library/` - Workshop parsing/import knowledge and library mechanics
- `crates/lwe-core/` - only shared domain or protocol pieces still useful after review

## Retire or Re-evaluate Aggressively

- old GUI framing under `crates/wayvid-gui/`
- old CLI/app product framing under `crates/wayvid-ctl/`
- daemon-vs-GUI transition concepts
- duplicated configuration models
- old docs and release story

## Review Criteria

- keep code only if it contributes directly to the new product blueprint
- prefer extracting proven low-level behavior over preserving old module boundaries
- delete stale product framing once replacement docs exist
```

- [ ] **Step 2: Create `docs/archive/legacy-wayvid-summary.md`**

Write a compact archive note:

```md
# Legacy wayvid Summary

The previous `wayvid` repository direction centered on a video-wallpaper application with an evolving GUI-first architecture.

That product framing is now retired.

The repository reset keeps technical knowledge that may still be valuable, but no longer treats the previous UI, docs, or product story as the basis for future work.
```

- [ ] **Step 3: Add a transition note to `Cargo.toml` workspace comments**

Replace the current opening comments with:

```toml
# Workspace Transition Note
# =========================
#
# This workspace is in repository-reset mode.
#
# The legacy `wayvid` product framing is being retired in favor of a new
# Linux dynamic wallpaper platform. Existing crates remain in place as
# migration candidates while follow-on plans decide what is retained,
# extracted, rewritten, or removed.
```

Keep the existing `[workspace]` section and members unchanged in this task.

- [ ] **Step 4: Verify the inventory docs and workspace note agree on retained assets**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
text = Path('docs/product/repository-reset.md').read_text()
cargo = Path('Cargo.toml').read_text()
for needle in ['wayvid-engine', 'wayvid-library', 'migration candidates']:
    assert needle in text or needle in cargo, needle
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 5: Commit**

```bash
git add docs/product/repository-reset.md docs/archive/legacy-wayvid-summary.md Cargo.toml
git commit -m "docs: define retained and retired assets for repo reset"
```

## Task 4: Remove Stale Documentation Trees Safely

**Files:**
- Delete: legacy files under `docs/src/**`
- Delete: legacy files under `docs/po/**` if still tied only to retired mdBook content
- Delete: legacy files under `docs/book/**` if present in the working tree
- Modify: `.gitignore` if new archive/build paths need coverage
- Test: path verification via `python3`

- [ ] **Step 1: Confirm replacement docs exist before deletion**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
required = [
    'docs/product/overview.md',
    'docs/product/roadmap.md',
    'docs/product/repository-reset.md',
    'docs/archive/legacy-wayvid-summary.md',
]
for rel in required:
    assert Path(rel).exists(), rel
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 2: Delete retired documentation trees**

Delete these paths if they exist and still only describe the retired product:

```text
docs/src/
docs/po/
docs/book/
```

If any file under those trees contains reusable technical material, move that content into a focused file under `docs/product/` before deleting the original file.

- [ ] **Step 3: Update `.gitignore` if needed**

If `docs/book/` or other generated paths were previously ignored by patterns that no longer apply cleanly, add a small documentation block such as:

```gitignore
# Legacy/generated docs artifacts
docs/book/
docs/po/
```

Only add entries that reflect actual retained ignore policy after the reset.

- [ ] **Step 4: Verify the retired trees are gone and the new docs remain**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
for retired in ['docs/src', 'docs/po', 'docs/book']:
    assert not Path(retired).exists(), retired
for kept in ['docs/product/overview.md', 'docs/product/roadmap.md', 'docs/product/repository-reset.md']:
    assert Path(kept).exists(), kept
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 5: Commit**

```bash
git add .gitignore docs
git commit -m "docs: remove retired documentation tree"
```

## Task 5: Add a Minimal Foundation Checkpoint

**Files:**
- Modify: `README.md`
- Modify: `docs/product/roadmap.md`
- Test: `cargo metadata`, `openspec-cn list --json`

- [ ] **Step 1: Add a clear checkpoint note to `README.md`**

Append this section near the end of `README.md`:

```md
## Foundation Checkpoint

The repository reset is complete when:

- OpenSpec reflects the new product direction.
- Legacy top-level docs have been replaced.
- Retained technical assets are documented.
- Follow-on plans can proceed without relying on the retired `wayvid` product framing.
```

- [ ] **Step 2: Add follow-on plan placeholders as roadmap items, not implementation placeholders**

Append this section to `docs/product/roadmap.md`:

```md
## Follow-on Planning Tracks

- Workshop loop plan
- Compatibility and content-type analysis plan
- Runtime architecture plan for `video + scene`
- Application shell and navigation plan
```

- [ ] **Step 3: Verify workspace and OpenSpec remain healthy after the reset**

Run:

```bash
cargo metadata --no-deps > /tmp/wayvid-metadata.json && openspec-cn list --json > /tmp/wayvid-openspec.json
```

Expected:

- `cargo metadata` exits successfully
- `openspec-cn list --json` exits successfully
- both output files are created

- [ ] **Step 4: Commit**

```bash
git add README.md docs/product/roadmap.md
git commit -m "chore: mark repository reset foundation complete"
```

## Self-Review Checklist

Before executing this plan, confirm:

- every requirement in the approved blueprint that belongs to repository reset appears in one of the tasks above
- no task implements Workshop runtime, compatibility engine logic, or application UI beyond documentation and planning boundaries
- all deletes happen only after replacement docs exist
- no task depends on a final product name being chosen
- OpenSpec, top-level docs, and repository inventory all describe the same product direction

## Expected Output of This Plan

When this plan is complete, the repository will have:

- a rewritten OpenSpec identity
- a rewritten top-level documentation story
- a documented retained/discarded asset inventory
- stale legacy docs removed
- a clean checkpoint for writing the next implementation plans
