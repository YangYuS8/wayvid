## Context
The current wayvid GUI layout is inefficient: the sidebar and main content panel have similar widths, wasting space. Additionally, users familiar with Wallpaper Engine expect a similar layout with wallpaper details on the right side. The default OpenGL renderer could be upgraded to Vulkan for better performance.

## Goals / Non-Goals

### Goals
- Improve space utilization with narrower sidebar and detail panel
- Match Wallpaper Engine UX patterns for familiarity
- Use Workshop preview images for faster loading and better accuracy
- Default to Vulkan renderer for modern GPU performance

### Non-Goals
- Complete Wallpaper Engine UI clone (keep wayvid's identity)
- Custom rendering pipeline (continue using iced framework)
- Dynamic renderer switching without restart

## Decisions

### Decision: Fixed-width sidebar (220px)
- **Why**: Variable/equal width wastes space; 220px fits icons + labels comfortably
- **Alternatives**: Percentage-based (rejected: too wide on large screens)

### Decision: Right-side detail panel
- **Why**: Matches Wallpaper Engine pattern; keeps library grid focused
- **Alternatives**: Bottom panel (rejected: reduces vertical scroll space)

### Decision: Vulkan default with OpenGL fallback
- **Why**: Vulkan provides better performance on modern GPUs; OpenGL needed for older systems
- **Alternatives**: OpenGL-only (rejected: misses performance opportunity)

### Decision: Prefer Workshop preview.jpg/gif
- **Why**: Preview files are curated by creators; faster than video frame extraction
- **Alternatives**: Always generate (rejected: slower, may pick poor frame)

## Layout Specification

### Main Window Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Header (Title Bar)                                          │
├────────┬────────────────────────────────────────┬───────────┤
│        │                                        │           │
│ Sidebar│       Library Grid Panel               │  Detail   │
│  220px │         (flexible width)               │  Panel    │
│        │                                        │   280px   │
│ - Home │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ │           │
│ - Lib  │  │      │ │      │ │      │ │      │ │ [Preview] │
│ - Sets │  │ Thumb│ │ Thumb│ │ Thumb│ │ Thumb│ │           │
│ - About│  │      │ │      │ │      │ │      │ │  Title    │
│        │  └──────┘ └──────┘ └──────┘ └──────┘ │  Author   │
│ Status │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ │  Tags     │
│        │  │      │ │      │ │      │ │      │ │           │
│        │  │ Thumb│ │ Thumb│ │ Thumb│ │ Thumb│ │ [Apply]   │
│        │  │      │ │      │ │      │ │      │ │ [Options] │
│        │  └──────┘ └──────┘ └──────┘ └──────┘ │           │
├────────┴────────────────────────────────────────┴───────────┤
│ Status Bar                                                  │
└─────────────────────────────────────────────────────────────┘
```

### Detail Panel Content
1. **Preview Image** - Large preview (280x180 or aspect-fit)
2. **Title** - Wallpaper name
3. **Author** - From project.json or "Unknown"
4. **Type Badge** - Video/Scene/GIF icon
5. **Tags** - Scrollable tag list
6. **Actions**:
   - Apply to Monitor dropdown
   - Open folder button
   - View on Workshop link (if Workshop item)

## Risks / Trade-offs

### Risk: Vulkan driver compatibility
- **Mitigation**: Automatic fallback to OpenGL; clear error messages
- **Detection**: Check for Vulkan support at startup

### Risk: Layout may feel cramped on small screens
- **Mitigation**: Collapsible sidebar and detail panel; responsive breakpoints

## Open Questions
- Should animated GIF previews play inline or show first frame only?
- Should detail panel width be user-adjustable?
