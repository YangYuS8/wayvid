# AUR Package for wayvid

This directory contains PKGBUILD files for publishing wayvid on the Arch User Repository (AUR).

## Available Packages

### wayvid (stable) - **Pre-compiled Binary**
- **Package name**: `wayvid`
- **Distribution**: Pre-compiled binary package from GitHub Releases
- **Installation time**: **5-10 seconds** (no compilation required)
- **Recommended for**: Most users seeking fast installation
- **PKGBUILD**: `PKGBUILD.stable`
- **Note**: Downloads pre-built `.pkg.tar.zst` directly from GitHub

### wayvid-git (development) - Source Build
- **Package name**: `wayvid-git`
- **Source**: Latest git main branch (compiled from source)
- **Installation time**: ~5-15 minutes (full Rust compilation)
- **Recommended for**: Developers, testing new features, M6 features (Steam Workshop, Niri integration)
- **PKGBUILD**: `PKGBUILD`
- **Note**: Compiles from source for cutting-edge features and customization

## New Features (M6)

The git version includes:
- **Steam Workshop Integration**: Import Wallpaper Engine projects directly
- **Niri Compositor Support**: Workspace-aware FPS throttling for better performance
- **Enhanced CLI**: `wayvid workshop list/info/import` commands

## Building and Installing

### Prerequisites

Install required tools:
```bash
sudo pacman -S base-devel git
```

### Installation Methods

#### Option 1: Install from AUR (Recommended)

Using an AUR helper (e.g., `yay`):
```bash
# Stable version (pre-compiled binary, FAST!)
yay -S wayvid

# Development version (compiled from source)
yay -S wayvid-git
```

Using `makepkg` manually:
```bash
# Clone AUR repository
git clone https://aur.archlinux.org/wayvid.git
cd wayvid

# Install (downloads pre-compiled package)
makepkg -si
```

**Performance Comparison:**
- **wayvid (binary)**: Installation completes in **5-10 seconds**
- **wayvid-git (source)**: Installation takes **5-15 minutes** (requires full Rust compilation)

#### Option 2: Build Locally (for testing)

From this directory:
```bash
# For git version
makepkg -si

# For stable version
cp PKGBUILD.stable PKGBUILD
makepkg -si
```

### Clean Build

Remove build artifacts:
```bash
makepkg -c
```

## Publishing to AUR

### First-time Setup

1. Create an AUR account at https://aur.archlinux.org/register/

2. Add your SSH key:
   ```bash
   ssh-keygen -t ed25519 -C "your.email@example.com"
   cat ~/.ssh/id_ed25519.pub
   # Add to https://aur.archlinux.org/account/
   ```

3. Test SSH connection:
   ```bash
   ssh -T aur@aur.archlinux.org
   ```

### Publishing Process

#### For wayvid-git

1. Clone or create AUR repository:
   ```bash
   git clone ssh://aur@aur.archlinux.org/wayvid-git.git aur-wayvid-git
   cd aur-wayvid-git
   ```

2. Copy PKGBUILD:
   ```bash
   cp ../PKGBUILD .
   ```

3. Generate .SRCINFO:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

4. Test build:
   ```bash
   makepkg -si
   ```

5. Commit and push:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Update to version X.Y.Z"
   git push
   ```

#### For wayvid (stable) - Binary Package Mode

**Important**: The stable package uses pre-compiled binaries from GitHub Releases.

1. Clone AUR repository:
   ```bash
   git clone ssh://aur@aur.archlinux.org/wayvid.git aur-wayvid
   cd aur-wayvid
   ```

2. Copy PKGBUILD:
   ```bash
   cp ../PKGBUILD.stable PKGBUILD
   ```

3. Update version:
   ```bash
   # Update pkgver to match the new release
   sed -i 's/^pkgver=.*/pkgver=0.4.0/' PKGBUILD
   ```

4. Generate .SRCINFO:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. **No checksum update needed**: Binary package uses `sha256sums=('SKIP')` as the `.pkg.tar.zst` is downloaded from GitHub Release and verified by pacman.

6. Test installation (optional):
   ```bash
   makepkg -si
   ```

7. Commit and push:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Update to version X.Y.Z (pre-compiled binary)"
   git push
   ```

### Binary vs Source Package

**wayvid (stable)** is now distributed as a **binary package** for faster installation:
- **No compilation required**: Downloads pre-built `.pkg.tar.zst` from GitHub
- **No build dependencies**: Users don't need rust, cargo, or other build tools
- **Fast installation**: Completes in seconds instead of minutes
- **Automatic updates**: Handled by CI/CD on each release

**wayvid-git (development)** remains a **source package**:
- Compiles from latest git commit
- Requires full build environment (rust, cargo, etc.)
- Longer installation time but provides cutting-edge features

## Package Validation

### Local Testing

```bash
# Validate PKGBUILD
namcap PKGBUILD

# Build and check package
makepkg
namcap wayvid-*.pkg.tar.zst
```

### Check Dependencies

```bash
# List runtime dependencies
pactree -d1 wayvid

# Check for missing dependencies
ldd /usr/bin/wayvid
```

## Troubleshooting

### Build Fails

**Error**: `cargo: command not found`
```bash
sudo pacman -S rust
```

**Error**: Missing system libraries
```bash
sudo pacman -S wayland libmpv
```

**Error**: `Cargo.lock does not exist` (versions before 2025-01-18)
```bash
# This was fixed in commit 9a9a0a9
# Cargo.lock is now included in the repository
# Update to latest git version or wait for next release
git pull origin main
```

### Test Fails

Tests are currently minimal and skipped with `|| true` in the PKGBUILD.

### Installation Issues

**Error**: Conflicting packages
```bash
# Remove old version
sudo pacman -R wayvid wayvid-git

# Reinstall
makepkg -si
```

## Maintenance

### Version Updates

**For wayvid (stable - binary package):**
1. Update `pkgver` in PKGBUILD.stable
2. Reset `pkgrel` to 1 for new versions
3. Increment `pkgrel` for packaging changes (same version)
4. **No checksum update needed** (uses SKIP for binary packages)
5. Update `.SRCINFO` after any PKGBUILD changes
6. Push to AUR (handled automatically by CI/CD)

**For wayvid-git (source package):**
1. Update dependencies if needed
2. Increment `pkgrel` for packaging changes
3. `pkgver()` function handles version automatically
4. Update `.SRCINFO` after any PKGBUILD changes
5. Test build on clean system
6. Push to AUR (handled automatically by CI/CD for pre-releases)

### Release Checklist

- [ ] Update version numbers
- [ ] Update checksums (stable only)
- [ ] Test build on clean system
- [ ] Verify runtime dependencies
- [ ] Update .SRCINFO
- [ ] Commit and push to AUR

## Support

- **Issues**: https://github.com/YangYuS8/wayvid/issues
- **AUR Comments**: https://aur.archlinux.org/packages/wayvid/
- **Documentation**: https://github.com/YangYuS8/wayvid/tree/main/docs

## License

PKGBUILD files are in the public domain. wayvid itself is dual-licensed under MIT or Apache-2.0.
