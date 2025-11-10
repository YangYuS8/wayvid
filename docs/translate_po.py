#!/usr/bin/env python3
"""
Quick translation helper for wayvid documentation PO file.
Adds Chinese translations for common terms and phrases.
"""

import re
from pathlib import Path

# Translation dictionary
TRANSLATIONS = {
    # Navigation
    "Summary# Summary": "目录",
    "Introduction\\- Chapter 1": "简介",
    "User Guide": "用户指南",
    "Quick Start": "快速开始",
    "Installation": "安装",
    "Configuration": "配置",
    "Video Sources": "视频源",
    "Multi-Monitor Setup": "多显示器设置",
    "Features": "功能特性",
    "HDR Support": "HDR 支持",
    "Steam Workshop": "Steam 创意工坊",
    "Niri Integration": "Niri 集成",
    "IPC Control": "IPC 控制",
    "Developer Guide": "开发者指南",
    "Building from Source": "从源码构建",
    "Development Workflow": "开发工作流",
    "Architecture": "系统架构",
    "Contributing": "贡献指南",
    "Reference": "参考文档",
    "Configuration Reference": "配置参考",
    "CLI Commands": "CLI 命令",
    "IPC Protocol": "IPC 协议",
    "WE Format": "WE 格式",
    
    # Common terms
    "wayvid": "wayvid",
    "Wayland Dynamic Video Wallpaper Daemon": "Wayland 动态壁纸守护进程",
    "Core Features": "核心特性",
    "Prerequisites": "前置要求",
    "Supported Compositors": "支持的合成器",
    "System Requirements": "系统要求",
    "Getting Help": "获取帮助",
    "License": "许可证",
    "Documentation": "文档",
    "Overview": "概览",
    "Example": "示例",
    "Usage": "使用方法",
    "Options": "选项",
    "Description": "描述",
    "Note": "注意",
    "Warning": "警告",
    "Tip": "提示",
    "See also": "另见",
    "Table of Contents": "目录",
    
    # Action words
    "See": "详见",
    "For more details": "详细信息请参阅",
    "Learn more": "了解更多",
}

def translate_po_file(po_path: Path):
    """Add translations to the PO file."""
    
    with open(po_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    translated_count = 0
    
    for english, chinese in TRANSLATIONS.items():
        # Escape special regex characters but handle backslashes in the text
        escaped = re.escape(english)
        
        # Pattern: find msgid "text" followed by empty msgstr ""
        # Match the entire msgid line and empty msgstr
        pattern = rf'msgid "{escaped}"\nmsgstr ""'
        
        # Replace with translated version
        replacement = f'msgid "{english}"\nmsgstr "{chinese}"'
        
        new_content, count = re.subn(pattern, replacement, content)
        
        if count > 0:
            content = new_content
            translated_count += count
            print(f"✓ Translated: {english} -> {chinese}")
    
    # Write back
    with open(po_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"\n✅ Total: {translated_count} translations added to {po_path.name}")

if __name__ == "__main__":
    po_file = Path(__file__).parent / "po" / "zh-CN.po"
    
    if not po_file.exists():
        print(f"❌ Error: {po_file} not found")
        exit(1)
    
    print(f"📝 Adding translations to {po_file.name}...\n")
    translate_po_file(po_file)
    print("\n🎉 Done! You can now build the Chinese documentation:")
    print("   MDBOOK_BOOK__LANGUAGE=zh-CN mdbook build -d book/zh-cn")
