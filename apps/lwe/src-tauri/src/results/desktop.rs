#[derive(Debug, Clone)]
pub struct DesktopPageResult {
    pub monitor_count: Option<usize>,
    pub stale: bool,
}
