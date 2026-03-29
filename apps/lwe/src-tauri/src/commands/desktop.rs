use crate::assembly::desktop_page::assemble_desktop_page;
use crate::models::DesktopPageSnapshot;
use crate::services::desktop_service::DesktopService;

#[tauri::command]
pub fn load_desktop_page() -> Result<DesktopPageSnapshot, String> {
    DesktopService::load_page().map(assemble_desktop_page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_desktop_snapshot_is_marked_stale() {
        let snapshot = load_desktop_page().unwrap();

        assert!(snapshot.monitors.is_empty());
        assert!(snapshot.stale);
    }
}
