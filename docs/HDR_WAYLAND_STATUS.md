# Wayland HDR Support Status

## 📊 Current Status (2025-10-25)

Wayland HDR support is actively being developed but is not yet widely available or standardized.

### Protocol Status

#### 1. **Color Management Protocol** (Staging)
- **Name**: `zwp_xx_color_management_v1`
- **Status**: ⚠️ Staging (not stable)
- **Repository**: https://gitlab.freedesktop.org/wayland/wayland-protocols/-/tree/main/staging/color-management
- **Support**: Limited

**Capabilities**:
- Color space selection
- Transfer function configuration
- Luminance metadata
- ICC profile support

**Limitations**:
- Not yet stable
- Limited compositor support
- API may change

#### 2. **Hyprland HDR Support**
- **Status**: ✅ Experimental support in Hyprland 0.40+
- **PR**: https://github.com/hyprwm/Hyprland/pull/2600
- **Features**:
  - HDR10 support
  - Per-output HDR toggle
  - Brightness control

**Limitations**:
- Compositor-specific (not portable)
- Requires recent Hyprland version
- Experimental quality

#### 3. **KDE Plasma HDR**
- **Status**: 🔄 In development
- **Version**: Plasma 6.0+
- **Features**: HDR support for Wayland

#### 4. **GNOME HDR**
- **Status**: 🔄 Planned
- **Timeline**: Future releases

### Current wayvid Approach

Given the current state of Wayland HDR support, wayvid uses a **conservative approach**:

1. **Assume SDR**: All outputs are treated as SDR by default
2. **MPV Tone Mapping**: Use MPV's built-in tone mapping for HDR content
3. **Future-Proof**: Infrastructure in place for when protocols mature

## 🔧 Implementation Details

### Phase 2 Completion

We have implemented:

✅ **OutputHdrCapabilities** structure:
```rust
pub struct OutputHdrCapabilities {
    pub hdr_supported: bool,
    pub max_luminance: Option<f64>,
    pub min_luminance: Option<f64>,
    pub supported_eotf: Vec<TransferFunction>,
}
```

✅ **Default SDR Capabilities**:
- `hdr_supported`: false
- `max_luminance`: 203 nits (typical SDR)
- `min_luminance`: 0 nits
- `supported_eotf`: [Srgb]

✅ **Placeholder Method** for future HDR detection:
```rust
pub fn query_hdr_capabilities(&mut self) {
    // TODO: Implement when Wayland protocols are available
}
```

### Why This Works

**MPV's Tone Mapping** is compositor-independent:
- Detects HDR content from video stream
- Applies tone mapping in software
- Works on any SDR display
- No Wayland protocol needed

**Benefits**:
- ✅ Works now on all compositors
- ✅ Handles HDR10, HLG, etc.
- ✅ User-configurable algorithms
- ✅ No external dependencies

**Limitations**:
- ❌ Can't do true HDR passthrough (yet)
- ❌ Can't query output HDR capabilities
- ⚠️ Slightly higher GPU load (tone mapping)

## 🚀 Future Plans

### When Wayland HDR Becomes Stable

When `zwp_xx_color_management_v1` or similar becomes stable:

1. **Detect Protocol**: Check for color management support
2. **Query Capabilities**: 
   - Is HDR supported?
   - Which EOTFs? (PQ, HLG)
   - Luminance range?
3. **Enable Passthrough**: 
   - Skip tone mapping if output supports HDR
   - Configure MPV for direct HDR output
   - Better quality, less GPU load

### Implementation Roadmap

```rust
// Future implementation pseudocode
if output.hdr_capabilities.hdr_supported {
    // HDR display detected
    if video_is_hdr && output_supports_same_format {
        // Direct passthrough
        set_mpv_option("target-colorspace-hint", "yes");
        // No tone mapping needed
    } else {
        // Tone mapping required
        configure_tone_mapping();
    }
} else {
    // SDR display (current path)
    configure_tone_mapping();
}
```

## 📚 References

### Wayland Protocols
- [Color Management Protocol](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/tree/main/staging/color-management)
- [Wayland Book - Color Management](https://wayland-book.com/)

### Compositor Support
- [Hyprland HDR PR](https://github.com/hyprwm/Hyprland/pull/2600)
- [KDE Plasma 6 HDR](https://invent.kde.org/plasma/kwin/-/merge_requests/3798)
- [wlroots HDR Tracking](https://gitlab.freedesktop.org/wlroots/wlroots/-/issues/2920)

### MPV Documentation
- [MPV HDR Options](https://mpv.io/manual/master/#options-target-colorspace-hint)
- [MPV Tone Mapping](https://mpv.io/manual/master/#options-tone-mapping)

## ✅ Conclusion

**Phase 2 Status**: ✅ Complete

We have:
1. ✅ Created `OutputHdrCapabilities` structure
2. ✅ Added to `OutputInfo`
3. ✅ Initialized with safe defaults
4. ✅ Added placeholder for future detection
5. ✅ Documented current state

**Next**: Phase 3 - Configure MPV based on detected capabilities (which will use tone mapping for now)

**Long-term**: When Wayland HDR protocols mature, we can easily enable true HDR passthrough by implementing the `query_hdr_capabilities()` method.
