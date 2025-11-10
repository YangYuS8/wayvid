# HDR Support

wayvid provides full HDR10 support with tone-mapping.

## Requirements

- HDR-capable display
- Compositor with HDR support (Hyprland, Niri)
- HDR-enabled output

## Enable HDR

```yaml
hdr:
  enabled: true
  target_nits: 1000  # Adjust to your display
```

## Target Nits

Common values:
- **400-600**: Entry HDR displays
- **1000**: Standard HDR10
- **1400+**: Premium HDR displays

Find your display's peak brightness in specifications.

## Tone-Mapping

Automatic conversion between HDR and SDR content.

### HDR Content on HDR Display
Direct passthrough, no conversion.

### SDR Content on HDR Display
Tone-mapped to HDR using BT.2020 color space.

### HDR Content on SDR Display
Tone-mapped to SDR using Reinhard algorithm.

## Verify HDR

```bash
# Check if HDR is active
wayvid-ctl status | grep hdr

# Expected output:
# hdr: enabled (1000 nits)
```

## Supported Formats

| Format | Support |
|--------|---------|
| HDR10 | ✅ Full |
| HDR10+ | ⚠️ Tone-map only |
| Dolby Vision | ⚠️ Tone-map only |
| HLG | ✅ Full |

## Performance

HDR adds minimal overhead:
- ~2-5% CPU increase
- Same GPU usage
- No memory impact

## Troubleshooting

**Colors look washed out:**
- Verify `target_nits` matches display
- Check compositor HDR settings
- Ensure video is true HDR

**HDR not working:**
- Verify display supports HDR
- Enable HDR in compositor config
- Check `wayvid-ctl status`

## Technical Details

- **Color Space**: BT.2020
- **Transfer Function**: PQ (SMPTE ST 2084)
- **Bit Depth**: 10-bit
- **Tone-Mapping**: Reinhard, Hable, Mobius
