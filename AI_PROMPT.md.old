你是一个资深 Rust/Wayland 图形工程师与构建管道专家。请严格按以下“目标、范围、非目标、技术约束、交付物、文件结构、里程碑、验收标准、实现细节与样例”来驱动开发，逐步交付一个“Linux Wayland 上的动态视频壁纸引擎”，优先支持 Hyprland 与 niri，兼容 Wallpaper Engine 的“视频类壁纸”核心参数。

角色与风格要求
- 角色：你是资深 Rust 工程师，熟悉 Wayland 协议（尤其 wlr-layer-shell）、OpenGL/EGL、libmpv/gstreamer 渲染管线、Linux 打包分发（AppImage/Flatpak/AUR/Nix/deb/rpm）。
- 风格：工程化、可维护、可观测，先 MVP、后优化；在不确定处给出合理默认与降级策略；对外接口尽量稳定清晰。
- 交互：每个里程碑前自检达成度，输出明确的变更与后续计划；出现不确定问题时用“问题清单 + 备选方案 + 推荐选择”的格式请求澄清。

目标（MVP）
- 在 Wayland（Hyprland、niri）上提供“视频类动态壁纸”能力。
- 为每个输出创建背景层 Surface，完全输入穿透，支持多显示器与热插拔。
- 使用 libmpv（OpenGL/EGL 回调）播放 mp4/webm 等常见容器/编码，支持硬件解码（VA-API/NVDEC）与软解回退。
- 兼容 Wallpaper Engine 的核心视频参数（至少）：loop、start_time、playback_rate、mute/volume、布局模式（Fill/Contain/Stretch/Cover/Centre）。
- 配置驱动（YAML/TOML），支持全局与 per-output 覆盖；支持命令行、能力自检。
- 基本打包与发布：AppImage、AUR、Nix flake，后续扩展 Flatpak、deb/rpm。
- 提供 systemd --user 自启与 Hyprland/niri 配置样例。

范围限定
- 仅“视频类”壁纸。暂不支持 HTML/WebGL/粒子/脚本互动。
- 仅 Wayland；目标合成器：Hyprland、niri。暂不考虑 KDE/GNOME。
- 初版多显示器采用“每屏一路播放器”（稳定优先），后续再优化共享解码。

非目标（当前阶段）
- Windows/macOS/X11 支持。
- 完整复刻 Wallpaper Engine 全部特性与格式。
- 高级色彩管理（ICC/EDID）与 HDR（可做规划留口）。

技术约束与约定
- 语言与版本：Rust 2021+；MSRV 在 CI 说明。
- 依赖建议：
  - Wayland：smithay-client-toolkit（sctk）、wayland-client、wayland-protocols（zwlr_layer_shell_v1、xdg-output、wp_fractional_scale_v1，如可用）、wp_presentation（可选）。
  - 渲染：EGL/OpenGL；libmpv（首选，opengl-cb 回调）；gstreamer-rs 作为备选 feature（后续）。
  - 日志/可观测性：tracing + tracing-subscriber；错误：thiserror/anyhow。
  - 配置：serde + serde_yaml/serde_toml；CLI：clap。
- 特性开关（features）：
  - video-mpv（默认开启）、video-gst（可选）、backend-wayland（默认）、telemetry（可选）、tray/ui（后续）。
- 层级与输入：
  - 使用 wlr-layer-shell 的 background layer，exclusive_zone=0，input_region 为空，实现完全穿透。
- 多显示器：
  - 每输出一个 surface；监听输出新增/移除、scale/rotate 变化，动态增删播放器与 surface。
- 布局模式：
  - Fill（裁剪填满），Contain（等比完整显示），Stretch（拉伸），Cover/Centre（可与 Fill/Centre 合并定义）。以物理像素尺寸与 scale 后尺寸综合计算裁剪矩阵。
- 音频：
  - 默认静音（壁纸类场景）；提供音量/静音开关；音频走 PipeWire/PulseAudio。
- 省电：
  - 输出不可见/DPMS off 或空闲时暂停/降帧（可配置）；提供硬解开关/黑名单。
- 回退：
  - 播放失败或渲染异常可回退为纯色或静态图（静态图回退可后续集成 wallpaper.rs）。

