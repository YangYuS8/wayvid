# Issue #1: HDR Support - Implementation Progress

## 📊 总体进度

**完成度**: 14% (2h/14h)

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

**提交**: f5759c3 - "feat: Add HDR detection infrastructure (Phase 1)"

**技术细节**:
```rust
// HDR 元数据查询
pub fn get_hdr_metadata(&self) -> Option<HdrMetadata> {
    let colorspace = self.get_property_string("video-params/colorspace")?;
    let gamma = self.get_property_string("video-params/gamma")?;
    let primaries = self.get_property_string("video-params/primaries")?;
    let peak_luminance = self.get_property_f64("video-params/sig-peak");
    
    // 解析并返回 HdrMetadata
    ...
}
```

---

## ⏳ Phase 2: 输出 HDR 能力查询 (3h) - 待开始

**任务**:
- [ ] 研究 Wayland HDR 协议
  - [ ] zwp_xx_color_management_v1 (标准协议)
  - [ ] Hyprland HDR 扩展
- [ ] 创建 `OutputHdrCapabilities` 结构体
- [ ] 查询输出是否支持 HDR
- [ ] 查询支持的 EOTF (传输函数)
- [ ] 查询最大/最小亮度范围
- [ ] 添加到 `OutputInfo`

**预期结果**:
```rust
pub struct OutputHdrCapabilities {
    pub hdr_supported: bool,
    pub max_luminance: Option<f64>,  // nits
    pub min_luminance: Option<f64>,  // nits
    pub supported_eotf: Vec<TransferFunction>,
}
```

---

## ⏳ Phase 3: MPV HDR 配置 (2h) - 待开始

**任务**:
- [ ] 在 `MpvPlayer::new()` 中检测 HDR 内容
- [ ] 根据 HDR 模式配置 MPV 选项
- [ ] HDR 直通模式配置
- [ ] 色调映射模式配置
- [ ] 添加详细日志输出

**MPV 配置选项**:

**直通模式** (HDR → HDR):
```rust
set_option("target-colorspace-hint", "yes");
set_option("icc-profile-auto", "yes");
```

**色调映射模式** (HDR → SDR):
```rust
set_option("tone-mapping", "hable");
set_option("tone-mapping-mode", "hybrid");
set_option("hdr-compute-peak", "yes");
set_option("target-trc", "srgb");
set_option("target-prim", "bt.709");
set_option("target-peak", "203");
```

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

1. **研究 Wayland HDR 协议**
   - 查看 Hyprland HDR 支持文档
   - 查看 wlroots HDR 实现
   - 确定查询方法

2. **实现输出能力查询**
   - 添加 Wayland 协议绑定
   - 查询输出 HDR 能力
   - 存储到 `OutputInfo`

3. **配置 MPV HDR 选项**
   - 根据检测结果配置 MPV
   - 实现智能 HDR/SDR 切换
   - 添加日志输出

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
**当前阶段**: Phase 1 完成,Phase 2 准备开始
