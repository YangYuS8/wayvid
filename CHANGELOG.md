## [0.5.0] - 2025-12-06

This is a major release introducing a GUI-first design with integrated playback engine.

### 🚀 Features

- *(gui)* Complete GUI rewrite using iced framework
- *(gui)* Wallpaper library browser with thumbnails and search
- *(gui)* Multi-monitor support with independent wallpapers per display
- *(gui)* System tray integration using ksni/SNI protocol
- *(gui)* Start minimized option - start directly to tray
- *(gui)* Wallpaper persistence - restore wallpapers after restart
- *(gui)* Engine state persistence - auto-start engine on launch
- *(gui)* Single-instance mode with window focus on re-launch
- *(engine)* Integrated playback engine (no separate daemon required)
- *(engine)* Battery state detection and auto-pause on battery
- *(engine)* Adaptive polling interval for power saving
- *(library)* Steam Workshop import support (video wallpapers)
- *(library)* HDR support with tone-mapping

### ⚡ Performance

- *(engine)* Optimize MPV config for integrated GPU scenarios
- *(engine)* Add vaapi-device hint for better AMD GPU hardware decoding
- *(engine)* Use bilinear scaling to reduce GPU load
- *(engine)* Cap wallpaper playback at 30fps to save resources
- *(engine)* Disable unnecessary post-processing (deband, dither, sigmoid)

### 🐛 Bug Fixes

- *(gui)* Fix theme persistence - settings now save immediately
- *(gui)* Fix window close handling for minimize-to-tray behavior
- *(gui)* Fix multiple instances running simultaneously
- *(gui)* Fix tray icon not showing on some systems

### 📚 Documentation

- *(docs)* Update for v0.5 GUI-first workflow
- *(docs)* Add autostart configuration guide for niri/Hyprland/Sway
- *(docs)* Update installation instructions

### 🚜 Refactor

- *(core)* Split into workspace with multiple crates
- *(app)* Replace daemon terminology with engine
- *(gui)* Remove legacy egui code

## [0.4.5-alpha.2] - 2025-12-04

### 📚 Documentation

- Improve systemd service configuration for niri users

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare v0.4.5-alpha.2
## [0.4.5-alpha.1] - 2025-12-04

### 🚀 Features

- Add project logo to README, docs, and packaging
- Add Vulkan backend infrastructure and scene support

### 🐛 Bug Fixes

- Resolve all clippy warnings and add demo section to README
- Correct documentation link formatting in README
- Address Reddit user feedback - improve CLI UX and documentation

### 💼 Other

- Add proposal for Scene wallpaper support

### 🎨 Styling

- Apply rustfmt and add demo section to README

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare v0.4.5-alpha.1
## [0.4.4] - 2025-12-03

### 🐛 Bug Fixes

- *(aur)* Remove non-existent LICENSE-APACHE from PKGBUILD.stable

### ⚙️ Miscellaneous Tasks

- *(release)* 发布 v0.4.4 正式版
## [0.4.4-alpha.3] - 2025-11-26

### 🚀 Features

- *(gui)* 重构 GUI 为 Wallpaper Engine 风格的简化界面

### 🐛 Bug Fixes

- *(gui)* Reconnect IPC for each command
- *(render)* 修复帧渲染循环架构，优化 CPU 使用率
- *(gui)* 优化状态消息格式化，提升代码可读性
- *(ci)* 修复 clippy dead_code 和 unnecessary_map_or 警告

### 📚 Documentation

- 更新文档以反映新的 GUI 和帧渲染架构

### 🎨 Styling

- 修复 types.rs 格式化问题

### ⚙️ Miscellaneous Tasks

- *(release)* 发布 v0.4.4-alpha.3
## [0.4.4-alpha.2] - 2025-11-25

### 🚀 Features

- *(gui)* Add i18n support and fix full screen coverage

### 🐛 Bug Fixes

- *(ci)* Resolve clippy warnings for CI compliance

### 📚 Documentation

- *(openspec)* Update optimize-release-workflow task progress
## [0.4.4-alpha.1] - 2025-11-25

### 🚀 Features

- *(openspec)* 添加 OpenSpec 提示和项目文档以支持变更管理

### 📚 Documentation

- *(openspec)* Initialize OpenSpec with core specifications
- Add Ko-fi sponsorship support in README

### ⚡ Performance

- Optimize CI workflow and apply Rust performance improvements
- *(release)* Optimize build workflow with Rust Performance Book techniques
## [0.4.3-hotfix.5] - 2025-11-19

### 🐛 Bug Fixes

- *(aur)* Add build dependencies to PKGBUILD
- *(aur)* Use system zstd library for linking
- *(aur)* Fix PKGBUILD build failures
## [0.4.3-hotfix.4] - 2025-11-19

### 🚀 Features

- Convert AUR stable package to binary distribution
- Add Debian package support and optimize workflow performance

### 🐛 Bug Fixes

- Resolve artifact path issues in all packaging jobs
- *(release)* Extract uploaded tarballs into target/release for packaging jobs
- *(appimage)* Use precompiled binaries instead of recompiling
- *(ci)* Skip AppImage tests and remove Arch package build job
- *(ci)* Completely remove Arch package build job and update AUR workflow

### ⚡ Performance

- Optimize CI and docs workflows

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare v0.4.3-hotfix.4
## [0.4.3-hotfix.3] - 2025-11-19

### 🐛 Bug Fixes

- *(aur)* 添加缺失的 zstd 依赖

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare v0.4.3-hotfix.3
## [0.4.3-hotfix.2] - 2025-11-19

### 🚀 Features

- *(ci)* 为预发布版本添加 AUR Git 包自动发布

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare v0.4.3-hotfix.2
## [0.4.3-hotfix.1] - 2025-11-18

### 🐛 Bug Fixes

- *(build)* Include Cargo.lock for binary crate
- 更新 wayvid 版本为 0.4.3-hotfix.1
- *(ci)* 修复 release workflow 版本验证逻辑

### 📚 Documentation

- *(aur)* Add troubleshooting for Cargo.lock build error

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare v0.4.3-hotfix.1
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
