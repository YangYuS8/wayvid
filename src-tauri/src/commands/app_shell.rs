use crate::assembly::app_shell::assemble_app_shell;
use crate::models::AppShellSnapshot;
use crate::services::app_shell_service::AppShellService;

#[tauri::command]
pub fn load_app_shell() -> Result<AppShellSnapshot, String> {
    Ok(assemble_app_shell(AppShellService::load_summary()?))
}
