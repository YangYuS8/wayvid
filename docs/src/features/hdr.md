# HDR

HDR10 support with automatic tone-mapping.

## Enable

```yaml
hdr_mode: auto  # auto, force, disable
```

## Requirements

- HDR-capable display
- Compositor with HDR support (Hyprland, Niri)

## Formats

| Format | Support |
|--------|---------|
| HDR10 | ✓ |
| HLG | ✓ |
| HDR10+ | Tone-map only |
| Dolby Vision | Tone-map only |

## Verify

```bash
wayvid-ctl status | grep hdr
```

## Troubleshooting

**Washed out colors:**
- Check compositor HDR settings
- Verify video is true HDR format
