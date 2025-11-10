# CI/CD 流程说明

## 📊 完整的发布流程

wayvid 采用分层的 CI/CD 策略，区分日常开发、测试版本和正式发布。

---

## 🔄 工作流概览

### 1. **ci.yml** - 日常质量检查

**触发时机**：
- ✅ Push 到 `main`, `develop`, `m6-*` 分支
- ✅ Pull Request 到 `main`
- ❌ **不在** tag 推送时触发

**执行内容**：
- 代码格式检查 (rustfmt)
- Lint 检查 (clippy)
- 单元测试（3种特性组合）
- 编译验证
- **PR 预览构建**（仅 PR 时）

**产物**：
- PR 预览二进制（保留 7 天）
- 不发布 Release

**运行时间**：~7 分钟（缓存命中）

---

### 2. **release.yml** - 版本发布 ✨ 核心

**触发时机**：
仅在推送符合以下格式的 tag 时：

| 版本类型 | Tag 格式 | 示例 | 是否预发布 |
|---------|---------|------|-----------|
| **正式版** | `v{major}.{minor}.{patch}` | `v1.2.3` | ❌ |
| **Alpha** | `v{major}.{minor}.{patch}-alpha.{n}` | `v1.2.3-alpha.1` | ✅ |
| **Beta** | `v{major}.{minor}.{patch}-beta.{n}` | `v1.2.3-beta.2` | ✅ |
| **RC** | `v{major}.{minor}.{patch}-rc.{n}` | `v1.2.3-rc.1` | ✅ |
| **Hotfix** | `v{major}.{minor}.{patch}-hotfix.{n}` | `v1.2.3-hotfix.1` | ✅ |

**执行流程**：

#### Phase 1: 验证 (validate)
```bash
✓ 提取版本号和类型
✓ 检查 tag 版本是否匹配 Cargo.toml
✓ 判断是否为预发布版本
```

#### Phase 2: 质量检查 (quality)
```bash
✓ 代码格式检查
✓ Clippy (所有特性)
✓ 单元测试 (所有特性)
```

#### Phase 3: 构建二进制 (build)
```bash
✓ 构建 3 个二进制文件 (wayvid, wayvid-ctl, wayvid-gui)
✓ Strip 优化
✓ 打包为 tarball
✓ 生成 SHA256 校验和
```

#### Phase 4: 构建 AppImage (appimage)
```bash
✓ 使用 packaging/appimage/build-appimage.sh
✓ 运行 AppImage 测试
✓ 生成 SHA256SUMS
```

#### Phase 5: 生成发布说明 (release-notes)
```bash
✓ 自动从 git commits 生成 changelog
✓ 分类为: Features, Bug Fixes, Documentation, Other
✓ 添加安装说明和下载链接
```

#### Phase 6: 创建 Release (release)
```bash
✓ 创建 GitHub Release
✓ 上传所有构建产物
✓ 标记预发布状态
✓ 附加完整的 Release Notes
```

#### Phase 7: 更新 AUR (update-aur) - 仅正式版
```bash
✓ 仅在非预发布版本时执行
✓ 自动更新 AUR 包元数据
```

**产物**：
- `wayvid-{version}-x86_64-unknown-linux-gnu.tar.gz`
- `wayvid-{version}-x86_64-unknown-linux-gnu.tar.gz.sha256`
- `wayvid-{version}-x86_64.AppImage`
- `SHA256SUMS`
- GitHub Release with notes

**运行时间**：~15-20 分钟

---

### 3. **m6-features.yml** - M6 专项测试

**触发时机**：
- Push 到 `m6-*` 分支
- PR 修改 M6 相关文件

**执行内容**：
- Workshop 集成测试
- Niri 后端测试
- GUI 构建验证
- AUR 包验证
- 完整集成测试

**运行时间**：~8 分钟

---

### 4. **appimage.yml** - 手动构建

**触发时机**：
- 仅手动触发 (workflow_dispatch)
- 用于测试 AppImage 构建流程

**用途**：
- 开发者本地测试
- 临时构建特定版本
- 不自动发布

---

## 🚀 发布流程实践

### 场景 1: 日常开发

```bash
# 开发新特性
git checkout -b feature/awesome-feature
# ... 编写代码 ...
git add .
git commit -m "feat: add awesome feature"
git push origin feature/awesome-feature

# 创建 PR
gh pr create --title "feat: add awesome feature"

# CI 自动运行:
# ✓ 质量检查
# ✓ 构建 PR 预览版 (保留 7 天)
```

**结果**：✅ 质量检查 + ✅ PR 预览构建

---

### 场景 2: 发布 Alpha 测试版

