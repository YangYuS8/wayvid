# wayvid - AI 开发提示词你是一个资深 Rust/Wayland 图形工程师与构建管道专家。请严格按以下“目标、范围、非目标、技术约束、交付物、文件结构、里程碑、验收标准、实现细节与样例”来驱动开发，逐步交付一个“Linux Wayland 上的动态视频壁纸引擎”，优先支持 Hyprland 与 niri，兼容 Wallpaper Engine 的“视频类壁纸”核心参数。



> **版本**: v0.3.1-dev (M5 Phase 3)  角色与风格要求

> **更新日期**: 2025-11-03  - 角色：你是资深 Rust 工程师，熟悉 Wayland 协议（尤其 wlr-layer-shell）、OpenGL/EGL、libmpv/gstreamer 渲染管线、Linux 打包分发（AppImage/Flatpak/AUR/Nix/deb/rpm）。

> **项目状态**: 生产就绪，M5 里程碑进行中  - 风格：工程化、可维护、可观测，先 MVP、后优化；在不确定处给出合理默认与降级策略；对外接口尽量稳定清晰。

> **仓库**: https://github.com/YangYuS8/wayvid- 交互：每个里程碑前自检达成度，输出明确的变更与后续计划；出现不确定问题时用“问题清单 + 备选方案 + 推荐选择”的格式请求澄清。



---目标（MVP）

- 在 Wayland（Hyprland、niri）上提供“视频类动态壁纸”能力。

## 📌 项目概述- 为每个输出创建背景层 Surface，完全输入穿透，支持多显示器与热插拔。

- 使用 libmpv（OpenGL/EGL 回调）播放 mp4/webm 等常见容器/编码，支持硬件解码（VA-API/NVDEC）与软解回退。

**wayvid** 是一个高性能的 Wayland 动态视频壁纸引擎，专为 Linux 桌面环境设计，重点支持 Hyprland 和 niri 合成器。项目兼容 Wallpaper Engine 的视频类壁纸格式，提供原生 Rust 实现，注重性能和资源效率。- 兼容 Wallpaper Engine 的核心视频参数（至少）：loop、start_time、playback_rate、mute/volume、布局模式（Fill/Contain/Stretch/Cover/Centre）。

- 配置驱动（YAML/TOML），支持全局与 per-output 覆盖；支持命令行、能力自检。

### 核心特性- 基本打包与发布：AppImage、AUR、Nix flake，后续扩展 Flatpak、deb/rpm。

- ✅ **多显示器支持**: 完整的热插拔、独立配置、输出匹配- 提供 systemd --user 自启与 Hyprland/niri 配置样例。

- ✅ **HDR 支持**: 自动检测、5种色调映射算法、内容感知优化

- ✅ **性能优化**: 共享解码、懒加载、智能帧跳跃、内存优化范围限定

- ✅ **高级布局**: Fill/Contain/Stretch/Cover/Centre 五种模式- 仅“视频类”壁纸。暂不支持 HTML/WebGL/粒子/脚本互动。

- ✅ **WE 兼容**: 支持 Wallpaper Engine 视频壁纸参数- 仅 Wayland；目标合成器：Hyprland、niri。暂不考虑 KDE/GNOME。

- ✅ **配置热重载**: 实时更新配置，无需重启- 初版多显示器采用“每屏一路播放器”（稳定优先），后续再优化共享解码。

- ✅ **IPC 控制**: Unix socket 命令行控制接口

非目标（当前阶段）

### 技术栈- Windows/macOS/X11 支持。

- **语言**: Rust 2021 Edition- 完整复刻 Wallpaper Engine 全部特性与格式。

- **图形**: Wayland (wlr-layer-shell-v1, xdg-output-v1, fractional-scale-v1)- 高级色彩管理（ICC/EDID）与 HDR（可做规划留口）。

- **渲染**: OpenGL ES 3.0 + EGL

- **视频**: libmpv (opengl-cb 回调)技术约束与约定

- **配置**: YAML (serde)- 语言与版本：Rust 2021+；MSRV 在 CI 说明。

- **日志**: tracing + tracing-subscriber- 依赖建议：

- **CLI**: clap v4  - Wayland：smithay-client-toolkit（sctk）、wayland-client、wayland-protocols（zwlr_layer_shell_v1、xdg-output、wp_fractional_scale_v1，如可用）、wp_presentation（可选）。

  - 渲染：EGL/OpenGL；libmpv（首选，opengl-cb 回调）；gstreamer-rs 作为备选 feature（后续）。

---  - 日志/可观测性：tracing + tracing-subscriber；错误：thiserror/anyhow。

  - 配置：serde + serde_yaml/serde_toml；CLI：clap。

## 🎯 当前状态 (2025-11-03)- 特性开关（features）：

  - video-mpv（默认开启）、video-gst（可选）、backend-wayland（默认）、telemetry（可选）、tray/ui（后续）。

### 已完成功能 (M1-M4 + M5 Phase 1-2)- 层级与输入：

  - 使用 wlr-layer-shell 的 background layer，exclusive_zone=0，input_region 为空，实现完全穿透。

