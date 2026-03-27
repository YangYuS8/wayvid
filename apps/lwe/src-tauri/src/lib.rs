pub const APP_CODE_NAME: &str = "lwe";

pub mod action_outcome;
pub mod models;

pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default().invoke_handler(tauri::generate_handler![])
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_name_uses_lwe_code_name() {
        assert_eq!(super::APP_CODE_NAME, "lwe");
    }
}
