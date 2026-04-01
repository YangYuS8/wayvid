use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const AUTOSTART_ENTRY_FILE_NAME: &str = "wayvid-lwe.desktop";
const AUTOSTART_ENTRY_NAME: &str = "LWE";
pub struct AutostartService;

pub struct ScopedAutostartService {
    config_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutostartStatus {
    pub state: AutostartState,
    pub entry_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutostartState {
    Enabled,
    Disabled,
    Unavailable { reason: String },
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

    pub fn status(&self, launch_command: &[&str]) -> AutostartStatus {
        let entry_path = self.entry_path();
        let state = match fs::read_to_string(&entry_path) {
            Ok(contents) => {
                if desktop_entry_is_active(&contents, launch_command) {
                    AutostartState::Enabled
                } else {
                    AutostartState::Disabled
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => AutostartState::Disabled,
            Err(error) => AutostartState::Unavailable {
                reason: format!(
                    "Failed to read autostart entry {}: {error}",
                    entry_path.display()
                ),
            },
        };

        AutostartStatus { state, entry_path }
    }

    pub fn enable(&self, launch_command: &[&str]) -> Result<(), String> {
        let entry_path = self.entry_path();

        if let Some(parent) = entry_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create autostart directory {}: {error}",
                    parent.display()
                )
            })?;
        }

        let contents = desktop_entry_contents(launch_command)?;

        fs::write(&entry_path, contents).map_err(|error| {
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

fn desktop_entry_contents(launch_command: &[&str]) -> Result<String, String> {
    let exec_value = desktop_entry_exec_value(launch_command)?;

    Ok(format!(
        "[Desktop Entry]\nType=Application\nName={AUTOSTART_ENTRY_NAME}\nExec={exec_value}\nTerminal=false\n"
    ))
}

fn desktop_entry_exec_value(launch_command: &[&str]) -> Result<String, String> {
    if launch_command.is_empty() {
        return Err(
            "Unable to encode autostart exec line from an empty launch command".to_string(),
        );
    }

    Ok(launch_command
        .iter()
        .map(|part| desktop_entry_exec_arg(part))
        .collect::<Vec<_>>()
        .join(" "))
}

fn desktop_entry_exec_arg(argument: &str) -> String {
    let needs_quotes = argument.is_empty()
        || argument
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\\' | '`' | '$'));

    let mut escaped = String::with_capacity(argument.len());

    for ch in argument.chars() {
        match ch {
            '%' => escaped.push_str("%%"),
            '"' | '\\' | '`' | '$' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }

    if !needs_quotes {
        return escaped;
    }

    format!("\"{escaped}\"")
}

fn desktop_entry_is_active(contents: &str, expected_launch_command: &[&str]) -> bool {
    let mut has_desktop_header = false;
    let mut has_application_type = false;
    let mut exec_matches = false;
    let mut hidden = false;

    for line in contents.lines().map(str::trim) {
        match line {
            "[Desktop Entry]" => has_desktop_header = true,
            "Type=Application" => has_application_type = true,
            "Hidden=true" => hidden = true,
            _ if line.starts_with("Exec=") => {
                exec_matches =
                    desktop_entry_exec_matches(&line["Exec=".len()..], expected_launch_command)
            }
            _ => {}
        }
    }

    has_desktop_header && has_application_type && exec_matches && !hidden
}

fn desktop_entry_exec_matches(exec_value: &str, expected_launch_command: &[&str]) -> bool {
    match parse_desktop_entry_exec_value(exec_value) {
        Ok(parsed_exec) => {
            parsed_exec
                == expected_launch_command
                    .iter()
                    .map(|part| part.to_string())
                    .collect::<Vec<_>>()
        }
        Err(_) => false,
    }
}

fn parse_desktop_entry_exec_value(exec_value: &str) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = exec_value.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '\\' => match chars.next() {
                Some(next) => current.push(next),
                None => return Err("Autostart Exec line ends with a trailing escape".to_string()),
            },
            '%' => match chars.next() {
                Some('%') => current.push('%'),
                Some(field_code) => {
                    return Err(format!(
                        "Autostart Exec line contains field code %{field_code}"
                    ))
                }
                None => return Err("Autostart Exec line ends with a trailing %".to_string()),
            },
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err("Autostart Exec line has an unclosed quote".to_string());
    }

    if !current.is_empty() {
        parts.push(current);
    }

    Ok(parts)
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
        let launch_command = [
            "/opt/lwe/bin/lwe",
            "--profile",
            "My Project",
            "say \"hi\"",
            "100%",
        ];

        assert_eq!(
            service.entry_path(),
            config_root.join("autostart").join("wayvid-lwe.desktop")
        );
        assert_eq!(
            service.status(&launch_command),
            super::AutostartStatus {
                state: super::AutostartState::Disabled,
                entry_path: config_root.join("autostart").join("wayvid-lwe.desktop"),
            }
        );

        service.enable(&launch_command).unwrap();

        let contents = std::fs::read_to_string(service.entry_path()).unwrap();
        assert!(contents.contains("[Desktop Entry]"));
        assert!(contents.contains("Type=Application"));
        assert!(contents.contains("Name=LWE"));
        assert!(contents
            .contains("Exec=/opt/lwe/bin/lwe --profile \"My Project\" \"say \\\"hi\\\"\" 100%%"));
        assert_eq!(
            service.status(&launch_command).state,
            super::AutostartState::Enabled
        );

        service.disable().unwrap();

        assert!(!service.entry_path().exists());
        assert_eq!(
            service.status(&launch_command).state,
            super::AutostartState::Disabled
        );
    }

    #[test]
    fn autostart_service_treats_equivalent_exec_with_different_name_as_enabled() {
        let config_root = test_config_root();
        let service = AutostartService::for_test(config_root);

        std::fs::create_dir_all(service.entry_path().parent().unwrap()).unwrap();
        std::fs::write(
            service.entry_path(),
            "[Desktop Entry]\nType=Application\nName=Launch Wallpaper Engine\nExec=\"/opt/lwe/bin/lwe\" --profile \"My Project\" \"say \\\"hi\\\"\" 100%%\nTerminal=false\n",
        )
        .unwrap();

        assert_eq!(
            service
                .status(&[
                    "/opt/lwe/bin/lwe",
                    "--profile",
                    "My Project",
                    "say \"hi\"",
                    "100%",
                ])
                .state,
            super::AutostartState::Enabled
        );
    }

    #[test]
    fn autostart_service_treats_stale_desktop_entry_as_disabled() {
        let config_root = test_config_root();
        let service = AutostartService::for_test(config_root);

        std::fs::create_dir_all(service.entry_path().parent().unwrap()).unwrap();
        std::fs::write(
            service.entry_path(),
            "[Desktop Entry]\nType=Application\nName=LWE\nTerminal=false\n",
        )
        .unwrap();

        assert_eq!(
            service
                .status(&[
                    "/opt/lwe/bin/lwe",
                    "--profile",
                    "My Project",
                    "say \"hi\"",
                    "100%",
                ])
                .state,
            super::AutostartState::Disabled
        );
    }

    #[test]
    fn autostart_service_treats_hidden_desktop_entry_as_disabled() {
        let config_root = test_config_root();
        let service = AutostartService::for_test(config_root);

        std::fs::create_dir_all(service.entry_path().parent().unwrap()).unwrap();
        std::fs::write(
            service.entry_path(),
            "[Desktop Entry]\nType=Application\nName=LWE\nExec=/opt/lwe/bin/lwe\nHidden=true\nTerminal=false\n",
        )
        .unwrap();

        assert_eq!(
            service
                .status(&[
                    "/opt/lwe/bin/lwe",
                    "--profile",
                    "My Project",
                    "say \"hi\"",
                    "100%",
                ])
                .state,
            super::AutostartState::Disabled
        );
    }

    #[test]
    fn autostart_service_treats_wrong_exec_target_as_disabled() {
        let config_root = test_config_root();
        let service = AutostartService::for_test(config_root);

        std::fs::create_dir_all(service.entry_path().parent().unwrap()).unwrap();
        std::fs::write(
            service.entry_path(),
            "[Desktop Entry]\nType=Application\nName=LWE\nExec=/usr/bin/other-app\nTerminal=false\n",
        )
        .unwrap();

        assert_eq!(
            service
                .status(&[
                    "/opt/lwe/bin/lwe",
                    "--profile",
                    "My Project",
                    "say \"hi\"",
                    "100%",
                ])
                .state,
            super::AutostartState::Disabled
        );
    }

    #[test]
    fn autostart_service_reports_unavailable_when_entry_cannot_be_read() {
        let config_root = test_config_root();
        let service = AutostartService::for_test(config_root);

        std::fs::create_dir_all(service.entry_path()).unwrap();

        assert!(matches!(
            service
                .status(&[
                    "/opt/lwe/bin/lwe",
                    "--profile",
                    "My Project",
                    "say \"hi\"",
                    "100%",
                ])
                .state,
            super::AutostartState::Unavailable { .. }
        ));
    }

    #[test]
    fn autostart_service_falls_back_to_home_config_root() {
        let root =
            autostart_config_root_from_env(None, Some(std::path::PathBuf::from("/tmp/home")))
                .unwrap();

        assert_eq!(root, std::path::PathBuf::from("/tmp/home/.config"));
    }
}
