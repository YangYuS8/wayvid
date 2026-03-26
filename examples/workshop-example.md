# Workshop Integration Example

This file is a reset-era reference note, not a guide to currently implemented commands.

The current repository direction includes future Workshop-centered discovery and acquisition work, but the repo does not currently provide active Workshop CLI commands. Treat the examples here as planning context for future product work.

## Concept Example

- A future Workshop flow may let users identify a Wallpaper Engine item by Workshop ID, review compatibility, import supported assets, and add them to a local wallpaper library.
- Until that support exists, use this document only as a conceptual example of the product direction.

## Finding Workshop IDs

Visit the [Steam Workshop](https://steamcommunity.com/app/431960/workshop/) and find wallpapers you like.

The ID is in the URL:
```
https://steamcommunity.com/sharedfiles/filedetails/?id=2815866033
                                                         ^^^^^^^^^^
                                                         Workshop ID
```

## Popular Video Wallpapers

Here are some popular video wallpapers to try (IDs may change):

- **Rainy Night City**: Search for "rainy night" in Workshop
- **Sakura Garden**: Search for "sakura" in Workshop
- **Space Scene**: Search for "space" with "Video" tag

## Concept Workflow

### Future Steam Client-Oriented Flow

1. Subscribe to wallpapers in Steam or locate already downloaded Workshop items.
2. Identify the Workshop item ID and inspect the downloaded asset layout.
3. Determine whether the item is a supported `video` or `scene` wallpaper.
4. Import or register supported assets in the future wallpaper library workflow once that support exists.

### Future Direct Acquisition Flow

1. Find a Workshop ID from a Steam URL.
2. Acquire the item through a future supported workflow.
3. Review compatibility details before adding it to the library.

## Multi-Monitor Planning Notes

- A future library-first application could let users choose different Workshop-derived wallpapers per display.
- Example asset mapping might associate one monitor with a scene wallpaper and another with a video wallpaper after import and compatibility review.

## Library and Cache Planning Notes

- Future Workshop support may need local cache management, imported asset tracking, and cleanup tools.
- Those workflows are not implemented in the current reset-era repository state.

## Troubleshooting

### No Steam Installation

The reset-era repository does not currently provide a direct download workflow. Keep this case as future product planning only.

### Item Not Downloading

Some Workshop items require Steam client authentication. In this case:

1. Install Steam
2. Subscribe to the item in Steam Workshop
3. Verify the item is available in the local Workshop content directory
4. Keep the item as future import/library work until supported commands exist

### Wrong Wallpaper Type

For the reset-era first release, focus on **video** and **scene** wallpapers. Check:

- Look for "Video" or "Scene" tags in Workshop
- Avoid "Web" or "Application" types for the current reset scope
- Check file list for video assets or scene project files before importing

## See Also

- [Product Overview](../docs/product/overview.md)
- [Product Roadmap](../docs/product/roadmap.md)
- [Repository Reset Inventory](../docs/product/repository-reset.md)
