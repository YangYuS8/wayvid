# HDR Support User Guide

## Overview

wayvid now supports HDR (High Dynamic Range) content including **HDR10** and **HLG** formats. The player automatically detects HDR content and applies tone mapping to display it correctly on SDR (Standard Dynamic Range) displays.

## Features

✨ **Automatic HDR Detection**: Detects HDR10, HLG, and Dolby Vision content  
🎨 **Smart Tone Mapping**: Converts HDR to SDR for standard displays  
🔧 **Content-Aware Optimization**: Automatically adjusts settings based on content type  
⚡ **Multiple Algorithms**: Choose from 5 different tone mapping algorithms  
📊 **Performance Modes**: Balance quality and GPU load  

## Quick Start

### Default Configuration (Recommended)

```yaml
source:
  type: file
  path: /path/to/hdr-video.mp4

hdr_mode: auto  # Automatically handle HDR

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: hybrid
```

This configuration:
- ✅ Automatically detects HDR content
- ✅ Applies optimal tone mapping for SDR displays
- ✅ Uses content-aware parameter optimization
- ✅ Works great for most content

### Running with HDR

```bash
# Simple run
wayvid run --config config.yaml

# With debug logging to see HDR info
wayvid run --config config.yaml --log-level debug
```

### Check HDR Detection

When running with `--log-level debug`, you'll see:

```
🎨 Configuring HDR handling...
📊 Video HDR metadata detected:
  Color space: Hdr10
  Transfer function: Pq
  Primaries: bt.2020-ncl
  Peak luminance: 1000.0 nits
✨ HDR content detected: HDR10
🖥️  Output is SDR - enabling tone mapping
  Algorithm: hable (Hable (Uncharted 2) - Best overall quality, good contrast)
  Mode: rgb
  Parameter: 1.20
✓ Tone mapping configured
```

## HDR Modes

### `auto` (Default)
- Automatically detects HDR content
- Applies tone mapping only when needed
- **Recommended for most users**

### `force`
- Forces HDR tone mapping even for SDR content
- Useful for testing or specific effects

### `disable`
- Disables all HDR processing
- Display content as-is (may look wrong for HDR)

## Tone Mapping Algorithms

### `hable` (Uncharted 2) ⭐ **Recommended**

**Best for**: Movies, general content  
**Quality**: Excellent  
**Performance**: Moderate  

```yaml
tone_mapping:
  algorithm: hable
  param: 1.0  # Higher = more contrast (0.8-1.5)
```

- ✅ Best overall quality
- ✅ Good contrast and detail balance
- ✅ Works well with most content
- ✅ Default choice

### `mobius`

**Best for**: Animation, bright scenes  
**Quality**: Excellent detail preservation  
**Performance**: Good  

```yaml
tone_mapping:
  algorithm: mobius
  param: 0.3  # Lower = more detail (0.2-0.5)
```

- ✅ Preserves highlight details
- ✅ Softer, more natural look
- ✅ Great for animation
- ⚠️ May look less punchy

### `reinhard`

**Best for**: Low-end hardware  
**Quality**: Good  
**Performance**: Excellent  

```yaml
tone_mapping:
  algorithm: reinhard
  param: 0.5  # Balance (0.4-0.8)
```

- ✅ Fast and simple
- ✅ Low GPU load
- ✅ Good enough for most content
- ⚠️ Less sophisticated than others

### `bt2390`

**Best for**: Professional, documentary  
**Quality**: Reference  
**Performance**: Good  

```yaml
tone_mapping:
  algorithm: bt2390
  param: 1.0  # ITU standard
```

- ✅ ITU broadcasting standard
- ✅ Natural, neutral look
- ✅ Professional reference
- ⚠️ May be less exciting

### `clip`

**Best for**: Testing, debugging  
**Quality**: Poor  
**Performance**: Excellent  

```yaml
tone_mapping:
  algorithm: clip
```

- ✅ Minimal GPU load
- ✅ No processing overhead
- ❌ Loses highlight details
- ❌ Not recommended for viewing

## Tone Mapping Modes

### `hybrid` (Default) ⭐

Balanced processing of RGB and luminance. **Good for most content**.

### `rgb`

Process each color channel separately. **Better for cinema and high-contrast content**, but may cause slight color shifts.

### `luma`

Process luminance only. **Preserves color saturation**, great for animation and vibrant content.

### `auto`

Let MPV choose based on content analysis.

## Content-Aware Optimization

wayvid automatically optimizes tone mapping parameters based on detected content type:

### Cinema (Peak > 2000 nits)
```yaml
# Automatically applied
tone_mapping:
  algorithm: hable
  param: 1.2    # Higher contrast
  mode: rgb     # Better cinema look
```

### Animation
```yaml
tone_mapping:
  algorithm: mobius
  param: 0.35   # Preserve details
  mode: luma    # Keep colors vibrant
```