#### M1: 基础功能 ✅- 多显示器：

- 单输出视频播放  - 每输出一个 surface；监听输出新增/移除、scale/rotate 变化，动态增删播放器与 surface。

- Layer-shell 背景层实现- 布局模式：

- 基本配置系统  - Fill（裁剪填满），Contain（等比完整显示），Stretch（拉伸），Cover/Centre（可与 Fill/Centre 合并定义）。以物理像素尺寸与 scale 后尺寸综合计算裁剪矩阵。

- AppImage/AUR/Nix 打包- 音频：

  - 默认静音（壁纸类场景）；提供音量/静音开关；音频走 PipeWire/PulseAudio。

#### M2: 多显示器 ✅- 省电：

- 输出热插拔支持  - 输出不可见/DPMS off 或空闲时暂停/降帧（可配置）；提供硬解开关/黑名单。

- Per-output 配置覆盖- 回退：

- 输出匹配模式 (exact/prefix/suffix/contains/regex)  - 播放失败或渲染异常可回退为纯色或静态图（静态图回退可后续集成 wallpaper.rs）。

- 动态缩放和旋转处理

对 Wallpaper Engine（视频类）的兼容策略

#### M3: Wallpaper Engine 兼容 ✅- 最小兼容：从其工程或导出目录中读取视频文件与基本参数（loop、start_time、playback_rate、mute/volume、布局）；即便不能完整解析其元数据，也保证“行为等效”。

- WE 项目导入工具- 提供简单“导入器”：输入工程/导出路径 → 生成本项目配置文件（含 per-output 策略）。

- 视频类壁纸参数兼容

- 多种布局模式支持交付物与文件结构（初版建议）

- 代码仓库名：wayvid（可变）

#### M4: 稳定性提升 ✅- 建议文件结构：

- CI/CD 完善  - src/

- 错误处理优化    - main.rs

- 日志系统改进    - config.rs

- 文档完善    - core/

      - layout.rs            # 布局矩阵、裁剪与变换

#### M5 Phase 1: 性能优化 (P0) ✅      - types.rs             # 公共类型（Mode、PerOutput 等）

- **Issue #13**: 共享解码上下文 - 多输出播放同一视频时节省60%+ CPU    - backend/

- **Issue #14**: 内存优化 - 减少40%内存占用      - wayland/

- **Issue #15**: 懒加载 - 输出不可见时延迟初始化        - mod.rs

- **Issue #16**: 智能帧跳跃 - 自适应帧率调整        - app.rs             # 事件循环、输出管理

        - surface.rs         # layer-shell surface 封装、EGL 上下文

#### M5 Phase 2: 高级功能 (P1) ✅        - output.rs          # 输出描述（名称、scale、尺寸）

- **Issue #1**: HDR 支持    - video/

  - 自动 HDR 检测 (HDR10/HLG/Dolby Vision)      - mpv.rs               # libmpv 封装（opengl-cb）

  - 5种色调映射算法 (Hable/Mobius/Reinhard/BT.2390/Clip)      - gst.rs               # 预留/可选

  - 内容感知优化 (Cinema/Animation/Documentary)    - ctl/

  - 配置验证和自动修正      - cli.rs               # 命令行解析

  - 完整文档 (450+ 行用户指南)      - ipc.rs               # 后续：unix socket/D-Bus

    - configs/

- **Issue #2**: 高级多显示器特性    - config.example.yaml

  - Per-output 视频源覆盖    - we-import.example.yaml

  - 输出名称模式匹配 (通配符、正则)  - packaging/

  - 输出优先级和回退    - appimage/

  - IPC 命令: set-source, list-outputs    - aur/

    - nix/

### 当前架构    - flatpak/              # 后续补充

    - deb/

