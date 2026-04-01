use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const AUTOSTART_ENTRY_FILE_NAME: &str = "wayvid-lwe.desktop";
const AUTOSTART_ENTRY_NAME: &str = "LWE";
const AUTOSTART_ENTRY_EXEC: &str = "lwe";

pub struct AutostartService;

pub struct ScopedAutostartService {
    config_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutostartStatus {
    pub enabled: bool,
    pub entry_path: PathBuf,
}

impl AutostartService {
    pub fn for_path(config_root: PathBuf) -> ScopedAutostartService {
        ScopedAutostartService { config_root }
    }

    pub fn for_test(config_root: PathBuf) -> ScopedAutostartService {
        Self::for_path(config_root)
    }

    #[allow(dead_code)]
    pub fn for_user_path() -> Result<ScopedAutostartService, String> {
        autostart_config_root().map(Self::for_path)
    }
}

impl ScopedAutostartService {
    pub fn entry_path(&self) -> PathBuf {
        self.config_root
            .join("autostart")
            .join(AUTOSTART_ENTRY_FILE_NAME)
    }

    pub fn status(&self) -> AutostartStatus {
        let entry_path = self.entry_path();

        AutostartStatus {
            enabled: entry_path.is_file(),
            entry_path,
        }
    }

    pub fn enable(&self) -> Result<(), String> {
        let entry_path = self.entry_path();

        if let Some(parent) = entry_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create autostart directory {}: {error}",
                    parent.display()
                )
            })?;
        }

        fs::write(&entry_path, desktop_entry_contents()).map_err(|error| {
            format!(
                "Failed to write autostart entry {}: {error}",
                entry_path.display()
            )
        })
    }

    pub fn disable(&self) -> Result<(), String> {
        let entry_path = self.entry_path();

        match fs::remove_file(&entry_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "Failed to remove autostart entry {}: {error}",
                entry_path.display()
            )),
        }
    }
}

fn desktop_entry_contents() -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={AUTOSTART_ENTRY_NAME}\nExec={AUTOSTART_ENTRY_EXEC}\nTerminal=false\n"
    )
}

fn autostart_config_root() -> Result<PathBuf, String> {
    autostart_config_root_from_env(
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
        std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
    )
}

fn autostart_config_root_from_env(
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let config_root = xdg_config_home.or_else(|| home.map(|home| home.join(".config")));

    match config_root {
        Some(path) if path.is_absolute() => Ok(path),
        Some(path) => Err(format!(
            "Unable to resolve autostart path from non-absolute config root {}",
            path.display()
        )),
        None => Err(
            "Unable to resolve autostart path because XDG_CONFIG_HOME and HOME are unset"
                .to_string(),
        ),
    }
}

#[allow(dead_code)]
fn autostart_dir_for(config_root: &Path) -> PathBuf {
    config_root.join("autostart")
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{autostart_config_root_from_env, AutostartService};

    fn test_config_root() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("autostart-service-{unique}"))
    }

    #[test]
    fn autostart_service_uses_graphical_session_desktop_entry_under_config_autostart() {
        let config_root = test_config_root();
        let service = AutostartService::for_test(config_root.clone());

        assert_eq!(
            service.entry_path(),
            config_root.join("autostart").join("wayvid-lwe.desktop")
        );
        assert_eq!(
            service.status(),
            super::AutostartStatus {
                enabled: false,
                entry_path: config_root.join("autostart").join("wayvid-lwe.desktop"),
            }
        );

        service.enable().unwrap();

        let contents = std::fs::read_to_string(service.entry_path()).unwrap();
        assert!(contents.contains("[Desktop Entry]"));
        assert!(contents.contains("Type=Application"));
        assert!(contents.contains("Name=LWE"));
        assert!(contents.contains("Exec=lwe"));
        assert!(service.status().enabled);

        service.disable().unwrap();

        assert!(!service.entry_path().exists());
        assert!(!service.status().enabled);
    }

    #[test]
    fn autostart_service_falls_back_to_home_config_root() {
        let root =
            autostart_config_root_from_env(None, Some(std::path::PathBuf::from("/tmp/home")))
                .unwrap();

        assert_eq!(root, std::path::PathBuf::from("/tmp/home/.config"));
    }
}
