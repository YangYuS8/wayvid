# LWE Makefile Entrypoints Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a minimal top-level `Makefile` that wraps the most common LWE development commands: install, dev, test, and check.

**Architecture:** The `Makefile` is a thin convenience layer only. It should delegate directly to the existing `pnpm` frontend commands and the active `lwe-shell` Rust test path without replacing the current scripts or introducing broader build/release logic.

**Tech Stack:** Make, pnpm, Cargo, Tauri

---

## File Map

### Files to create

- `Makefile` - top-level convenience entrypoints for install/dev/test/check

### Files to modify

- `docs/superpowers/specs/2026-03-30-lwe-makefile-entrypoints-design.md` - no modification required unless the implementation reveals a spec mismatch

## Task 1: Add Minimal Makefile Entrypoints

**Files:**
- Create: `Makefile`
- Test: `make -n install && make -n dev && make -n test && make -n check`

- [ ] **Step 1: Write the failing smoke check**

Run:

```bash
make -n install
```

Expected: FAIL with `No rule to make target 'install'` because the `Makefile` does not exist yet.

- [ ] **Step 2: Create the Makefile**

Write `Makefile` with exactly these targets:

```make
.PHONY: install dev test check

install:
	pnpm --dir  install

dev:
	cd  && cargo tauri dev

test:
	cargo test -p lwe-shell
	pnpm --dir  test

check:
	cargo test -p lwe-shell
	pnpm --dir  check
```

- [ ] **Step 3: Run dry-run verification**

Run:

```bash
make -n install && make -n dev && make -n test && make -n check
```

Expected: PASS and print the intended command lines without executing them.

- [ ] **Step 4: Run one real command path**

Run:

```bash
make check
```

Expected:

- `cargo test -p lwe-shell` passes
- `pnpm --dir  check` passes

- [ ] **Step 5: Commit**

```bash
git add Makefile
git commit -m "build: add lwe makefile entrypoints"
```

## Self-Review Checklist

- Spec coverage:
  - `install` → present
  - `dev` → present
  - `test` → present
  - `check` → present
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check: no release/build/package/legacy targets were added.

## Expected Output of This Plan

When this plan is complete, you can run:

- `make install`
- `make dev`
- `make test`
- `make check`

from the repository root to drive the active LWE shell workflow.
