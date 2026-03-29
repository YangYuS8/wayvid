use crate::assembly::library_detail::assemble_library_detail;
use crate::assembly::library_page::assemble_library_page;
use crate::models::{LibraryItemDetail, LibraryPageSnapshot};
use crate::services::library_service::LibraryService;

#[tauri::command]
pub fn load_library_page() -> Result<LibraryPageSnapshot, String> {
    Ok(assemble_library_page(LibraryService::load_projection()?))
}

#[tauri::command]
pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    Ok(assemble_library_detail(LibraryService::inspect_item(
        &item_id,
    )?))
}