```bash
# 确保在 main 分支
git checkout main
git pull

# 检查版本号 (Cargo.toml 应为 0.4.0)
grep "^version" Cargo.toml
# version = "0.4.0"

# 创建并推送 alpha tag
git tag v0.4.0-alpha.1
git push origin v0.4.0-alpha.1

# Release workflow 自动运行:
# ✓ 验证版本号
# ✓ 质量检查
# ✓ 构建所有二进制
# ✓ 构建 AppImage
# ✓ 生成 Release Notes
# ✓ 创建 GitHub Release (标记为 Pre-release)
```

**结果**：✅ Alpha 版本发布，标记为 Pre-release

---

### 场景 3: 发布正式版

```bash
# 确保在 main 分支且代码稳定
git checkout main
git pull

# 更新 CHANGELOG.md
vim CHANGELOG.md

# 确认版本号
grep "^version" Cargo.toml
# version = "0.4.0"

# 创建并推送正式版 tag
git tag v0.4.0
git push origin v0.4.0

# Release workflow 自动运行:
# ✓ 验证版本号
# ✓ 质量检查
# ✓ 构建所有二进制
# ✓ 构建 AppImage
# ✓ 生成 Release Notes
# ✓ 创建 GitHub Release (正式版)
# ✓ 更新 AUR 包
```

**结果**：✅ 正式版本发布 + ✅ AUR 自动更新

---

### 场景 4: 发布 Hotfix

```bash
# 从 main 创建 hotfix 分支
git checkout main
git checkout -b hotfix/critical-bug
# ... 修复 bug ...
git commit -m "fix: critical bug in module X"
git push origin hotfix/critical-bug

# 合并到 main
git checkout main
git merge hotfix/critical-bug

# 更新版本号为 0.4.1
vim Cargo.toml  # version = "0.4.1"
git commit -am "chore: bump version to 0.4.1"
git push

# 创建 hotfix tag
git tag v0.4.1-hotfix.1
git push origin v0.4.1-hotfix.1

# Release workflow 自动运行 (标记为 Pre-release)
```

**结果**：✅ Hotfix 版本发布，标记为 Pre-release

---

## 📋 版本号管理

### 版本号格式

