use crate::action_outcome::{ActionOutcome, AppShellPatch};
use crate::models::WorkshopPageSnapshot;
use crate::policies::shared::invalidation_policy::pages_after_workshop_refresh;
use crate::results::desktop::DesktopApplyResult;
use crate::results::workshop::WorkshopRefreshResult;

use super::workshop_page::assemble_workshop_page;

pub fn assemble_workshop_refresh_outcome(
    result: &WorkshopRefreshResult,
) -> ActionOutcome<WorkshopPageSnapshot> {
    ActionOutcome {
        ok: true,
        message: Some("Workshop catalog refreshed".to_string()),
        shell_patch: Some(AppShellPatch {
            workshop_synced_count: Some(result.synced_entry_count()),
            library_count: None,
            monitor_count: None,
        }),
        current_update: Some(assemble_workshop_page(result)),
        invalidations: if result.library_refresh_required {
            pages_after_workshop_refresh()
        } else {
            Vec::new()
        },
    }
}

pub fn assemble_desktop_apply_outcome(result: DesktopApplyResult) -> ActionOutcome<()> {
    match result {
        DesktopApplyResult::Applied {
            monitor_id,
            item_id,
        } => ActionOutcome {
            ok: true,
            message: Some(format!("Applied {item_id} to {monitor_id}")),
            shell_patch: None,
            current_update: None,
            invalidations: Vec::new(),
        },
        DesktopApplyResult::Cleared { monitor_id } => ActionOutcome {
            ok: true,
            message: Some(format!("Cleared desktop assignment for {monitor_id}")),
            shell_patch: None,
            current_update: None,
            invalidations: Vec::new(),
        },
        DesktopApplyResult::MonitorNotFound { monitor_id } => ActionOutcome {
            ok: false,
            message: Some(format!("Monitor {monitor_id} was not found")),
            shell_patch: None,
            current_update: None,
            invalidations: Vec::new(),
        },
        DesktopApplyResult::MonitorDiscoveryUnavailable { reason }
        | DesktopApplyResult::PersistenceUnavailable { reason } => ActionOutcome {
            ok: false,
            message: Some(reason),
            shell_patch: None,
            current_update: None,
            invalidations: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_outcome::InvalidatedPage;
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn assembler_uses_refresh_result_for_invalidations() {
        let outcome = assemble_workshop_refresh_outcome(&WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: true,
        });

        assert_eq!(outcome.invalidations.len(), 1);
        assert!(matches!(outcome.invalidations[0], InvalidatedPage::Library));
    }

    #[test]
    fn desktop_apply_flow_action_outcome_marks_unavailable_apply_as_failure() {
        let outcome = assemble_desktop_apply_outcome(DesktopApplyResult::PersistenceUnavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        });

        assert!(!outcome.ok);
        assert_eq!(
            outcome.message.as_deref(),
            Some("Desktop persistence is not available yet")
        );
    }
}
