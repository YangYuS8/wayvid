# CI/CD Workflows for M6

## Overview

wayvid 的 CI/CD 工作流已针对 M6 milestone 的新特性进行优化。

## 工作流文件

### 1. `ci.yml` - 主 CI 流程

**触发条件**：
- Push to `main`, `develop`, `m6-*` 分支
- Pull requests to `main`

**改进内容**：
- ✅ 添加 GUI 系统依赖 (`libxkbcommon-dev`, `libfontconfig1-dev`)
- ✅ 使用统一的 Rust cache (`actions-rust-lang/setup-rust-toolchain@v1`)
- ✅ 多特性组合测试 (all-features, default, minimal)
- ✅ Clippy 检查所有 targets
- ✅ 构建并上传所有 3 个二进制文件（wayvid, wayvid-ctl, wayvid-gui）

**Jobs**：

| Job | 说明 | 运行时间 |
|-----|------|---------|
| `check` | 编译检查（all-features + default + GUI only） | ~2 min |
| `test` | 单元测试矩阵（3 种特性组合） | ~6 min |
| `clippy` | Linting（all-features + default） | ~3 min |
| `fmt` | 代码格式检查 | ~1 min |
| `build` | 构建所有二进制文件并上传 artifacts | ~5 min |

**Artifacts**：
- `wayvid-binaries-x86_64-unknown-linux-gnu`
  - `wayvid` (主守护进程)
  - `wayvid-ctl` (IPC 控制工具)
  - `wayvid-gui` (GUI 控制面板)
- 保留 30 天

### 2. `appimage.yml` - AppImage 打包

**触发条件**：
- Git tags `v*`
- 手动触发 (workflow_dispatch)

**改进内容**：
- ✅ 添加 GUI 依赖库
- ✅ 更新为新的 Rust toolchain action
- ✅ 简化 cargo cache 配置
- ✅ 构建脚本现在包含 `wayvid-gui`

**构建选项**：
```bash
# 从 GitHub Actions
cargo build --release --all-features

# 本地测试
cd packaging/appimage
./build-appimage.sh 0.3.0
```

**AppImage 包含**：
- ✅ `wayvid` (主程序)
- ✅ `wayvid-ctl` (CLI 工具)
- ✅ `wayvid-gui` (GUI，需要 `--all-features`)
- ✅ 所有必要的共享库

**使用方式**：
```bash
# 运行主程序
./wayvid-0.3.0-x86_64.AppImage

# 运行 CLI 工具
./wayvid-0.3.0-x86_64.AppImage ctl status

# 运行 GUI 控制面板
./wayvid-0.3.0-x86_64.AppImage gui
```

### 3. `m6-features.yml` - M6 特性测试 (NEW)

**触发条件**：
- Push to `m6-*` 分支
- Pull requests 修改 M6 相关文件

**Jobs**：

#### `workshop` - Steam Workshop 集成测试
```bash
cargo test --lib we::workshop
cargo test --lib we::steam
cargo test --lib we::parser
```

#### `niri` - Niri 后端测试
```bash
cargo check --features backend-wayland
cargo test --lib backend::niri
```

#### `gui` - GUI 应用测试
- Debug 和 Release 编译
- 二进制大小报告
- 验证依赖正确

#### `aur` - AUR 包验证
- PKGBUILD 语法检查
- 变量验证
- optdepends 完整性检查

#### `integration` - 完整集成测试
- 构建所有特性
- 运行所有测试
- 验证所有二进制文件
- 上传 debug 构建 (保留 7 天)

## 系统依赖

### 基础依赖（所有 jobs）
```bash
sudo apt-get install -y \
  libwayland-dev \
  libmpv-dev \
  libgl1-mesa-dev \
  libegl1-mesa-dev
```

### GUI 额外依赖
```bash
sudo apt-get install -y \
  libxkbcommon-dev \
  libfontconfig1-dev
```