遵循 [Semantic Versioning 2.0.0](https://semver.org/)：

```
{major}.{minor}.{patch}[-{prerelease}.{number}]
```

### 版本号规则

| 类型 | 何时递增 | 示例 |
|------|---------|------|
| **Major** | 不兼容的 API 变更 | `1.0.0` → `2.0.0` |
| **Minor** | 向后兼容的功能新增 | `1.0.0` → `1.1.0` |
| **Patch** | 向后兼容的 bug 修复 | `1.0.0` → `1.0.1` |
| **Prerelease** | 预发布版本标识 | `1.0.0-alpha.1` |

### 预发布版本类型

| 类型 | 用途 | 稳定性 |
|------|------|--------|
| **alpha** | 早期测试版，功能不完整 | ⚠️ 不稳定 |
| **beta** | 功能完整，需要测试 | ⚠️ 可能有 bug |
| **rc** | 发布候选，准备正式发布 | ✅ 基本稳定 |
| **hotfix** | 紧急修复，独立于主版本 | ✅ 修复特定问题 |

### Cargo.toml 版本号

**重要规则**：
- ⚠️ `Cargo.toml` 中的 `version` **不包含** 预发布后缀
- ✅ Tag 可以包含预发布后缀
- ✅ Release workflow 会自动验证版本号匹配

**示例**：
```toml
# Cargo.toml
version = "0.4.0"  # ← 不包含 -alpha.1
```

```bash
# 可以推送的 tags:
git tag v0.4.0          # ✅ 正式版
git tag v0.4.0-alpha.1  # ✅ Alpha
git tag v0.4.0-beta.1   # ✅ Beta
git tag v0.4.0-rc.1     # ✅ RC
```

---

## 🔍 版本号验证

Release workflow 会自动检查：

```yaml
Cargo.toml version:  0.4.0
Tag base version:    0.4.0  (去除 -alpha.1 后缀)
                     ✅ Match!
```

如果不匹配：
```yaml
Cargo.toml version:  0.3.0
Tag base version:    0.4.0
                     ❌ Mismatch! Build fails.
```

---

## 📦 构建产物

### 每个 Release 包含：

| 文件 | 说明 | 大小 (约) |
|------|------|----------|
| `wayvid-{ver}-x86_64-unknown-linux-gnu.tar.gz` | 二进制 tarball | ~25 MB |
| `wayvid-{ver}-x86_64-unknown-linux-gnu.tar.gz.sha256` | SHA256 校验和 | ~100 B |
| `wayvid-{ver}-x86_64.AppImage` | AppImage 包 | ~35 MB |
| `SHA256SUMS` | 所有文件的校验和 | ~500 B |

### Tarball 内容：
```
wayvid          # 主守护进程
wayvid-ctl      # CLI 控制工具
wayvid-gui      # GUI 控制面板
```

---

## 🛠️ 开发者命令

### 查看当前版本
```bash
grep "^version" Cargo.toml
```

### 列出所有 tags
```bash
git tag -l "v*" --sort=-version:refname
```

### 删除错误的 tag
```bash
# 本地删除
git tag -d v0.4.0-alpha.1

# 远程删除
git push --delete origin v0.4.0-alpha.1
```

### 创建正式版 tag
```bash
# 确保代码已提交
git status

# 创建 annotated tag (推荐)
git tag -a v0.4.0 -m "Release v0.4.0"

# 推送
git push origin v0.4.0
```

### 查看 tag 详情
```bash
git show v0.4.0
```

### 手动触发 AppImage 构建
```bash
gh workflow run appimage.yml -f version=0.4.0
```

---

## 📊 Release Notes 自动生成

### Commit 消息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 支持的类型：

| Type | 分类 | 示例 |
|------|------|------|
| `feat` | ✨ Features | `feat(gui): add dark mode support` |
| `fix` | 🐛 Bug Fixes | `fix(niri): resolve workspace detection` |
| `docs` | 📚 Documentation | `docs: update installation guide` |
| `chore` | 🔧 Other Changes | `chore: bump dependencies` |
| `ci` | 🔧 Other Changes | `ci: optimize build cache` |
| `refactor` | 🔧 Other Changes | `refactor: simplify config parser` |

### 生成的 Release Notes 结构：

```markdown
## wayvid 0.4.0

**Release Type:** Stable
**Build Date:** 2025-11-10

### What's New

#### ✨ Features
- Add dark mode support (a1b2c3d)
- Implement workshop search (e4f5g6h)

#### 🐛 Bug Fixes
- Resolve workspace detection issue (i7j8k9l)

#### 📚 Documentation
- Update installation guide (m0n1o2p)

### 📦 Installation

[安装说明...]

### 📋 Checksums

[校验和信息...]
```

---

## 🔔 通知和监控

### GitHub Actions 通知

- ✅ 成功：无通知
- ❌ 失败：GitHub 自动发送邮件

### Release 订阅

用户可以通过以下方式获取更新通知：
- GitHub "Watch" → "Releases only"
- RSS 订阅: `https://github.com/YangYuS8/wayvid/releases.atom`

---

## 🐛 故障排查

### 版本号不匹配

**问题**：
```
❌ Version mismatch!
   Cargo.toml: 0.3.0
   Tag:        0.4.0
```

**解决**：
```bash
# 更新 Cargo.toml
vim Cargo.toml  # version = "0.4.0"
git commit -am "chore: bump version to 0.4.0"
git push

# 删除错误的 tag
git tag -d v0.4.0
git push --delete origin v0.4.0

# 重新创建
git tag v0.4.0
git push origin v0.4.0
```

### Release workflow 失败

**问题**：构建或测试失败

**解决**：
```bash
# 查看失败原因
gh run view --log-failed

# 修复代码后，删除并重新推送 tag
git tag -d v0.4.0
git push --delete origin v0.4.0

# 修复提交
git commit --amend
git push

# 重新创建 tag
git tag v0.4.0
git push origin v0.4.0
```

### AppImage 构建失败

**问题**：依赖缺失

**解决**：
1. 检查 `packaging/appimage/build-appimage.sh` 的依赖列表
2. 更新 `.github/workflows/release.yml` 的系统依赖
3. 手动测试 AppImage 构建脚本

---

## 📈 性能优化

### 缓存策略

| Workflow | 缓存策略 | 理由 |
|----------|---------|------|
| ci.yml | ✅ 启用 | 加速日常开发 |
| release.yml (prerelease) | ✅ 启用 | 加速测试版本 |
| release.yml (stable) | ❌ 禁用 | 确保干净构建 |

### 并行执行

- ✅ build 和 appimage 并行运行
- ✅ 多特性测试并行执行
- ✅ 独立的验证步骤

---

## 📚 相关资源

- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Actions - Creating releases](https://docs.github.com/en/actions/creating-actions/creating-a-composite-action)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)

---

## ✅ 检查清单

### 发布前

- [ ] 所有 CI 检查通过
- [ ] 版本号已更新 (`Cargo.toml`)
- [ ] CHANGELOG.md 已更新（可选）
- [ ] 重大变更已文档化
- [ ] 本地测试通过

### 发布时

- [ ] Tag 格式正确
- [ ] Tag 版本号匹配 Cargo.toml
- [ ] 推送到 origin

### 发布后

- [ ] 验证 Release 创建成功
- [ ] 验证所有 artifacts 可下载
- [ ] 验证 Release Notes 完整
- [ ] 测试 AppImage 可运行
- [ ] 更新文档链接（如需要）

---

## 🎯 最佳实践

1. **小步快跑**：经常发布 alpha/beta 版本测试
2. **语义化版本**：严格遵循版本号规范
3. **清晰的提交信息**：便于自动生成 Release Notes
4. **充分测试**：PR 预览构建 + alpha/beta 测试
5. **文档同步**：每次发布更新相关文档