```    - rpm/

wayvid/  - scripts/

├── src/    - dev-check.sh          # 能力自检脚本（可选）

│   ├── main.rs                 # 入口点  - systemd/

│   ├── config/    - wayvid.service

│   │   ├── types.rs           # 配置类型定义  - .github/workflows/

│   │   ├── loader.rs          # 配置加载和验证    - ci.yml

│   │   ├── watcher.rs         # 配置热重载  - README.md

│   │   └── pattern.rs         # 输出匹配模式  - LICENSE

│   ├── core/

│   │   ├── types.rs           # 核心类型 (OutputInfo, VideoSource)请先创建仓库骨架、最小可运行 MVP（单输出），并附完整 README 与示例配置、systemd 单元与 Hyprland/niri 自启样例。

│   │   └── layout.rs          # 布局矩阵计算

│   ├── backend/里程碑与任务拆分

│   │   └── wayland/- M1（2–4 周）：单输出 MVP

│   │       ├── app.rs         # Wayland 事件循环  - 建立项目结构与依赖；实现 layer-shell 背景层（输入穿透），单输出视频播放（libmpv/opengl-cb），布局 Fill/Contain/Stretch，CLI 与配置读取，日志与自检命令（列出输出、协议支持、硬解可用性），AppImage/AUR/Nix 初版。

│   │       ├── surface.rs     # Layer-shell surface 封装- M2（3–5 周）：多输出与热插拔

│   │       └── output.rs      # 输出管理和热插拔  - 输出监听、动态增删 surface 与播放器；per-output 覆盖；省电与暂停策略；能力报告增强（硬解状态、丢帧率、FPS）。

│   ├── video/- M3（3–5 周）：WE 视频导入与分发完善

│   │   ├── mpv.rs             # MPV 播放器封装  - 导入器：识别视频与参数，生成配置；Flatpak 与 .deb/.rpm；systemd --user 自启；文档与故障排查。

│   │   ├── hdr.rs             # HDR 检测和色调映射- M4（持续）：性能与共享解码优化

│   │   ├── shared_decode.rs   # 共享解码管理器  - 共享解码/多路渲染（高阶优化）、高分辨率/高帧率优化；更细的色彩/色域处理；回退静态图；Tray/UI 与 IPC。

│   │   └── frame_timing.rs    # 帧率控制

│   ├── we/验收标准（每个里程碑需通过）

│   │   ├── parser.rs          # WE 项目解析- 功能：

│   │   └── converter.rs       # WE 到 wayvid 配置转换  - 在 Hyprland 与 niri 上能稳定置底播放视频壁纸，输入完全穿透，不抢焦点。

│   └── ipc/  - 布局模式正确，窗口/输出尺寸与 scale 变化时画面无撕裂/拉伸异常。

│       ├── server.rs          # Unix socket 服务器- 性能：

│       └── commands.rs        # IPC 命令处理  - 同分辨率下优先硬解；4K@60 在有硬解时不卡顿（设备允许前提下），软解可降帧或提示。

├── docs/- 可靠性：

│   ├── QUICKSTART.md          # 快速开始指南  - 输出断开/接入能自动增删 surface；播放异常自动回退或提示。

│   ├── HDR_USER_GUIDE.md      # HDR 使用指南- 可观测：

│   ├── MULTI_MONITOR_EXAMPLES.md  # 多显示器示例  - 日志包含输出信息、解码模式、FPS、丢帧/渲染耗时等指标（至少 debug 级可见）。

│   ├── IPC.md                 # IPC 命令参考- 分发：

│   └── WE_FORMAT.md           # WE 格式说明  - 提供 AppImage（二进制可运行）、AUR 与 Nix flake 的构建与基本安装说明。

├── examples/

│   ├── config.yaml            # 基础配置示例实现细节与接口规范

│   ├── hdr-config.yaml        # HDR 配置示例

│   └── multi-monitor.yaml     # 多显示器配置1) CLI（示例）

├── scripts/- wayvid run --config path/to/config.yaml

│   ├── verify-hdr-implementation.sh- wayvid check            # 打印 Wayland 能力自检（合成器、layer-shell、输出与 scale、硬解可用性）

│   ├── test-hdr-functionality.sh- wayvid reload           # 后续：通过 IPC 重载配置

│   └── test-multi-monitor.sh

└── packaging/2) 配置文件（YAML 示意，需在 README 中说明 TOML 等价语法）

    ├── appimage/```yaml

    ├── aur/source: { File: "/home/user/Videos/loop.mp4" }  # 也可 Directory / WeProject

    └── nix/layout: Fill        # Fill | Contain | Stretch | Cover | Centre

```loop: true

start_time: 0.0

### 关键代码指标playback_rate: 1.0

- **总代码行数**: ~8,000 行 Rustmute: true

- **测试覆盖率**: 40% (目标 70%)volume: 0.0

- **编译时间**: ~45s (debug), ~2m (release)hwdec: true

- **二进制大小**: ~4MB (stripped)per_output:

- **依赖数量**: 156 crates  HDMI-A-1:

    layout: Contain

---  eDP-1:

    source: { File: "/home/user/Videos/lowpower.mp4" }

## 🚀 下一步开发计划 (M5 Phase 3)    start_time: 10.5

```

### 优先级 P1: 核心功能完善

3) Wayland 后端要点

#### Issue #3: 播放列表支持 🔴 HIGH- 使用 sctk 与 wlr-layer-shell 建立 layer=background surface，exclusive_zone=0，input_region=空；为每个 wl_output 配置 surface，绑定 xdg-output 获取名称与逻辑尺寸；若可用，使用 wp_fractional_scale 适配分数缩放。

**目标**: 支持多个视频按顺序或随机播放- 帧同步：mpv 的渲染节奏为主，使用 frame callback 做节流与空闲；DPMS/不可见时暂停。

- 热插拔：监听输出全生命周期事件，动态创建/销毁 surface 与播放器。

**需求**:

