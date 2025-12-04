# Change: Fix AUR PKGBUILD LICENSE-APACHE download error

## Why
AUR package build fails because `PKGBUILD.stable` tries to download `LICENSE-APACHE` file that doesn't exist in the repository. The project only uses MIT license (`LICENSE-MIT`).

## What Changes
- Remove `LICENSE-APACHE` from PKGBUILD source array
- Remove conditional Apache license installation from package() function
- Update version to 0.4.4 for consistency

## Impact
- Affected files: `packaging/aur/PKGBUILD.stable`
- No breaking changes
- Fixes AUR package installation for wayvid 0.4.4
