use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};

const PERSISTENCE_UNAVAILABLE_REASON: &str = "Desktop persistence is not available yet";

pub struct DesktopPersistenceService;

impl DesktopPersistenceService {
    pub fn load_state() -> DesktopPersistenceLoad {
        DesktopPersistenceLoad::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }

    pub fn save_assignment(_monitor_id: &str, _item_id: &str) -> DesktopPersistenceWrite {
        DesktopPersistenceWrite::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }

    pub fn clear_assignment(_monitor_id: &str) -> DesktopPersistenceWrite {
        DesktopPersistenceWrite::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};

    use super::DesktopPersistenceService;

    #[test]
    fn load_state_returns_unavailable_until_real_persistence_is_implemented() {
        let result = DesktopPersistenceService::load_state();

        assert!(matches!(
            result,
            DesktopPersistenceLoad::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }

    #[test]
    fn save_assignment_returns_unavailable_until_real_persistence_is_implemented() {
        let result = DesktopPersistenceService::save_assignment("DISPLAY-1", "wallpaper-1");

        assert!(matches!(
            result,
            DesktopPersistenceWrite::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }

    #[test]
    fn clear_assignment_returns_unavailable_until_real_persistence_is_implemented() {
        let result = DesktopPersistenceService::clear_assignment("DISPLAY-1");

        assert!(matches!(
            result,
            DesktopPersistenceWrite::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }
}
