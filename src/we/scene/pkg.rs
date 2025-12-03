//! PKG file parser for Wallpaper Engine packed assets
//!
//! Wallpaper Engine uses .pkg files to bundle scene assets.
//! This module provides functionality to extract files from these packages.

use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use tracing::{debug, info, warn};

/// PKG file magic number
const PKG_MAGIC: &[u8; 8] = b"PKGV0023";

/// Entry in a PKG file
#[derive(Debug, Clone)]
pub struct PkgEntry {
    /// File name/path within the package
    pub name: String,
    /// Offset in the PKG file where data starts
    pub offset: u64,
    /// Size of the file data
    pub size: u64,
}

/// PKG file reader for Wallpaper Engine packages
pub struct PkgReader {
    /// Path to the PKG file
    path: std::path::PathBuf,
    /// File entries in the package
    entries: HashMap<String, PkgEntry>,
    /// Header size (start of data section)
    data_offset: u64,
}

impl PkgReader {
    /// Open a PKG file and read its directory
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(
            File::open(path).with_context(|| format!("Failed to open PKG: {:?}", path))?,
        );

        // Read and verify header
        let mut header = [0u8; 4];
        file.read_exact(&mut header)?;
        let _unknown = u32::from_le_bytes(header);

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;

        if &magic != PKG_MAGIC {
            return Err(anyhow!(
                "Invalid PKG magic: expected {:?}, got {:?}",
                PKG_MAGIC,
                magic
            ));
        }

        // Read entry count
        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let entry_count = u32::from_le_bytes(count_bytes);

        info!("PKG file has {} entries", entry_count);

        // Read all entries
        let mut entries = HashMap::new();
        let mut current_pos: u64 = 12 + 4; // After header + count

        for i in 0..entry_count {
            let entry = Self::read_entry(&mut file, &mut current_pos)?;
            debug!("PKG entry {}: {} (offset={}, size={})", i, entry.name, entry.offset, entry.size);
            entries.insert(entry.name.clone(), entry);
        }

        let data_offset = current_pos;
        debug!("PKG data section starts at offset {}", data_offset);

        Ok(Self {
            path: path.to_path_buf(),
            entries,
            data_offset,
        })
    }

    /// Read a single entry from the PKG directory
    fn read_entry(file: &mut BufReader<File>, pos: &mut u64) -> Result<PkgEntry> {
        // Format: name_len (4 bytes) -> name (variable) -> offset (4 bytes) -> size (4 bytes)
        
        // Read name length (4 bytes)
        let mut name_len_bytes = [0u8; 4];
        file.read_exact(&mut name_len_bytes)?;
        let name_len = u32::from_le_bytes(name_len_bytes) as usize;
        *pos += 4;

        // Read name
        let mut name_bytes = vec![0u8; name_len];
        file.read_exact(&mut name_bytes)?;
        *pos += name_len as u64;

        // Name may be null-terminated
        let name = String::from_utf8_lossy(&name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read offset (4 bytes)
        let mut offset_bytes = [0u8; 4];
        file.read_exact(&mut offset_bytes)?;
        let offset = u32::from_le_bytes(offset_bytes) as u64;
        *pos += 4;

        // Read size (4 bytes)
        let mut size_bytes = [0u8; 4];
        file.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes) as u64;
        *pos += 4;

        Ok(PkgEntry { name, offset, size })
    }

    /// List all files in the package
    pub fn list_files(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a file exists in the package
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Read a file from the package
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .entries
            .get(name)
            .ok_or_else(|| anyhow!("File not found in PKG: {}", name))?;

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.data_offset + entry.offset))?;

        let mut data = vec![0u8; entry.size as usize];
        file.read_exact(&mut data)?;

        debug!("Read {} bytes for '{}'", data.len(), name);
        Ok(data)
    }

    /// Read a file as string
    pub fn read_file_string(&self, name: &str) -> Result<String> {
        let data = self.read_file(name)?;
        String::from_utf8(data).with_context(|| format!("File is not valid UTF-8: {}", name))
    }

    /// Get entry info for a file
    pub fn get_entry(&self, name: &str) -> Option<&PkgEntry> {
        self.entries.get(name)
    }

    /// Get the number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Extract a file to disk
    pub fn extract_file(&self, name: &str, dest: &Path) -> Result<()> {
        let data = self.read_file(name)?;

        // Create parent directories if needed
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(dest, &data)?;
        debug!("Extracted '{}' to {:?}", name, dest);
        Ok(())
    }

    /// Extract all files to a directory
    pub fn extract_all(&self, dest_dir: &Path) -> Result<usize> {
        std::fs::create_dir_all(dest_dir)?;

        let mut count = 0;
        for name in self.entries.keys() {
            let dest_path = dest_dir.join(name);
            if let Err(e) = self.extract_file(name, &dest_path) {
                warn!("Failed to extract '{}': {}", name, e);
            } else {
                count += 1;
            }
        }

        info!("Extracted {} files to {:?}", count, dest_dir);
        Ok(count)
    }
}

