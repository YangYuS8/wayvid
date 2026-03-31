use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedDesktopAssignments {
    pub assignments: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct JsonFilePersistenceBackend {
    path: PathBuf,
}

impl JsonFilePersistenceBackend {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn desktop_state_path_from_env(
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let base = xdg_config_home.or_else(|| home.map(|home| home.join(".config")));

    match base {
        Some(path) if path.is_absolute() => Ok(path.join("wayvid").join("desktop-state.json")),
        Some(path) => Err(format!(
            "Unable to resolve desktop persistence path from non-absolute config root {}",
            path.display()
        )),
        None => Err(
            "Unable to resolve desktop persistence path because XDG_CONFIG_HOME and HOME are unset"
                .to_string(),
        ),
    }
}

pub fn desktop_state_path() -> Result<PathBuf, String> {
    desktop_state_path_from_env(
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
        std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
    )
}

pub trait PersistenceBackend {
    fn load_assignments(&self) -> Result<PersistedDesktopAssignments, String>;

    fn save_assignments(&self, assignments: &PersistedDesktopAssignments) -> Result<(), String>;

    fn clear_assignments(&self) -> Result<(), String>;
}

impl PersistenceBackend for JsonFilePersistenceBackend {
    fn load_assignments(&self) -> Result<PersistedDesktopAssignments, String> {
        match fs::read_to_string(self.path()) {
            Ok(contents) => serde_json::from_str(&contents).map_err(|error| {
                format!(
                    "Failed to parse desktop assignments from {}: {error}",
                    self.path().display()
                )
            }),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                Ok(PersistedDesktopAssignments::default())
            }
            Err(error) => Err(format!(
                "Failed to read desktop assignments from {}: {error}",
                self.path().display()
            )),
        }
    }

    fn save_assignments(&self, assignments: &PersistedDesktopAssignments) -> Result<(), String> {
        if let Some(parent) = self.path().parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create desktop assignments directory {}: {error}",
                    parent.display()
                )
            })?;
        }

        let contents = serde_json::to_string_pretty(assignments).map_err(|error| {
            format!(
                "Failed to serialize desktop assignments for {}: {error}",
                self.path().display()
            )
        })?;

        fs::write(self.path(), contents).map_err(|error| {
            format!(
                "Failed to write desktop assignments to {}: {error}",
                self.path().display()
            )
        })
    }

    fn clear_assignments(&self) -> Result<(), String> {
        match fs::remove_file(self.path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "Failed to clear desktop assignments at {}: {error}",
                self.path().display()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::desktop_state_path_from_env;

    #[test]
    fn desktop_state_path_returns_unavailable_when_env_is_missing() {
        let path = desktop_state_path_from_env(None, None);

        assert!(
            matches!(path, Err(reason) if reason.contains("XDG_CONFIG_HOME and HOME are unset"))
        );
    }

    #[test]
    fn desktop_state_path_returns_unavailable_for_relative_config_root() {
        let path = desktop_state_path_from_env(Some(PathBuf::from("relative-config")), None);

        assert!(matches!(path, Err(reason) if reason.contains("non-absolute config root")));
    }

    #[test]
    fn desktop_state_path_uses_absolute_config_root() {
        let path = desktop_state_path_from_env(Some(PathBuf::from("/tmp/config")), None).unwrap();

        assert_eq!(path, PathBuf::from("/tmp/config/wayvid/desktop-state.json"));
    }
}