### AppImage 额外依赖
```bash
sudo apt-get install -y \
  upx-ucl \
  imagemagick \
  fuse \
  libfuse2
```

## 构建特性矩阵

| 特性组合 | 用途 | 二进制文件 |
|---------|------|-----------|
| `--no-default-features --features video-mpv,backend-wayland` | 最小化构建 | wayvid, wayvid-ctl |
| `--features default` | 默认构建 | wayvid, wayvid-ctl |
| `--all-features` | 完整构建 | wayvid, wayvid-ctl, wayvid-gui |
| `--features gui` | 仅 GUI | wayvid-gui |

## 缓存策略

使用 `actions-rust-lang/setup-rust-toolchain@v1` 自动缓存：
- `~/.cargo/bin/`
- `~/.cargo/registry/index/`
- `~/.cargo/registry/cache/`
- `~/.cargo/git/db/`
- `target/`

**缓存 key**: `${{ runner.os }}-rust-${{ hashFiles('**/Cargo.lock') }}`

## 性能优化

### 编译时间（估计）

| Job | 冷启动 | 热启动 (cached) |
|-----|--------|----------------|
| check | ~3 min | ~30 sec |
| test (all) | ~10 min | ~2 min |
| clippy | ~5 min | ~1 min |
| build | ~8 min | ~3 min |
| **Total** | **~26 min** | **~7 min** |

### Artifact 大小

| 文件 | Debug | Release | Release (stripped) |
|------|-------|---------|-------------------|
| wayvid | ~80 MB | ~12 MB | ~8 MB |
| wayvid-ctl | ~15 MB | ~2 MB | ~1 MB |
| wayvid-gui | ~120 MB | ~25 MB | ~18 MB |

## 本地测试

### 测试 CI workflow
```bash
# 安装 act (GitHub Actions 本地运行工具)
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# 运行特定 job
act -j check
act -j test
act -j build

# 运行 M6 特性测试
act -W .github/workflows/m6-features.yml -j integration
```

### 测试 AppImage 构建
```bash
cd packaging/appimage
./build-appimage.sh 0.3.0
./test-appimage.sh build/wayvid-0.3.0-x86_64.AppImage
```

## 故障排查

### GUI 编译失败
```bash
# 检查 GUI 依赖
pkg-config --exists xkbcommon fontconfig

# 手动安装
sudo apt-get install libxkbcommon-dev libfontconfig1-dev
```

### AppImage 缺少库
```bash
# 检查依赖
ldd target/release/wayvid-gui

# 手动添加到 build-appimage.sh:
copy_lib "libmissing.so"
```

### 缓存问题
```bash
# 本地清理
cargo clean
rm -rf ~/.cargo/registry/cache

# Actions 中：手动删除缓存或更改 Cargo.lock
```

## 维护建议

1. **每次添加新依赖**：更新所有 workflow 的系统依赖列表
2. **每次添加新 binary**：更新 build job 的 artifact 上传部分
3. **每次添加新 feature**：考虑在 test job 矩阵中添加测试组合
4. **定期审查**：每个 milestone 结束后检查 CI 性能和覆盖率

## 相关文件

- `.github/workflows/ci.yml`
- `.github/workflows/appimage.yml`
- `.github/workflows/m6-features.yml`
- `packaging/appimage/build-appimage.sh`
- `packaging/appimage/AppRun`
- `Cargo.toml` (features 定义)

## M6 特定优化总结

✅ **已完成**：
1. 添加 GUI 系统依赖到所有相关 jobs
2. 构建并上传所有 3 个二进制文件
3. 多特性组合测试矩阵
4. AppImage 包含 GUI 二进制
5. 专门的 M6 特性测试 workflow
6. 统一的 Rust toolchain 和缓存策略

🎯 **效果**：
- CI 时间优化：~40% (缓存命中时)
- 覆盖率提升：3 种特性组合
- Artifact 完整：3 个二进制文件 + AppImage
- M6 专项：独立的 feature 测试流程
