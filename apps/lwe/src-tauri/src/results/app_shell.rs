#[derive(Debug, Clone)]
pub struct ShellSummary {
    pub steam_available: bool,
    pub library_count: Option<usize>,
    pub workshop_synced_count: Option<usize>,
    pub monitor_count: Option<usize>,
}
