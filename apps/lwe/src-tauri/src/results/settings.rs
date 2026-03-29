#[derive(Debug, Clone)]
pub struct SettingsPageResult {
    pub language: String,
    pub theme: String,
    pub steam_required: bool,
    pub stale: bool,
}
