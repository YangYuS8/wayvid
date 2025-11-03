# Issue #1: HDR Support - Implementation Plan

## 🎯 目标

添加 HDR10/HLG 直通和色调映射支持,使 wayvid 能够在 HDR 显示器上正确播放 HDR 内容,并在 SDR 显示器上提供优雅的降级。

## 📋 任务清单

### Phase 1: HDR 检测 (2h)
- [ ] 添加 `get_property_string()` 方法到 `MpvPlayer`
- [ ] 检测视频的色彩空间 (`video-params/colorspace`)
- [ ] 检测视频的传输函数 (`video-params/gamma`)
- [ ] 检测视频的色域 (`video-params/primaries`)
- [ ] 检测峰值亮度 (`video-params/sig-peak`)
- [ ] 创建 `HdrMetadata` 结构体

### Phase 2: 输出 HDR 能力查询 (3h)
- [ ] 研究 Wayland HDR 协议 (zwp_xx_color_management_v1 或 Hyprland 扩展)
- [ ] 查询输出是否支持 HDR
- [ ] 查询支持的 EOTF (Electro-Optical Transfer Function)
- [ ] 查询最大亮度范围
- [ ] 创建 `OutputHdrCapabilities` 结构体
- [ ] 添加到 `OutputInfo`

### Phase 3: MPV HDR 配置 (2h)
- [ ] 配置 `target-colorspace-hint`
- [ ] 配置 `target-trc` (传输函数)
- [ ] 配置 `target-prim` (色域)
- [ ] 配置 `target-peak` (峰值亮度)
- [ ] 启用 HDR 直通时禁用色调映射

### Phase 4: 色调映射 (3h)
- [ ] 研究 MPV 色调映射算法 (`tone-mapping` 选项)
- [ ] 配置默认色调映射算法 (hable/mobius/reinhard)
- [ ] 配置 `tone-mapping-param`
- [ ] 配置 `hdr-compute-peak` (动态峰值检测)
- [ ] 配置 `tone-mapping-mode` (auto/rgb/hybrid/luma)

### Phase 5: 配置选项 (1h)
- [ ] 添加 `hdr_mode` 到 `Config` (auto/force/disable)
- [ ] 添加 `tone_mapping_algorithm` 配置
- [ ] 添加 `tone_mapping_param` 配置
- [ ] 更新配置文档

### Phase 6: 测试 (3h)
- [ ] 下载 HDR 测试视频 (HDR10, HLG)
- [ ] 测试 HDR → HDR 直通 (如果有 HDR 显示器)
- [ ] 测试 HDR → SDR 色调映射
- [ ] 测试配置选项切换
- [ ] 创建测试报告

## 🔧 技术细节

### HDR 元数据结构

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    Sdr,         // BT.709
    Hdr10,       // BT.2020
    Hlg,         // Hybrid Log-Gamma
    DolbyVision, // Future
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferFunction {
    Srgb,        // SDR
    Pq,          // PQ (Perceptual Quantizer) - HDR10
    Hlg,         // HLG
}

#[derive(Debug, Clone)]
pub struct HdrMetadata {
    pub color_space: ColorSpace,
    pub transfer_function: TransferFunction,
    pub max_luminance: Option<f64>,  // nits
    pub avg_luminance: Option<f64>,  // nits
    pub min_luminance: Option<f64>,  // nits
}
```

### MPV 属性查询

```rust
// 在 MpvPlayer 中添加
pub fn get_hdr_metadata(&self) -> Option<HdrMetadata> {
    let colorspace = self.get_property_string("video-params/colorspace")?;
    let gamma = self.get_property_string("video-params/gamma")?;
    let primaries = self.get_property_string("video-params/primaries")?;
    let sig_peak = self.get_property_i64("video-params/sig-peak");
    
    // 解析并构建 HdrMetadata
    ...
}
```

### MPV HDR 配置选项

**直通模式** (HDR → HDR):
```rust
set_option("target-colorspace-hint", "yes");
set_option("icc-profile-auto", "yes");  // 如果支持
```

**色调映射模式** (HDR → SDR):
```rust
set_option("tone-mapping", "hable");  // 或 mobius/reinhard/bt2390
set_option("tone-mapping-mode", "hybrid");
set_option("hdr-compute-peak", "yes");
set_option("target-trc", "srgb");
set_option("target-prim", "bt.709");
set_option("target-peak", "203");  // SDR 峰值亮度
```

### 配置文件格式

```yaml
# HDR 配置
hdr_mode: auto  # auto, force, disable
tone_mapping:
  algorithm: hable  # hable, mobius, reinhard, bt2390
  param: 1.0        # 算法参数
  compute_peak: yes # 动态峰值检测
```

## 📊 实现策略

### 1. 逐步实现

1. **先实现检测**: 添加 HDR 元数据检测,输出日志
2. **再添加色调映射**: 为 SDR 显示器启用色调映射
3. **最后实现直通**: 如果检测到 HDR 显示器支持,启用直通

### 2. 优雅降级

- 如果 Wayland 不支持 HDR 查询 → 假设 SDR,启用色调映射
- 如果 MPV 不支持某个选项 → 记录警告,继续运行
- 如果检测失败 → 使用默认 SDR 模式

### 3. 日志输出

```
🎨 HDR Detection:
  Colorspace: BT.2020
  Transfer: PQ (HDR10)
  Peak Luminance: 1000 nits
  
🖥️  Output Capabilities:
  HDR Support: Yes
  Max Luminance: 1000 nits
  EOTFs: PQ, HLG
  
⚙️  HDR Mode: Passthrough
  Target Colorspace: BT.2020
  Target TRC: PQ
```

## 🧪 测试计划

### 测试场景

1. **HDR10 视频 + SDR 显示器**
   - 应启用色调映射
   - 视频应正常播放,无过曝
   
2. **HLG 视频 + SDR 显示器**
   - 应启用色调映射
   - 视频应正常播放
   
3. **SDR 视频**
   - 不应触发 HDR 处理
   - 正常播放

4. **配置测试**
   - `hdr_mode: disable` → 强制 SDR
   - `hdr_mode: force` → 强制 HDR 处理
   - `hdr_mode: auto` → 自动检测

### 测试视频

- HDR10: https://4kmedia.org/lg-hdr-picture-quality-demo-comparison/
- HLG: BBC HLG 测试片段
- 或使用 FFmpeg 生成测试视频

## 📝 依赖

- **MPV**: 需要 libmpv >= 0.35 (支持 HDR 色调映射)
- **Compositor**: Hyprland 0.40+ (如果需要 HDR 直通)
- **Wayland 协议**: zwp_xx_color_management_v1 或供应商扩展

## 🔗 参考资料

- [MPV HDR Documentation](https://mpv.io/manual/master/#options-target-colorspace-hint)
- [MPV Tone Mapping](https://mpv.io/manual/master/#options-tone-mapping)
- [Wayland Color Management Protocol](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/tree/main/staging/color-management)

## ⏱️ 时间估算

- Phase 1: 2h (HDR 检测)
- Phase 2: 3h (输出能力查询)
- Phase 3: 2h (MPV 配置)
- Phase 4: 3h (色调映射)
- Phase 5: 1h (配置选项)
- Phase 6: 3h (测试)

**总计**: 14h

## ✅ 成功标准

- [ ] HDR 视频在 SDR 显示器上正确显示(无过曝)
- [ ] 色调映射算法可配置
- [ ] 优雅降级(不支持 HDR 时正常工作)
- [ ] 完整的日志输出
- [ ] 配置文档完整
- [ ] 测试覆盖主要场景
