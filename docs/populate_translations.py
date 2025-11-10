#!/usr/bin/env python3
"""
Populate Chinese translations in messages.po from existing zh_cn markdown files.
"""
import re

# Translation mapping from English to Chinese
translations = {
    # Navigation
    "Summary# Summary": "目录",
    "Introduction\\- Chapter 1": "简介 - 第一章",
    "User Guide": "用户指南",
    "Quick Start": "快速开始",
    "Installation": "安装",
    "Configuration": "配置",
    "Video Sources": "视频源",
    "Multi-Monitor Setup": "多显示器设置",
    "Features": "功能特性",
    "HDR Support": "HDR 支持",
    "Steam Workshop Integration": "Steam 创意工坊集成",
    "Niri Integration": "Niri 集成",
    "IPC Control": "IPC 控制",
    "Developer Guide": "开发者指南",
    "Building from Source": "从源码构建",
    "Development Workflow": "开发流程",
    "Architecture": "系统架构",
    "Contributing": "贡献指南",
    "Reference": "参考文档",
    "Configuration Reference": "配置参考",
    "CLI Reference": "命令行参考",
    "IPC Protocol": "IPC 协议",
    "Wallpaper Engine Format": "Wallpaper Engine 格式规范",
    
    # Common terms
    "wayvid": "wayvid",
    "Wayland": "Wayland",
    "video wallpaper": "动态壁纸",
    "compositor": "合成器",
    "output": "输出",
    "monitor": "显示器",
    "display": "显示器",
    "hardware decode": "硬件解码",
    "tone mapping": "色调映射",
}

# Read the PO file
with open('i18n/zh-CN/messages.po', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace empty msgstr with translations
for en, zh in translations.items():
    # Pattern: msgid "text"\nmsgstr ""
    pattern = f'msgid "{re.escape(en)}"\\nmsgstr ""'
    replacement = f'msgid "{en}"\\nmsgstr "{zh}"'
    content = re.sub(pattern, replacement, content)

# Write back
with open('i18n/zh-CN/messages.po', 'w', encoding='utf-8') as f:
    f.write(content)

print("✅ Translations populated in i18n/zh-CN/messages.po")
