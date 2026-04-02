use lwe_library::SteamLibrary;

use crate::models::SettingsUpdateInput;
use crate::results::settings_persistence::{
    PersistedSettings, SettingsPersistenceLoad, SettingsPersistenceWrite,
};
use crate::services::autostart_service::{AutostartService, AutostartState};
use crate::services::settings_persistence_service::SettingsPersistenceService;

pub(crate) struct SettingsPageData {
    pub language: String,
    pub theme: String,
    pub launch_on_login: bool,
    pub launch_on_login_available: bool,
    pub steam_required: bool,
    pub steam_status_message: String,
    pub stale: bool,
}

pub struct SettingsService;

impl SettingsService {
    pub(crate) fn load_page() -> Result<SettingsPageData, String> {
        let persistence = match SettingsPersistenceService::for_user_path() {
            Ok(service) => service.load_settings(),
            Err(reason) => SettingsPersistenceLoad::Unavailable { reason },
        };

        let autostart = match AutostartService::for_user_path() {
            Ok(service) => match current_launch_command() {
                Ok(command) => service.status(&command_refs(&command)).state,
                Err(reason) => AutostartState::Unavailable { reason },
            },
            Err(reason) => AutostartState::Unavailable { reason },
        };

        Ok(Self::build_page_data(persistence, autostart))
    }

    pub(crate) fn update_settings(input: SettingsUpdateInput) -> Result<SettingsPageData, String> {
        let persistence = SettingsPersistenceService::for_user_path()?;
        let mut settings = match persistence.load_settings() {
            SettingsPersistenceLoad::Loaded(settings) => settings,
            SettingsPersistenceLoad::Unavailable { reason } => return Err(reason),
        };

        if let Some(language) = input.language {
            settings.language = language;
        }

        if let Some(theme) = input.theme {
            settings.theme = theme;
        }

        if let Some(launch_on_login) = input.launch_on_login {
            settings.launch_on_login = launch_on_login;
            Self::apply_launch_on_login(launch_on_login)?;
        }

        match persistence.save_settings(&settings) {
            SettingsPersistenceWrite::Saved => Self::load_page(),
            SettingsPersistenceWrite::Unavailable { reason } => Err(reason),
        }
    }

    fn apply_launch_on_login(enabled: bool) -> Result<(), String> {
        let autostart = AutostartService::for_user_path()?;
        let command = current_launch_command()?;
        let command_refs = command_refs(&command);

        if enabled {
            autostart.enable(&command_refs)
        } else {
            autostart.disable(&command_refs)
        }
    }

    fn build_page_data(
        persistence: SettingsPersistenceLoad,
        autostart: AutostartState,
    ) -> SettingsPageData {
        let (settings, persistence_available) = match persistence {
            SettingsPersistenceLoad::Loaded(settings) => (settings, true),
            SettingsPersistenceLoad::Unavailable { .. } => (PersistedSettings::default(), false),
        };

        let (launch_on_login, launch_on_login_available) = match autostart {
            AutostartState::Enabled => (true, true),
            AutostartState::Disabled => (false, true),
            AutostartState::Unavailable { .. } => (settings.launch_on_login, false),
        };

        let (steam_required, steam_status_message) = steam_status();

        SettingsPageData {
            language: settings.language,
            theme: settings.theme,
            launch_on_login,
            launch_on_login_available,
            steam_required,
            steam_status_message,
            stale: !persistence_available || !launch_on_login_available,
        }
    }

    #[cfg(test)]
    fn load_page_for_test(
        settings_path: std::path::PathBuf,
        config_root: std::path::PathBuf,
    ) -> Result<SettingsPageData, String> {
        let persistence = SettingsPersistenceService::for_test(settings_path).load_settings();
        let command = vec![test_launch_command()];
        let autostart = AutostartService::for_test(config_root)
            .status(&command_refs(&command))
            .state;

        Ok(Self::build_page_data(persistence, autostart))
    }

    #[cfg(test)]
    fn update_settings_for_test(
        settings_path: std::path::PathBuf,
        config_root: std::path::PathBuf,
        input: SettingsUpdateInput,
    ) -> Result<SettingsPageData, String> {
        let persistence = SettingsPersistenceService::for_test(settings_path.clone());
        let mut settings = match persistence.load_settings() {
            SettingsPersistenceLoad::Loaded(settings) => settings,
            SettingsPersistenceLoad::Unavailable { reason } => return Err(reason),
        };

        if let Some(language) = input.language {
            settings.language = language;
        }

        if let Some(theme) = input.theme {
            settings.theme = theme;
        }

        if let Some(launch_on_login) = input.launch_on_login {
            settings.launch_on_login = launch_on_login;
            let autostart = AutostartService::for_test(config_root.clone());
            let command = vec![test_launch_command()];
            let command_refs = command_refs(&command);

            if launch_on_login {
                autostart.enable(&command_refs)?;
            } else {
                autostart.disable(&command_refs)?;
            }
        }

        match persistence.save_settings(&settings) {
            SettingsPersistenceWrite::Saved => Self::load_page_for_test(settings_path, config_root),
            SettingsPersistenceWrite::Unavailable { reason } => Err(reason),
        }
    }
}