```yaml4) libmpv 集成要点

source:- 初始化：mpv_create → 设置选项（hwdec=auto-safe、loop、mute、speed、start、vid/aid 选择等）→ mpv_initialize。

  type: Playlist- 渲染：mpv_render_context_create(opengl-cb)；在每个输出的 EGL 上下文与 FBO 下调用 mpv_opengl_cb_draw()；按布局模式计算矩阵（保持像素等比/裁剪）。

  items:- 音频：默认 mute；音量可设定；后续可暴露切换。

    - "/path/to/video1.mp4"- 可观测：查询属性（vo、hwdec、dwidth/dheight、fps）、事件循环（丢帧/缓冲事件）。

    - "/path/to/video2.mp4"

  mode: sequential  # or shuffle, random5) 兼容 Wallpaper Engine（视频）

  interval: 300     # seconds per video- 导入规则：若给出 WE 工程/导出目录，则解析其中视频主文件与简单参数映射（loop/start/speed/mute/layout）；生成等价配置文件供本引擎使用。

  transition: fade  # fade, cut, blend- 不要求 1:1 完整解析；优先“行为等效”。

  

# 或者目录模式6) 省电策略

source:- 空闲/不可见暂停渲染；电池模式降帧或暂停（可配置）；提供一键禁用硬解的选项（处理兼容问题）。

  type: Directory

  path: "/path/to/videos/"7) 错误处理与日志

  pattern: "*.mp4"- 使用 thiserror/anyhow 统一错误；对外部命令/驱动失败、协议缺失、上下文创建失败等分类清晰。

  shuffle: true- tracing 提供 info/debug/trace 级别；关键路径指标打点。

  interval: 600

```8) 打包与分发

- AppImage：覆盖通用运行环境；注意 OpenGL/驱动；尽量减小体积。

**实现要点**:- AUR：提供 PKGBUILD。

1. 新增 `VideoSource::Playlist` 和 `VideoSource::Directory` 变体- Nix：flake.nix 提供包与 devShell；兼容 Hyprland/niri 用户常见环境。

2. 实现播放列表管理器 (`src/video/playlist.rs`)- 后续：Flatpak（声明 GL、Wayland socket 与硬解权限）、.deb/.rpm（分别提供打包脚本）。

3. 添加过渡效果支持 (交叉淡入淡出)- 提供 systemd --user 单元与 Hyprland/niri 自启样例。

4. IPC 命令: `next`, `prev`, `current`, `list`

5. 配置验证: 检查文件存在性、格式支持9) 文档（README）

- 快速开始、能力矩阵（Hyprland/niri 版本与协议要求）、安装方式、配置说明、常见问题（黑屏、层级冲突、硬解失败）、性能建议（限帧/省电）。

**技术挑战**:

- 预加载下一个视频避免卡顿示例与样板（请在仓库中生成相应文件）

- 平滑过渡需要双缓冲- 示例 systemd --user 单元（安装到 ~/.config/systemd/user/）

- 共享解码情况下的切换逻辑```ini

[Unit]

**预估时间**: 3-4 天Description=Wayland Video Wallpaper (wayvid)

After=graphical-session.target

#### Issue #4: 音频反应性 (基础) 🟡 MEDIUM

**目标**: 支持基于音频频谱的视觉效果[Service]

Type=simple

**需求**:ExecStart=%h/.local/bin/wayvid run --config %h/.config/wayvid/config.yaml

```yamlRestart=on-failure

audio_reactivity:

  enabled: true[Install]

  source: default  # PulseAudio/PipeWire sourceWantedBy=graphical-session.target

  fft_size: 2048```

  smoothing: 0.8

  sensitivity: 1.0- Hyprland 自启样例（hyprland.conf 中）

  ```

# MPV 脚本可访问 audio_fft 属性exec-once = wayvid run --config ~/.config/wayvid/config.yaml

``````



**实现要点**:- niri 自启样例（niri config 中，依据 niri 配置语法版本调整）

1. 添加 PipeWire/PulseAudio 音频捕获 (`src/audio/capture.rs`)```

2. 实现 FFT 频谱分析spawn "wayvid" "--config" "/home/user/.config/wayvid/config.yaml"

3. 通过 MPV Lua 脚本暴露频谱数据```

4. 提供示例 Lua 脚本 (视觉化频谱、颜色调制)

5. 可选功能 (feature gate: `audio-reactivity`)- 初版 README 内容要包含：支持的合成器、依赖、安装命令、示例配置、已知限制与路线图。



**技术挑战**:质量门槛与代码规范

- 低延迟音频捕获- 代码通过 clippy 与 rustfmt；CI 构建矩阵：x86_64/aarch64（最少），Wayland 构建检查。

- FFT 性能优化- 错误与日志有一致的语义；重要接口有文档注释与示例。

- 与视频渲染同步- 模块边界清晰：backend（Wayland）、video（mpv/gst）、core（布局/类型）、ctl（CLI/IPC）。



**预估时间**: 4-5 天执行顺序与你需要输出的内容（第一轮）

1) 创建项目骨架与 Cargo.toml（features、依赖齐全，注释说明）。

---2) 填充最小可运行的单输出 MVP：Wayland 背景层 + libmpv 渲染 + 布局 Fill/Contain/Stretch + CLI/config + 自检命令。

3) 提交 README、config.example.yaml、systemd 单元与 Hyprland/niri 自启样例、AUR/Nix/AppImage 初版脚手架。

### 优先级 P2: 用户体验提升4) 运行说明（包括硬解可用性排查）、已知限制与后续里程碑。



