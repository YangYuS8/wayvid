use crate::models::AppShellSnapshot;
use crate::results::app_shell::{ObservedCount, ShellSummary};

fn observed_count_to_option(count: ObservedCount) -> Option<usize> {
    match count {
        ObservedCount::Known(value) => Some(value),
        ObservedCount::Unknown => None,
    }
}

pub fn assemble_app_shell(summary: ShellSummary) -> AppShellSnapshot {
    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available: summary.steam_available,
        library_count: observed_count_to_option(summary.library_items),
        workshop_synced_count: observed_count_to_option(summary.synced_workshop_items),
        monitor_count: observed_count_to_option(summary.connected_monitors),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembler_turns_shell_summary_into_snapshot() {
        let snapshot = assemble_app_shell(ShellSummary {
            steam_available: true,
            library_items: ObservedCount::Known(3),
            synced_workshop_items: ObservedCount::Unknown,
            connected_monitors: ObservedCount::Known(1),
        });

        assert_eq!(snapshot.library_count, Some(3));
        assert_eq!(snapshot.workshop_synced_count, None);
        assert_eq!(snapshot.monitor_count, Some(1));
    }
}
