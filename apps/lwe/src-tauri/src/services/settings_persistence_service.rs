use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::results::settings_persistence::{
    PersistedSettings, SettingsPersistenceLoad, SettingsPersistenceWrite,
};

pub struct SettingsPersistenceService;

pub struct ScopedSettingsPersistenceService {
    path: PathBuf,
}

impl SettingsPersistenceService {
    pub fn for_path(path: PathBuf) -> ScopedSettingsPersistenceService {
        ScopedSettingsPersistenceService { path }
    }

    #[allow(dead_code)]
    pub fn for_user_path() -> Result<ScopedSettingsPersistenceService, String> {
        settings_path().map(Self::for_path)
    }

    pub fn for_test(path: PathBuf) -> ScopedSettingsPersistenceService {
        Self::for_path(path)
    }
}

impl ScopedSettingsPersistenceService {
    pub fn load_settings(&self) -> SettingsPersistenceLoad {
        match fs::read_to_string(&self.path) {
            Ok(contents) => match parse_settings(&contents) {
                Ok(settings) => SettingsPersistenceLoad::Loaded(settings),
                Err(reason) => SettingsPersistenceLoad::Unavailable {
                    reason: format!(
                        "Failed to parse settings from {}: {reason}",
                        self.path.display()
                    ),
                },
            },
            Err(error) if error.kind() == ErrorKind::NotFound => {
                SettingsPersistenceLoad::Loaded(PersistedSettings::default())
            }
            Err(error) => SettingsPersistenceLoad::Unavailable {
                reason: format!(
                    "Failed to read settings from {}: {error}",
                    self.path.display()
                ),
            },
        }
    }

    pub fn save_settings(&self, settings: &PersistedSettings) -> SettingsPersistenceWrite {
        if let Some(parent) = self.path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                return SettingsPersistenceWrite::Unavailable {
                    reason: format!(
                        "Failed to create settings directory {}: {error}",
                        parent.display()
                    ),
                };
            }
        }

        let contents = serialize_settings(settings);

        match fs::write(&self.path, contents) {
            Ok(()) => SettingsPersistenceWrite::Saved,
            Err(error) => SettingsPersistenceWrite::Unavailable {
                reason: format!(
                    "Failed to write settings to {}: {error}",
                    self.path.display()
                ),
            },
        }
    }
}

fn serialize_settings(settings: &PersistedSettings) -> String {
    format!(
        "language = {}\ntheme = {}\nlaunch_on_login = {}\n",
        quote_toml_string(&settings.language),
        quote_toml_string(&settings.theme),
        settings.launch_on_login
    )
}

fn quote_toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn parse_settings(contents: &str) -> Result<PersistedSettings, String> {
    let mut settings = PersistedSettings::default();

    for raw_line in contents.lines() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("invalid TOML line `{line}`"));
        };

        match key.trim() {
            "language" => settings.language = parse_toml_string(value.trim())?,
            "theme" => settings.theme = parse_toml_string(value.trim())?,
            "launch_on_login" => settings.launch_on_login = parse_toml_bool(value.trim())?,
            _ => {}
        }
    }

    Ok(settings)
}

fn parse_toml_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("expected boolean, found `{value}`")),
    }
}

fn parse_toml_string(value: &str) -> Result<String, String> {
    if !(value.starts_with('"') && value.ends_with('"')) {
        return Err(format!("expected quoted string, found `{value}`"));
    }

    let mut parsed = String::new();
    let mut chars = value[1..value.len() - 1].chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            parsed.push(ch);
            continue;
        }

        match chars.next() {
            Some('"') => parsed.push('"'),
            Some('\\') => parsed.push('\\'),
            Some(other) => return Err(format!("unsupported string escape `\\{other}`")),
            None => return Err("unterminated string escape".to_string()),
        }
    }

    Ok(parsed)
}

#[allow(dead_code)]
fn settings_path_from_env(
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let base = xdg_config_home.or_else(|| home.map(|home| home.join(".config")));

    match base {
        Some(path) if path.is_absolute() => Ok(path.join("wayvid").join("settings.toml")),
        Some(path) => Err(format!(
            "Unable to resolve settings path from non-absolute config root {}",
            path.display()
        )),
        None => Err(
            "Unable to resolve settings path because XDG_CONFIG_HOME and HOME are unset"
                .to_string(),
        ),
    }
}

#[allow(dead_code)]
fn settings_path() -> Result<PathBuf, String> {
    settings_path_from_env(
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
        std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
    )
}

impl ScopedSettingsPersistenceService {
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::results::settings_persistence::{PersistedSettings, SettingsPersistenceLoad};

    use super::SettingsPersistenceService;

    fn test_settings_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("settings-persistence-service-{unique}.toml"))
    }

    #[test]
    fn settings_persistence_returns_defaults_when_file_is_missing() {
        let path = test_settings_path();
        let service = SettingsPersistenceService::for_test(path);

        let loaded = service.load_settings();

        assert_eq!(
            loaded,
            SettingsPersistenceLoad::Loaded(PersistedSettings::default())
        );
    }

    #[test]
    fn settings_persistence_round_trips_mvp_settings_through_toml_file() {
        let path = test_settings_path();
        let service = SettingsPersistenceService::for_test(path.clone());
        let settings = PersistedSettings {
            language: "en".to_string(),
            theme: "dark".to_string(),
            launch_on_login: true,
        };

        assert!(matches!(
            service.save_settings(&settings),
            crate::results::settings_persistence::SettingsPersistenceWrite::Saved
        ));

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("language = \"en\""));
        assert!(contents.contains("theme = \"dark\""));
        assert!(contents.contains("launch_on_login = true"));

        let loaded = service.load_settings();

        assert_eq!(loaded, SettingsPersistenceLoad::Loaded(settings));
    }
}
