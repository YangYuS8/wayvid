## [unreleased]

### 🐛 Bug Fixes

- *(aur)* Add build dependencies to PKGBUILD
- *(aur)* Use system zstd library for linking
## [0.4.3-hotfix.4] - 2025-11-19

### 🚀 Features

- Add Wayland backend for dynamic video wallpaper engine
- *(M4-3)* Add AUR packaging
- *(packaging)* Update AUR packages for M6 features (#28)
- Convert AUR stable package to binary distribution
- Add Debian package support and optimize workflow performance

### 🐛 Bug Fixes

- *(arch+niri)* Correct documentation and add GUI to AUR packages
- *(aur)* 修复依赖包名 libmpv -> mpv
- *(aur)* 添加缺失的 zstd 依赖
- Resolve artifact path issues in all packaging jobs
- *(ci)* Completely remove Arch package build job and update AUR workflow

### 📚 Documentation

- *(aur)* Add troubleshooting for Cargo.lock build error

### ⚙️ Miscellaneous Tasks

- *(aur)* Update stable package to v0.4.0
