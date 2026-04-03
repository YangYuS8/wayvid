use crate::assembly::library_detail::assemble_library_detail;
use crate::assembly::library_page::assemble_library_page;
use crate::models::{LibraryItemDetail, LibraryPageSnapshot};
use crate::services::desktop_service::DesktopService;
use crate::services::library_service::LibraryService;

#[tauri::command]
pub fn load_library_page() -> Result<LibraryPageSnapshot, String> {
    let projection = LibraryService::load_projection()?;
    let desktop = DesktopService::load_page_with_projection(Ok(projection.clone()))?;

    Ok(assemble_library_page(projection, &desktop))
}

#[tauri::command]
pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    let projection = LibraryService::load_projection()?;
    let desktop = DesktopService::load_page_with_projection(Ok(projection.clone()))?;

    Ok(assemble_library_detail(
        LibraryService::inspect_item_in_projection(&projection, &item_id)?,
        &desktop,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_apply_flow_library_page_reuses_desktop_state_in_snapshot() {
        let snapshot = load_library_page().unwrap();

        assert!(snapshot.desktop_assignment_issue.is_none());
        assert!(snapshot.desktop_assignments_available);

        if snapshot.monitors_available {
            assert!(snapshot.monitor_discovery_issue.is_none());
        } else {
            assert!(snapshot.monitor_discovery_issue.is_some());
        }
    }
}
