use crate::we::types::WeProject;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Detect if a directory contains a Wallpaper Engine project
pub fn detect_we_project(path: &Path) -> Result<PathBuf> {
    let project_file = path.join("project.json");

    if !project_file.exists() {
        return Err(anyhow!(
            "No project.json found in {}. Not a Wallpaper Engine project.",
            path.display()
        ));
    }

    debug!("Found project.json at: {}", project_file.display());
    Ok(project_file)
}

/// Parse a Wallpaper Engine project.json file
pub fn parse_we_project(project_file: &Path) -> Result<(WeProject, PathBuf)> {
    info!("📂 Parsing Wallpaper Engine project: {}", project_file.display());

    // Read and parse JSON
    let content = fs::read_to_string(project_file)
        .with_context(|| format!("Failed to read {}", project_file.display()))?;

    let project: WeProject = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from {}", project_file.display()))?;

    // Validate project type
    if project.project_type != "video" {
        return Err(anyhow!(
            "Unsupported project type: '{}'. Only 'video' wallpapers are supported.",
            project.project_type
        ));
    }

    info!("✅ Project type: video");
    info!("📝 Title: {}", project.title);

    if let Some(ref workshop_id) = project.workshopid {
        info!("🔗 Workshop ID: {}", workshop_id);
    }

    // Resolve video file path
    let project_dir = project_file
        .parent()
        .ok_or_else(|| anyhow!("Invalid project file path"))?;

    let video_path = resolve_video_path(project_dir, &project.file)?;

    info!("🎬 Video file: {}", video_path.display());

    Ok((project, video_path))
}

/// Resolve video file path relative to project directory
fn resolve_video_path(project_dir: &Path, file: &str) -> Result<PathBuf> {
    let video_path = project_dir.join(file);

    if !video_path.exists() {
        return Err(anyhow!(
            "Video file not found: {}. Ensure the project directory is complete.",
            video_path.display()
        ));
    }

    // Check if it's a readable file
    if !video_path.is_file() {
        return Err(anyhow!(
            "Video path is not a file: {}",
            video_path.display()
        ));
    }

    // Optionally check if it's a video file (basic check)
    let extension = video_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let common_video_extensions = ["mp4", "webm", "mkv", "avi", "mov", "m4v"];

    if !common_video_extensions.contains(&extension.to_lowercase().as_str()) {
        warn!(
            "⚠️  File extension '{}' is not a common video format. Proceeding anyway.",
            extension
        );
    }

    Ok(video_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_detect_we_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_file = temp_dir.path().join("project.json");

        // No project.json
        assert!(detect_we_project(temp_dir.path()).is_err());

        // Create project.json
        File::create(&project_file).unwrap();

        // Should detect
        let result = detect_we_project(temp_dir.path()).unwrap();
        assert_eq!(result, project_file);
    }

    #[test]
    fn test_parse_we_project_basic() {
        let temp_dir = TempDir::new().unwrap();
        let project_file = temp_dir.path().join("project.json");
        let video_file = temp_dir.path().join("video.mp4");

        // Create dummy video file
        File::create(&video_file).unwrap();

        // Create minimal project.json
        let content = r#"{
            "type": "video",
            "file": "video.mp4",
            "title": "Test Video",
            "description": "A test video wallpaper"
        }"#;

        let mut file = File::create(&project_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        // Parse
        let (project, video_path) = parse_we_project(&project_file).unwrap();

        assert_eq!(project.project_type, "video");
        assert_eq!(project.file, "video.mp4");
        assert_eq!(project.title, "Test Video");
        assert_eq!(video_path, video_file);
    }

    #[test]
    fn test_parse_we_project_unsupported_type() {
        let temp_dir = TempDir::new().unwrap();
        let project_file = temp_dir.path().join("project.json");

        let content = r#"{
            "type": "web",
            "file": "index.html",
            "title": "Web Wallpaper"
        }"#;

        let mut file = File::create(&project_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        // Should fail with unsupported type
        let result = parse_we_project(&project_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported project type"));
    }

    #[test]
    fn test_resolve_video_path_missing_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = resolve_video_path(temp_dir.path(), "nonexistent.mp4");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Video file not found"));
    }
}