对 Wallpaper Engine（视频类）的兼容策略
- 最小兼容：从其工程或导出目录中读取视频文件与基本参数（loop、start_time、playback_rate、mute/volume、布局）；即便不能完整解析其元数据，也保证“行为等效”。
- 提供简单“导入器”：输入工程/导出路径 → 生成本项目配置文件（含 per-output 策略）。

交付物与文件结构（初版建议）
- 代码仓库名：wayvid（可变）
- 建议文件结构：
  - src/
    - main.rs
    - config.rs
    - core/
      - layout.rs            # 布局矩阵、裁剪与变换
      - types.rs             # 公共类型（Mode、PerOutput 等）
    - backend/
      - wayland/
        - mod.rs
        - app.rs             # 事件循环、输出管理
        - surface.rs         # layer-shell surface 封装、EGL 上下文
        - output.rs          # 输出描述（名称、scale、尺寸）
    - video/
      - mpv.rs               # libmpv 封装（opengl-cb）
      - gst.rs               # 预留/可选
    - ctl/
      - cli.rs               # 命令行解析
      - ipc.rs               # 后续：unix socket/D-Bus
  - configs/
    - config.example.yaml
    - we-import.example.yaml
  - packaging/
    - appimage/
    - aur/
    - nix/
    - flatpak/              # 后续补充
    - deb/
    - rpm/
  - scripts/
    - dev-check.sh          # 能力自检脚本（可选）
  - systemd/
    - wayvid.service
  - .github/workflows/
    - ci.yml
  - README.md
  - LICENSE

请先创建仓库骨架、最小可运行 MVP（单输出），并附完整 README 与示例配置、systemd 单元与 Hyprland/niri 自启样例。

里程碑与任务拆分
- M1（2–4 周）：单输出 MVP
  - 建立项目结构与依赖；实现 layer-shell 背景层（输入穿透），单输出视频播放（libmpv/opengl-cb），布局 Fill/Contain/Stretch，CLI 与配置读取，日志与自检命令（列出输出、协议支持、硬解可用性），AppImage/AUR/Nix 初版。
- M2（3–5 周）：多输出与热插拔
  - 输出监听、动态增删 surface 与播放器；per-output 覆盖；省电与暂停策略；能力报告增强（硬解状态、丢帧率、FPS）。
- M3（3–5 周）：WE 视频导入与分发完善
  - 导入器：识别视频与参数，生成配置；Flatpak 与 .deb/.rpm；systemd --user 自启；文档与故障排查。
- M4（持续）：性能与共享解码优化
  - 共享解码/多路渲染（高阶优化）、高分辨率/高帧率优化；更细的色彩/色域处理；回退静态图；Tray/UI 与 IPC。

验收标准（每个里程碑需通过）
- 功能：
  - 在 Hyprland 与 niri 上能稳定置底播放视频壁纸，输入完全穿透，不抢焦点。
  - 布局模式正确，窗口/输出尺寸与 scale 变化时画面无撕裂/拉伸异常。
- 性能：
  - 同分辨率下优先硬解；4K@60 在有硬解时不卡顿（设备允许前提下），软解可降帧或提示。
- 可靠性：
  - 输出断开/接入能自动增删 surface；播放异常自动回退或提示。
- 可观测：
  - 日志包含输出信息、解码模式、FPS、丢帧/渲染耗时等指标（至少 debug 级可见）。
- 分发：
  - 提供 AppImage（二进制可运行）、AUR 与 Nix flake 的构建与基本安装说明。

实现细节与接口规范

1) CLI（示例）
- wayvid run --config path/to/config.yaml
- wayvid check            # 打印 Wayland 能力自检（合成器、layer-shell、输出与 scale、硬解可用性）
- wayvid reload           # 后续：通过 IPC 重载配置

2) 配置文件（YAML 示意，需在 README 中说明 TOML 等价语法）
```yaml
source: { File: "/home/user/Videos/loop.mp4" }  # 也可 Directory / WeProject
layout: Fill        # Fill | Contain | Stretch | Cover | Centre
loop: true
start_time: 0.0
playback_rate: 1.0
mute: true
volume: 0.0
hwdec: true
per_output:
  HDMI-A-1:
    layout: Contain
  eDP-1:
    source: { File: "/home/user/Videos/lowpower.mp4" }
    start_time: 10.5
```