fn steam_status() -> (bool, String) {
    match SteamLibrary::try_discover() {
        Some(steam) if steam.has_wallpaper_engine() => (
            true,
            "Steam and Wallpaper Engine are available for Workshop features".to_string(),
        ),
        Some(_) => (
            true,
            "Steam is available, but Wallpaper Engine was not found for Workshop features"
                .to_string(),
        ),
        None => (
            true,
            "Steam was not detected. Steam is required for Workshop features".to_string(),
        ),
    }
}

fn current_launch_command() -> Result<Vec<String>, String> {
    let path = std::env::current_exe()
        .map_err(|error| format!("Unable to resolve current executable for autostart: {error}"))?;

    Ok(vec![path_to_command_part(&path)?])
}

#[cfg(test)]
fn test_launch_command() -> String {
    "/opt/lwe/bin/lwe".to_string()
}

fn path_to_command_part(path: &std::path::Path) -> Result<String, String> {
    path.to_str().map(str::to_string).ok_or_else(|| {
        format!(
            "Unable to use non-UTF-8 executable path {} for autostart",
            path.display()
        )
    })
}

fn command_refs(command: &[String]) -> Vec<&str> {
    command.iter().map(String::as_str).collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::models::SettingsUpdateInput;
    use crate::services::autostart_service::{AutostartService, AutostartState};

    use super::SettingsService;

    fn unique_test_path(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{unique}"))
    }

    #[test]
    fn settings_service_load_page_reflects_persisted_settings_and_autostart_state() {
        let settings_path = unique_test_path("settings-service").with_extension("toml");
        let config_root = unique_test_path("settings-service-config");

        std::fs::write(
            &settings_path,
            "language = \"en\"\ntheme = \"dark\"\nlaunch_on_login = false\n",
        )
        .unwrap();

        let autostart = AutostartService::for_test(config_root.clone());
        autostart.enable(&["/opt/lwe/bin/lwe"]).unwrap();

        let result = SettingsService::load_page_for_test(settings_path, config_root).unwrap();

        assert_eq!(result.language, "en");
        assert_eq!(result.theme, "dark");
        assert!(result.launch_on_login);
        assert!(result.launch_on_login_available);
        assert!(result.steam_required);
        assert!(!result.stale);
        assert!(result.steam_status_message.contains("Steam"));
    }

    #[test]
    fn settings_service_uses_persisted_launch_preference_when_autostart_is_unavailable() {
        let result = SettingsService::build_page_data(
            crate::results::settings_persistence::SettingsPersistenceLoad::Loaded(
                crate::results::settings_persistence::PersistedSettings {
                    language: "system".to_string(),
                    theme: "system".to_string(),
                    launch_on_login: true,
                },
            ),
            AutostartState::Unavailable {
                reason: "missing XDG config root".to_string(),
            },
        );

        assert!(result.launch_on_login);
        assert!(!result.launch_on_login_available);
        assert!(result.stale);
    }

    #[test]
    fn settings_service_update_persists_values_and_updates_autostart() {
        let settings_path = unique_test_path("settings-update").with_extension("toml");
        let config_root = unique_test_path("settings-update-config");

        let result = SettingsService::update_settings_for_test(
            settings_path.clone(),
            config_root.clone(),
            SettingsUpdateInput {
                language: Some("fr".to_string()),
                theme: Some("dark".to_string()),
                launch_on_login: Some(true),
            },
        )
        .unwrap();

        assert_eq!(result.language, "fr");
        assert_eq!(result.theme, "dark");
        assert!(result.launch_on_login);

        let contents = std::fs::read_to_string(settings_path).unwrap();
        assert!(contents.contains("language = \"fr\""));
        assert!(contents.contains("theme = \"dark\""));
        assert!(contents.contains("launch_on_login = true"));

        assert_eq!(
            AutostartService::for_test(config_root)
                .status(&["/opt/lwe/bin/lwe"])
                .state,
            AutostartState::Enabled
        );
    }
}
