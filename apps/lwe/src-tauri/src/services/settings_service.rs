use crate::results::settings::SettingsPageResult;

pub struct SettingsService;

impl SettingsService {
    pub fn load_page() -> Result<SettingsPageResult, String> {
        Ok(SettingsPageResult {
            language: "system".to_string(),
            theme: "system".to_string(),
            steam_required: true,
            stale: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_service_returns_placeholder_result() {
        let result = SettingsService::load_page().unwrap();

        assert_eq!(result.language, "system");
        assert_eq!(result.theme, "system");
        assert!(result.steam_required);
        assert!(result.stale);
    }
}
