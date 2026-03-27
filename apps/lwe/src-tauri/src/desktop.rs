use crate::models::DesktopPageSnapshot;

#[tauri::command]
pub fn load_desktop_page() -> DesktopPageSnapshot {
    DesktopPageSnapshot {
        monitors: Vec::new(),
        stale: false,
    }
}
