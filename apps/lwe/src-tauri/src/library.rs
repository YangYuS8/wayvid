use crate::models::{LibraryItemDetail, LibraryPageSnapshot};
use crate::services::library_service::LibraryService;

#[tauri::command]
pub fn load_library_page() -> Result<LibraryPageSnapshot, String> {
    LibraryService::load_page()
}

#[tauri::command]
pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    LibraryService::load_item_detail(&item_id)
}
