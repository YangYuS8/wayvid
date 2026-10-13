# 贡献指南

## 开发环境

安装核心工具链：

- Node.js 24，与 CI 一致；
- pnpm 11.1.3，由 `package.json` 固定；
- Rust 1.99.0，由 `rust-toolchain.toml` 选择；最低支持的 Rust 版本另行声明在 `Cargo.toml` 中；
- 当前发行版所需的 Tauri 2 平台依赖。

Cargo Tauri CLI 版本由 `.tauri-cli-version` 固定（2.12.1）。使用共享安装脚本验证或安装该精确版本，不使用未限定的最新 CLI。

安装依赖：

```bash
pnpm install --frozen-lockfile
pnpm exec svelte-kit sync
./scripts/ensure-tauri-cli.sh
export PATH="$PWD/target/tauri-cli/bin:$PATH"
```

安装脚本默认将 CLI 单独安装到 `target/tauri-cli`，与已有 Cargo 工具分开。如需使用其他位置，在运行安装脚本时设置 `CARGO_INSTALL_ROOT`，并在执行 `pnpm tauri:dev` 或 `cargo tauri build` 前把该目录下的 `bin/` 加入 `PATH`。

运行前端检查：

```bash
pnpm check
pnpm test
pnpm build
pnpm docs:build
```

