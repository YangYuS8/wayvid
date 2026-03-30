use std::path::PathBuf;

use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};
use crate::services::backends::persistence_backend::{
    desktop_state_path, JsonFilePersistenceBackend, PersistenceBackend,
};

const PERSISTENCE_UNAVAILABLE_REASON: &str = "Desktop persistence is not available yet";

pub struct DesktopPersistenceService;

pub struct ScopedDesktopPersistenceService {
    backend: JsonFilePersistenceBackend,
}

impl DesktopPersistenceService {
    pub fn load_state() -> DesktopPersistenceLoad {
        DesktopPersistenceLoad::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }

    pub fn save_assignment(monitor_id: &str, item_id: &str) -> DesktopPersistenceWrite {
        let _ = (monitor_id, item_id);

        DesktopPersistenceWrite::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }

    pub fn clear_assignment(monitor_id: &str) -> DesktopPersistenceWrite {
        let _ = monitor_id;

        DesktopPersistenceWrite::Unavailable {
            reason: PERSISTENCE_UNAVAILABLE_REASON.to_string(),
        }
    }

    pub fn for_user_path() -> Result<ScopedDesktopPersistenceService, String> {
        desktop_state_path().map(Self::for_path)
    }

    pub fn for_path(path: PathBuf) -> ScopedDesktopPersistenceService {
        ScopedDesktopPersistenceService {
            backend: JsonFilePersistenceBackend::new(path),
        }
    }

    pub fn for_test(path: PathBuf) -> ScopedDesktopPersistenceService {
        Self::for_path(path)
    }

    fn load_with_backend(backend: &impl PersistenceBackend) -> DesktopPersistenceLoad {
        match backend.load_assignments() {
            Ok(assignments) => DesktopPersistenceLoad::Loaded(assignments.assignments),
            Err(reason) => DesktopPersistenceLoad::Unavailable { reason },
        }
    }

    fn save_with_backend(
        backend: &impl PersistenceBackend,
        monitor_id: &str,
        item_id: &str,
    ) -> DesktopPersistenceWrite {
        let mut assignments = match backend.load_assignments() {
            Ok(assignments) => assignments,
            Err(reason) => return DesktopPersistenceWrite::Unavailable { reason },
        };

        assignments
            .assignments
            .insert(monitor_id.to_string(), item_id.to_string());

        match backend.save_assignments(&assignments) {
            Ok(()) => DesktopPersistenceWrite::Saved,
            Err(reason) => DesktopPersistenceWrite::Unavailable { reason },
        }
    }

    fn clear_with_backend(
        backend: &impl PersistenceBackend,
        monitor_id: &str,
    ) -> DesktopPersistenceWrite {
        let mut assignments = match backend.load_assignments() {
            Ok(assignments) => assignments,
            Err(reason) => return DesktopPersistenceWrite::Unavailable { reason },
        };

        assignments.assignments.remove(monitor_id);

        let result = if assignments.assignments.is_empty() {
            backend.clear_assignments()
        } else {
            backend.save_assignments(&assignments)
        };

        match result {
            Ok(()) => DesktopPersistenceWrite::Cleared,
            Err(reason) => DesktopPersistenceWrite::Unavailable { reason },
        }
    }
}

impl ScopedDesktopPersistenceService {
    pub fn load_state(&self) -> DesktopPersistenceLoad {
        DesktopPersistenceService::load_with_backend(&self.backend)
    }

    pub fn save_assignment(&self, monitor_id: &str, item_id: &str) -> DesktopPersistenceWrite {
        DesktopPersistenceService::save_with_backend(&self.backend, monitor_id, item_id)
    }

    pub fn clear_assignment(&self, monitor_id: &str) -> DesktopPersistenceWrite {
        DesktopPersistenceService::clear_with_backend(&self.backend, monitor_id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};

    use super::DesktopPersistenceService;

    fn test_state_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("desktop-persistence-service-{unique}.json"))
    }

    #[test]
    fn persistence_service_round_trips_assignments() {
        let path = test_state_path();
        let service = DesktopPersistenceService::for_test(path.clone());

        assert!(matches!(
            service.load_state(),
            crate::results::desktop_persistence::DesktopPersistenceLoad::Loaded(_)
        ));

        assert!(matches!(
            service.save_assignment("eDP-1", "item-1"),
            crate::results::desktop_persistence::DesktopPersistenceWrite::Saved
        ));

        let persisted: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(persisted["assignments"]["eDP-1"], "item-1");

        let loaded = service.load_state();
        match loaded {
            crate::results::desktop_persistence::DesktopPersistenceLoad::Loaded(assignments) => {
                assert_eq!(assignments.get("eDP-1").unwrap(), "item-1");
            }
            _ => panic!("expected loaded assignments"),
        }
    }

    #[test]
    fn default_service_methods_remain_unavailable_until_activation_is_threaded() {
        assert!(matches!(
            DesktopPersistenceService::load_state(),
            DesktopPersistenceLoad::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));

        assert!(matches!(
            DesktopPersistenceService::save_assignment("eDP-1", "item-1"),
            DesktopPersistenceWrite::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));

        assert!(matches!(
            DesktopPersistenceService::clear_assignment("eDP-1"),
            DesktopPersistenceWrite::Unavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }

    #[test]
    fn load_state_returns_empty_assignments_when_file_is_missing() {
        let path = test_state_path();
        let service = DesktopPersistenceService::for_test(path);

        let result = service.load_state();

        assert!(matches!(
            result,
            DesktopPersistenceLoad::Loaded(assignments) if assignments.is_empty()
        ));
    }

    #[test]
    fn clear_assignment_removes_saved_monitor_assignment() {
        let path = test_state_path();
        let service = DesktopPersistenceService::for_test(path);

        assert!(matches!(
            service.save_assignment("DISPLAY-1", "wallpaper-1"),
            DesktopPersistenceWrite::Saved
        ));

        let result = service.clear_assignment("DISPLAY-1");

        assert!(matches!(result, DesktopPersistenceWrite::Cleared));

        assert!(matches!(
            service.load_state(),
            DesktopPersistenceLoad::Loaded(assignments) if assignments == BTreeMap::new()
        ));
    }

    #[test]
    fn load_state_returns_unavailable_for_invalid_json() {
        let path = test_state_path();
        std::fs::write(&path, "not json").unwrap();

        let service = DesktopPersistenceService::for_test(path);
        let result = service.load_state();

        assert!(matches!(
            result,
            DesktopPersistenceLoad::Unavailable { reason }
                if !reason.is_empty()
        ));
    }
}