#### Issue #5: 更好的错误处理 🟡 MEDIUM遇到不确定点时，请列出问题清单并给出推荐项后再继续实现。

**目标**: 提供用户友好的错误信息和恢复机制

现在请开始：生成仓库骨架与最小 MVP 所需的全部文件与代码，保证可以在 Hyprland 与 niri 上编译运行并渲染单输出视频为背景层；随后补充 README 与示例配置与自检命令。
**实现要点**:
1. 桌面通知集成 (libnotify)
2. 错误分类和恢复策略
3. 回退到默认壁纸 (纯色或静态图)
4. 错误码系统和故障排查指南
5. `wayvid-ctl health` 命令

**预估时间**: 2 天

#### Issue #6: 配置验证器 🟡 MEDIUM
**目标**: 启动前验证配置有效性

**实现要点**:
1. `wayvid check-config <file>` 命令
2. 检查项:
   - 文件存在性和权限
   - 视频格式支持
   - 硬件能力匹配
   - 输出名称有效性
3. 提供修复建议

**预估时间**: 2 天

#### Issue #7: 交互式设置向导 🟢 LOW
**目标**: 首次运行时引导用户配置

**实现要点**:
```bash
$ wayvid setup
🔍 检测合成器... Hyprland v0.42.0 ✓
🔍 检查硬件解码... VA-API (Intel) ✓
🔍 扫描视频文件...

找到 3 个视频壁纸:
  1. ~/Videos/ocean.mp4 (1920x1080, 60fps) [推荐]
  2. ~/Videos/space.mp4 (3840x2160, 30fps)
  3. ~/Downloads/abstract.webm (2560x1440, 24fps)

选择默认壁纸 [1]: 1
应用到所有输出？ [Y/n]: y
启用硬件解码？ [Y/n]: y
启用 HDR？ [Y/n]: n

✓ 配置已保存到 ~/.config/wayvid/config.yaml
✓ systemd 服务已安装

运行 `systemctl --user start wayvid` 启动
```

**预估时间**: 3 天

#### Issue #8: 诊断工具 🟡 MEDIUM
**目标**: 性能监控和问题诊断

**实现要点**:
1. `wayvid-ctl stats` 命令:
   ```
   Output eDP-1:
     FPS: 59.8 (target: 60)
     Dropped frames: 3 (0.05%)
     CPU usage: 4.2%
     GPU usage: 15.3%
     Memory: 185 MB
     Decoder: vaapi (hw)
     Resolution: 1920x1080 @ 1.5x scale
   ```
2. 性能覆盖层 (debug 模式)
3. 帧时序直方图
4. 导出统计数据为 JSON

**预估时间**: 2-3 天

---

### 优先级 P3: 平台支持

#### Issue #9: Debian/Ubuntu 包 🟢 LOW
**目标**: .deb 包和 PPA 支持

**实现要点**:
1. 创建 `debian/` 目录结构
2. 编写控制文件和构建规则
3. 设置 Launchpad PPA
4. 测试 Ubuntu 22.04, 24.04, Debian 12

**预估时间**: 2 天

#### Issue #10: Fedora/RPM 包 🟢 LOW
**目标**: .rpm 包和 COPR 仓库

**实现要点**:
1. 编写 `wayvid.spec` 文件
2. 提交到 Fedora COPR
3. 测试 Fedora 39, 40

**预估时间**: 2 天

#### Issue #11: Flatpak 🟢 LOW
**目标**: Flatpak 包和 Flathub 发布

**实现要点**:
1. 编写 manifest (`org.github.YangYuS8.wayvid.yaml`)
2. 配置 Wayland socket 和硬件访问权限
3. 提交到 Flathub

**预估时间**: 3 天

#### Issue #12: ARM64 支持 🟢 LOW
**目标**: aarch64 交叉编译和 ARM64 设备支持

**实现要点**:
1. 添加 ARM64 CI 构建
2. 设置交叉编译工具链
3. 在树莓派 4/5 上测试

**预估时间**: 2 天

---

## 📝 开发指南

### 代码风格规范

1. **Rust 惯例**
   - 遵循 `rustfmt` 和 `clippy` 规则
   - 所有公共 API 必须有文档注释
   - 使用 `thiserror` 定义错误类型
   - 使用 `tracing` 而非 `log`

2. **错误处理**
   ```rust
   // ✅ 好的做法
   fn load_config(path: &Path) -> Result<Config> {
       let content = fs::read_to_string(path)
           .context("Failed to read config file")?;
       serde_yaml::from_str(&content)
           .context("Failed to parse config")
   }
   
   // ❌ 避免 unwrap/expect
   let config = load_config(path).unwrap(); // 不要这样做
   ```

3. **日志级别**
   - `error!`: 严重错误，可能导致功能失效
   - `warn!`: 警告，功能降级但可继续
   - `info!`: 重要事件 (启动、配置加载、输出变化)
   - `debug!`: 调试信息 (帧渲染、属性变化)
   - `trace!`: 详细追踪 (每帧事件)

