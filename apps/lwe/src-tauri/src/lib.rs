pub mod action_outcome;
pub mod app_shell;
pub mod assembly;
pub mod desktop;
pub mod library;
pub mod models;
pub mod policies;
pub mod results;
pub mod services;
pub mod settings;
pub mod workshop;

pub const APP_CODE_NAME: &str = "lwe";

pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        app_shell::load_app_shell,
        workshop::load_workshop_page,
        workshop::load_workshop_item_detail,
        workshop::refresh_workshop_catalog,
        workshop::open_workshop_in_steam,
        library::load_library_page,
        library::load_library_item_detail,
        desktop::load_desktop_page,
        settings::load_settings_page,
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
