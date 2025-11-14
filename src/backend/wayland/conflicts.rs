//! Detect and handle conflicts with other wallpaper managers

use std::process::Command;
use tracing::warn;

/// Check for conflicting wallpaper managers
pub fn check_wallpaper_conflicts() {
    // Check for swww
    if is_process_running("swww-daemon") {
        warn!("⚠️  Detected swww-daemon running");
        warn!("⚠️  swww and wayvid both use the Background layer");
        warn!("⚠️  This may cause wayvid to be hidden behind swww");
        warn!("⚠️  To fix: run 'killall swww-daemon' before starting wayvid");
        warn!("⚠️  Or remove swww from your compositor's autostart");
    }

    // Check for hyprpaper
    if is_process_running("hyprpaper") {
        warn!("⚠️  Detected hyprpaper running - may conflict with wayvid");
        warn!("⚠️  Consider stopping hyprpaper if wayvid is not visible");
    }

    // Check for swaybg
    if is_process_running("swaybg") {
        warn!("⚠️  Detected swaybg running - may conflict with wayvid");
        warn!("⚠️  Consider stopping swaybg if wayvid is not visible");
    }
}

/// Check if a process is running by name
fn is_process_running(name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_check() {
        // Should not panic
        check_wallpaper_conflicts();
    }
}
