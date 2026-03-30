use crate::assembly::library_detail::assemble_library_detail;
use crate::assembly::library_page::assemble_library_page;
use crate::models::{LibraryItemDetail, LibraryPageSnapshot};
use crate::services::desktop_service::DesktopService;
use crate::services::library_service::LibraryService;

#[tauri::command]
pub fn load_library_page() -> Result<LibraryPageSnapshot, String> {
    let desktop = DesktopService::load_page()?;

    Ok(assemble_library_page(
        LibraryService::load_projection()?,
        &desktop,
    ))
}

#[tauri::command]
pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    let desktop = DesktopService::load_page()?;

    Ok(assemble_library_detail(
        LibraryService::inspect_item(&item_id)?,
        &desktop,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_apply_flow_library_page_reuses_desktop_state_in_snapshot() {
        let snapshot = load_library_page().unwrap();

        assert!(!snapshot.desktop_assignments_available);
        assert_eq!(
            snapshot.desktop_assignment_issue.as_deref(),
            Some("Desktop persistence is not available yet")
        );

        if snapshot.monitors_available {
            assert!(snapshot.monitor_discovery_issue.is_none());
        } else {
            assert!(snapshot.monitor_discovery_issue.is_some());
        }
    }
}
