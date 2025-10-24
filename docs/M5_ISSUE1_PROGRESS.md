# Issue #1: HDR Support - Implementation Progress

## 📊 总体进度

**完成度**: 50% (7h/14h)

**状态**: 🟢 进行中

**分支**: `m5-hdr-support`

---

## ✅ Phase 1: HDR 检测 (2h) - 完成

**完成时间**: 2025-10-25

**实现内容**:
- ✅ 创建 `src/video/hdr.rs` 模块
- ✅ 定义 `ColorSpace` 枚举 (Sdr, Hdr10, Hlg, DolbyVision)
- ✅ 定义 `TransferFunction` 枚举 (Srgb, Pq, Hlg)
- ✅ 定义 `HdrMetadata` 结构体
- ✅ 实现解析函数 `parse_colorspace()`, `parse_transfer_function()`
- ✅ 添加 `HdrMode` 配置枚举 (Auto/Force/Disable)
- ✅ 添加 `ToneMappingAlgorithm` 枚举
- ✅ 添加 `ToneMappingConfig` 结构体
- ✅ 在 `MpvPlayer` 中添加 `get_property_string()` 方法
- ✅ 在 `MpvPlayer` 中添加 `get_property_f64()` 方法
- ✅ 实现 `get_hdr_metadata()` 方法
- ✅ 添加 `hdr_mode` 到 `Config`
- ✅ 添加 `tone_mapping` 到 `Config`
- ✅ 更新 `EffectiveConfig` 包含 HDR 字段
- ✅ 更新 WE converter

**提交**: f5759c3, c027f11

---

## ✅ Phase 2: 输出 HDR 能力查询 (3h) - 完成

**完成时间**: 2025-10-25

**实现内容**:
- ✅ 创建 `OutputHdrCapabilities` 结构体
  - `hdr_supported`: 是否支持 HDR
  - `max_luminance`: 最大亮度 (nits)
  - `min_luminance`: 最小亮度 (nits)
  - `supported_eotf`: 支持的传输函数列表
- ✅ 添加 `hdr_capabilities` 字段到 `OutputInfo`
- ✅ 实现保守的默认值 (假设 SDR)
- ✅ 添加 `query_hdr_capabilities()` 占位方法
- ✅ 更新 `Output::new()` 初始化 HDR 能力
- ✅ 创建 Wayland HDR 支持状况文档

**提交**: 4cbc198

**技术决策**:

由于当前 Wayland HDR 协议仍在开发中:
- **Color Management Protocol**: 处于 staging 阶段,未稳定
- **Hyprland HDR**: 实验性支持 (0.40+)
- **其他合成器**: KDE/GNOME 正在开发中

**采用策略**: 保守方法
1. 默认假设所有输出为 SDR
2. 依赖 MPV 的软件色调映射
3. 为未来协议预留扩展点

**优势**:
- ✅ 在所有合成器上立即可用
- ✅ 不依赖特定协议或合成器
- ✅ MPV 色调映射质量高且可配置
- ✅ 未来可无缝升级到硬件 HDR

**文档**: `docs/HDR_WAYLAND_STATUS.md`

---

## ✅ Phase 3: MPV HDR 配置 (2h) - 完成

**完成时间**: 2025-10-25

**实现内容**:
- ✅ 实现 `configure_hdr()` 主配置方法
  - 检查 `HdrMode` (Auto/Force/Disable)
  - 检测 HDR 元数据
  - 详细日志输出 HDR 信息
  - 选择色调映射或直通模式
- ✅ 实现 `configure_tone_mapping()` (HDR → SDR)
  - 配置 MPV 色调映射选项
  - 设置目标色彩空间 (sRGB, BT.709, 203 nits)
  - 应用用户的 `ToneMappingConfig`
- ✅ 实现 `configure_hdr_passthrough()` (HDR → HDR)
  - 为未来 HDR 显示器准备
  - 禁用色调映射,启用直通
- ✅ 集成到 `SharedDecoder`
  - 在 `init_render_context()` 后调用
  - 存储 `config` 以供 HDR 配置使用
- ✅ 更新测试代码以包含新的 HDR 字段

**提交**: 0a372f1

**MPV 配置选项**:

**色调映射模式** (HDR → SDR):
```rust
set_option("tone-mapping", algorithm);  // hable, mobius, reinhard, etc.
set_option("tone-mapping-mode", mode);  // hybrid, auto, rgb, luma
set_option("hdr-compute-peak", "yes");  // 动态峰值检测
set_option("target-trc", "srgb");       // 目标传输函数
set_option("target-prim", "bt.709");    // 目标色域
set_option("target-peak", "203");       // 目标亮度 (nits)
```

