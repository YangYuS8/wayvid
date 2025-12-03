//! Scene JSON parser for Wallpaper Engine scene files
//!
//! Parses project.json and scene.json files to extract scene structure.

use super::pkg::SceneContainer;
use super::types::*;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Parsed scene project with metadata
#[derive(Debug)]
pub struct SceneProject {
    /// Project title
    pub title: String,
    /// Project description
    pub description: Option<String>,
    /// Workshop ID (if from Steam Workshop)
    pub workshop_id: Option<String>,
    /// All scene objects
    pub objects: Vec<SceneObject>,
    /// Scene resolution
    pub resolution: (u32, u32),
    /// Loaded scene data
    pub loaded_scene: LoadedScene,
}

/// Parser for Wallpaper Engine scene files
pub struct SceneParser;

impl SceneParser {
    /// Load a scene from a Wallpaper Engine project directory
    ///
    /// This is the primary entry point for scene loading.
    ///
    /// # Arguments
    /// * `project_path` - Path to the project directory containing project.json
    ///
    /// # Returns
    /// A SceneProject containing the parsed scene and metadata
    pub fn load(project_path: &Path) -> Result<SceneProject> {
        let project_json_path = project_path.join("project.json");
        info!("Loading scene from: {:?}", project_json_path);

        // Read and parse project.json for metadata
        let project_content = fs::read_to_string(&project_json_path)
            .with_context(|| format!("Failed to read project.json: {:?}", project_json_path))?;
        let project_meta: Value = serde_json::from_str(&project_content)
            .with_context(|| "Failed to parse project.json")?;

        // Extract metadata
        let title = project_meta["title"]
            .as_str()
            .unwrap_or("Untitled Scene")
            .to_string();
        let description = project_meta["description"].as_str().map(String::from);
        let workshop_id = project_meta["workshopid"]
            .as_str()
            .or_else(|| project_meta["workshopid"].as_u64().map(|_| ""))
            .map(String::from);

        // Parse the full scene
        let loaded_scene = Self::parse(project_path)?;

        let resolution = loaded_scene.resolution;
        let objects = loaded_scene.scene.objects.clone();

        Ok(SceneProject {
            title,
            description,
            workshop_id,
            objects,
            resolution,
            loaded_scene,
        })
    }

