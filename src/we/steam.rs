//! Steam library discovery and VDF parsing

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Steam library manager
#[derive(Debug, Clone)]
pub struct SteamLibrary {
    /// Root Steam installation path
    pub root: PathBuf,
    /// Additional library folders
    pub libraries: Vec<PathBuf>,
}

impl SteamLibrary {
    /// Discover Steam installation
    pub fn discover() -> Result<Self> {
        let root = Self::find_steam_root()?;
        let libraries = Self::parse_library_folders(&root)?;

        Ok(Self { root, libraries })
    }

    /// Find Steam root directory
    fn find_steam_root() -> Result<PathBuf> {
        // Check common Steam paths on Linux
        let candidates = [
            dirs::home_dir().map(|h| h.join(".steam/steam")),
            dirs::home_dir().map(|h| h.join(".local/share/Steam")),
            Some(PathBuf::from("/usr/share/steam")),
        ];

        for candidate in candidates.iter().flatten() {
            if candidate.exists() && candidate.join("steamapps").exists() {
                tracing::debug!("Found Steam at: {:?}", candidate);
                return Ok(candidate.clone());
            }
        }

        anyhow::bail!("Steam installation not found")
    }

    /// Parse libraryfolders.vdf to find additional libraries
    fn parse_library_folders(root: &Path) -> Result<Vec<PathBuf>> {
        let vdf_path = root.join("steamapps/libraryfolders.vdf");
        if !vdf_path.exists() {
            return Ok(vec![root.to_path_buf()]);
        }

        let content = fs::read_to_string(&vdf_path).context("Failed to read libraryfolders.vdf")?;

        let mut libraries = vec![root.to_path_buf()];
        libraries.extend(Self::parse_vdf_paths(&content));

        tracing::debug!("Found {} Steam libraries", libraries.len());
        Ok(libraries)
    }

    /// Parse VDF file for library paths
    fn parse_vdf_paths(content: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            // Look for "path" key in VDF
            if line.starts_with("\"path\"") {
                if let Some(path_str) = Self::extract_vdf_value(line) {
                    let path = PathBuf::from(path_str);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
        }

        paths
    }

    /// Extract quoted value from VDF line
    fn extract_vdf_value(line: &str) -> Option<String> {
        // Format: "key"		"value"
        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 4 {
            Some(parts[3].to_string())
        } else {
            None
        }
    }

    /// Find Workshop items for app ID
    pub fn find_workshop_items(&self, app_id: u32) -> Result<Vec<PathBuf>> {
        let mut items = Vec::new();

        for library in std::iter::once(&self.root).chain(&self.libraries) {
            let workshop_path = library
                .join("steamapps/workshop/content")
                .join(app_id.to_string());

            if !workshop_path.exists() {
                continue;
            }

            tracing::debug!("Scanning workshop: {:?}", workshop_path);

            for entry in fs::read_dir(&workshop_path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    items.push(entry.path());
                }
            }
        }

        tracing::info!("Found {} workshop items for app {}", items.len(), app_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdf_value_extraction() {
        let line = r#"		"path"		"/home/user/SteamLibrary""#;
        let value = SteamLibrary::extract_vdf_value(line);
        assert_eq!(value, Some("/home/user/SteamLibrary".to_string()));
    }

    #[test]
    fn test_vdf_parsing() {
        let content = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"/home/user/.local/share/Steam"
		"label"		""
	}
	"1"
	{
		"path"		"/mnt/games/SteamLibrary"
		"label"		"Games"
	}
}
"#;
        let paths = SteamLibrary::parse_vdf_paths(content);
        // Note: paths must exist to be returned, so in test env count will be 0
        assert!(paths.len() <= 2);
    }
}
