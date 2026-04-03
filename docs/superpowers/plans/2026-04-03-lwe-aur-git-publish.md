# LWE AUR `lwe-git` Publish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a GitHub Actions pipeline that publishes Tauri v2-style AUR metadata for `lwe-git` to AUR on every push to `main`, with no GitHub Release publishing.

**Architecture:** Keep AUR packaging sources in-repo under `packaging/aur/lwe-git`, regenerate `.SRCINFO` from `PKGBUILD` in CI, then sync those two files into `aur@aur.archlinux.org:lwe-git.git`. The workflow exits without commit when nothing changed and fails fast on auth, generation, or push errors.

**Tech Stack:** GitHub Actions, Arch `makepkg`, PKGBUILD/.SRCINFO, SSH deploy key, Tauri v2 AUR packaging pattern

---

## File Map

- Create: `.github/workflows/aur-publish.yml`
  - Trigger on `push` to `main`, regenerate `.SRCINFO`, sync to AUR repo, and push.
- Create: `packaging/aur/lwe-git/PKGBUILD`
  - Define `lwe-git` AUR package using Tauri v2 official recommended `-git` package structure.
- Create: `packaging/aur/lwe-git/.SRCINFO`
  - Machine-readable metadata generated from `PKGBUILD`.
- Modify: `README.md`
  - Add a short section documenting AUR workflow purpose and required GitHub secrets.

### Task 1: Add AUR `PKGBUILD` Source File

**Files:**
- Create: `packaging/aur/lwe-git/PKGBUILD`

- [ ] **Step 1: Write the failing existence check**

Run:

```bash
test -f packaging/aur/lwe-git/PKGBUILD
```

Expected: FAIL (file does not exist yet).

- [ ] **Step 2: Create `PKGBUILD` with Tauri v2 `-git` package structure**

Create `packaging/aur/lwe-git/PKGBUILD` with this full content:

```bash
pkgname=lwe-git
pkgver=0.0.0
pkgrel=1
pkgdesc="Linux Wallpaper Engine companion app built with Tauri"
arch=('x86_64' 'aarch64')
url="https://github.com/YangYuS8/lwe"
license=('MIT')
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'libsoup' 'pango' 'webkit2gtk-4.1')
makedepends=('git' 'openssl' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'cargo' 'pnpm' 'nodejs')
provides=('lwe')
conflicts=('lwe')
source=("git+${url}.git")
sha256sums=('SKIP')

pkgver() {
  cd "${srcdir}/lwe"
  if git describe --long --abbrev=7 --tags >/dev/null 2>&1; then
    git describe --long --abbrev=7 --tags | sed 's/^v//;s/-/./g'
  else
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short=7 HEAD)"
  fi
}

prepare() {
  cd "${srcdir}/lwe"
  pnpm install --frozen-lockfile
}

build() {
  cd "${srcdir}/lwe"
  pnpm tauri build -b deb
}

package() {
  cd "${srcdir}/lwe/src-tauri"
  install -Dm755 "target/release/lwe-shell" "${pkgdir}/usr/bin/lwe-shell"
  install -Dm644 "../target/release/bundle/deb/lwe-shell/usr/share/applications/lwe-shell.desktop" "${pkgdir}/usr/share/applications/lwe-shell.desktop"
}
```

- [ ] **Step 3: Run syntax validation for `PKGBUILD`**

Run:

```bash
bash -n packaging/aur/lwe-git/PKGBUILD
```

Expected: PASS (no output).

- [ ] **Step 4: Commit PKGBUILD seed file**

Run:

```bash
git add packaging/aur/lwe-git/PKGBUILD
git commit -m "build: add lwe-git PKGBUILD source"
```

Expected: commit created.

### Task 2: Generate and Track `.SRCINFO`

**Files:**
- Create: `packaging/aur/lwe-git/.SRCINFO`
- Modify: `packaging/aur/lwe-git/PKGBUILD` (only if `makepkg --printsrcinfo` reveals missing metadata)

- [ ] **Step 1: Write the failing `.SRCINFO` existence check**

Run:

```bash
test -f packaging/aur/lwe-git/.SRCINFO
```

Expected: FAIL (file does not exist yet).

- [ ] **Step 2: Generate `.SRCINFO` from `PKGBUILD`**

Run:

```bash
cd packaging/aur/lwe-git && makepkg --printsrcinfo > .SRCINFO
```

Expected: PASS and `.SRCINFO` created.

- [ ] **Step 3: Verify `.SRCINFO` contains package identity and source**

Run:

```bash
rg "pkgbase = lwe-git|pkgname = lwe-git|source = git\+https://github.com/YangYuS8/lwe.git" packaging/aur/lwe-git/.SRCINFO
```

Expected: three matches found.

- [ ] **Step 4: Commit generated `.SRCINFO`**

Run:

```bash
git add packaging/aur/lwe-git/.SRCINFO packaging/aur/lwe-git/PKGBUILD
git commit -m "build: add generated AUR SRCINFO for lwe-git"
```

Expected: commit created.

### Task 3: Add GitHub Actions AUR Publish Workflow

**Files:**
- Create: `.github/workflows/aur-publish.yml`

- [ ] **Step 1: Write the failing workflow existence check**

Run:

```bash
test -f .github/workflows/aur-publish.yml
```

Expected: FAIL.

- [ ] **Step 2: Create workflow with push-main trigger and AUR sync**