/// Container that can read from PKG files or filesystem
pub struct SceneContainer {
    /// PKG reader if scene uses a package
    pkg: Option<PkgReader>,
    /// Base path for filesystem access
    base_path: std::path::PathBuf,
}

impl SceneContainer {
    /// Create a new container for a scene directory
    pub fn new(project_dir: &Path) -> Result<Self> {
        let pkg_path = project_dir.join("scene.pkg");

        let pkg = if pkg_path.exists() {
            info!("Loading scene from PKG: {:?}", pkg_path);
            Some(PkgReader::open(&pkg_path)?)
        } else {
            debug!("No scene.pkg found, using filesystem");
            None
        };

        Ok(Self {
            pkg,
            base_path: project_dir.to_path_buf(),
        })
    }

    /// Check if using a PKG file
    pub fn uses_pkg(&self) -> bool {
        self.pkg.is_some()
    }

    /// Read a file (from PKG or filesystem)
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        if let Some(ref pkg) = self.pkg {
            if pkg.contains(name) {
                return pkg.read_file(name);
            }
        }

        // Fall back to filesystem
        let path = self.base_path.join(name);
        std::fs::read(&path).with_context(|| format!("Failed to read file: {:?}", path))
    }

    /// Read a file as string
    pub fn read_file_string(&self, name: &str) -> Result<String> {
        let data = self.read_file(name)?;
        String::from_utf8(data).with_context(|| format!("File is not valid UTF-8: {}", name))
    }

    /// Check if a file exists
    pub fn exists(&self, name: &str) -> bool {
        if let Some(ref pkg) = self.pkg {
            if pkg.contains(name) {
                return true;
            }
        }
        self.base_path.join(name).exists()
    }

    /// Get base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// List all available files
    pub fn list_files(&self) -> Vec<String> {
        let mut files = Vec::new();

        if let Some(ref pkg) = self.pkg {
            files.extend(pkg.list_files().iter().map(|s| s.to_string()));
        }

        files
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pkg_entry_name_parsing() {
        // Test that null terminator is properly removed
        let name_with_null = "test.json\0";
        let cleaned = name_with_null.trim_end_matches('\0');
        assert_eq!(cleaned, "test.json");
    }

    #[test]
    fn test_pkg_reading_real_file() {
        // Try to read a real PKG file if available
        let home = std::env::var("HOME").unwrap_or_default();
        let pkg_path = std::path::Path::new(&home)
            .join(".steam/steam/steamapps/workshop/content/431960/3578699777/scene.pkg");

        if !pkg_path.exists() {
            println!("Skipping test: PKG file not found at {:?}", pkg_path);
            return;
        }

        let pkg = super::PkgReader::open(&pkg_path).expect("Failed to open PKG");
        
        // Should have entries
        assert!(pkg.entry_count() > 0, "PKG should have entries");
        
        // Should contain scene.json
        assert!(pkg.contains("scene.json"), "PKG should contain scene.json");
        
        // Should be able to read scene.json
        let content = pkg.read_file_string("scene.json").expect("Failed to read scene.json");
        assert!(content.contains("objects") || content.contains("camera"), 
                "scene.json should contain scene data");
        
        println!("PKG contains {} files", pkg.entry_count());
        println!("scene.json is {} bytes", content.len());
    }
}
