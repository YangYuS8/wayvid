use std::collections::BTreeMap;

use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};

const PERSISTENCE_UNAVAILABLE_REASON: &str = "Desktop persistence is not available yet";

pub struct DesktopPersistenceService;

impl DesktopPersistenceService {
    pub fn load() -> DesktopPersistenceLoad {
        DesktopPersistenceLoad::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }

    pub fn write(_assignments: &BTreeMap<String, String>) -> DesktopPersistenceWrite {
        DesktopPersistenceWrite::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};

    use super::DesktopPersistenceService;

    #[test]
    fn load_returns_unavailable_until_real_persistence_is_implemented() {
        let result = DesktopPersistenceService::load();

        assert!(matches!(
            result,
            DesktopPersistenceLoad::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }

    #[test]
    fn write_returns_unavailable_until_real_persistence_is_implemented() {
        let assignments = BTreeMap::from([("DISPLAY-1".to_string(), "wallpaper-1".to_string())]);

        let result = DesktopPersistenceService::write(&assignments);

        assert!(matches!(
            result,
            DesktopPersistenceWrite::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }
}