**直通模式** (HDR → HDR,未来):
```rust
set_option("target-colorspace-hint", "yes");  // 启用色彩空间提示
set_option("icc-profile-auto", "yes");        // 自动 ICC 配置文件
set_option("tone-mapping", "clip");           // 禁用色调映射
```

**日志输出**:
- 🎨 HDR 配置启动信息
- 📺 输出 HDR 能力信息
- ✨ HDR 内容检测信息 (色彩空间、传输函数、峰值亮度)
- 🖥️ 色调映射配置信息
- ✓ 配置成功确认

---

## ⏳ Phase 4: 色调映射配置 (3h) - 待开始

**任务**:
- [ ] 实现不同色调映射算法的配置
- [ ] 配置 `tone-mapping-param`
- [ ] 配置 `hdr-compute-peak`
- [ ] 配置 `tone-mapping-mode`
- [ ] 测试不同算法效果
- [ ] 优化默认参数

**算法选项**:
- `hable`: Hable (Uncharted 2) - 适合大多数内容
- `mobius`: 保留细节
- `reinhard`: 经典算法
- `bt.2390`: ITU 标准
- `clip`: 无色调映射

---

## ⏳ Phase 5: 配置选项和文档 (1h) - 待开始

**任务**:
- [ ] 创建 HDR 配置示例
- [ ] 更新 README.md
- [ ] 创建 HDR 使用指南
- [ ] 添加配置验证
- [ ] 添加配置模板

**配置示例**:
```yaml
# HDR 配置
hdr_mode: auto  # auto, force, disable

tone_mapping:
  algorithm: hable  # hable, mobius, reinhard, bt2390, clip
  param: 1.0        # 算法参数
  compute_peak: yes # 动态峰值检测
  mode: hybrid      # auto, rgb, hybrid, luma
```

---

## ⏳ Phase 6: 测试和验证 (3h) - 待开始

**任务**:
- [ ] 下载 HDR 测试视频
  - [ ] HDR10 测试视频
  - [ ] HLG 测试视频
  - [ ] SDR 对照视频
- [ ] 测试 HDR → SDR 色调映射
- [ ] 测试不同色调映射算法
- [ ] 测试配置选项切换
- [ ] 性能测试
- [ ] 创建测试报告

**测试场景**:
1. HDR10 视频 + SDR 显示器 → 应启用色调映射
2. HLG 视频 + SDR 显示器 → 应启用色调映射
3. SDR 视频 → 不应触发 HDR 处理
4. `hdr_mode: disable` → 强制 SDR
5. `hdr_mode: force` → 强制 HDR 处理

---

## 📝 下一步行动

1. **Phase 4: 优化色调映射配置**
   - 测试不同色调映射算法的效果
   - 优化 `tone-mapping-param` 默认值
   - 配置不同内容类型的预设
   - 性能测试和优化

2. **Phase 5: 配置选项和文档**
   - 创建 HDR 配置示例
   - 更新 README 添加 HDR 部分
   - 编写用户使用指南
   - 添加配置验证

3. **Phase 6: 全面测试**
   - 下载 HDR 测试视频 (HDR10, HLG)
   - 测试所有色调映射算法
   - 测试配置选项切换
   - 性能基准测试
   - 创建测试报告

---

## 🔗 参考资料

- [MPV HDR Documentation](https://mpv.io/manual/master/#options-target-colorspace-hint)
- [MPV Tone Mapping](https://mpv.io/manual/master/#options-tone-mapping)
- [Wayland Color Management Protocol](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/tree/main/staging/color-management)
- [Hyprland HDR Support](https://github.com/hyprwm/Hyprland/pull/2600)

---

## ⚠️ 注意事项

1. **优雅降级**: 如果 Wayland 不支持 HDR 查询,假设 SDR 并启用色调映射
2. **兼容性**: MPV 需要 >= 0.35 版本才支持完整的 HDR 色调映射
3. **性能**: 色调映射会增加 GPU 负载,需要测试性能影响
4. **日志**: 添加详细的 HDR 检测和配置日志,方便调试

---

**最后更新**: 2025-10-25
**当前阶段**: Phase 3 完成,Phase 4 准备开始
**进度**: 50% (7h/14h)