### Documentary/Nature
```yaml
tone_mapping:
  algorithm: bt2390  # Natural standard
  param: 1.0
  mode: auto
```

### Low Dynamic Range (Peak < 400 nits)
```yaml
tone_mapping:
  algorithm: reinhard
  param: 0.6    # Gentle mapping
```

## Performance Tuning

### Maximum Quality

```yaml
tone_mapping:
  algorithm: hable
  compute_peak: true   # Dynamic peak detection
  mode: hybrid
```

**GPU Load**: High  
**Quality**: Maximum  
**Use for**: High-end hardware, reference viewing

### Balanced (Default)

```yaml
tone_mapping:
  algorithm: hable
  compute_peak: true
  mode: hybrid
```

**GPU Load**: Moderate  
**Quality**: Excellent  
**Use for**: Most systems

### Performance Mode

```yaml
tone_mapping:
  algorithm: reinhard
  compute_peak: false  # Disable dynamic analysis
  mode: luma          # Faster than RGB
```

**GPU Load**: Low  
**Quality**: Good  
**Use for**: Low-end hardware, battery saving

## Advanced Configuration

### Fine-Tuning Parameters

```yaml
tone_mapping:
  # Algorithm selection
  algorithm: hable
  
  # Algorithm parameter
  # - hable: 0.8-1.5 (higher = more contrast)
  # - mobius: 0.2-0.5 (lower = more detail)
  # - reinhard: 0.4-0.8 (balance)
  param: 1.0
  
  # Dynamic peak detection
  # true = analyze each frame (better quality)
  # false = use static value (faster)
  compute_peak: true
  
  # Processing mode
  # hybrid = balanced (default)
  # rgb = better cinema
  # luma = preserve colors
  # auto = let MPV decide
  mode: hybrid
```

### Per-Output Configuration

You can configure HDR settings per output:

```yaml
outputs:
  HDMI-A-1:
    source:
      type: file
      path: /path/to/movie.mp4
    
    # Cinema-optimized for TV
    tone_mapping:
      algorithm: hable
      param: 1.2
      mode: rgb
  
  eDP-1:
    source:
      type: file
      path: /path/to/anime.mkv
    
    # Animation-optimized for laptop
    tone_mapping:
      algorithm: mobius
      param: 0.35
      mode: luma
```

## Troubleshooting

### HDR Content Looks Too Dark

**Solution**: Increase tone mapping parameter

```yaml
tone_mapping:
  algorithm: hable
  param: 1.3  # Increase from 1.0
```

### HDR Content Looks Too Bright/Washed Out

**Solution**: Decrease tone mapping parameter

```yaml
tone_mapping:
  algorithm: hable
  param: 0.8  # Decrease from 1.0
```

### Colors Look Wrong

**Solution**: Try different mode

```yaml
tone_mapping:
  mode: luma  # Preserves colors better
```

### Performance Issues

**Solution 1**: Disable dynamic peak detection

```yaml
tone_mapping:
  compute_peak: false
```

**Solution 2**: Use faster algorithm

```yaml
tone_mapping:
  algorithm: reinhard
```

### HDR Not Detected

**Check logs with debug level**:

```bash
wayvid run --config config.yaml --log-level debug
```

Look for:
```
📊 Video HDR metadata detected:
  Color space: ...
  Transfer function: ...
```

If metadata shows `Unknown`, the video might not be HDR, or MPV couldn't detect it.

## Testing HDR

### Download Test Videos

**HDR10 Test Patterns**:
- LG HDR Demo: https://4kmedia.org
- Samsung HDR Tests: https://demo-uhd3d.com

**HLG Test Content**:
- BBC HLG Tests: https://www.bbc.co.uk/rd/blog

### Quick Test Script

```bash
# Test all algorithms with your HDR video
./scripts/test-hdr-tonemapping.sh /path/to/hdr-video.mp4 5 debug
```

This will:
- Test all 5 tone mapping algorithms
- Run each for 5 seconds
- Show detailed logs

### Visual Comparison

1. Start with `hable` (default)
2. If highlights are clipped, try `mobius`
3. If too slow, try `reinhard`
4. For reference, try `bt2390`

## Requirements

### Software
- MPV >= 0.35 (for full HDR support)
- GPU with OpenGL 3.3+ support

### Hardware
- Any GPU that supports hardware decoding
- More VRAM = better for high-resolution HDR

## Future Features

🚧 **HDR Passthrough**: Direct HDR output to HDR displays (when Wayland protocols mature)  
🚧 **Dolby Vision**: Full Dolby Vision support  
🚧 **HDR10+**: Dynamic metadata support  

## References

- [MPV HDR Documentation](https://mpv.io/manual/master/#options-target-colorspace-hint)
- [MPV Tone Mapping Guide](https://mpv.io/manual/master/#options-tone-mapping)
- [ITU BT.2390 Standard](https://www.itu.int/rec/R-REC-BT.2390)

## Examples

See `examples/hdr-config.yaml` for complete configuration examples for different content types.