Create `.github/workflows/aur-publish.yml` with this full content:

```yaml
name: Publish AUR lwe-git

on:
  push:
    branches:
      - main

concurrency:
  group: aur-lwe-git-main
  cancel-in-progress: false

jobs:
  publish-aur:
    runs-on: ubuntu-latest
    permissions:
      contents: read

    steps:
      - name: Checkout source
        uses: actions/checkout@v4

      - name: Install makepkg tools
        run: |
          sudo apt-get update
          sudo apt-get install -y pacman-package-manager

      - name: Regenerate .SRCINFO
        run: |
          cd packaging/aur/lwe-git
          makepkg --printsrcinfo > .SRCINFO

      - name: Setup SSH key
        uses: webfactory/ssh-agent@v0.9.0
        with:
          ssh-private-key: ${{ secrets.AUR_SSH_PRIVATE_KEY }}

      - name: Configure known hosts
        run: |
          mkdir -p ~/.ssh
          chmod 700 ~/.ssh
          printf "%s\n" "${{ secrets.AUR_KNOWN_HOSTS }}" >> ~/.ssh/known_hosts
          chmod 644 ~/.ssh/known_hosts

      - name: Clone AUR repo
        run: git clone "ssh://aur@aur.archlinux.org/lwe-git.git" aur-repo

      - name: Sync package files
        run: |
          cp packaging/aur/lwe-git/PKGBUILD aur-repo/PKGBUILD
          cp packaging/aur/lwe-git/.SRCINFO aur-repo/.SRCINFO

      - name: Commit and push if changed
        run: |
          cd aur-repo
          if git diff --quiet -- PKGBUILD .SRCINFO; then
            echo "No AUR metadata changes to publish"
            exit 0
          fi

          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add PKGBUILD .SRCINFO
          git commit -m "chore: update lwe-git to ${GITHUB_SHA::7}"
          git push origin HEAD:master
```

- [ ] **Step 3: Validate workflow YAML parses**

Run:

```bash
python - <<'PY'
import yaml
yaml.safe_load(open('.github/workflows/aur-publish.yml', 'r', encoding='utf-8'))
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 4: Commit workflow file**

Run:

```bash
git add .github/workflows/aur-publish.yml
git commit -m "ci: publish lwe-git metadata to AUR on main"
```

Expected: commit created.

### Task 4: Document Operation and Secrets

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write failing doc-content check**

Run:

```bash
rg "AUR|lwe-git|AUR_SSH_PRIVATE_KEY|AUR_KNOWN_HOSTS" README.md
```

Expected: no section documenting AUR workflow secrets.

- [ ] **Step 2: Add README section for AUR publishing flow**

Add a concise section to `README.md` stating:

```markdown
## AUR `lwe-git` Publish Workflow

The repository includes `.github/workflows/aur-publish.yml`, which publishes `PKGBUILD` and `.SRCINFO` to AUR package `lwe-git` on every push to `main`.

Required GitHub Actions repository secrets:

- `AUR_SSH_PRIVATE_KEY`: SSH private key with push access to `aur@aur.archlinux.org:lwe-git.git`
- `AUR_KNOWN_HOSTS`: known_hosts entry for `aur.archlinux.org`

This workflow publishes to AUR only and does not create GitHub Releases.
```

- [ ] **Step 3: Verify README now includes required keys and behavior**

Run:

```bash
rg "AUR `lwe-git` Publish Workflow|AUR_SSH_PRIVATE_KEY|AUR_KNOWN_HOSTS|does not create GitHub Releases" README.md
```

Expected: all matches present.

- [ ] **Step 4: Commit README documentation update**

Run:

```bash
git add README.md
git commit -m "docs: add AUR lwe-git workflow setup notes"
```

Expected: commit created.

### Task 5: End-to-End Verification

**Files:**
- Verify only: `packaging/aur/lwe-git/PKGBUILD`, `packaging/aur/lwe-git/.SRCINFO`, `.github/workflows/aur-publish.yml`, `README.md`

- [ ] **Step 1: Re-generate `.SRCINFO` and ensure clean git state after regeneration**

Run:

```bash
cd packaging/aur/lwe-git && makepkg --printsrcinfo > .SRCINFO
cd ../../..
git diff -- packaging/aur/lwe-git/.SRCINFO
```

Expected: no diff output.

- [ ] **Step 2: Validate all changed files are tracked and intentional**

Run:

```bash
git status --short
```

Expected: only intended AUR workflow/package/doc changes.

- [ ] **Step 3: Final verification commit (if this plan is executed without per-task commits)**

Run:

```bash
git add .github/workflows/aur-publish.yml packaging/aur/lwe-git/PKGBUILD packaging/aur/lwe-git/.SRCINFO README.md
git commit -m "feat: add automated AUR publish workflow for lwe-git"
```

Expected: commit created if no earlier commit strategy was used.

## Plan Self-Review

- Spec coverage check: includes in-repo PKGBUILD/.SRCINFO, push-on-main workflow, AUR-only publishing, secret handling, fail-fast behavior, and no-release scope.
- Placeholder scan: no `TODO`/`TBD` placeholders remain; each task has explicit file paths and commands.
- Consistency check: package name remains `lwe-git`, AUR remote remains `aur@aur.archlinux.org:lwe-git.git`, trigger remains `push` on `main`.
