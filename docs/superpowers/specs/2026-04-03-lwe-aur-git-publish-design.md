# LWE AUR `lwe-git` Publish Workflow Design

## Goal

Add a GitHub Actions workflow that publishes LWE development package metadata to AUR package `lwe-git` on every push to `main`, using the Tauri v2 official recommended AUR packaging approach. The pipeline publishes to AUR only and does not create GitHub Releases.

## Architecture

The solution has two layers:

1. **Source packaging metadata in this repository**
   - `packaging/aur/lwe-git/PKGBUILD`
   - `packaging/aur/lwe-git/.SRCINFO`
2. **CI sync pipeline**
   - `.github/workflows/aur-publish.yml`
   - Trigger: `push` on `main`
   - Action: regenerate `.SRCINFO`, sync files into AUR git repo `lwe-git`, commit, push

This keeps packaging definitions auditable in the main project while using AUR repo as the distribution endpoint.

## Packaging Components

### PKGBUILD

Use Tauri v2 docs-recommended AUR structure for `-git` packages:

- `pkgname=lwe-git`
- `source=("git+https://github.com/YangYuS8/lwe.git")`
- `sha256sums=('SKIP')`
- runtime deps include GTK/WebKit stack required by Tauri Linux apps
- makedepends include `cargo`, `nodejs`, `pnpm`, plus required Linux build packages

Version behavior:

- `pkgver()` derives version from git history (`git describe` with fallback format)
- `pkgrel=1`

Build behavior:

- `prepare()` installs frontend dependencies via `pnpm install --frozen-lockfile`
- `build()` runs Tauri build with deb bundle path recommended in official AUR page

### .SRCINFO

- Generated from `PKGBUILD` by `makepkg --printsrcinfo > .SRCINFO`
- Never manually hand-maintained in CI

## CI Data Flow

1. Trigger on `push` to `main`
2. Checkout repository
3. Install tools required for `makepkg --printsrcinfo`
4. Regenerate `packaging/aur/lwe-git/.SRCINFO`
5. Configure SSH agent with `AUR_SSH_PRIVATE_KEY`
6. Clone `aur@aur.archlinux.org:lwe-git.git`
7. Copy `PKGBUILD` and `.SRCINFO` into cloned AUR repo
8. Commit only when file content changed
9. Push to AUR

No release artifacts are uploaded to GitHub.

## Secrets and Security

- `AUR_SSH_PRIVATE_KEY`: private key allowed to push `lwe-git`
- `AUR_KNOWN_HOSTS`: pinned host key entry for `aur.archlinux.org`
- CI uses strict host checking and fails fast on auth or push failure

## Error Handling

- If `.SRCINFO` generation fails, workflow fails
- If AUR clone/auth fails, workflow fails
- If push fails due to race or permission issue, workflow fails with explicit error
- Use workflow `concurrency` to prevent overlapping pushes to the same target

## Testing and Verification

Repository-level verification:

- local/CI command: `makepkg --printsrcinfo > .SRCINFO`
- validate that generated `.SRCINFO` reflects `PKGBUILD`

Workflow-level verification:

- dry-run by manually triggering on branch (optional extension)
- production path: push to `main` and confirm AUR repo commit appears

Success criteria:

- each `main` push updates AUR `lwe-git` when packaging metadata changes
- no GitHub Release is created
- pipeline exits cleanly when no AUR file changes are detected

## Scope Boundaries

In scope:

- AUR-only publication pipeline
- Tauri v2 official AUR-style PKGBUILD layout
- automated `.SRCINFO` generation and sync

Out of scope:

- Publishing GitHub Releases
- Maintaining additional AUR package names
- Building/signing custom binary artifacts outside standard AUR flow
