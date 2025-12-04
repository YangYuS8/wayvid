//! Thumbnail generation for wallpapers
//!
//! Generates preview thumbnails for video and image wallpapers.

use std::io::Cursor;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use image::{ImageFormat, GenericImageView};
use tracing::{debug, warn};

/// Default thumbnail width
pub const THUMBNAIL_WIDTH: u32 = 320;
/// Default thumbnail height  
pub const THUMBNAIL_HEIGHT: u32 = 180;
/// JPEG quality for thumbnails
pub const THUMBNAIL_QUALITY: u8 = 85;

/// Thumbnail generator
pub struct ThumbnailGenerator {
    /// Target width
    pub width: u32,
    /// Target height
    pub height: u32,
    /// Whether ffmpeg is available
    ffmpeg_available: bool,
}

impl ThumbnailGenerator {
    pub fn new() -> Self {
        let ffmpeg_available = check_ffmpeg();
        if !ffmpeg_available {
            warn!("⚠️ ffmpeg not found - video thumbnails will not be generated");
        }

        Self {
            width: THUMBNAIL_WIDTH,
            height: THUMBNAIL_HEIGHT,
            ffmpeg_available,
        }
    }

    pub fn with_size(width: u32, height: u32) -> Self {
        let mut gen = Self::new();
        gen.width = width;
        gen.height = height;
        gen
    }

    /// Generate thumbnail for a wallpaper file
    pub fn generate(&self, path: &Path) -> Result<ThumbnailResult> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            // Video files
            "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" | "wmv" | "flv" => {
                self.generate_video_thumbnail(path)
            }
            // Animated GIF - treat as video
            "gif" => {
                // Try to get first frame using image crate
                self.generate_image_thumbnail(path)
            }
            // Image files
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" | "tif" => {
                self.generate_image_thumbnail(path)
            }
            _ => {
                anyhow::bail!("Unsupported file type: {}", extension)
            }
        }
    }

    /// Generate thumbnail for image file
    fn generate_image_thumbnail(&self, path: &Path) -> Result<ThumbnailResult> {
        debug!("Generating image thumbnail for: {}", path.display());

        let img = image::open(path)
            .context("Failed to open image")?;

        let (orig_width, orig_height) = img.dimensions();

        // Resize maintaining aspect ratio
        let thumbnail = img.thumbnail(self.width, self.height);
        let (thumb_width, thumb_height) = thumbnail.dimensions();

        // Encode to PNG
        let mut buffer = Vec::new();
        thumbnail.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .context("Failed to encode thumbnail")?;

        Ok(ThumbnailResult {
            data: buffer,
            width: thumb_width,
            height: thumb_height,
            original_width: orig_width,
            original_height: orig_height,
            format: "png".to_string(),
        })
    }

    /// Generate thumbnail for video file using ffmpeg
    fn generate_video_thumbnail(&self, path: &Path) -> Result<ThumbnailResult> {
        if !self.ffmpeg_available {
            anyhow::bail!("ffmpeg not available for video thumbnail generation");
        }

        debug!("Generating video thumbnail for: {}", path.display());

        // Create temp file for output
        let temp_path = std::env::temp_dir()
            .join(format!("wayvid_thumb_{}.png", std::process::id()));

        // Run ffmpeg to extract frame at 10% duration
        // -ss 1 seeks to 1 second (good for most videos)
        // -vframes 1 extracts single frame
        // -vf scale maintains aspect ratio
        let output = Command::new("ffmpeg")
            .args([
                "-y",  // Overwrite output
                "-ss", "1",  // Seek to 1 second
                "-i", path.to_str().unwrap_or_default(),
                "-vframes", "1",
                "-vf", &format!("scale={}:{}:force_original_aspect_ratio=decrease", self.width, self.height),
                "-f", "image2",
                temp_path.to_str().unwrap_or_default(),
            ])
            .output()
            .context("Failed to run ffmpeg")?;

        if !output.status.success() {
            // Try at 0 seconds for very short videos
            let output = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-ss", "0",
                    "-i", path.to_str().unwrap_or_default(),
                    "-vframes", "1",
                    "-vf", &format!("scale={}:{}:force_original_aspect_ratio=decrease", self.width, self.height),
                    "-f", "image2",
                    temp_path.to_str().unwrap_or_default(),
                ])
                .output()
                .context("Failed to run ffmpeg (retry)")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("ffmpeg failed: {}", stderr);
            }
        }

        // Read the generated thumbnail
        let data = std::fs::read(&temp_path)
            .context("Failed to read thumbnail")?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        // Get dimensions from the generated image
        let img = image::load_from_memory(&data)
            .context("Failed to load generated thumbnail")?;
        let (width, height) = img.dimensions();

        // Get original video dimensions using ffprobe
        let (orig_width, orig_height) = get_video_dimensions(path)
            .unwrap_or((0, 0));

        Ok(ThumbnailResult {
            data,
            width,
            height,
            original_width: orig_width,
            original_height: orig_height,
            format: "png".to_string(),
        })
    }

    /// Check if video thumbnail generation is available
    pub fn can_generate_video_thumbnails(&self) -> bool {
        self.ffmpeg_available
    }
}

impl Default for ThumbnailGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of thumbnail generation
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    /// PNG image data
    pub data: Vec<u8>,
    /// Thumbnail width
    pub width: u32,
    /// Thumbnail height  
    pub height: u32,
    /// Original media width
    pub original_width: u32,
    /// Original media height
    pub original_height: u32,
    /// Image format (always "png" currently)
    pub format: String,
}

/// Check if ffmpeg is available
fn check_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get video dimensions using ffprobe
fn get_video_dimensions(path: &Path) -> Option<(u32, u32)> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=p=0:s=x",
            path.to_str()?,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split('x').collect();
    
    if parts.len() == 2 {
        let width = parts[0].parse().ok()?;
        let height = parts[1].parse().ok()?;
        Some((width, height))
    } else {
        None
    }
}

/// Get video duration using ffprobe
pub fn get_video_duration(path: &Path) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path.to_str()?,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use image::DynamicImage;

    #[test]
    fn test_thumbnail_generator_creation() {
        let gen = ThumbnailGenerator::new();
        assert_eq!(gen.width, THUMBNAIL_WIDTH);
        assert_eq!(gen.height, THUMBNAIL_HEIGHT);
    }

    #[test]
    fn test_thumbnail_generator_custom_size() {
        let gen = ThumbnailGenerator::with_size(640, 360);
        assert_eq!(gen.width, 640);
        assert_eq!(gen.height, 360);
    }

    #[test]
    fn test_image_thumbnail() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.png");
        
        // Create a simple test image
        let img = DynamicImage::new_rgb8(100, 100);
        img.save(&image_path).unwrap();

        let gen = ThumbnailGenerator::new();
        let result = gen.generate(&image_path).unwrap();

        assert!(!result.data.is_empty());
        assert!(result.width <= THUMBNAIL_WIDTH);
        assert!(result.height <= THUMBNAIL_HEIGHT);
        assert_eq!(result.original_width, 100);
        assert_eq!(result.original_height, 100);
    }
}