4. **测试要求**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_pattern_matching() {
           let pattern = OutputPattern::Prefix("HDMI".into());
           assert!(pattern.matches("HDMI-A-1"));
           assert!(!pattern.matches("eDP-1"));
       }
   }
   ```

### 性能考虑

1. **避免不必要的分配**
   ```rust
   // ✅ 使用引用
   fn process_output(info: &OutputInfo) { }
   
   // ❌ 避免克隆
   fn process_output(info: OutputInfo) { } // 会复制整个结构
   ```

2. **使用 Arc 共享数据**
   ```rust
   struct SharedDecoder {
       decoder: Arc<Mutex<MpvPlayer>>,
       consumers: Vec<Consumer>,
   }
   ```

3. **异步操作使用 tokio**
   ```rust
   #[tokio::main]
   async fn main() {
       // IPC 服务器、配置监视器等
   }
   ```

### Git 工作流

1. **分支命名**
   - `feature/issue-N-short-desc`: 新功能
   - `fix/issue-N-short-desc`: Bug 修复
   - `docs/topic`: 文档更新
   - `refactor/component`: 代码重构

2. **提交信息格式**
   ```
   type(scope): short description
   
   - Detailed change 1
   - Detailed change 2
   
   Closes #N
   ```
   
   类型: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

3. **Pull Request 流程**
   - 创建 draft PR 并关联 Issue
   - 确保 CI 全部通过
   - 请求至少一个 review
   - 使用 squash merge 合并

### 测试策略

1. **单元测试**
   - 覆盖核心逻辑 (配置解析、模式匹配、布局计算)
   - 使用 `cargo test` 运行

2. **集成测试**
   - 位于 `tests/` 目录
   - 测试多组件协作

3. **手动测试清单**
   ```bash
   # 基础功能
   - [ ] 单输出播放
   - [ ] 多输出播放
   - [ ] 热插拔处理
   - [ ] 配置热重载
   - [ ] HDR 视频播放
   
   # 边界情况
   - [ ] 无效配置文件
   - [ ] 不存在的视频文件
   - [ ] 快速插拔 10+ 次
   - [ ] 内存泄漏 (24小时运行)
   ```

---

## 🔧 技术深入解析

### 1. Wayland Layer Shell 实现

**关键协议**:
```
zwlr_layer_shell_v1: 背景层实现
xdg_output_v1: 输出名称和逻辑尺寸
wp_fractional_scale_v1: 分数缩放支持
wp_viewporter: 视口裁剪
```

**Layer Surface 配置**:
```rust
layer_surface.set_layer(Layer::Background);
layer_surface.set_exclusive_zone(0);
layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

// 输入穿透 - 关键!
let region = compositor.create_region();
// 空 region = 完全穿透
surface.set_input_region(Some(&region));
```

### 2. OpenGL 渲染流程

**EGL 上下文创建**:
```rust
// 1. 获取 EGLDisplay
let display = egl::get_platform_display(...);

// 2. 选择配置
let config = egl::choose_config(display, &[
    egl::SURFACE_TYPE, egl::WINDOW_BIT,
    egl::RED_SIZE, 8,
    egl::GREEN_SIZE, 8,
    egl::BLUE_SIZE, 8,
    egl::ALPHA_SIZE, 8,
    egl::RENDERABLE_TYPE, egl::OPENGL_ES3_BIT,
]);

// 3. 创建上下文
let context = egl::create_context(display, config, ...);
```

**MPV 渲染集成**:
```rust
// MPV opengl-cb 回调
fn render_frame(fbo: i32, width: i32, height: i32) {
    mpv.render_context_render(&[
        mpv_render_param { type: MPV_RENDER_PARAM_OPENGL_FBO, data: &fbo },
        mpv_render_param { type: MPV_RENDER_PARAM_FLIP_Y, data: &1 },
    ]);
}
```

### 3. 共享解码架构

**核心思想**: 一个 MPV 实例解码，多个输出消费相同帧

```rust
pub struct SharedDecodeManager {
    // 视频源 -> 解码器映射
    decoders: HashMap<String, Arc<Mutex<Decoder>>>,
    
    // 输出 ID -> 消费者映射
    consumers: HashMap<u32, Consumer>,
}

pub struct Decoder {
    mpv: MpvPlayer,
    current_frame: Arc<RwLock<Frame>>,
    ref_count: AtomicUsize,
}

pub struct Consumer {
    output_id: u32,
    frame_ref: Arc<RwLock<Frame>>,
    last_rendered: Instant,
}
```

**关键流程**:
1. 检测多个输出使用相同视频源
2. 创建单个解码器
3. 每个输出作为消费者注册
4. 解码器渲染到共享纹理
5. 各消费者从共享纹理绘制到自己的 FBO

### 4. HDR 处理管线

**HDR 检测**:
```rust
// 从 MPV 属性读取
let color_space = mpv.get_property("video-params/primaries");
let transfer = mpv.get_property("video-params/gamma");
let peak_luma = mpv.get_property("video-params/sig-peak");

