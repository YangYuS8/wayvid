#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedCount {
    Known(usize),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSummary {
    pub steam_available: bool,
    pub library_items: ObservedCount,
    pub synced_workshop_items: ObservedCount,
    pub connected_monitors: ObservedCount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_policy_shell_summary_keeps_unknown_counts_explicit() {
        let summary = ShellSummary {
            steam_available: true,
            library_items: ObservedCount::Known(12),
            synced_workshop_items: ObservedCount::Unknown,
            connected_monitors: ObservedCount::Known(2),
        };

        assert_eq!(summary.library_items, ObservedCount::Known(12));
        assert_eq!(summary.synced_workshop_items, ObservedCount::Unknown);
    }
}
