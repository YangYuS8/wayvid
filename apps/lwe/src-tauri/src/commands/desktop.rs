use crate::models::DesktopPageSnapshot;

#[tauri::command]
pub fn load_desktop_page() -> Result<DesktopPageSnapshot, String> {
    crate::desktop::load_desktop_page()
}