match (color_space, transfer) {
    ("bt.2020", "pq") => HdrFormat::Hdr10,
    ("bt.2020", "hlg") => HdrFormat::Hlg,
    _ => HdrFormat::Sdr,
}
```

**色调映射配置**:
```rust
mpv.set_property("tone-mapping", "hable");
mpv.set_property("tone-mapping-param", 1.0);
mpv.set_property("tone-mapping-mode", "hybrid");
mpv.set_property("hdr-compute-peak", "yes");
```

### 5. IPC 命令系统

**Unix Socket 服务器**:
```rust
let socket_path = format!("/run/user/{}/wayvid.sock", getuid());
let listener = UnixListener::bind(&socket_path)?;

for stream in listener.incoming() {
    let cmd = read_command(&stream)?;
    let response = handle_command(cmd)?;
    write_response(&stream, response)?;
}
```

**命令格式** (JSON):
```json
{
  "command": "set-source",
  "args": {
    "output": "HDMI-A-1",
    "source": {
      "type": "File",
      "path": "/path/to/video.mp4"
    }
  }
}
```

---

## 📚 关键配置示例

### 基础配置
```yaml
# ~/.config/wayvid/config.yaml
source:
  type: File
  path: "/home/user/Videos/wallpaper.mp4"

layout: Fill
loop: true
start_time: 0.0
playback_rate: 1.0
mute: true
volume: 0.0
hwdec: true

log_level: info
```

### 多显示器配置
```yaml
source:
  type: File
  path: "/home/user/Videos/default.mp4"

per_output:
  # 精确匹配
  "eDP-1":
    source:
      type: File
      path: "/home/user/Videos/laptop.mp4"
    layout: Contain
  
  # 前缀匹配
  "HDMI-*":
    source:
      type: File
      path: "/home/user/Videos/external.mp4"
    layout: Fill
  
  # 正则匹配
  "/DP-[0-9]+/":
    source:
      type: File
      path: "/home/user/Videos/displayport.mp4"
```

### HDR 配置
```yaml
source:
  type: File
  path: "/home/user/Videos/hdr-video.mp4"

hdr_mode: auto  # auto | force | disable

tone_mapping:
  algorithm: hable  # hable | mobius | reinhard | bt2390 | clip
  param: 1.0        # 算法特定参数
  compute_peak: true
  mode: hybrid      # rgb | luma | hybrid | auto

per_output:
  "HDMI-A-1":  # HDR 显示器
    hdr_mode: force
  
  "eDP-1":     # SDR 笔记本屏
    tone_mapping:
      algorithm: mobius
      param: 0.3
```

### 播放列表配置 (规划中)
```yaml
source:
  type: Playlist
  items:
    - "/home/user/Videos/morning.mp4"
    - "/home/user/Videos/afternoon.mp4"
    - "/home/user/Videos/evening.mp4"
  mode: sequential
  interval: 3600  # 每小时切换
  transition: fade
  fade_duration: 2.0

# 或使用目录
source:
  type: Directory
  path: "/home/user/Videos/collection/"
  pattern: "*.{mp4,webm,mkv}"
  shuffle: true
  interval: 600
```

---

## 🐛 常见问题和解决方案

### 问题 1: 黑屏/壁纸不显示
**排查步骤**:
```bash
# 1. 检查合成器支持
wayvid check

# 2. 查看日志
journalctl --user -u wayvid -f

# 3. 验证配置
wayvid check-config ~/.config/wayvid/config.yaml

# 4. 测试最小配置
wayvid run --config examples/minimal.yaml --log-level debug
```

**常见原因**:
- Layer-shell 协议不支持 (使用 `wayvid check` 确认)
- 视频文件不存在或格式不支持
- 硬件解码失败 (尝试 `hwdec: false`)
- 输出名称匹配错误

### 问题 2: 高 CPU/GPU 使用率
**优化建议**:
```yaml
# 启用共享解码 (多输出相同视频)
shared_decode: true

# 限制帧率
target_fps: 30

# 禁用硬件解码 (某些驱动问题)
hwdec: false

# 降低视频质量
per_output:
  "*":
    source:
      type: File
      path: "/path/to/lower-resolution.mp4"
```

### 问题 3: 热插拔后壁纸消失
**检查**:
```bash
# 实时监控输出变化
wayvid-ctl list-outputs --watch

# 查看输出匹配规则
wayvid-ctl debug match-outputs
```

**配置建议**:
```yaml
# 使用通配符确保新输出被覆盖
per_output:
  "*":  # 匹配所有输出
    source:
      type: File
      path: "/home/user/Videos/default.mp4"
```

### 问题 4: HDR 视频颜色异常
**HDR 检测**:
```bash
# 查看 HDR 检测结果
wayvid-ctl hdr-status

# 手动测试不同算法
wayvid-ctl set-tone-mapping hable
wayvid-ctl set-tone-mapping mobius
```

**推荐设置**:
```yaml
tone_mapping:
  # 电影内容
  algorithm: hable
  param: 1.0
  
  # 动画内容
  # algorithm: mobius
  # param: 0.3
