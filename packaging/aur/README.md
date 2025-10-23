# AUR Package for wayvid

This directory contains PKGBUILD files for publishing wayvid on the Arch User Repository (AUR).

## Available Packages

### wayvid (stable)
- **Package name**: `wayvid`
- **Source**: Official release tarballs from GitHub
- **Recommended for**: Most users
- **PKGBUILD**: `PKGBUILD.stable`

### wayvid-git (development)
- **Package name**: `wayvid-git`
- **Source**: Latest git main branch
- **Recommended for**: Testing new features
- **PKGBUILD**: `PKGBUILD`

## Building and Installing

### Prerequisites

Install required tools:
```bash
sudo pacman -S base-devel git
```

### Build from Source

#### Option 1: Install from AUR (once published)

Using an AUR helper (e.g., `yay`):
```bash
# Stable version
yay -S wayvid

# Development version
yay -S wayvid-git
```

Using `makepkg` manually:
```bash
# Clone AUR repository
git clone https://aur.archlinux.org/wayvid.git
cd wayvid

# Build and install
makepkg -si
```

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

#### For wayvid (stable)

Same process as above, but:
- Use `PKGBUILD.stable` as the source
- Clone `ssh://aur@aur.archlinux.org/wayvid.git`
- Update `pkgver` and `sha256sums` for each release

### Updating Checksums

For stable releases, update SHA256 checksum:
```bash
curl -sL https://github.com/YangYuS8/wayvid/archive/v0.3.0.tar.gz | sha256sum
```

Update the `sha256sums` array in PKGBUILD with the result.

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

1. Update `pkgver` in PKGBUILD
2. Update `pkgrel` to 1 for new versions
3. Increment `pkgrel` for packaging changes (same version)
4. Update `.SRCINFO` after any PKGBUILD changes
5. Test build before publishing

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
