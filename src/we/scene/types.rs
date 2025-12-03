//! Scene data types for Wallpaper Engine scene parsing
//!
//! These types represent the structure of Wallpaper Engine scene wallpapers.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main scene structure containing all objects and settings
#[derive(Debug, Clone, Default)]
pub struct Scene {
    /// Scene camera settings
    pub camera: SceneCamera,
    /// Scene orthographic projection settings
    pub orthogonal_projection: OrthogonalProjection,
    /// All objects in the scene
    pub objects: Vec<SceneObject>,
    /// Scene general settings
    pub general: SceneGeneral,
}

/// Scene camera configuration
#[derive(Debug, Clone, Default)]
pub struct SceneCamera {
    /// Camera center point
    pub center: Vec3,
    /// Camera eye position
    pub eye: Vec3,
    /// Camera up vector
    pub up: Vec3,
}

/// Orthogonal projection settings
#[derive(Debug, Clone)]
pub struct OrthogonalProjection {
    /// Auto detection enabled
    pub auto: bool,
    /// Projection width
    pub width: u32,
    /// Projection height
    pub height: u32,
}

impl Default for OrthogonalProjection {
    fn default() -> Self {
        Self {
            auto: true,
            width: 1920,
            height: 1080,
        }
    }
}

/// Scene general settings
#[derive(Debug, Clone, Default)]
pub struct SceneGeneral {
    /// Ambient color
    pub ambient: Vec3,
    /// Background color
    pub background_color: Vec3,
    /// Clear color
    pub clear_color: Vec3,
    /// Ceiling height
    pub ceiling_height: f32,
    /// Nearz clipping plane
    pub nearz: f32,
    /// Farz clipping plane
    pub farz: f32,
    /// Zoom level
    pub zoom: f32,
}

/// 3D vector for positions, scales, colors
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn from_str(s: &str) -> Self {
        let parts: Vec<f32> = s
            .split_whitespace()
            .filter_map(|p| p.parse().ok())
            .collect();
        Self {
            x: parts.first().copied().unwrap_or(0.0),
            y: parts.get(1).copied().unwrap_or(0.0),
            z: parts.get(2).copied().unwrap_or(0.0),
        }
    }
}

/// 2D vector for sizes
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_str(s: &str) -> Self {
        let parts: Vec<f32> = s
            .split_whitespace()
            .filter_map(|p| p.parse().ok())
            .collect();
        Self {
            x: parts.first().copied().unwrap_or(0.0),
            y: parts.get(1).copied().unwrap_or(0.0),
        }
    }
}

/// A scene object (image, particle, sound, etc.)
#[derive(Debug, Clone)]
pub struct SceneObject {
    /// Unique object ID
    pub id: u64,
    /// Object name
    pub name: String,
    /// Object origin/position
    pub origin: Vec3,
    /// Object scale
    pub scale: Vec3,
    /// Object rotation angles (degrees)
    pub angles: Vec3,
    /// Whether object is visible
    pub visible: bool,
    /// Object-specific data
    pub data: SceneObjectData,
    /// Effects applied to this object
    pub effects: Vec<SceneEffect>,
    /// Object dependencies (IDs of other objects)
    pub dependencies: Vec<u64>,
    /// Parallax depth for scroll effect
    pub parallax_depth: Vec2,
}

impl Default for SceneObject {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            origin: Vec3::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            angles: Vec3::default(),
            visible: true,
            data: SceneObjectData::Unknown,
            effects: Vec::new(),
            dependencies: Vec::new(),
            parallax_depth: Vec2::default(),
        }
    }
}

/// Object-specific data based on type
#[derive(Debug, Clone)]
pub enum SceneObjectData {
    /// Image layer
    Image(ImageObject),
    /// Particle system (not fully supported yet)
    Particle(ParticleObject),
    /// Sound object (not rendered)
    Sound(SoundObject),
    /// Unknown/unsupported type
    Unknown,
}