    /// Parse a scene from a Wallpaper Engine project directory
    ///
    /// # Arguments
    /// * `project_path` - Path to the project directory containing project.json
    ///
    /// # Returns
    /// A LoadedScene containing the parsed scene and metadata
    pub fn parse(project_path: &Path) -> Result<LoadedScene> {
        let project_json_path = project_path.join("project.json");
        debug!("Parsing scene from: {:?}", project_json_path);

        // Read and parse project.json
        let project_content = fs::read_to_string(&project_json_path)
            .with_context(|| format!("Failed to read project.json: {:?}", project_json_path))?;
        let project: Value = serde_json::from_str(&project_content)
            .with_context(|| "Failed to parse project.json")?;

        // Verify it's a scene type
        let project_type = project["type"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'type' field in project.json"))?;

        if project_type != "scene" {
            return Err(anyhow!(
                "Project is not a scene type, got: {}",
                project_type
            ));
        }

        // Get the scene file path
        let scene_file = project["file"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'file' field in project.json"))?;

        // Create container for reading scene files (supports PKG or filesystem)
        let container = SceneContainer::new(project_path)?;

        // Read scene.json (from PKG or filesystem)
        let scene_content = container
            .read_file_string(scene_file)
            .with_context(|| format!("Failed to read scene file: {}", scene_file))?;
        let scene_json: Value =
            serde_json::from_str(&scene_content).with_context(|| "Failed to parse scene file")?;

        // Parse scene structure
        let scene = Self::parse_scene_with_container(&scene_json, &container)?;

        // Determine resolution from orthogonal projection
        let resolution = (
            scene.orthogonal_projection.width,
            scene.orthogonal_projection.height,
        );

        Ok(LoadedScene {
            scene,
            base_path: project_path.to_path_buf(),
            resolution,
        })
    }

    /// Parse the scene JSON structure using a container
    fn parse_scene_with_container(json: &Value, container: &SceneContainer) -> Result<Scene> {
        let base_path = container.base_path();
        Self::parse_scene(json, base_path)
    }

    /// Parse the scene JSON structure
    fn parse_scene(json: &Value, base_path: &Path) -> Result<Scene> {
        let mut scene = Scene::default();

        // Parse camera
        if let Some(camera) = json.get("camera") {
            scene.camera = Self::parse_camera(camera);
        }

        // Parse orthogonal projection
        if let Some(ortho) = json.get("orthogonalprojection") {
            scene.orthogonal_projection = Self::parse_orthogonal_projection(ortho);
        }

        // Parse general settings
        if let Some(general) = json.get("general") {
            scene.general = Self::parse_general(general);
        }

        // Parse objects
        if let Some(objects) = json.get("objects").and_then(|o| o.as_array()) {
            for obj_json in objects {
                match Self::parse_object(obj_json, base_path) {
                    Ok(obj) => scene.objects.push(obj),
                    Err(e) => warn!("Failed to parse object: {}", e),
                }
            }
        }

        debug!("Parsed scene with {} objects", scene.objects.len());
        Ok(scene)
    }

    /// Parse camera settings
    fn parse_camera(json: &Value) -> SceneCamera {
        SceneCamera {
            center: Self::parse_vec3_field(json, "center"),
            eye: Self::parse_vec3_field(json, "eye"),
            up: Self::parse_vec3_field(json, "up"),
        }
    }

    /// Parse orthogonal projection settings
    fn parse_orthogonal_projection(json: &Value) -> OrthogonalProjection {
        OrthogonalProjection {
            auto: json.get("auto").and_then(|v| v.as_bool()).unwrap_or(true),
            width: json.get("width").and_then(|v| v.as_u64()).unwrap_or(1920) as u32,
            height: json.get("height").and_then(|v| v.as_u64()).unwrap_or(1080) as u32,
        }
    }

    /// Parse general settings
    fn parse_general(json: &Value) -> SceneGeneral {
        SceneGeneral {
            ambient: Self::parse_vec3_field(json, "ambient"),
            background_color: Self::parse_vec3_field(json, "backgroundcolor"),
            clear_color: Self::parse_vec3_field(json, "clearcolor"),
            ceiling_height: json
                .get("ceilingheight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            nearz: json.get("nearz").and_then(|v| v.as_f64()).unwrap_or(0.01) as f32,
            farz: json.get("farz").and_then(|v| v.as_f64()).unwrap_or(10000.0) as f32,
            zoom: json.get("zoom").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
        }
    }

    /// Parse a scene object
    fn parse_object(json: &Value, base_path: &Path) -> Result<SceneObject> {
        let mut obj = SceneObject::default();

        // Parse common fields
        obj.id = json.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        obj.name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        obj.visible = Self::parse_bool_or_setting(json, "visible", true);
        obj.origin = Self::parse_vec3_or_setting(json, "origin");
        obj.scale = Self::parse_vec3_or_setting_default(json, "scale", Vec3::new(1.0, 1.0, 1.0));
        obj.angles = Self::parse_vec3_field(json, "angles");
        obj.parallax_depth = Self::parse_vec2_field(json, "parallaxDepth");

        // Parse dependencies
        if let Some(deps) = json.get("dependencies").and_then(|v| v.as_array()) {
            obj.dependencies = deps.iter().filter_map(|v| v.as_u64()).collect();
        }

        // Parse effects
        if let Some(effects) = json.get("effects").and_then(|v| v.as_array()) {
            for effect_json in effects {
                if let Some(effect) = Self::parse_effect(effect_json) {
                    obj.effects.push(effect);
                }
            }
        }

        // Determine object type and parse type-specific data
        if let Some(image_path) = json.get("image").and_then(|v| v.as_str()) {
            obj.data =
                SceneObjectData::Image(Self::parse_image_object(json, image_path, base_path)?);
        } else if let Some(particle_path) = json.get("particle").and_then(|v| v.as_str()) {
            obj.data = SceneObjectData::Particle(ParticleObject {
                particle_path: particle_path.to_string(),
                start_time: json
                    .get("starttime")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32,
                max_count: json
                    .get("maxcount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as u32,
            });
            warn!("Particle objects are not fully supported yet: {}", obj.name);
        } else if json.get("sound").is_some() {
            obj.data = SceneObjectData::Sound(Self::parse_sound_object(json));
            debug!("Sound object parsed (not rendered): {}", obj.name);
        } else {
            obj.data = SceneObjectData::Unknown;
            warn!("Unknown object type for: {}", obj.name);
        }

        Ok(obj)
    }

    /// Parse image object data
    fn parse_image_object(json: &Value, image_path: &str, base_path: &Path) -> Result<ImageObject> {
        let mut img = ImageObject {
            image_path: image_path.to_string(),
            size: Self::parse_vec2_field(json, "size"),
            alignment: json
                .get("alignment")
                .and_then(|v| v.as_str())
                .map(ImageAlignment::from_str)
                .unwrap_or_default(),
            alpha: Self::parse_float_or_setting(json, "alpha", 1.0),
            color: Self::parse_vec3_or_setting_default(json, "color", Vec3::new(1.0, 1.0, 1.0)),
            brightness: json
                .get("brightness")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32,
            color_blend_mode: json
                .get("colorBlendMode")
                .and_then(|v| v.as_i64())
                .map(|n| BlendMode::from_int(n as i32))
                .unwrap_or_default(),
            ..Default::default()
        };

        // Try to load the image content file to get material and texture info
        let image_content_path = base_path.join(image_path);
        if image_content_path.exists() {
            if let Ok(content) = fs::read_to_string(&image_content_path) {
                if let Ok(content_json) = serde_json::from_str::<Value>(&content) {
                    img.fullscreen = content_json
                        .get("fullscreen")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    img.passthrough = content_json
                        .get("passthrough")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    img.autosize = content_json
                        .get("autosize")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    // Get material path
                    if let Some(material) = content_json.get("material").and_then(|v| v.as_str()) {
                        img.material = Some(material.to_string());

                        // Try to resolve texture from material
                        if let Some(texture) =
                            Self::resolve_texture_from_material(base_path, material)
                        {
                            img.texture_path = Some(texture);
                        }
                    }
                }
            }
        }

        // If no texture found via material, try common patterns
        if img.texture_path.is_none() {
            img.texture_path = Self::find_texture_file(base_path, image_path);
        }

        Ok(img)
    }

    /// Try to resolve texture path from material file
    fn resolve_texture_from_material(
        base_path: &Path,
        material_path: &str,
    ) -> Option<std::path::PathBuf> {
        let material_full_path = base_path.join(material_path);
        if let Ok(content) = fs::read_to_string(&material_full_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                // Look for texture in passes -> textures
                if let Some(passes) = json.get("passes").and_then(|v| v.as_array()) {
                    for pass in passes {
                        if let Some(textures) = pass.get("textures").and_then(|v| v.as_array()) {
                            for tex in textures {
                                if let Some(tex_path) = tex.as_str() {
                                    let full_path = base_path.join(tex_path);
                                    if full_path.exists() {
                                        return Some(full_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Find texture file with common extensions
    fn find_texture_file(base_path: &Path, image_path: &str) -> Option<std::path::PathBuf> {
        // Remove .json extension and try image extensions
        let stem = image_path.trim_end_matches(".json");
        let extensions = ["png", "jpg", "jpeg", "gif", "webp", "tex"];

        for ext in extensions {
            let path = base_path.join(format!("{}.{}", stem, ext));
            if path.exists() {
                return Some(path);
            }
        }

        // Also check materials folder
        let materials_path = base_path.join("materials");
        if materials_path.exists() {
            if let Ok(entries) = fs::read_dir(&materials_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_str().unwrap_or("")) {
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    /// Parse sound object data
    fn parse_sound_object(json: &Value) -> SoundObject {
        let mut sound = SoundObject::default();

        if let Some(sounds) = json.get("sound").and_then(|v| v.as_array()) {
            sound.sounds = sounds
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        sound.repeat = json
            .get("repeat")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        sound
    }

    /// Parse an effect
    fn parse_effect(json: &Value) -> Option<SceneEffect> {
        Some(SceneEffect {
            name: json.get("name").and_then(|v| v.as_str())?.to_string(),
            file: json
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            visible: json
                .get("visible")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            id: json.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }

    // Helper functions for parsing values that can be direct or user settings

    fn parse_vec3_field(json: &Value, field: &str) -> Vec3 {
        json.get(field)
            .and_then(|v| v.as_str())
            .map(Vec3::from_str)
            .unwrap_or_default()
    }

    fn parse_vec2_field(json: &Value, field: &str) -> Vec2 {
        json.get(field)
            .and_then(|v| v.as_str())
            .map(Vec2::from_str)
            .unwrap_or_default()
    }

    fn parse_vec3_or_setting(json: &Value, field: &str) -> Vec3 {
        if let Some(value) = json.get(field) {
            if let Some(s) = value.as_str() {
                return Vec3::from_str(s);
            }
            // Handle user setting object
            if let Some(val) = value.get("value").and_then(|v| v.as_str()) {
                return Vec3::from_str(val);
            }
        }
        Vec3::default()
    }

    fn parse_vec3_or_setting_default(json: &Value, field: &str, default: Vec3) -> Vec3 {
        if let Some(value) = json.get(field) {
            if let Some(s) = value.as_str() {
                return Vec3::from_str(s);
            }
            if let Some(val) = value.get("value").and_then(|v| v.as_str()) {
                return Vec3::from_str(val);
            }
        }
        default
    }

    fn parse_bool_or_setting(json: &Value, field: &str, default: bool) -> bool {
        if let Some(value) = json.get(field) {
            if let Some(b) = value.as_bool() {
                return b;
            }
            if let Some(b) = value.get("value").and_then(|v| v.as_bool()) {
                return b;
            }
        }
        default
    }

    fn parse_float_or_setting(json: &Value, field: &str, default: f32) -> f32 {
        if let Some(value) = json.get(field) {
            if let Some(f) = value.as_f64() {
                return f as f32;
            }
            if let Some(f) = value.get("value").and_then(|v| v.as_f64()) {
                return f as f32;
            }
        }
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_from_str() {
        let v = Vec3::from_str("1.0 2.5 3.0");
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.5);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vec2_from_str() {
        let v = Vec2::from_str("100 200");
        assert_eq!(v.x, 100.0);
        assert_eq!(v.y, 200.0);
    }

    #[test]
    fn test_blend_mode_from_int() {
        assert_eq!(BlendMode::from_int(0), BlendMode::Normal);
        assert_eq!(BlendMode::from_int(1), BlendMode::Additive);
        assert_eq!(BlendMode::from_int(99), BlendMode::Normal);
    }

    #[test]
    fn test_image_alignment_from_str() {
        assert_eq!(ImageAlignment::from_str("center"), ImageAlignment::Center);
        assert_eq!(ImageAlignment::from_str("topleft"), ImageAlignment::TopLeft);
        assert_eq!(ImageAlignment::from_str("unknown"), ImageAlignment::Center);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_scene_parsing_from_pkg() {
        let home = std::env::var("HOME").unwrap_or_default();
        let scene_dir =
            PathBuf::from(&home).join(".steam/steam/steamapps/workshop/content/431960/3578699777");

        if !scene_dir.exists() {
            println!("Skipping test: scene directory not found");
            return;
        }

        let project = SceneParser::load(&scene_dir).expect("Failed to load scene");

        println!("Title: {}", project.title);
        println!("Resolution: {:?}", project.resolution);
        println!("Objects: {}", project.objects.len());

        // Should have parsed something
        assert!(!project.title.is_empty(), "Should have a title");
        assert!(project.resolution.0 > 0, "Should have resolution");

        // Print first few objects
        for (i, obj) in project.objects.iter().take(5).enumerate() {
            println!("Object {}: {} (id={})", i, obj.name, obj.id);
        }
    }
}
