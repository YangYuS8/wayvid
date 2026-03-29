use crate::action_outcome::ActionOutcome;
use crate::assembly::action_outcome::assemble_workshop_refresh_outcome;
use crate::assembly::workshop_detail::assemble_workshop_detail;
use crate::assembly::workshop_page::assemble_workshop_page;
use crate::models::{WorkshopItemDetail, WorkshopPageSnapshot};
use crate::services::workshop_service::WorkshopService;

#[tauri::command]
pub fn load_workshop_page() -> Result<WorkshopPageSnapshot, String> {
    Ok(assemble_workshop_page(&WorkshopService::refresh_catalog()?))
}

#[tauri::command]
pub fn load_workshop_item_detail(workshop_id: String) -> Result<WorkshopItemDetail, String> {
    Ok(assemble_workshop_detail(WorkshopService::inspect_item(
        &workshop_id,
    )?))
}

#[tauri::command]
pub fn refresh_workshop_catalog() -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
    Ok(assemble_workshop_refresh_outcome(
        &WorkshopService::refresh_catalog()?,
    ))
}

#[tauri::command]
pub fn open_workshop_in_steam(workshop_id: String) -> Result<ActionOutcome<()>, String> {
    open::that_detached(crate::workshop::steam_openurl(&workshop_id))
        .map_err(|error| error.to_string())?;

    Ok(ActionOutcome {
        ok: true,
        message: Some("Opened item in Steam".to_string()),
        shell_patch: None,
        current_update: None,
        invalidations: Vec::new(),
    })
}