/// Image object data
#[derive(Debug, Clone, Default)]
pub struct ImageObject {
    /// Path to the image content file
    pub image_path: String,
    /// Image size
    pub size: Vec2,
    /// Image alignment
    pub alignment: ImageAlignment,
    /// Alpha/transparency (0.0 - 1.0)
    pub alpha: f32,
    /// Tint color
    pub color: Vec3,
    /// Brightness multiplier
    pub brightness: f32,
    /// Color blend mode
    pub color_blend_mode: BlendMode,
    /// Whether image is fullscreen
    pub fullscreen: bool,
    /// Pass through mouse events
    pub passthrough: bool,
    /// Auto-size based on image dimensions
    pub autosize: bool,
    /// Material path (for rendering)
    pub material: Option<String>,
    /// Loaded texture path (resolved from material)
    pub texture_path: Option<PathBuf>,
}

/// Particle system object (placeholder for future support)
#[derive(Debug, Clone, Default)]
pub struct ParticleObject {
    /// Particle definition file path
    pub particle_path: String,
    /// Start time
    pub start_time: f32,
    /// Maximum particle count
    pub max_count: u32,
}

/// Sound object (not rendered, but parsed)
#[derive(Debug, Clone, Default)]
pub struct SoundObject {
    /// Sound file paths
    pub sounds: Vec<String>,
    /// Whether to repeat
    pub repeat: bool,
}

/// Image alignment options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ImageAlignment {
    #[default]
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl ImageAlignment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "center" => Self::Center,
            "topleft" => Self::TopLeft,
            "topcenter" | "top" => Self::TopCenter,
            "topright" => Self::TopRight,
            "centerleft" | "left" => Self::CenterLeft,
            "centerright" | "right" => Self::CenterRight,
            "bottomleft" => Self::BottomLeft,
            "bottomcenter" | "bottom" => Self::BottomCenter,
            "bottomright" => Self::BottomRight,
            _ => Self::Center,
        }
    }
}

/// Blend modes for layer compositing
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Normal,
    Additive,
    Multiply,
    Screen,
    Overlay,
}

impl BlendMode {
    pub fn from_int(n: i32) -> Self {
        match n {
            0 => Self::Normal,
            1 => Self::Additive,
            2 => Self::Multiply,
            3 => Self::Screen,
            4 => Self::Overlay,
            _ => Self::Normal,
        }
    }
}

/// Scene effect applied to an object
#[derive(Debug, Clone, Default)]
pub struct SceneEffect {
    /// Effect name
    pub name: String,
    /// Effect file path
    pub file: String,
    /// Whether effect is visible/enabled
    pub visible: bool,
    /// Effect ID
    pub id: u64,
}

/// Animation definition for scene objects
#[derive(Debug, Clone)]
pub struct Animation {
    /// Animation type
    pub animation_type: AnimationType,
    /// Animation duration in seconds
    pub duration: f32,
    /// Whether animation loops
    pub repeat: bool,
    /// Animation easing function
    pub easing: EasingFunction,
}

/// Types of animations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    /// Scroll/pan animation
    Scroll {
        direction: ScrollDirection,
        speed: i32,
    },
    /// Scale animation
    Scale { from: i32, to: i32 },
    /// Rotation animation
    Spin { speed: i32 },
    /// Opacity animation
    Opacity { from: i32, to: i32 },
}

/// Scroll direction for scroll animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    #[default]
    Horizontal,
    Vertical,
    Both,
}

/// Easing functions for animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EasingFunction {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// Result of loading a scene
#[derive(Debug)]
pub struct LoadedScene {
    /// The parsed scene structure
    pub scene: Scene,
    /// Base path for resolving resources
    pub base_path: PathBuf,
    /// Scene resolution
    pub resolution: (u32, u32),
}

impl LoadedScene {
    /// Get absolute path for a resource relative to scene
    pub fn resolve_path(&self, relative: &str) -> PathBuf {
        self.base_path.join(relative)
    }
}