```

---

## 🎓 学习资源

### Wayland 协议文档
- wlr-layer-shell: https://wayland.app/protocols/wlr-layer-shell-unstable-v1
- xdg-output: https://wayland.app/protocols/xdg-output-unstable-v1
- fractional-scale: https://wayland.app/protocols/fractional-scale-v1

### libmpv 文档
- MPV 手册: https://mpv.io/manual/master/
- libmpv 客户端 API: https://github.com/mpv-player/mpv/blob/master/libmpv/client.h
- OpenGL 渲染回调: https://github.com/mpv-player/mpv/blob/master/libmpv/render_gl.h

### Rust 相关
- smithay-client-toolkit: https://github.com/Smithay/client-toolkit
- wayland-rs: https://github.com/Smithay/wayland-rs
- mpv-rs: https://github.com/ParadoxSpiral/mpv-rs

---

## 📊 性能基准

### 当前性能 (v0.3.1)
```
单输出 (1080p@60fps):
  CPU: 3-5% (硬解) / 15-20% (软解)
  GPU: 8-12%
  内存: 120-150 MB
  启动时间: 300-500ms

多输出 (4x 1080p@60fps, 相同视频, 共享解码):
  CPU: 5-8% (硬解) / 25-35% (软解)
  GPU: 20-30%
  内存: 300-400 MB
  输出切换: <100ms

多输出 (4x 1080p@60fps, 不同视频):
  CPU: 12-18% (硬解) / 60-80% (软解)
  GPU: 35-50%
  内存: 500-700 MB
```

### 优化目标 (M5)
- 单输出 CPU: <3%
- 共享解码 CPU: <5%
- 内存占用: <200MB (4输出)
- 启动时间: <300ms
- 配置重载: <100ms

---

## 🤝 贡献指南

### 如何贡献
1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码审查要点
- [ ] 代码通过 `cargo clippy` 无警告
- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] CI 检查全部通过
- [ ] 功能符合设计文档
- [ ] 性能没有明显退化

### 测试环境
我们在以下环境中测试:
- **Hyprland**: 0.40+, 0.41+, 0.42+
- **niri**: latest git
- **Sway**: 1.9+ (基础支持)
- **硬件**: Intel (VA-API), AMD (VA-API), NVIDIA (NVDEC)
- **发行版**: Arch, NixOS, Ubuntu 24.04, Fedora 40

---

## 📈 项目路线图

### v0.4.0 (M5 - 当前) - 2025 Q4
- ✅ 共享解码上下文
- ✅ 内存优化
- ✅ HDR 支持
- ✅ 高级多显示器
- 🔄 播放列表支持
- 🔄 音频反应性
- 🔄 用户体验提升

### v0.5.0 (M6) - 2026 Q1
- Wallpaper Engine 完整兼容
- HTML/WebGL 壁纸支持
- 交互式壁纸 (鼠标/键盘)
- GUI 配置工具
- 插件系统

### v1.0.0 (稳定版) - 2026 Q2
- 全平台支持 (KDE, GNOME, Sway)
- 生产级稳定性
- 完整文档和教程
- 社区插件生态

---

## 📞 联系方式

- **GitHub Issues**: 技术问题和功能请求
- **GitHub Discussions**: 一般讨论和问题
- **Email**: YangYuS8@163.com
- **Matrix**: #wayvid:matrix.org (规划中)

---

## 📄 许可证

wayvid 采用 GPL-3.0 许可证。详见 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

- **Hyprland**: 提供优秀的 Wayland 合成器
- **MPV**: 强大的媒体播放引擎
- **smithay-client-toolkit**: Wayland 客户端库
- **Wallpaper Engine**: 灵感来源

---

**最后更新**: 2025-11-03  
**文档版本**: 2.0  
**项目版本**: v0.3.1-dev  
**维护者**: YangYuS8

---

## 🔄 迁移设备清单

### 代码仓库迁移
- [ ] `git clone https://github.com/YangYuS8/wayvid.git`
- [ ] `git checkout main`
- [ ] 确认所有分支已推送

### 开发环境设置
```bash
# 1. 安装依赖
sudo pacman -S rust wayland wayland-protocols mesa libmpv pkgconf

# 2. 配置 Rust
rustup default stable
rustup component add clippy rustfmt

# 3. 验证构建
cd wayvid
cargo build
cargo test
cargo clippy

# 4. 运行
cargo run -- check
cargo run -- run --config examples/config.yaml
```

### IDE 配置
- VSCode 扩展: rust-analyzer, crates, Error Lens
- 配置文件: `.vscode/settings.json` (已在仓库中)

### 记得复制的本地文件
- 测试视频: `~/Videos/test-*.mp4`
- 配置文件: `~/.config/wayvid/config.yaml`
- SSH 密钥: `~/.ssh/id_rsa` (用于 GitHub push)

### 环境变量
```bash
export RUST_LOG=debug
export WAYLAND_DEBUG=1
```

### 下一步工作
参考本文档 "下一步开发计划" 部分，优先实现:
1. Issue #3: 播放列表支持
2. Issue #4: 音频反应性
3. Issue #5-8: 用户体验提升
