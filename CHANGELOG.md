## [unreleased]

### 🐛 Bug Fixes

- *(build)* Include Cargo.lock for binary crate

### 📚 Documentation

- *(aur)* Add troubleshooting for Cargo.lock build error
## [0.4.3] - 2025-11-18

### 🐛 Bug Fixes

- *(aur)* 修复依赖包名 libmpv -> mpv

### 📚 Documentation

- *(i18n)* 完善中文翻译并修正机器翻译错误

### ⚙️ Miscellaneous Tasks

- Release v0.4.3
## [0.4.2] - 2025-11-14

### 🚀 Features

- *(workshop)* Implement Steam Workshop download and cache management
- *(wayland)* 添加壁纸管理器冲突检测
- *(video)* 添加 MPV 缩放参数以消除黑边
- *(gui)* 完整实现图形控制面板功能

### 🐛 Bug Fixes

- *(ci)* Suppress dead_code warning for search method
- *(workshop)* 修复项目去重和音量转换问题
- *(gui)* 修复 CI dead_code 警告

### 🚜 Refactor

- *(ci)* Optimize CI trigger conditions
- 优化代码格式和可读性

### 📚 Documentation

- *(i18n)* Update Chinese translations (43/896 translated)
- Complete Chinese translation and add CHANGELOG
- *(workshop)* Update documentation and add usage examples
- *(conflicts)* Rewrite in English per international standards
- 更新 Layout 模式说明
- 添加 GUI 控制面板文档和测试工具
- Update CHANGELOG for v0.4.2

### 🧪 Testing

- *(workshop)* Add comprehensive test scripts
- 添加 Layout 模式测试脚本

### ⚙️ Miscellaneous Tasks

- 删除 CHANGELOG.md 文件
- Bump version to 0.4.2
## [0.4.1] - 2025-11-11

### 🚀 Features

- *(ci)* Add automatic AUR package publishing to release workflow
- *(ci)* Add automatic CHANGELOG generation with git-cliff

### 🐛 Bug Fixes

- *(ci)* Correct git-cliff template syntax
- *(ci)* Upload CHANGELOG as artifact instead of committing
- *(ci)* Use Docker to generate .SRCINFO for AUR
- *(ci)* Create builder user for makepkg in Docker

### ⚙️ Miscellaneous Tasks

- *(aur)* Update stable package to v0.4.0
- Improve CI trigger rules and remove redundant docs
## [0.4.0] - 2025-11-10

### 🚀 Features

- *(M5-P0)* Implement Shared Decode Context for Multi-Display Performance (#17)
- Implement frame skip intelligence (Issue #16) (#20)
- Add output name pattern matching support
- Add priority-based pattern matching for outputs
- Add VideoSource-based IPC commands and multi-monitor docs
- Add comprehensive HDR support (Issue #1) (#22)
- *(packaging)* Update AUR packages for M6 features (#28)
- *(m6)* Desktop GUI Control Panel with egui (#30)
- *(gui)* Implement real IPC communication

### 🐛 Bug Fixes

- *(ci)* Update actions to v4 (v3 deprecated)
- *(build)* Fix missing ipc module compilation error
- *(ctl)* Add version flag to wayvid-ctl
- *(appimage)* Handle AppImage extraction failure in CI
- *(ci)* Fix CI failures - unused imports, variables, clippy warnings, formatting
- *(ci)* Fix test failures and remaining clippy warnings
- *(ci)* Remove unused imports and dead code warnings
- *(ci)* Remove unused PathBuf import
- *(ci)* Mark send_command as allow(dead_code)
- *(ci)* Remove unused Path import in test module
- *(test)* Update test YAML to match new VideoSource format
- Correctly bind XDG output manager and use connector names
- Mark unused qh parameter with underscore
- Allow dead_code for find_best_match function
- *(ci)* 修正分支名称格式以保持一致性
- *(arch+niri)* Correct documentation and add GUI to AUR packages
- *(ci)* Correct bash syntax in release workflow

### 💼 Other

- V0.4.0 - Enhanced diagnostics, GUI, and error messages

### 📚 Documentation

- Add M4 milestone completion summary
- Add M5 milestone planning
- Add M5 quick reference guide
- Add M5 GitHub Project setup documentation
- *(m5)* Update progress - Issue #13 completed, starting #14
- Update M5 progress - Issue #14 merged
- Update M5 progress - Issue #15 merged
- Add M5 Phase 1 completion summary
- Add Issue #2 progress report (Part 1)
- Update test report with XDG fix verification
- 重构文档结构，准备设备迁移
- 清理 docs 目录，移除测试文档和 RFC
- Refactor to mdbook-i18n-helpers with gettext workflow (#31)

### 🎨 Styling

- Format code (automated formatting)

### 🧪 Testing

- Complete multi-monitor testing for Issue #2
- Fix test assertion to match enhanced error message

### ⚙️ Miscellaneous Tasks

- Clean up outdated test scripts and files
- Update .gitignore to exclude test artifacts
## [0.3.0] - 2025-10-23

### 🚀 Features

- Add Wayland backend for dynamic video wallpaper engine
- *(M3-5)* Implement extended IPC command set
- *(M3-6)* Implement configuration hot reload
- *(M3-7)* Implement multi-video source support
- *(M4-1)* Implement Wallpaper Engine project parser
- *(M4-3)* Add AUR packaging
- *(M4-4)* Complete Nix flake configuration
- *(M4-5)* Add AppImage packaging
- *(M4-6)* Complete documentation updates

### 📚 Documentation

- *(M3-8)* Complete M3 documentation