运行 Rust 检查：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
```

本地构建文档：

```bash
pnpm docs:build
```

本地预览文档：

```bash
pnpm docs:preview
```

## 修改产品行为前

把本站中维护的文档作为产品变更的政策来源。

每个重要变更都应说明它如何支持 Linux 动态壁纸平台，而不是已退役的 wayvid 产品故事。当范围可能被误解时，也应写明首发非目标。

设计工作应标明所属领域：

- 产品基础；
- 创意工坊循环；
- 兼容性；
- 运行时；
- 应用壳。

当内容类型或导入路径受到影响时，需要说明用户可见的兼容性影响。

## 用户可见文本和 i18n

LWE 的用户可见界面目标支持英文和简体中文。

修改用户可见文本时：

1. 同时更新两种语言；
2. 检查更长的中文或英文字符串对布局的影响；
3. 在任务或 Pull Request 说明中包含 i18n 验证；
4. 保持术语与文档一致。

文档站点也遵循该规则：英文根目录下的每个维护页面，都必须在 `docs/zh/` 下有对应的简体中文页面。

## 文档规则

- 所有维护中的文档都放在 `docs/` 下。
- 优先编写任务导向页面，而不是归档堆积。
- 过时规划文档中的有用信息合并进维护页面后，应删除原文档。
- 不要重新引入已删除的 `openspec/` 目录；项目指导现在由本文档站维护。
- 根 README 保持简洁，并链接到发布文档获取详情。
- 不要把不支持的运行时行为写成已支持。
- 如果某功能只在已验证环境中工作，需要明确写出该环境。

## 测试期望

提交变更前，运行最小且有意义的检查。对于范围较大的维护变更，使用：

```bash
pnpm check
pnpm test
pnpm build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
pnpm docs:build
```

Quality Check 还运行轻量打包回归：

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

前端测试包含服务端渲染和模拟 DOM 中的真实组件交互；`pnpm test` 会运行两个测试项目。单独检查时可使用 `pnpm test --project server` 或 `pnpm test --project interaction`。交互测试模拟 IPC，并点击实际挂载的组件；覆盖业务失败（`ActionOutcome.ok=false`）、IPC 拒绝，以及动作成功后刷新数据失败。失败后保留已有内容，区分重新加载数据与重复执行已经成功的动作。

Tauri 命令通过 `commands/background.rs` 在后台执行同步服务逻辑。依赖桌面状态的命令会等待后台启动恢复完成，写入操作在服务层串行执行。会话和设置使用私有临时文件后原子替换；持久化测试必须验证部分写入或替换失败时旧文件仍然有效。网络错误脱敏测试使用虚构 API key 和本地 HTTP 测试服务。

对于仅文档变更，`pnpm docs:build` 和 `git diff --check` 是必需验证项。

## 打包验证

两条发布 workflow 都读取 `.tauri-cli-version`，将其纳入 CLI 缓存键，并在恢复缓存后验证 `cargo-tauri`。缓存中的版本不匹配时，必须替换并重新核验，才能开始构建。

AppImage 在 Ubuntu 22.04 上构建，以保持 glibc 要求与 AppImage 目录的测试主机兼容。原生 deb/rpm 包使用 Ubuntu 24.04，以保留 Arch 软件包使用的 libmpv.so.2 ABI。发布 CLI 和 Rust 缓存按发行版隔离；静态元数据校验通过不能证明 AppImage 可以在最低支持系统上启动。Quality CI 使用校验过 SHA-256 的 actionlint 检查所有 workflow。

AppImage 验证需要 Python 3、`unsquashfs`（来自 squashfs-tools）、`file` 和 `desktop-file-validate`（来自 desktop-file-utils）：

```bash
python3 scripts/validate-appimage.py target/release/bundle/appimage/*.AppImage
# 也可验证已经解包的 AppDir：
python3 scripts/validate-appimage.py --appdir path/to/LWE.AppDir
```

校验器检查可执行的 `AppRun`、根目录唯一且有效的 desktop 文件、它引用的图标，以及可读取的 PNG `.DirIcon`。相关软链接必须使用相对路径、解析后位于 AppDir 内且目标存在；普通图标文件同样可用。检查不会启动应用或修补失败的包。所有选中上传的 AppImage 都必须在发布产物前通过该门禁。检查选中的版本，避免把旧构建产物纳入上传范围。

回归套件覆盖正常包，以及缺失、断链、绝对路径或越界的图标链接。测试样本通过不能替代真实发布产物验证或真实桌面验收。

## GitHub 到 CNB 的单向镜像

GitHub `main` 及其发布标签是权威源码与构建源。CNB `Nesoriel/lwe` 仅作为代码和发布的单向镜像。CNB 仓库设置为 `auto_trigger=false`；`.cnb.yml` 只保留用于 EdgeOne 文档部署的 `api_trigger_docs`。CNB 不重复运行 Quality Check、Rust 或安装包构建。

`Sync CNB` workflow 在 GitHub `main` 推送时镜像代码。两条发布 workflow 在 GitHub release 发布后，以 reusable job 传入 `release_tag` 和 `expected_source_sha`，不依赖 `GITHUB_TOKEN` 产生的 release 事件。`workflow_dispatch` 可回填单个 `release_tag`，或通过 `deploy_docs=true` 显式请求文档 API 触发器。

CNB 写入共用不取消任务的 GitHub 队列（`queue: max`）。CI 严格检查该队列声明，只精确排除固定 actionlint 版本不支持队列字段的误报；其他 workflow 保持完整检查。上传后核对 CNB 服务端 SHA-256 和大小；旧附件没有 SHA-256 时下载文件校验。

GitHub 仓库 Secret `CNB_TOKEN` 使用限定到 `Nesoriel/lwe` 的长期 CNB 个人访问令牌（PAT），所需权限为：

- `repo-code:rw`；
- `repo-release:rw`；
- `repo-cnb-trigger:rw`；
- `repo-manage:r`。

本机 OAuth 令牌有效期为八小时，不适合长期 CI。管理员在本机完成一次性 CNB 仓库设置；CI 不授予仓库管理写权限。令牌值不得进入源码、日志或发布说明。

保留 GitHub tag 对象，并核验其源码提交与 `expected_source_sha` 一致；不得改写 tag 或强制更新 CNB `main`，后者只允许快进。公开 GitHub release 附件原样同步，核对两侧 `.deb`、`.rpm` 和 `.AppImage` 的 SHA-256，CNB 附件以 `ttl=0` 永久保留。重跑时补齐缺失内容、跳过相同的已有附件，拒绝同名但不同内容的附件。任何同步或校验失败都必须保持为失败的运行结果。

源码变更不能证明镜像或 EdgeOne 部署已上线。合并并执行 workflow 后，应读回 CNB 提交、tag 对象、发布附件及哈希；文档触发和部署结果单独报告。

## 发布冒烟清单

发布稳定标签或合并到 `main` 触发预发布前，先验证该操作将触发的发布路径：

1. 如果变更范围较大，在本地运行等价的必需检查：`pnpm check`、`pnpm test`、`pnpm build`、`pnpm docs:build`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo check --workspace`、`cargo test --workspace`，以及上述打包回归。
2. 确认待打标签提交上的 GitHub Actions 质量检查通过。
3. 确认发布 workflow 会构建预期 Linux 制品（`.deb`、`.rpm` 和 `.AppImage`），并在上传前通过 AppImage 验证。发布后下载公开的 AppImage，记录 SHA-256 并再次验证。
4. 确认包渠道元数据匹配发布渠道：稳定标签使用 AUR stable（`lwe`），开发/预发布路径使用 AUR git（`lwe-git`）。
5. 对影响运行时的发布候选版，在写成受支持前，必须在已验证的 Wayland + `niri` 会话上运行下面的真实桌面运行时验收清单。
6. 条件允许时执行新安装冒烟流程：安装一个包制品，打开并移动/缩放窗口，从托盘恢复窗口，打开 Settings 并生成诊断信息，检查 Workshop/Library 状态，应用一张兼容视频壁纸，清除它，重启 LWE，并确认保存分配行为可见。改变 Wayland 启动行为的 Tauri 升级也需要在 `niri` 上完成此项检查。
7. 读回发布及包渠道结果。修复 AppImage 目录收录时，应在新稳定版 AppImage 已公开且验证通过后请求上游复测，并确认上游日志选中了新版本。

包安装成功只表示应用可从该包启动；它不保证未验证的合成器、GPU/EGL 栈、显示器布局或壁纸类型具备运行时支持。

## 真实桌面运行时验收

运行时变更在写成“已支持”前，必须先在真实受支持桌面会话上验证。Wayland + `niri` 上的视频是主要已验证目标；实验性 Scene 验收仅限下方记录的样本与环境。

会话恢复使用 LWE 会话文件：当设置了 `XDG_CONFIG_HOME` 时为 `$XDG_CONFIG_HOME/lwe/session.toml`，否则为 `$HOME/.config/lwe/session.toml`。验收时使用独立的 `XDG_CONFIG_HOME`，检查恢复行为时读取该测试会话文件。不要把删除现有用户设置或分配作为常规测试准备。

运行时变更请使用以下清单：

1. 在 LWE 中发现至少一个活动显示器；
2. 从本地库把一张兼容的视频壁纸应用到一个显示器；
3. 确认桌面可见变化；
4. 如果存在多个显示器，把壁纸应用到第二个显示器，然后只清除其中一个显示器；
5. 确认清除一个显示器不会停止其他显示器上的壁纸；
6. 重启 LWE，确认已保存分配能恢复，或恢复失败能在 Desktop 页面中显示；
7. 清除所有分配，并确认保存的会话不会再次恢复它们；
8. 在同一进程中至少三轮应用、清除并重新应用视频，检查 EGL 资源复用。

如果某个运行时步骤失败，请保留终端日志中指出失败阶段的行：后端启动、输出发现、首帧应用、按显示器清除或启动时恢复。这些信息是区分缺失视频资源、输出不匹配、Wayland layer-shell/EGL 失败和后端超时的受支持方式。

Rust 测试套件中的真实桌面测试需要显式启用，因为它们依赖当前合成器、显示器布局、GPU/EGL 栈、Steam 创意工坊内容和本地视频资源。请只在已验证机器上运行：

```bash
lwe_test_config="$(mktemp -d /tmp/lwe-real-desktop.XXXXXX)" && \
XDG_CONFIG_HOME="$lwe_test_config" LWE_REAL_DESKTOP_TESTS=1 \
  cargo test --locked -p lwe-shell --lib \
  services::desktop_service::tests::desktop_apply_flow_reapplies_video_after_clear_on_same_backend \
  -- --exact --nocapture --test-threads=1
```

验收实验性的原生 2D Scene 子集时，请先确保本地库中至少有一张通过本地验证的 Scene 和一张可运行视频，再使用独立测试配置目录：

```bash
lwe_scene_test_config="$(mktemp -d /tmp/lwe-scene-desktop.XXXXXX)" && \
XDG_CONFIG_HOME="$lwe_scene_test_config" LWE_REAL_DESKTOP_TESTS=1 \
  cargo test --locked -p lwe-shell --lib \
  services::desktop_service::tests::desktop_apply_flow_scene_video_switch_clear_and_restore \
  -- --exact --nocapture --test-threads=1
```

该测试默认检查三轮应用/清除，以及同一后端中的 Scene → Video → Scene 切换、隔离会话保存失败后的回滚、commit 命令已处理后的取消、后端关闭/join/重启后的 Scene 分配恢复和最后清除。`LWE_REAL_DESKTOP_SCENE_CYCLES` 可指定 1–1,000 轮；重跑已记录的混合输入源压力测试可使用 100 轮，并复用同一个隔离配置目录：

```bash
XDG_CONFIG_HOME="$lwe_scene_test_config" LWE_REAL_DESKTOP_TESTS=1 \
LWE_REAL_DESKTOP_SCENE_CYCLES=100 \
  cargo test --locked -p lwe-shell --lib \
  services::desktop_service::tests::desktop_apply_flow_scene_video_switch_clear_and_restore \
  -- --exact --nocapture --test-threads=1
```

保留 `HOME` 供 Steam 内容发现使用，检查持久化时读取隔离目录中的会话文件。还需确认桌面上壁纸可见，且受支持的效果会动画。增加轮数是操作步骤，不代表已记录通过。此测试只覆盖当前 2D 子集，不能证明完整 Scene 兼容性。独立输出渲染与清除需要真实多显示器硬件，单显示器测试无法覆盖这些场景。

### Scene inventory

只读 inventory 示例接收单个项目目录，或包含最多 64 个打包项目的目录。请替换为实际 Steam 库路径，并把报告保留在本地；不需要桌面 opt-in 标志：

```bash
lwe_scene_projects="$HOME/.local/share/Steam/steamapps/workshop/content/431960"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
lwe_scene_inventory_report="$(mktemp /tmp/lwe-scene-inventory.XXXXXX.json)"
cargo run --locked -p lwe-scene --example inventory -- \
  "$lwe_scene_projects" "$lwe_scene_assets" > "$lwe_scene_inventory_report"
```

`SceneInventory::collect` 沿已建模引用遍历，并扫描包内 JSON/TEX/着色器和项目元数据。未打包文件只按引用读取，不递归枚举目录。报告区分可达与仅存在于包中的声明、`user`/`value` 绑定与 JS、HDR 与 bloom、资源来源、结构和未知项。脚本/着色器/文字正文及私有值不导出，未知/文字子树中看似资源的字符串也省略。它不把内容标为可运行或已支持。预算耗尽及无效/缺失资源会明确标为部分覆盖；动态/脚本生成的引用仍未知。

默认每次遍历包含 8,192 个资源、32 MiB JSON、16 MiB 着色器来源、16,384 个 mip 与 16,384 个动画帧的上限，同时限制文档、深度/节点、观察及报告字符串。结构读取器在循环与分配前消耗共享的 mip/帧剩余预算，跳过媒体负载且不执行解码。PKG 索引、范围读取和哈希复用同一打开的 descriptor；元数据检查能发现普通原地修改，路径替换也不会把指纹转向新文件。这是保守的一致性检查，不是文件系统不可变快照。完成有界遍历不等于完整运行时语义已验证。

### Scene UI 与运行时测量

以下工具的运行时模式会应用和清除真实壁纸，只能在具备条件的 Wayland + `niri` 会话中，为真实桌面验收显式启用。保留 `HOME` 供 Steam 发现使用，并填入本地已获取的项目/资源路径。工具不能替代原版 Wallpaper Engine 视觉 A/B、多显示器或长期验收。

`scene_probe` 启动自有引擎后端，设置 30 FPS 引擎上限，在频繁 `GetStatus` 请求下测量动态、暂停与恢复阶段。JSON 输出包含 draw/swap/callback 计数、时长与逻辑 Scene 负载；不保存 settings/session，也不测试 Tauri UI。请把 `eDP-1` 替换为实际输出：

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
lwe_scene_probe_report="$(mktemp /tmp/lwe-scene-probe.XXXXXX.json)"
LWE_REAL_DESKTOP_TESTS=1 cargo run --locked -p lwe-engine --example scene_probe -- \
  "$lwe_scene_project" "$lwe_scene_assets" eDP-1 60 \
  > "$lwe_scene_probe_report"
```

追加 `--static /absolute/path/to/an-authored-static-project` 可检查暂停时的静态第一帧和独立静态空闲阶段。静态项目须通过准备验证且无动画特效；未提供时，报告没有静态阶段证据。工具检查命令流量下的限帧、稳定暂停/静态时零活动、清除及 shutdown/join；它主动产生状态请求，不能把该过程当作普通空闲 CPU/功耗基线。报告的逻辑 RGBA 纹理/FBO 字节不含真实驱动 VRAM、EGL 缓冲或视频资源。

添加 `--properties /absolute/path/to/choices.json` 可使用作者属性覆盖值准备并测量项目。JSON 必须是声明名称到对应类型值的对象，文件上限为 64 KiB。准备过程共用 LWE 的验证；Probe 不修改原始项目，也不把这些选择保存到 settings/session 文件。

`scene_resources` 分开诊断 CPU 准备、Scene 渲染、视频渲染、混合输入源切换和后端反复重启。包括 `cpu` 模式在内都要求 `LWE_REAL_DESKTOP_TESTS=1`；运行时模式要求恰好一个物理输出。每次清除先确认活动会话/指标为空的状态 ACK，再采样；后端关闭后等待 join。工具逐条输出 JSON 行，不在进程中累计报告，不写 settings/session：

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
lwe_scene_video="/absolute/path/to/local/video.mp4"
lwe_scene_resource_report="$(mktemp /tmp/lwe-scene-resources.XXXXXX.jsonl)"
LWE_REAL_DESKTOP_TESTS=1 cargo run --locked -p lwe-engine --example scene_resources -- \
  "$lwe_scene_project" "$lwe_scene_assets" "$lwe_scene_video" mixed 30 \
  > "$lwe_scene_resource_report"
```

把 `mixed` 替换为 `cpu`、`scene`、`video` 或 `restart`，循环次数为 1–200。`cpu` 只准备/释放 Scene，不启动桌面后端；`restart` 每轮应用/清除 Scene，并关闭/join 后端。其余模式在多轮之间保留同一后端。

JSON 记录 Rust 请求的活跃/峰值字节、glibc 已用/空闲/mmap 块、`/proc/self/smaps` RSS 分类、可用的 DRM fdinfo 内存字段、FD 和线程数。Rust 计数不含直接原生分配，其逻辑 realloc 峰值也不包含原生分配器临时保留的新旧缓冲。glibc 空闲块是分配器保留空间，不是活跃壁纸资源；mmap 与驱动分配需要分别解释。采样自身会临时分配 `/proc`/JSON 缓冲，并有固定预热开销。这些进程快照不是总 VRAM、帧耗时、功耗或全部原生分配无泄漏的证明。在全新子进程中设置 `MALLOC_ARENA_MAX=2` 只用于受控分配器诊断，不是生产设置或已验收的内存修复。

`scripts/scene-ui-smoke.py` 使用原生 Tauri WebDriver DOM 操作与真实 IPC，以原生鼠标点击选择 Library 项目与显示器、应用、读取 Desktop 反馈与隔离会话、检查标题栏关闭隐藏但壁纸继续、重启自己的应用进程、确认恢复、清除，并可检查阻断项目。它输出 UI/桌面截图及 `report.json`。当前工具要求恰好一个物理输出、已有 shell 构建、`PATH` 中可用的 `tauri-driver` 和 `WebKitWebDriver`、`grim`、当前输出上未命名的空工作区，以及已获取的 Scene 内容。UI 显示器标签可与后端输出名不同，报告会同时记录。测试 debug shell 时，在另一终端运行 `pnpm dev`，并先执行 `cargo build --locked -p lwe-shell`：

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3337606455"
lwe_scene_blocked_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/2454418355"
lwe_scene_driver="$(command -v tauri-driver)"
lwe_scene_ui_report="$(mktemp -d /tmp/lwe-scene-ui-report.XXXXXX)"
LWE_REAL_DESKTOP_TESTS=1 python3 scripts/scene-ui-smoke.py \
  --binary "$PWD/target/debug/lwe-shell" --driver "$lwe_scene_driver" \
  --project "$lwe_scene_project" --blocked-project "$lwe_scene_blocked_project" \
  --blocked-reason unsupported_hdr \
  --output "$lwe_scene_ui_report"
```

脚本创建并删除自己的隔离 `XDG_CONFIG_HOME`，保留 `HOME`，临时命名验收工作区，结束时恢复此前获得焦点的工作区。壁纸截图前会检查该工作区获得焦点且没有遮挡。`--output` 是报告目录，不是显示器名。除业务状态断言外还需检查截图，显示 Running 本身不足以证明画面。缺少驱动、合适的空工作区或桌面条件时，应记为验收阻断。工具不能证明硬件电池切换、实测 VRAM、功耗、24 小时运行或独立多输出行为。

追加 `--playback-settings` 可执行原生 Settings 操作：保存 15 FPS 和电池暂停，在壁纸运行时改为 30 FPS，随后在自有进程重启后核验保存偏好。示例显式检查 `2454418355` 的 `unsupported_hdr`，复跑前应重新核对阻断项目/原因。`3659880761` 现在通过原始包准备，不再是当前泛光阻断示例。UI 保存值不测量实际节奏或硬件电池转换，下方独立桌面服务测量覆盖引擎节奏。

原生属性/失败/全屏验收应使用默认 `3659880761` 项目，追加 `--scene-properties --scene-properties-write-failure --long-occlusion`，可同时加 `--playback-settings`。`--scene-properties` 需要作者声明的 bool 和可修改 slider，保存编辑后验证重启的会话/控件恢复，并重置全部作者默认值。写入失败标志要求非 root 用户，仅更改工具自有隔离配置的权限，验证回滚/草稿保留后重试。`--long-occlusion` 只用自有 Tauri 全屏窗口覆盖至少十二秒，不使系统休眠/锁屏，也不测 callback 暂停。应复核真实壁纸、编辑器、回滚与恢复截图，自动业务断言本身不足以通过画面验收。

控件较多的项目可添加 `--all-scene-properties`，它会启用属性流程并编辑全部声明控件。bool 和 slider 使用原生 WebDriver 鼠标/键盘操作；color 通过已渲染颜色输入的实际 input 事件修改，再走正常 Save/IPC 流程，不测试系统颜色选择器。原始 `2903458386` 的二十个控件使用此参数，并同时启用 `--scene-properties-write-failure --playback-settings --long-occlusion`。

### 已记录的实验性 Scene 验收

2026-10-09 的开发版本验收环境为 Wayland 与 `niri`、AMD Radeon 680M（`1002:1681`，`amdgpu`），仅一个 `eDP-1` 输出，2160×1440、缩放 1.25。场景样本 `3337606455` 使用默认零 X/Y 相机，包含一个图片图层与 Shine：5 个特效 pass、总计 6 个 pass。全分辨率截图确认 Shine 实际可见渲染。

最终兼容性门控在本地目录中仅允许该 Scene 样本播放。`desktop_apply_flow_scene_video_switch_clear_and_restore` 通过了三轮 Scene ↔ Video/应用/清除、隔离会话保存失败回滚、commit 已处理后的取消，以及后端关闭并 join 后的 Scene 分配恢复。原有三轮视频重新应用测试也通过。测试使用隔离的 `XDG_CONFIG_HOME`，已恢复原工作区画面，用户原配置未修改。

在 2026-10-09 的这次验收中，`3258032485` 因非零 X/Y 相机语义未确认，以 `unsupported_camera` 阻断。此前截图属于研究，不证明支持。这段历史记录只覆盖单显示器上的一个已验收 Scene，不代表完整 Scene 兼容；独立多显示器行为当时未测试。场景路径由 Rust 独立实现，不依赖 Open Wallpaper Engine 运行时。

### 2026-10-10 开发记录

新的受限相机分支基于本机原版 `wallpaper64.exe` 的 REA 证据：正交加载器在活动相机路径向量为空时重置保存的 eye/center/up，运行时相机更新使用重置后的值。LWE 仅允许显式固定尺寸、保存向量平行且向前、默认 up、未启用自动投影、路径缺失/为空，其他相机模式仍阻断。该证据支持 `3258032485` 所用的受限实现，不代表与原引擎视觉一致。原版视觉 A/B 尚未执行。

| 范围 | 已实现的证据/工具 | 本版验收状态 |
| --- | --- | --- |
| P0 inventory | 有界声明/结构 API、同 descriptor PKG 指纹、隐私与循环前总预算回归 | 最终 `scene-inventory-p0.json`：11 项、2,695,798 字节，11 个 PKG 的 SHA-256 均匹配基线。可达声明包括五个项目中的 31 处 JS、五个项目中的 20 处 puppet，三个项目存在层级声明；不判定运行时资格。 |
| P1 运行时 | 每输出截止时间、稳定静态/暂停空闲、计数器、包含回滚资源的 512 MiB 逻辑 RGBA/FBO 核算、一个/两个临时目标；EGL 生命周期清理和单次初始视频加载 | `scene-probe-shine.json` 已通过频繁状态请求下的 30 FPS swap 完成率上限和独立 60 秒暂停/静态空闲窗口。资源后续记录见下文。Settings FPS/电池持久化及实时接入、独立 15 → 30 FPS 桌面服务测量和有限原生 Settings/重启流程已通过。显示时间戳、GPU p95、硬件电池/交流电转换、VRAM/功耗/长期运行仍未验证。 |
| P0 真实 UI | 原生鼠标选择显示器 → 应用 → Desktop → 标题栏关闭/隐藏 → 自有进程重启/恢复 → 目标清除，隔离会话与无遮挡截图 | `3337606455`、`3258032485` 和原始默认 `3659880761` 通过原生鼠标应用/隐藏/新进程恢复/清除与截图复核；旧报告保留 365 当时的阻断解释。365 默认流程另通过 Settings 15 → 30 FPS/重启偏好，下文记录的独立属性/失败/重置/全屏流程也已通过。清理与原工作区恢复均通过。 |
| P2 相机 | 自建正向/阻断样本、原版二进制控制流/常量证据 | 原引擎视觉 A/B 与日用一致性未验证。 |
| P2 属性 | 精确 bool/slider/color 绑定默认值和经验证的项目/输出编辑；分配/属性原子持久化与回滚 | 未知表达式/类型/形状及错误声明继续阻断。原生 bool/slider 编辑、真实写入失败回滚/重试、inode 原子替换、新进程控件恢复与完整默认重置已通过；CPU/持久化测试仍与 UI 证据分开。 |

此前开发快照通过了资源 crate 的 18 个测试及 clippy；独立隐私/预算/指纹复核没有剩余阻断。Rust fmt、workspace clippy、check 和测试已通过。普通引擎测试通过 49 项、忽略一个 GPU 样本；随后在 surfaceless EGL 下显式运行该样本并通过，普通 ignored 结果不作为其 GPU 证据。最终 inventory 已确认共享的 16,384 mip/16,384 动画帧预算在循环前扣除，哈希后再次核验索引 descriptor stamp。CSS 修复后的前端类型检查报告零错误/零警告，25 个文件中的 113 个测试及生产构建均通过。Python 套件通过 88 个测试，包含五个 UI 工具安全回归。上述数量属于此前快照，后续设置/属性/资源改动需要独立记录最终检查。文档构建结果单独记录。100 轮结果见下文。

2026-10-10 的 `3337606455` UI 流程在所记录的 Wayland + `niri`、AMD 680M 单 `eDP-1` 机器（2160×1440，缩放 1.25）上通过。原生鼠标选择核验了 Library 身份 `d6d309c547f3c764` 与显示器身份 `niri:BOE:0x0893:eDP-1`（后端 `eDP-1`，UI 标签 `BOE 0x0893`）。应用保存分配后，对应 Desktop 卡片显示 Running。正常标题栏 Close 隐藏窗口，同一应用 PID、壁纸和会话保留。测试随后结束自己的进程组，启动新应用进程，确认保存恢复并清除目标。`3659880761` 保持可浏览，Apply 禁用且显示属性绑定解释。应用/恢复/清除的背景截图已确认获得焦点且无遮挡。

原生鼠标测试发现 CSS 作用域问题：`body > div` 把应用根尺寸套到了显示器菜单的 portal wrapper，菜单因此超出可视区。应用根新增 ID `lwe-app`，相关样式改为 `body > #lwe-app`，修复后原生鼠标选择通过。验收使用原生 WebDriver 点击，没有合成 DOM 输入。

最终 Shine 与 Shake 定时截图在静态桌面部件以外的背景区域都有像素变化（ROI x=100–1700，y=400–1400）。两份 UI 报告均完成画面审查，并有独立 visual-review 与 animation-difference 记录。这证明像素变化，不证明与原引擎视觉一致。普通托盘 Show/Quit 未测试，结束自有进程仅是测试中的重启步骤。`3258032485` 最终 UI 复跑也通过：Library 身份 `18bab41e6ab0c721`、同一显示器的原生鼠标选择、同 PID 关闭隐藏、自有进程退出后新进程恢复、目标清除，以及 `3659880761` 当时的属性阻断解释。报告无清理错误，恢复原工作区 ID 1。swap 完成率/空闲 Probe 及 100 轮生命周期压力均通过，记录见下文；后续资源诊断仍不足以证明日用 RSS 稳定。

最终 `scene-probe-shine.json` 记录了 `eDP-1` 上设置 30 FPS 上限的 engine-only Probe，不写 session/settings，也不测试 UI。动态窗口每秒处理约 475.93 个状态请求，稳定的逻辑 Scene RGBA/FBO 负载为 51,871,748 字节（49.47 MiB），不等于实测 VRAM。

| 阶段 | 窗口 | 完成的 swap | 其他观察 |
| --- | --- | --- | --- |
| 动态 Shine | 60.000055 秒 | 1,773 次，约 29.54997/s | 频繁命令唤醒下 draw 为 1,773 次。 |
| 稳定后的暂停 | 60.000080 秒 | 0 | Render attempt、draw、empty frame、callback 请求/接收均为零。 |
| 恢复 | 3 秒 | 89 次，约 29.66595/s | 动画恢复且仍受配置上限约束。 |
| 暂停时的自建静态候选 | 第一帧 | 1 | 恰好一次 draw 和 swap，完成候选接纳。 |
| 稳定后的静态播放 | 60 秒 | 0 | Render attempt、draw、swap、callback 请求/接收均为零，逻辑负载 1,032 字节。 |
| 清除与关闭 | Probe 结束 | — | 输出指标移除，后端关闭后 join 成功。 |

这些是引擎完成 swap/callback 的计数及有限空闲窗口观察，证明命令流量下的截止时间约束，以及该窗口内没有持续暂停/静态渲染循环。它们不测量合成器显示时间戳、GPU 渲染耗时 p95、普通空闲 CPU/功耗、VRAM、24 小时运行或多个输出。下方已完成的 100 轮生命周期结果不证明内存平坦或日用就绪。

最终 `scene-stress-100.json` 和日志记录了同一后端上交替 Shake/Shine 的 100 轮循环，每轮 Scene → Video → Scene → Clear。Rust 测试在 424.37 秒内通过，外层流程耗时 430.392 秒。隔离保存失败回滚、commit ACK 迟到后的截止时间取消、后端关闭/join、保存的 Scene 恢复和最终清除均通过。用户原配置未改变，自有应用进程已退出，工作区 1 已恢复。

整个测试进程的 RSS 包含准备/缓存/引擎/libmpv，峰值 832,548,864 字节（793.98 MiB），结束时 714,219,520 字节（681.13 MiB）。四个时间段的中位数从 562.84 → 620.44 → 641.48 → 664.35 MiB 上升。这不是应用专属内存或 VRAM，在该次验收时增长尚未归因。P1 日用资源稳定性未通过。四段 FD 最大值均为 22，结束为 14；起始 4 不是稳定运行基线。这些窗口内未观察到 FD 增长，不证明长期稳定。

资源后续工作增加了 context/display 的显式销毁和幂等 surface 清理，覆盖初始化失败路径；display/library 的所有者保留到最后一个 context/surface 释放。视频构建改为只加载一次初始输入源，移除了连续两次 `loadfile replace`。这些改动处理实际的生命周期和重复工作问题，不宣称它们解释了此前全部 RSS 增长。

私有 `resources-{cpu,scene,video,mixed,restart}.jsonl` 和 `resources-summary.json` 记录同一单输出 680M 机器上的独立 engine-only 子进程，均成功退出，用户原配置未改变。下表比较首轮与末轮释放/清除后的采样，不是冷启动基线：

| 模式 | 循环次数 | 释放/清除后 Rust 请求的活跃字节 | 其他观察 |
| --- | --- | --- | --- |
| CPU 准备/释放 | 30 | 2,177 → 2,177 | 不启动桌面后端，解码的 Scene 数据每轮释放。 |
| Scene 应用/清除 | 30 | 49,259 → 51,539 | 后端保持运行，该小幅变化不证明已经达到稳态。 |
| 视频应用/清除 | 30 | 49,259 → 49,259 | Rust 活跃字节固定，RSS 仍上升。 |
| Scene → Video → Scene → Clear | 50 | 49,637 → 63,029 | glibc 空闲块：166,546,944 → 654,284,608 字节；RSS：290,889,728 → 614,535,168 字节。glibc 已用块从 15,622,656 变为 16,472,256 字节。 |
| 后端重启 | 20 | 每次关闭/join 后均为 2,229 | 每个停止后的快照均为四个 FD、一个线程。 |
| 全新子进程 `MALLOC_ARENA_MAX=2` 混合模式 | 100 | 49,260 → 51,846 | glibc 空闲块：134,691,232 → 198,131,456 字节；清除后 FD 均为 12，末轮清除后线程为 19。 |

这些计数支持分配器保留的大量空闲空间是视频/混合模式进程内存趋势的一个来源，不证明全部原生分配正确，也不证明 RSS 达到稳定上界。Rust System 分配计数与 glibc 块计数的范围不同，Probe 也未包含 shell 的目录/缓存/UI 工作。P1 继续开放。

私有 `resources-mixed-arena2.jsonl`/日志中的受控 arena 对照在 174.26 秒内通过了 100 轮。最终关闭/join 后 Rust 活跃字节为 2,228，四个 FD、一个线程、无 DRM client。末轮清除后的 RSS 为 226,689,024 字节，另有 29,519,872 字节换出，文件页驻留也发生变化。观察结果与 arena 策略影响空闲块保留相符，但默认模式运行 50 轮、受控模式运行 100 轮，内存压力不同；它们不是生产性能对照或 RSS 收益证明。环境变量只传给诊断子进程，未采用产品分配器调优或强制 trim。

后续实现还接通了可持久化的 FPS/电池控制与引擎实时策略更新。实时更新失败时，已保存设置仍为权威状态，UI 会暴露运行时问题。FPS 选项限制按节奏执行的渲染/swap；电池暂停保留用户手动暂停意图。显式启用的 `desktop_settings_live_updates_preserve_assignments_and_backend` 测试在隔离配置下用时 22.05 秒通过，私有 `settings-live.log` 记录如下：

| 保存的上限 | 测量窗口 | 完成的 swap/速率 | 状态请求 |
| --- | --- | --- | --- |
| 15 FPS | 10.0003 秒 | 147 次；14.6996/s | 4,589 |
| 实时改为 30 FPS | 10.0004 秒 | 296 次；29.5987/s | 4,758 |

两份采样均确认后端相同、分配字节未变，并回读保存的偏好。电池暂停偏好已保存并进入引擎策略，机器报告 `on_battery=false`、`power_paused=false`，没有测试实际电池/交流电转换。这些是实际服务路径完成的 swap 计数，不是合成器呈现时间戳或 Settings UI 操作。

独立原生 UI 流程在私有 `ui-settings-verified/report.json` 中通过，`visual-review.json` 记录画面复核完成。原生英文 Settings 操作保存了 15 FPS 和电池暂停，在壁纸运行时改为 30 FPS，并在新的自有进程恢复壁纸后核验 30 FPS 与电池偏好；中文 Settings 布局也已复核。应用、同 PID 关闭隐藏、自有进程组完整退出后重启、保存恢复、目标清除和 `3659880761` 当时的可见泛光阻断均通过。清理错误为空，隔离配置已删除，原工作区 ID 1 已恢复。此前 `ui-settings-final` 自动结果的 15/30 FPS 截图取景不完整，不计为这次新的视觉验收。这个有限流程不证明硬件电池转换、锁屏/休眠恢复、功耗、24 小时运行或完整 P1 就绪。结果来自开发 checkout，安装包验收仍待完成。

### 混合 Scene 与每输出属性：2026-10-10

独立 Rust CPU/GPU 路径已实现 `3659880761` 原始包的活动粒子树、WaterRipple/Shake/Iris 特效、LDR 泛光和作者配置的单条 FLAC 循环层。准备成功，包含 7 个渲染图层、5 个特效和 21 个 pass。默认配置为 6 个可见粒子根系统加全部 5 个子系统，最大粒子/顶点/work 为 3,965/26,790/30,980。第七个 Reactive 根系统是字面量 `visible:false`，保留作者隐藏状态才是正确默认播放；启用其音频分析仍阻断。没有为通过准备而关闭作者开启的粒子/特效/泛光层。

粒子按 SceneClock 执行有界的发射器、初始化器、运动/阻力/重力、透明度/尺寸振荡、角运动、湍流、sprite/rope 渲染及 spawn/follow 子系统。未知行为明确报错。OWE 源码仅供行为参考，Rust 运行时使用独立状态结构、RNG 和梯度/curl 场；原版噪声、子系统生命周期/变换、rope 采样及完整画面等价仍未验证。

REA 确认了 LDR 四 pass 输入链、RGBA8 颜色目标、向下取整且最小为二像素的缩放尺寸，以及来自完整输出尺寸的 texel。另确认单条循环无随机 min/max 间隔、图层音量采用平方。这些是原生静态发现，与视觉或音频验收分开。

最终有限 engine-only Probe 私有记录为 `runtime/365-engine-final-60s.json`，在所记录的 Wayland + `niri`、AMD 680M、单 `eDP-1` 机器上通过，shutdown 已 join、清除后输出指标移除；不写 session/settings，不进行 Tauri UI 操作：

| 阶段 | 观察 |
| --- | --- |
| 默认混合 Scene，30 FPS 上限 | 六十秒完成 1,677 次 swap，约 27.95/s；约 466 次状态请求/s，empty frames 为零。 |
| 稳定暂停 | 六十秒与 28,000 次状态请求下，render attempt、draw、swap、empty frame、callback 均为零。 |
| 恢复 | 三秒完成 85 次 swap，约 28.33/s。 |
| 逻辑 GPU 负载 | RGBA 纹理/FBO 与粒子 VBO 共 134,862,328 字节，不含驱动/EGL/视频开销。 |

另一次私有 Pulse sink/PCM Probe 在候选、暂停、静音和 drop 阶段观测到零 PCM，激活后 RMS 为 2,894.53。自有 sink 已清理，原有 sinks、默认 sink/source 未变。这不证明硬件扬声器输出、每个循环边界或完整属性/音频回滚事务。

独立的默认原生 UI 流程在私有 `runtime/ui-365-default-final/report.json` 中通过，`mockIpc=false`，使用隔离配置并人工复核截图。原生 Library/显示器应用达到 Desktop Running；标题栏关闭隐藏同一 PID，壁纸/session 保留；整个自有进程组退出后，新 Tauri 进程恢复 Scene，目标清除还原基线。原生 Settings 保存 15 FPS 和电池暂停，在 PID/分配字节不变时改为 30 FPS，新进程恢复后回读 30 FPS 与电池偏好。英/中文截图已核验完整构图、两帧粒子变化、恢复可见、清除基线和标签布局。cleanupErrors 为空，工作区 1 已恢复，没有关闭原始默认活动特性。这通过的是默认 UI 流程，独立属性与有限全屏流程记录于下文，日用验收仍未完成。

有限 `runtime/365-resources-mixed-20.jsonl` 及摘要记录 20 轮 `3659880761` → 视频 `2848041076` → `3659880761` → 清除。每次清除均确认 active status/metrics 为空，关闭已 join。清除后 FD 恒为 12、Rust 活跃字节恒为 48,811；首/末轮进程 RSS 为 257,609,728 → 287,825,920 字节，glibc 已用块为 14,897,040 → 15,433,312 字节。关闭后为四个 FD、一个线程、2,227 Rust 活跃字节、191,705,088 字节 RSS 和 250,144,480 字节 glibc 空闲块。这支持该有限窗口的资源归还及无 FD 累加；显著分配器/驱动 RSS 保留不支持内存稳定或全部原生分配无泄漏声明，功耗与日用长期运行仍未验收。

最终实现快照通过 workspace fmt/clippy/check/测试和 98 个 Python 测试。错误状态重置修复后，前端类型检查零错误/零警告，25 个组件回归及 26 个文件中的全部 148 个测试通过，生产构建通过。新增错误恢复仍属于组件证据，与已记录的原生正常保存/重置流程分开。三个 ignored GPU 样本在独立 surfaceless EGL 运行中显式通过：自建图片/GL 状态、泛光 halo/状态，以及本机原版资源的 bloom/WaterRipple/Iris/generic particle/填充方向/cutout 像素。此结果与 CPU 准备及原生桌面/UI 验收分开，不是原引擎完整画面 A/B。文档构建作为这些记录完成后的最终整合检查。

经验证的 bool/slider/color 编辑按项目和输出保存，与分配同处 `session.toml`。“保存并应用”事务为第一帧/commit 接纳 → 一次原子保存分配与属性 → Finalize 并启用音频。原子发布前的准备、候选接纳或保存失败保留原渲染器与选择；保存成功后的持久分配保持权威，延迟 Finalize 清理不能撤销它。普通保存/清除保留其他显示器分配和所有无关选择，清除最后一个分配也不删除已保存属性；无文件视为空状态，损坏或超限状态禁止覆写，诊断省略属性值。下述独立原生属性流程已通过。作者 schema 变化使形状合法的保存选择失效时，无快照的属性加载错误状态也提供重置默认值；它让 `{}` 通过默认准备，并仅在应用/保存成功后删除该显示器/项目选择，普通保存仍要求经验证的声明/值。不支持的默认值、无法读取或损坏的 session 仍明确报错。此新增错误恢复路径由组件回归验证；已记录原生流程覆盖正常保存/重置，不是独立作者 schema 变化场景的原生验收。

原生属性/失败/全屏流程私有记录为 `runtime/ui-365-properties-final/report.json`，真实 checkbox/number 输入与人工截图复核均通过。作者属性 `hdr` 从 true 改为 false 是关闭 **LDR 泛光**，不是启用 HDR 渲染；`newproperty2` 从 0 改为 0.1，对应作者音频层音量。仅把隔离 `config/lwe` 目录设为 0500 后，真实 Save 因 PermissionDenied 失败，session 字节/inode 未变，编辑草稿保留，原 Scene 仍 Running 且实际可见。恢复 0700 后用相同选择重试成功：完整 TOML 通过新 inode 原子替换，已打开 descriptor 仍读到旧值；新 Tauri 进程恢复保存的属性与真实控件。重置默认值删除 overrides，恢复所有作者值（包括 bloom=true）和原 session 字节哈希，并保留分配。

随后自有 Tauri 窗口完整覆盖逻辑 1728×1152 输出 12.000 秒，揭示后仍 Running，两份截图可见粒子变化；没有直接测量 callback/遮挡暂停，不能证明系统休眠或锁屏恢复。所有自有进程组停止，隔离配置删除，工作区 1 恢复，cleanupErrors 为空。通过的是有限属性/全屏流程，不代表原版视觉一致、24 小时运行、真实休眠/锁屏、硬件电池转换、功耗/VRAM 稳定或安装包验收。

原引擎对照尝试使用隔离的引擎副本和 Wine prefix；对照 UI 在获得可用截图前以 `c0000005` 失败，自有清理无错误、源文件哈希未变。这不是原引擎 A/B 结果。只读 11 项 CPU 复跑已私有保存为 `scene-prepare-corpus-2026-10-10-mixed.json`，包含二进制/inventory SHA-256、原包哈希、耗时、退出码及原始首项阻断。通过 `3258032485`、`3337606455` 和 `3659880761`，八项首阻断为五项 SceneScript、`2798622011` 相机、`2454418355` HDR 和 `3485147659` 图层数量上限；后者的 feature code 为 null，附精确图层数量验证错误。11 个原包均与 P0 一致。复用现有编译示例，没有构建、启动 backend/GL 或写 settings/session。3/11 CPU 结果不是 Workshop 兼容率或原生 UI 验收。有限 CPU/GPU/PCM 证据及已通过的默认原生 UI 流程须与尚未完成的原版视觉对照、24 小时运行、十次真实休眠恢复、硬件电池/交流电转换、功耗/VRAM 测量、独立多显示器及安装包验收分开，不能据此宣称日用就绪。

2026-10-09 验收及此前显示属性/泛光阻断的 UI 截图保留为历史证据。新增原始包混合 Scene 实现和有限 Probe 不扩大旧流程的验收边界，也不证明日用支持。

三个独立 GPU 像素/GL 状态样本可按以下方式重跑，无须应用桌面壁纸；原版资源覆盖需要本机默认 365 项目与已安装资源：

```bash
lwe_scene_project="$HOME/.local/share/Steam/steamapps/workshop/content/431960/3659880761"
lwe_scene_assets="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
for lwe_scene_gpu_test in \
  scene::renderer::tests::offscreen_pixels_and_gl_state_are_preserved \
  scene::postprocess::tests::offscreen_bloom_halo_threshold_tint_and_state \
  scene::postprocess::tests::offscreen_original_bloom_and_effect_shader_assets
do
  EGL_PLATFORM=surfaceless LWE_SCENE_TEST_PROJECT="$lwe_scene_project" \
    LWE_SCENE_TEST_ASSETS="$lwe_scene_assets" \
    cargo test --locked -p lwe-engine --lib "$lwe_scene_gpu_test" \
    -- --ignored --exact --test-threads=1 --nocapture
done
```

这些样本在可用 EGL/GPU context 上验证相应像素用例与 GL 状态恢复，不比较原引擎截图，也不证明 Wayland/多输出生命周期行为。

### Space 2026-10-10

原创意工坊 [2.5D Space（`2903458386`）](https://steamcommunity.com/sharedfiles/filedetails/?id=2903458386) 在不改写原包、不删减图层/特效的条件下，通过共用 CPU 准备与真实 LWE 应用：十个图片图层、十一个特效和二十四个 pass。八个 Scroll 实例、音频 Pulse/色差和四 pass 项目自带泛光保留作者顺序。该项目关闭通用泛光，但自定义泛光继续启用；项目资源保持高于本机 builtins 的优先级。

十四段未修改的标量脚本通过七十个独立数值用例，覆盖默认、最小、最大、零值与混合属性快照。Oxc 先检查模块 AST，再由有界 QuickJS 在 CPU worker 执行，渲染线程不运行脚本回调。十二个原始着色器程序以 GL 130 链接成功。真实桌面输出 monitor 采集另通过立体声低/高频、暂停/恢复、静音和清理检查；每通道十六档归一化频谱不能证明 Windows 分频或视觉一致。

私有 `space-runtime/gl-acceptance.json` 的二十六项 GPU 像素比较全部通过：八个图片层分别将不透明度设为零均改变超过一百万像素；六组 opacity、色差/Pulse/Bloom 开关、图层顺序和背景颜色按预期改变完整场景。原始着色器合成模式 0/30/31 通过各自像素检查。非零频谱下，暂停强制重绘及 resize 后回到原尺寸保持像素完全一致，恢复后接受新频谱。这些 EGL 检查证明相应作者管线和输入保留，不是 Windows 引擎截图对照。独立的生产 monitor 到桌面流程也已通过：Global Speed 设为零后，安静帧及停止声音后返回的帧完全一致；自有低音量立体声测试音改变壁纸，暂停时在声音变化期间保留该帧，恢复后再次响应声音。像素比较使用现有桌面部件以下 2160×1090 的壁纸区域；用户音频路由、配置、原始资产和工作区均保留。

真实原生 Tauri 流程保存在私有 `runtime/space-ui-delivery/report.json`，在记录的单输出 Wayland + `niri`/AMD 680M 机器上通过：Library/显示器 Apply 达到 Running；全部二十个作者控件（十六 slider、三 bool、一 color）修改并保存；隔离 PermissionDenied 保留旧分配/选择和草稿，重试原子替换完整会话。新的自有应用进程恢复已保存控件与壁纸，Reset 恢复全部作者默认值。Settings 15 → 30 FPS、十二秒自有全屏覆盖/揭示、目标清除及自有资源清理也通过。颜色使用已渲染控件的 input 事件，未操作系统颜色选择器。这个有限 UI 流程未测量硬件休眠/锁屏或遮挡 callback 策略。

最终 engine-only Probe 使用 30 FPS 上限并频繁查询状态：十二秒约 27.25 次完成 swap/s，稳定暂停十二秒 render attempts/draw/swap/empty frames/callback 均为零，恢复约 27.33/s。按图层尺寸共享的临时目标、命名目标和完整画面目标共计 202,180,100 逻辑 GPU 字节。配置上限不保证达到 30 FPS；统计不含驱动/EGL/视频开销，也不是实测 VRAM。Windows A/B、24 小时运行、十次真实休眠恢复、硬件电池转换、功耗测量、独立多显示器与安装包验收仍未完成。

### Yakuza 扩展 2026-10-12

本批在纯标量路径之外增加有界动态 SceneScript worker。接纳的生命周期为模块初始化、`init`、初次声明属性交付及重复 `update`；回调可用有限 `Vec3`/`copy`、声明控件和图层 `origin`/`scale`/`angles`/`color`/`alpha`/`visible`，`init` 返回值可更新所绑定属性。`createLayer(path)` 仅在初始化期间接受安全的图片模型 JSON，并经过正常共用准备/解码；渲染线程不运行 JavaScript VM。先保留作者对象 handle，再分配创建 handle；已验证 worker 状态先于首帧 ACK，暂停强制重绘保留已接纳状态，可逆应用/保存/Finalize 边界继续负责失败和回滚。该子集未实现 config/handle 重载、任意 clone、销毁、场景查询、Vec2、engine 时间、光标/媒体事件或通用交互。官方 [init](https://docs.wallpaperengine.io/en/scene/scenescript/reference/event/init.html)、[update](https://docs.wallpaperengine.io/en/scene/scenescript/reference/event/update.html) 和[场景 API](https://docs.wallpaperengine.io/en/scene/scenescript/reference/class/IScene.html) 的完整契约大于 LWE 当前接纳范围。

Oxc 在 QuickJS 前校验源码与 AST：最多 32 个绑定、每段 64 KiB/合计 256 KiB 源码、AST 深度 48、作者/创建对象 handle 合计 128、VM 内存 8 MiB、栈 256 KiB，以及初始化 100 ms/每帧执行 10 ms。原型/全局逃逸、未经验证的计算属性写入、导入、未知 API 与不支持输出严格阻断。运行异常、执行超时、worker 退出与取消须保留上次持久分配；CPU 测试覆盖有限规则，不能替代 GPU 或原生应用验收。

`registerAudioBuffers(64)` 仅允许模块全局注册，`init`、`update` 或属性回调中注册会被拒绝；其 `average` 是实际固定长度 `Float32Array`，由采集桌面输出后的真实 64 档变换就地刷新。十六档立体声 shader uniform 保持独立，扩展它们不能产生 64 个独立频档。LWE 目前仅接纳这一分辨率的 average 通道，输入不可用时静音，暂停/回滚保留已接纳输入与状态。官方[音频缓冲](https://docs.wallpaperengine.io/en/scene/scenescript/reference/class/AudioBuffers.html) 还有其他通道/分辨率；Windows 分频、幅度归一化和画面一致性仍未验证。

Combo 声明现保留作者标签及非空唯一字符串选项，默认值与全部覆盖值须属于声明选项；界面提供带可访问标签的 select，字符串保存至同一项目/输出选择快照。精确 `{name, condition}` combo 相等条件产生 bool 可见性；混合 script/user 包装先提取脚本，再验证剩余 user 条件。标量 shader 映射不消费 combo 字符串。原始 Yakuza 控件为 `av` 的 `av`/`0`、`pic` 的四个 variant（`Majima_1`、`Kiryu_1`、`YakuzaDragon`、`Majima_2`）及 `schemecolor`，无效声明/选择不得静默替换为默认值。

单图片 `TEXS0002` 粒子图集保留经验证的矩形 UV 帧、序列/随机帧选择及序列模式下受支持的相邻帧插值。带时长 `TEXS0003`、多图片图集及作为普通图片/特效纹理使用仍门控。新增平面粒子组件包括湍流初速度、随机透明度/角速度、角运动、透明度/尺寸振荡与湍流，有限模拟/几何/work 预算继续执行。保留的原生结构/组件证据支持特定字段，噪声/随机行为及原引擎完整粒子 A/B 尚未核验。

Yakuza 作者的 `perspective:true` 音频条仅进入共面固定正交子集：origin Z=0、无 X/Y 旋转及无相机运动/视差。保留的原二进制证据支持规范相机/投影行为，未确认逐层 perspective 分支是否附加其他映射。平面等价须标为数学推断，保留原引擎视觉一致性缺口。

共用 `genericimage2` 图片适配现保留缺省 VERSION 的旧版 Brightness/Alpha 及显式 VERSION 1/2 Color4，不增加 renderer API，也不放宽未知输入/combo 门控。下述最终 GPU 用例已验证原始图片 color/alpha/可见性，原引擎视觉一致性仍未核验；最终适配也在 VERSION 缺省时保留作者对象 color/alpha，修正原 owner 音频条的黑色色调且保留 handle；此前 Space VERSION 2 记录保留。

**已记录有限验收：**私有 `yakuza/cpu-prepare-report.json` 通过全部八组原始包组合：四个 `pic` 背景 × `av`/`0`。私有 `yakuza/gpu-report-final.json` 通过五组完整场景原生离屏 OpenGL 用例：每个背景开启音频条，另加默认背景关闭音频条。保留 64 个音频条 handle、63 个初始化创建层、原始 4×4 音频条材质/纹理、66–67 个渲染层与 67–68 个 pass。二十五项像素比较和十五轮 worker 销毁/重建均通过，包含原始图片 color、alpha=0 与隐藏断言；注入 64 档音频使每组开启音频条的隔离区域改变 8,070 像素，关闭时差异为零，二十次暂停强制重绘与 resize 后回原尺寸保留已接纳画面。注入输入的 GPU 证据不能证明实际采集路径。

私有 `yakuza/ui-run-2/report.json` 使用最终冻结构建，在单 `eDP-1`、Wayland + `niri`/AMD 680M 上通过真实 `mockIpc=false` Tauri Library/Desktop 流程。私有 `yakuza/visual-review-final.json` 记录完整默认与 Kiryu 场景、恢复控件及中文 Settings 布局画面复核；作者颜色标签仍为原始本地化 token，不代表作者标签已完整翻译。原生控件保存 `av=0`、`pic=Kiryu_1` 和颜色，真实隔离 PermissionDenied 保留 session 字节/inode、可编辑草稿及原 Running 渲染器。重试原子保存完整选择，新进程恢复选择，Reset 删除覆盖值并保留分配、恢复全部默认值。Settings 15 → 30 FPS、同 PID 关闭/隐藏、十二秒自有全屏恢复与目标清除通过；cleanupErrors 为空，工作区 1 已恢复。颜色使用已渲染 input 事件，未操作系统颜色选择器；私有 `yakuza/native-temporal-pixels-final.json` 对桌面部件下方（y≥300）区域进行精确 RGB8 像素比较，默认两帧差 1,297 像素，全屏恢复后两帧差 1,848，证明 Scene 区域存在时间变化，不把每个差异像素归因于某个组件；全屏恢复未测 callback 遮挡暂停或物理休眠。

私有 `yakuza/gpu-report-final-live-audio.json` 另通过真实 PulseAudio/PipeWire PCM 自有静音 null-sink monitor 采集 → 真实 64 档 FFT average → 未修改的原始脚本经 LWE worker → 原生离屏音频条 GPU 像素链路。渲染器为 AMD Radeon Graphics（`radeonsi`、`rembrandt`、ACO），OpenGL 4.6/Mesa 26.2.4。开始播放改变 1,285 个音频条像素，暂停断开/恢复重连后改变 690 像素；暂停期间已显示像素完全一致，停止播放后完全回到静音音频条画面，初始与释放后的 64 元素缓冲均为零。已核验采集源为自有 sink monitor，采集及脚本 worker 释放、自有 sink 卸载，默认 sink/source 未变。该结果证明隔离路由下真实采集/FFT/脚本/GPU 链路，使用离屏像素，不包含扬声器输出或默认设备采集，与已通过的可见桌面 UI 流程分开；不证明用户当前音乐/视频路由或 Windows FFT/视觉一致性。

最终源码/二进制身份私有保存在 `yakuza/final-source-identity.json`，记录的 125 个 Rust 源码/manifest/lock 文件均与冻结源码一致。最终 workspace fmt/clippy/check、466 项常规 Rust 测试及 shell 构建通过，此前前端 154 项测试与 Python 144 项回归也通过；源码检查与运行时验收分开。

最终 `yakuza/space-regression-gpu-report-final.json` 另通过本扩展后的全部二十六项 2.5D Space GPU 比较，未修改的原项目保留十层、十一个特效、二十四个 pass、二十个控件及 202,180,100 逻辑 GPU 字节，覆盖原透明度/特效/顺序/合成、注入立体声频谱及暂停重绘/resize 保留。这是新增确定性离屏 GPU 回归，保留旧 Space 验收记录，不代表另一次桌面/实际音频采集或 Windows 一致性验收。

**剩余验收：**保留原始文件哈希及私有证据，旧阻断结果作为带日期快照。用户明确延后物理休眠恢复，因为它会挂起 Codex，本批不执行，也不作为完成条件。旧 24 小时任务为释放显示器，通过核验自有 PID 后正常结束，证据保留；这是中断运行，不是长期验收通过。真实多显示器、安装包、Windows A/B、硬件电池/功耗、高级脚本、骨骼与 HDR 继续单独保留未验证或未支持状态；这些有限结果不能证明日用就绪。

## 报告问题

有效的问题报告应包含：

- 安装包类型（`lwe`、`lwe-git`、`.deb`、`.rpm` 或 `.AppImage`）；
- 发行版和版本；
- 会话类型和合成器/桌面环境；
- 显示器布局；
- 已知的壁纸类型（`video`、`scene` 或 `web`）；
- LWE 显示的兼容性状态；
- Settings 中可复制的诊断信息，其中 Steam Web API Key 会被隐藏；
- 可用的终端日志。

### 持久记录的场景可靠性验收

用户因休眠会挂起 Codex，明确将物理休眠恢复移出当前批次。下列不休眠记录器示例要求零次休眠，不能证明休眠恢复；完整未来恢复目标与辅助脚本单独保留。

使用 `scripts/scene-reliability.py` 启动已编译 `lwe-shell` 库测试二进制的固定副本，选择确切的忽略测试 `services::desktop_service::reliability_tests::desktop_scene_daily_use_reliability`。先执行 `cargo test --locked -p lwe-shell --lib --no-run --message-format=json`，从输出的 `compiler-artifact.executable` 取得路径。验收期间保留完整未提交源码指纹和原始输入。记录器隔离 `XDG_CONFIG_HOME`，Steam 发现仍保留 `HOME`。

设置 `LWE_REAL_DESKTOP_TESTS=1`，参数为 `--hours 24 --cycles 100 --suspend-cycles 0 --sample-seconds 5 --memory-growth-mib 64`，并提供实际 `--binary`、`--project`、`--assets`、`--video` 和 Git 外全新私有 `--output` 目录。固定样本集使用 `--scene-ids 2903458386,3659880761,3337606455,3258032485`，最后追加 `-- services::desktop_service::reliability_tests::desktop_scene_daily_use_reliability --ignored --exact --nocapture --test-threads=1`。首项是连续运行主样本，其余参加最终确认后的往返切换。记录器不会发起休眠。

未来任务明确包含物理恢复验收时，待 `suspend-readiness.json` 显示新鲜、健康的主场景后，先用 `scripts/scene-suspend-cycles.py --check --readiness PATH` 只读检查。RTC 真实循环需要获授权的本机特权执行，提供 `--readiness PATH --output-dir NEW_PRIVATE_HELPER_DIRECTORY --cycles 10`。脚本保留正常 logind inhibitor 和锁屏，等待正常密码认证解锁；不得覆盖已有 RTC 闹钟。凭证不得放入参数、环境、文件、报告或自动化提示。

已结束记录器的 `passed=true` 只证明实际记录的目标，`--suspend-cycles 0` 不能证明物理恢复；未来恢复验收另需真实休眠报告。`report.json`、`manifest.json` 和追加的 `evidence.jsonl` 区分实际呈现、完整切换、内核休眠、资源增长、帧耗时和清理。任务启动、mock/单测、经过一天、中断后重启或重编译源码均不算通过 24 小时验收。

可选的 `--finish-after-soak` 会在有效场景呈现与最终确认的完整往返达到目标后，正常清除、关闭并等待后端退出，即使单独要求的休眠目标尚未完成。此部分结果仍为 `passed=false`、记录器退出码 1，但 `status=completed`、子进程成功退出，并列明完成/未完成组件与 `stop_reason=soak_and_switch_targets_reached_suspend_incomplete`；它不是应自动重启的运行故障。该策略写入不可变 manifest，不能用 `--resume` 更改。中断记录实际收到的信号。首次完整验收可以通过；历史连续性片段仍各自独立，不得追加拼接成不中断的通过记录。
