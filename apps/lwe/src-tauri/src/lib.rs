// Layered application core for the Tauri shell.
pub mod action_outcome;
pub mod assembly;
pub mod commands;
pub mod models;
pub mod policies;
pub mod results;
pub mod services;

pub const APP_CODE_NAME: &str = "lwe";

pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        commands::app_shell::load_app_shell,
        commands::workshop::load_workshop_page,
        commands::workshop::load_workshop_item_detail,
        commands::workshop::refresh_workshop_catalog,
        commands::workshop::open_workshop_in_steam,
        commands::library::load_library_page,
        commands::library::load_library_item_detail,
        commands::desktop::load_desktop_page,
        commands::desktop::apply_library_item_to_monitor,
        commands::desktop::clear_library_item_from_monitor,
        commands::settings::load_settings_page,
        commands::settings::update_settings,
    ])
}

pub fn builder() -> tauri::Builder<tauri::Wry> {
    register_commands(tauri::Builder::default())
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_name_uses_lwe_code_name() {
        assert_eq!(super::APP_CODE_NAME, "lwe");
    }
}
