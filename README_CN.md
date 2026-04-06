<p align="center">
  <img src="logo.svg" alt="LWE logo" width="180" />
</p>

<p align="center">
  <img alt="AUR lwe-git" src="https://img.shields.io/aur/version/lwe-git?label=AUR%20lwe-git" />
  <img alt="AUR lwe" src="https://img.shields.io/aur/version/lwe?label=AUR%20lwe" />
  <img alt="Quality Check" src="https://github.com/YangYuS8/lwe/actions/workflows/quality-check.yml/badge.svg" />
  <img alt="Prerelease" src="https://github.com/YangYuS8/lwe/actions/workflows/release-prerelease.yml/badge.svg" />
  <img alt="Stable release" src="https://github.com/YangYuS8/lwe/actions/workflows/release-stable.yml/badge.svg" />
</p>

# LWE

[English](README.md) | 简体中文

LWE 是一个 Linux 桌面应用，用于浏览、管理并应用 Wallpaper Engine 壁纸内容。

它主要面向 Linux 用户的迁移场景，提供以下能力：

- 在应用内浏览创意工坊内容
- 将受支持的壁纸导入本地库
- 在应用前查看兼容性信息
- 按显示器分配壁纸的桌面工作流

## 已测试桌面环境

当前已验证环境：

- `niri` + Wayland 会话

## 壁纸支持范围

首发运行时重点支持：

- 视频类壁纸

暂不作为首发运行时目标：

- 场景类壁纸（逆向私有格式成本目前过高）
- 网页类壁纸（用于兼容性识别，不是首发主要运行时目标）

## 安装方式

### Arch Linux (AUR)

- 稳定版：`lwe`
- 开发版：`lwe-git`

可使用你常用的 AUR 助手安装，例如：

```bash
yay -S lwe
```

或

```bash
yay -S lwe-git
```

### GitHub Releases

测试版与正式版都会发布以下 Linux 安装包：

- `.deb`
- `.rpm`
- `.AppImage`

可在仓库的 Releases 页面下载。

## 面向贡献者与 Agent 的说明

贡献/Agent 相关文档请查看：`docs/agent/README_AGENT.md`。
