# 安装

在不同发行版上安装 wayvid。

## Arch Linux

### AUR (推荐)

```bash
# 使用 yay
yay -S wayvid-git

# 或使用 paru
paru -S wayvid-git

# 手动构建
git clone https://aur.archlinux.org/wayvid-git.git
cd wayvid-git
makepkg -si
```

**可选依赖**（硬件加速）:
```bash
# Intel GPU
sudo pacman -S intel-media-driver libva-intel-driver

# AMD GPU
sudo pacman -S mesa libva-mesa-driver

# NVIDIA GPU
sudo pacman -S nvidia-utils nvidia-vaapi-driver
```

## Ubuntu / Debian

### 系统依赖

```bash
sudo apt update
sudo apt install -y \
    libwayland-dev \
    libmpv-dev \
    libgl1-mesa-dev \
    libegl1-mesa-dev \
    cargo \
    rustc
```

### 硬件解码（可选但推荐）

```bash
# Intel
sudo apt install intel-media-va-driver i965-va-driver

# AMD
sudo apt install mesa-va-drivers

# NVIDIA (替换版本号)
sudo apt install nvidia-driver-535 libnvidia-encode-535
```

### 从源码构建

```bash
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
cargo build --release
sudo install -Dm755 target/release/wayvid /usr/local/bin/
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

## Fedora

### 系统依赖

```bash
sudo dnf install -y \
    wayland-devel \
    mpv-libs-devel \
    mesa-libGL-devel \
    mesa-libEGL-devel \
    cargo \
    rust
```

### 硬件解码

```bash
# Intel/AMD
sudo dnf install libva-intel-driver mesa-va-drivers

# NVIDIA
sudo dnf install nvidia-driver nvidia-vaapi-driver
```

### 构建安装

```bash
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
cargo build --release
sudo install -Dm755 target/release/wayvid /usr/local/bin/
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

## NixOS / Nix

### Flake 方式

```nix
{
  inputs.wayvid.url = "github:YangYuS8/wayvid";
  
  outputs = { self, nixpkgs, wayvid }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [{
        environment.systemPackages = [ 
          wayvid.packages.x86_64-linux.default 
        ];
      }];
    };
  };
}
```

### 直接运行

```bash
nix run github:YangYuS8/wayvid
```

### 安装到 Profile

```bash
nix profile install github:YangYuS8/wayvid
```

## 从源码构建（通用）

### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. 克隆仓库

```bash
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
```

### 3. 构建

```bash
# 完整功能构建
cargo build --release --all-features

# 仅核心功能
cargo build --release
```

### 4. 安装

```bash
# 用户安装
install -Dm755 target/release/wayvid ~/.local/bin/
install -Dm755 target/release/wayvid-ctl ~/.local/bin/

# 系统安装
sudo install -Dm755 target/release/wayvid /usr/local/bin/
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

## 验证安装

```bash
# 检查版本
wayvid --version

# 系统能力检查
wayvid check
```

预期输出：
```
=== wayvid 系统能力检查 ===

[Wayland]
  ✓ WAYLAND_DISPLAY: wayland-1
  ✓ 连接: 已建立
  ✓ 协议: 可用
    - wl_compositor
    - wl_output
    - zwlr_layer_shell_v1

[视频后端]
  ✓ 后端: libmpv
  ℹ mpv 0.37.0

[硬件解码]
  ✓ VA-API 可用
  ℹ VDPAU 不可用
```

## 故障排除

### 找不到 wayvid 命令

确保 `~/.local/bin` 在 PATH 中：
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 缺少依赖

检查错误信息并安装对应的包：
```bash
# Arch
sudo pacman -S wayland libmpv mesa

# Ubuntu
sudo apt install libwayland-client0 libmpv1 libgl1

# Fedora
sudo dnf install wayland-client mpv-libs mesa-libGL
```

### VA-API 不可用

```bash
# 安装 VA-API 驱动
sudo pacman -S libva-intel-driver  # Intel
sudo apt install i965-va-driver     # Ubuntu/Debian

# 检查
vainfo
```

### 编译错误

确保 Rust 版本 >= 1.75:
```bash
rustc --version
rustup update
```

## 下一步

- [配置 wayvid](configuration.md)
- [准备视频文件](video-sources.md)
- [多显示器设置](multi-monitor.md)
