//! Steam Workshop downloader
//!
//! This module handles downloading wallpapers from Steam Workshop using
//! the Steam Web API and SteamCMD-like download mechanisms.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Steam Workshop API endpoint
const STEAM_API_BASE: &str = "https://api.steampowered.com";

/// Workshop item metadata from Steam API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopItemDetails {
    /// Workshop item ID
    pub publishedfileid: String,
    /// Item title
    pub title: String,
    /// Item description
    #[serde(default)]
    pub description: String,
    /// Creator Steam ID
    #[serde(default)]
    pub creator: String,
    /// File size in bytes
    #[serde(default)]
    pub file_size: String,
    /// Preview image URL
    #[serde(default)]
    pub preview_url: String,
    /// Download URL (if available)
    #[serde(default)]
    pub file_url: String,
    /// Number of subscriptions
    #[serde(default)]
    pub subscriptions: u64,
    /// Number of favorites
    #[serde(default)]
    pub favorited: u64,
    /// Tags
    #[serde(default)]
    pub tags: Vec<WorkshopTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopTag {
    pub tag: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    response: ApiResponseData,
}

#[derive(Debug, Deserialize)]
struct ApiResponseData {
    #[serde(default)]
    publishedfiledetails: Vec<WorkshopItemDetails>,
}

/// Workshop downloader
pub struct WorkshopDownloader {
    /// Cache directory for downloads
    cache_dir: PathBuf,
    /// HTTP client
    client: reqwest::blocking::Client,
}

impl WorkshopDownloader {
    /// Create new downloader
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("wayvid")
            .join("workshop");

        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        let client = reqwest::blocking::Client::builder()
            .user_agent("wayvid/0.4.1")
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self { cache_dir, client })
    }

    /// Get item details from Steam API
    pub fn get_item_details(&self, item_id: u64) -> Result<WorkshopItemDetails> {
        tracing::info!("Fetching details for Workshop item {}", item_id);

        let url = format!(
            "{}/ISteamRemoteStorage/GetPublishedFileDetails/v1/",
            STEAM_API_BASE
        );

        let params = [
            ("itemcount", "1"),
            ("publishedfileids[0]", &item_id.to_string()),
        ];

        let response = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .context("Failed to fetch item details")?;

        if !response.status().is_success() {
            anyhow::bail!("Steam API returned error: {}", response.status());
        }

        let api_response: ApiResponse = response.json().context("Failed to parse API response")?;

        let details = api_response
            .response
            .publishedfiledetails
            .into_iter()
            .next()
            .context("Item not found")?;

        Ok(details)
    }

    /// Search Workshop items
    ///
    /// Note: Steam's public Web API has limitations for search functionality.
    /// This method provides item details lookup if you have specific IDs.
    /// For browsing, users should use Steam Workshop directly:
    /// https://steamcommunity.com/app/431960/workshop/
    pub fn search(&self, _app_id: u32, _query: &str, _page: u32) -> Result<Vec<WorkshopItemDetails>> {
        // Steam's public API doesn't provide good search capabilities without API key
        // Users should:
        // 1. Browse Workshop in Steam client or web browser
        // 2. Subscribe to items (they'll appear in 'wayvid workshop list')
        // 3. Or use item IDs directly: 'wayvid workshop download <id>'
        
        tracing::warn!(
            "Steam Web API search requires authentication. \
             Please browse Workshop at: https://steamcommunity.com/app/431960/workshop/"
        );

        Ok(Vec::new())
    }

    /// Download Workshop item
    pub fn download(&self, item_id: u64) -> Result<PathBuf> {
        let details = self.get_item_details(item_id)?;

        // Check if already downloaded
        let item_dir = self.cache_dir.join(item_id.to_string());
        if item_dir.exists() {
            tracing::info!("Item {} already cached at {:?}", item_id, item_dir);
            return Ok(item_dir);
        }

        tracing::info!("Downloading Workshop item: {}", details.title);

        // Note: Direct download URLs are not always available from Steam API
        // This is a limitation - we may need to use SteamCMD or require manual subscription
        if details.file_url.is_empty() {
            anyhow::bail!(
                "Direct download not available for this item.\n\
                 Please subscribe to the item in Steam and use 'wayvid workshop list' to access it."
            );
        }

        // Download the file
        tracing::info!("Downloading from: {}", details.file_url);
        let response = self
            .client
            .get(&details.file_url)
            .send()
            .context("Failed to download item")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        // Save to temporary file
        let temp_file = self.cache_dir.join(format!("{}.tmp", item_id));
        let mut file = fs::File::create(&temp_file).context("Failed to create download file")?;

        let content = response.bytes().context("Failed to read response")?;
        file.write_all(&content)
            .context("Failed to write download")?;
        drop(file);

        // Extract if it's a zip
        if self.is_zip_file(&temp_file)? {
            tracing::info!("Extracting archive...");
            self.extract_zip(&temp_file, &item_dir)?;
            fs::remove_file(&temp_file)?;
        } else {
            // Not a zip, just rename
            fs::create_dir_all(&item_dir)?;
            let target = item_dir.join("wallpaper");
            fs::rename(&temp_file, &target)?;
        }

        // Save metadata
        self.save_metadata(&item_dir, &details)?;

        tracing::info!("✅ Downloaded to: {:?}", item_dir);
        Ok(item_dir)
    }

    /// Check if file is a ZIP archive
    fn is_zip_file(&self, path: &Path) -> Result<bool> {
        let file = fs::File::open(path)?;
        Ok(zip::ZipArchive::new(file).is_ok())
    }

    /// Extract ZIP archive
    fn extract_zip(&self, zip_path: &Path, target_dir: &Path) -> Result<()> {
        let file = fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        fs::create_dir_all(target_dir)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(())
    }

    /// Save item metadata
    fn save_metadata(&self, item_dir: &Path, details: &WorkshopItemDetails) -> Result<()> {
        let metadata_path = item_dir.join("metadata.json");
        let json = serde_json::to_string_pretty(details)?;
        fs::write(metadata_path, json)?;
        Ok(())
    }

    /// Get cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// List cached items
    pub fn list_cached(&self) -> Result<Vec<u64>> {
        let mut items = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(items);
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(id) = entry.file_name().to_string_lossy().parse::<u64>() {
                    items.push(id);
                }
            }
        }

        items.sort_unstable();
        Ok(items)
    }

    /// Clear cache for specific item
    pub fn clear_cache(&self, item_id: u64) -> Result<()> {
        let item_dir = self.cache_dir.join(item_id.to_string());
        if item_dir.exists() {
            fs::remove_dir_all(&item_dir)?;
            tracing::info!("Cleared cache for item {}", item_id);
        }
        Ok(())
    }

    /// Clear all cache
    pub fn clear_all_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
            tracing::info!("Cleared all Workshop cache");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let downloader = WorkshopDownloader::new();
        assert!(downloader.is_ok());
    }

    #[test]
    #[ignore] // Requires network
    fn test_get_item_details() {
        let downloader = WorkshopDownloader::new().unwrap();
        // Test with a known public item (replace with actual ID if needed)
        let result = downloader.get_item_details(123456789);
        // This might fail if item doesn't exist, but tests API call structure
        println!("{:?}", result);
    }
}
