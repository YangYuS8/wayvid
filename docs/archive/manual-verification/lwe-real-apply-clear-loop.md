# LWE Real Apply/Clear Loop Manual Verification

## Environment

- Platform: Linux
- Display stack: Wayland + niri
- Verification branch: `lwe-real-apply-clear-main`

## Verified Flow

The following end-to-end loop was manually verified on the local machine:

1. Open `Library`
2. Select a compatible video wallpaper item
3. Choose the real active monitor
4. Click `Apply`
5. Confirm the wallpaper is visibly applied to the desktop
6. Switch to a second compatible video wallpaper
7. Click `Apply` again and confirm the wallpaper visibly changes
8. Open `Desktop`
9. Click `Clear`
10. Confirm the wallpaper is visibly cleared from the desktop

## Outcome

- Initial apply: success
- Apply while switching to another wallpaper: success
- Clear from Desktop: success

This confirms that the current LWE shell now has one real, user-visible desktop action loop on the local Wayland + niri environment rather than only a simulated or placeholder path.
