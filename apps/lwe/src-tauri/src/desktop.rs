use crate::models::DesktopPageSnapshot;

fn placeholder_desktop_page() -> DesktopPageSnapshot {
    DesktopPageSnapshot {
        monitors: Vec::new(),
        stale: true,
    }
}

#[tauri::command]
pub fn load_desktop_page() -> DesktopPageSnapshot {
    placeholder_desktop_page()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_desktop_snapshot_is_marked_stale() {
        let snapshot = placeholder_desktop_page();

        assert!(snapshot.monitors.is_empty());
        assert!(snapshot.stale);
    }
}