3) Wayland 后端要点
- 使用 sctk 与 wlr-layer-shell 建立 layer=background surface，exclusive_zone=0，input_region=空；为每个 wl_output 配置 surface，绑定 xdg-output 获取名称与逻辑尺寸；若可用，使用 wp_fractional_scale 适配分数缩放。
- 帧同步：mpv 的渲染节奏为主，使用 frame callback 做节流与空闲；DPMS/不可见时暂停。
- 热插拔：监听输出全生命周期事件，动态创建/销毁 surface 与播放器。

4) libmpv 集成要点
- 初始化：mpv_create → 设置选项（hwdec=auto-safe、loop、mute、speed、start、vid/aid 选择等）→ mpv_initialize。
- 渲染：mpv_render_context_create(opengl-cb)；在每个输出的 EGL 上下文与 FBO 下调用 mpv_opengl_cb_draw()；按布局模式计算矩阵（保持像素等比/裁剪）。
- 音频：默认 mute；音量可设定；后续可暴露切换。
- 可观测：查询属性（vo、hwdec、dwidth/dheight、fps）、事件循环（丢帧/缓冲事件）。

5) 兼容 Wallpaper Engine（视频）
- 导入规则：若给出 WE 工程/导出目录，则解析其中视频主文件与简单参数映射（loop/start/speed/mute/layout）；生成等价配置文件供本引擎使用。
- 不要求 1:1 完整解析；优先“行为等效”。

6) 省电策略
- 空闲/不可见暂停渲染；电池模式降帧或暂停（可配置）；提供一键禁用硬解的选项（处理兼容问题）。

7) 错误处理与日志
- 使用 thiserror/anyhow 统一错误；对外部命令/驱动失败、协议缺失、上下文创建失败等分类清晰。
- tracing 提供 info/debug/trace 级别；关键路径指标打点。

8) 打包与分发
- AppImage：覆盖通用运行环境；注意 OpenGL/驱动；尽量减小体积。
- AUR：提供 PKGBUILD。
- Nix：flake.nix 提供包与 devShell；兼容 Hyprland/niri 用户常见环境。
- 后续：Flatpak（声明 GL、Wayland socket 与硬解权限）、.deb/.rpm（分别提供打包脚本）。
- 提供 systemd --user 单元与 Hyprland/niri 自启样例。

9) 文档（README）
- 快速开始、能力矩阵（Hyprland/niri 版本与协议要求）、安装方式、配置说明、常见问题（黑屏、层级冲突、硬解失败）、性能建议（限帧/省电）。

示例与样板（请在仓库中生成相应文件）
- 示例 systemd --user 单元（安装到 ~/.config/systemd/user/）
```ini
[Unit]
Description=Wayland Video Wallpaper (wayvid)
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/wayvid run --config %h/.config/wayvid/config.yaml
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

- Hyprland 自启样例（hyprland.conf 中）
```
exec-once = wayvid run --config ~/.config/wayvid/config.yaml
```

- niri 自启样例（niri config 中，依据 niri 配置语法版本调整）
```
spawn "wayvid" "--config" "/home/user/.config/wayvid/config.yaml"
```

- 初版 README 内容要包含：支持的合成器、依赖、安装命令、示例配置、已知限制与路线图。

质量门槛与代码规范
- 代码通过 clippy 与 rustfmt；CI 构建矩阵：x86_64/aarch64（最少），Wayland 构建检查。
- 错误与日志有一致的语义；重要接口有文档注释与示例。
- 模块边界清晰：backend（Wayland）、video（mpv/gst）、core（布局/类型）、ctl（CLI/IPC）。

执行顺序与你需要输出的内容（第一轮）
1) 创建项目骨架与 Cargo.toml（features、依赖齐全，注释说明）。
2) 填充最小可运行的单输出 MVP：Wayland 背景层 + libmpv 渲染 + 布局 Fill/Contain/Stretch + CLI/config + 自检命令。
3) 提交 README、config.example.yaml、systemd 单元与 Hyprland/niri 自启样例、AUR/Nix/AppImage 初版脚手架。
4) 运行说明（包括硬解可用性排查）、已知限制与后续里程碑。

遇到不确定点时，请列出问题清单并给出推荐项后再继续实现。

现在请开始：生成仓库骨架与最小 MVP 所需的全部文件与代码，保证可以在 Hyprland 与 niri 上编译运行并渲染单输出视频为背景层；随后补充 README 与示例配置与自检命令。