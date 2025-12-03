//! Scene renderer for Wallpaper Engine scenes
//!
//! Renders scene objects (images, layers) using OpenGL.

use super::pkg::SceneContainer;
use super::shader::{
    mat4_multiply, ortho_matrix, rotation_z_matrix, scale_matrix, translation_matrix, QuadMesh,
    ShaderProgram, LAYER_FRAGMENT_SHADER, LAYER_VERTEX_SHADER, SOLID_FRAGMENT_SHADER,
};
use super::texture::Texture;
use super::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Scene renderer that composites layers using OpenGL
pub struct SceneRenderer {
    /// Loaded scene
    scene: LoadedScene,
    /// Scene container for file access (PKG or directory)
    scene_container: Option<SceneContainer>,
    /// Loaded textures (object_id -> TextureInfo)
    textures: HashMap<u64, TextureInfo>,
    /// Current time for animations
    current_time: f64,
    /// Scene width
    width: u32,
    /// Scene height
    height: u32,
    /// OpenGL resources (initialized lazily)
    gl_resources: Option<GlResources>,
}

/// OpenGL resources for rendering
struct GlResources {
    /// Shader for textured layers
    layer_shader: ShaderProgram,
    /// Shader for solid color layers
    solid_shader: ShaderProgram,
    /// Quad mesh for rendering
    quad: QuadMesh,
    /// GPU textures (object_id -> gl handle)
    gpu_textures: HashMap<u64, u32>,
}

/// Information about a loaded texture
#[derive(Debug, Clone)]
pub struct TextureInfo {
    /// OpenGL texture handle (0 if not loaded)
    pub gl_handle: u32,
    /// Texture width
    pub width: u32,
    /// Texture height
    pub height: u32,
    /// Raw pixel data (RGBA)
    pub data: Vec<u8>,
}

impl SceneRenderer {
    /// Create a new scene renderer
    pub fn new(scene: LoadedScene) -> Self {
        let width = scene.resolution.0;
        let height = scene.resolution.1;

        Self {
            scene,
            scene_container: None,
            textures: HashMap::new(),
            current_time: 0.0,
            width,
            height,
            gl_resources: None,
        }
    }

    /// Create a new scene renderer with scene container for file access
    pub fn with_container(scene: LoadedScene, container: SceneContainer) -> Self {
        let width = scene.resolution.0;
        let height = scene.resolution.1;

        Self {
            scene,
            scene_container: Some(container),
            textures: HashMap::new(),
            current_time: 0.0,
            width,
            height,
            gl_resources: None,
        }
    }

    /// Initialize OpenGL resources (must be called with GL context active)
    pub fn init_gl(&mut self) -> Result<()> {
        if self.gl_resources.is_some() {
            return Ok(());
        }

        info!("Initializing scene OpenGL resources");

        // Create shaders
        let layer_shader = ShaderProgram::new(LAYER_VERTEX_SHADER, LAYER_FRAGMENT_SHADER)
            .context("Failed to create layer shader")?;
        let solid_shader = ShaderProgram::new(LAYER_VERTEX_SHADER, SOLID_FRAGMENT_SHADER)
            .context("Failed to create solid shader")?;

        // Create quad mesh
        let quad = QuadMesh::new().context("Failed to create quad mesh")?;

        // Upload textures to GPU
        let mut gpu_textures = HashMap::new();
        for (object_id, tex_info) in &self.textures {
            let gl_handle = Self::upload_texture(tex_info)?;
            gpu_textures.insert(*object_id, gl_handle);
            debug!(
                "Uploaded texture for object {}: GL handle {}",
                object_id, gl_handle
            );
        }

        let texture_count = gpu_textures.len();

        self.gl_resources = Some(GlResources {
            layer_shader,
            solid_shader,
            quad,
            gpu_textures,
        });

        info!("Scene OpenGL initialized: {} textures", texture_count);
        Ok(())
    }

    /// Upload a texture to GPU
    fn upload_texture(tex_info: &TextureInfo) -> Result<u32> {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            // Upload pixel data
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                tex_info.width as i32,
                tex_info.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                tex_info.data.as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Ok(texture)
    }

    /// Render the scene to the current framebuffer
    pub fn render(&self) -> Result<()> {
        let gl_res = self
            .gl_resources
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenGL not initialized, call init_gl() first"))?;

        // Set up viewport and clear
        let (bg_r, bg_g, bg_b) = self.background_color();
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::ClearColor(bg_r, bg_g, bg_b, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Enable blending for transparency
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        // Create projection matrix (orthographic, origin at top-left)
        let projection = ortho_matrix(0.0, self.width as f32, self.height as f32, 0.0, -1.0, 1.0);

        // Get visible layers sorted by render order
        let layers = self.get_visible_layers();

        // Render each layer
        gl_res.layer_shader.use_program();
        gl_res.layer_shader.set_mat4("u_projection", &projection);
        gl_res.layer_shader.set_int("u_texture", 0);

        for (obj, tex_info) in layers {
            if let Some(tex_info) = tex_info {
                if let Some(&gl_handle) = gl_res.gpu_textures.get(&obj.id) {
                    self.render_layer(gl_res, obj, tex_info, gl_handle)?;
                }
            }
        }

        unsafe {
            gl::Disable(gl::BLEND);
        }

        Ok(())
    }

    /// Render a single layer
    fn render_layer(
        &self,
        gl_res: &GlResources,
        obj: &SceneObject,
        tex_info: &TextureInfo,
        gl_handle: u32,
    ) -> Result<()> {
        let rect = self.calculate_render_rect(obj, Some(tex_info));

        // Build transform matrix: translate -> rotate -> scale
        let scale = scale_matrix(rect.width, rect.height, 1.0);
        let rotation = rotation_z_matrix(rect.rotation.to_radians());
        let translation = translation_matrix(rect.x, rect.y, 0.0);

        // Apply transforms: T * R * S (scale first, then rotate, then translate)
        let transform = mat4_multiply(&translation, &mat4_multiply(&rotation, &scale));

        // Set uniforms
        gl_res.layer_shader.set_mat4("u_transform", &transform);
        gl_res.layer_shader.set_float("u_alpha", rect.alpha);
        gl_res
            .layer_shader
            .set_vec3("u_color", rect.color.x, rect.color.y, rect.color.z);
        gl_res.layer_shader.set_float("u_brightness", 1.0);
        gl_res
            .layer_shader
            .set_int("u_blend_mode", rect.blend_mode as i32);

        // Set blend mode
        self.apply_blend_mode(rect.blend_mode);

        // Bind texture and draw
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gl_handle);
        }

        gl_res.quad.draw();

        // Reset blend mode
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Ok(())
    }

    /// Apply OpenGL blend mode
    fn apply_blend_mode(&self, mode: BlendMode) {
        unsafe {
            match mode {
                BlendMode::Normal => {
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }
                BlendMode::Additive => {
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE);
                }
                BlendMode::Multiply => {
                    // Multiply blend requires different approach
                    gl::BlendFunc(gl::DST_COLOR, gl::ZERO);
                }
                BlendMode::Screen => {
                    gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_COLOR);
                }
                BlendMode::Overlay => {
                    // Overlay is complex, use normal for now
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }
            }
        }
    }

    /// Clean up OpenGL resources
    pub fn cleanup_gl(&mut self) {
        if let Some(gl_res) = self.gl_resources.take() {
            // Delete GPU textures
            for (_, gl_handle) in gl_res.gpu_textures {
                unsafe {
                    gl::DeleteTextures(1, &gl_handle);
                }
            }
            // ShaderProgram and QuadMesh will clean up in their Drop impls
            info!("Scene OpenGL resources cleaned up");
        }
    }

    /// Check if OpenGL is initialized
    pub fn is_gl_initialized(&self) -> bool {
        self.gl_resources.is_some()
    }

    /// Load all scene resources (textures)
    pub fn load_resources(&mut self) -> Result<()> {
        info!(
            "Loading scene resources, {} objects",
            self.scene.scene.objects.len()
        );

        // Collect texture paths first to avoid borrow issues
        let texture_tasks: Vec<(u64, String)> = self
            .scene
            .scene
            .objects
            .iter()
            .filter_map(|obj| {
                if let SceneObjectData::Image(ref img) = obj.data {
                    // Get the image path from the image_path field
                    if !img.image_path.is_empty() {
                        Some((obj.id, img.image_path.clone()))
                    } else {
                        img.texture_path
                            .as_ref()
                            .map(|p| (obj.id, p.to_string_lossy().into_owned()))
                    }
                } else {
                    None
                }
            })
            .collect();

        // Log objects without textures
        for obj in &self.scene.scene.objects {
            if let SceneObjectData::Image(ref img) = obj.data {
                if img.image_path.is_empty() && img.texture_path.is_none() {
                    warn!("No texture path for image object: {}", obj.name);
                }
            }
        }

        // Now load textures
        for (obj_id, texture_path) in texture_tasks {
            if let Err(e) = self.load_texture_from_path(obj_id, &texture_path) {
                warn!(
                    "Failed to load texture '{}' for object {}: {}",
                    texture_path, obj_id, e
                );
            }
        }

        info!("Loaded {} textures", self.textures.len());
        Ok(())
    }

    /// Load a texture from a path (supports PKG, TEX, and regular images)
    fn load_texture_from_path(&mut self, object_id: u64, path: &str) -> Result<()> {
        debug!("Loading texture for object {}: {}", object_id, path);

        // Determine texture path - materials/*.tex or direct image file
        let tex_path = if path.ends_with(".tex") {
            path.to_string()
        } else {
            // Try to find .tex version in materials folder
            let basename = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(path);
            format!("materials/{}.tex", basename)
        };

        // Try to load from scene container (PKG or directory)
        if let Some(ref container) = self.scene_container {
            // First try the .tex path
            if container.exists(&tex_path) {
                let data = container
                    .read_file(&tex_path)
                    .with_context(|| format!("Failed to read TEX file: {}", tex_path))?;
                return self.load_tex_data(object_id, &data, &tex_path);
            }

            // Fall back to original path
            if container.exists(path) {
                let data = container
                    .read_file(path)
                    .with_context(|| format!("Failed to read file: {}", path))?;

                // Check if it's a TEX file by magic
                if data.len() >= 8 && &data[..8] == b"TEXV0005" {
                    return self.load_tex_data(object_id, &data, path);
                }

                // Otherwise treat as regular image
                return self.load_image_data(object_id, &data, path);
            }
        }

        // Fall back to file system
        let full_path = self.scene.base_path.join(path);
        if full_path.exists() {
            let data = std::fs::read(&full_path)
                .with_context(|| format!("Failed to read file: {:?}", full_path))?;

            if data.len() >= 8 && &data[..8] == b"TEXV0005" {
                return self.load_tex_data(object_id, &data, path);
            }

            return self.load_image_data(object_id, &data, path);
        }

        Err(anyhow::anyhow!("Texture file not found: {}", path))
    }

    /// Load texture data from TEX format
    fn load_tex_data(&mut self, object_id: u64, data: &[u8], path: &str) -> Result<()> {
        let texture =
            Texture::parse(data).with_context(|| format!("Failed to parse TEX file: {}", path))?;

        // Use the first mipmap level
        let mip = texture
            .mipmaps
            .first()
            .ok_or_else(|| anyhow::anyhow!("TEX file has no mipmaps: {}", path))?;

        if mip.data.is_empty() {
            return Err(anyhow::anyhow!("TEX mipmap has no data: {}", path));
        }

        let texture_info = TextureInfo {
            gl_handle: 0,
            width: mip.width,
            height: mip.height,
            data: mip.data.clone(),
        };

        self.textures.insert(object_id, texture_info);
        debug!(
            "Loaded TEX texture {}x{} {:?} for object {}",
            mip.width, mip.height, texture.format, object_id
        );

        Ok(())
    }

    /// Load texture data from regular image format
    fn load_image_data(&mut self, object_id: u64, data: &[u8], path: &str) -> Result<()> {
        let format = image::guess_format(data)
            .with_context(|| format!("Failed to detect image format: {}", path))?;

        let img = image::load_from_memory_with_format(data, format)
            .with_context(|| format!("Failed to decode image: {}", path))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture_info = TextureInfo {
            gl_handle: 0,
            width,
            height,
            data: rgba.into_raw(),
        };

        self.textures.insert(object_id, texture_info);
        debug!(
            "Loaded image texture {}x{} for object {}",
            width, height, object_id
        );

        Ok(())
    }

    /// Get texture info for an object
    pub fn get_texture(&self, object_id: u64) -> Option<&TextureInfo> {
        self.textures.get(&object_id)
    }

    /// Get scene resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get number of loaded textures
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get visible image objects sorted by render order (z-order based on ID)
    pub fn get_visible_layers(&self) -> Vec<(&SceneObject, Option<&TextureInfo>)> {
        let mut layers: Vec<_> = self
            .scene
            .scene
            .objects
            .iter()
            .filter(|obj| obj.visible)
            .filter(|obj| matches!(obj.data, SceneObjectData::Image(_)))
            .map(|obj| (obj, self.textures.get(&obj.id)))
            .collect();

        // Sort by ID (lower IDs render first, i.e., behind)
        layers.sort_by_key(|(obj, _)| obj.id);

        layers
    }

    /// Update animation time
    pub fn update(&mut self, delta_time: f64) {
        self.current_time += delta_time;
    }

    /// Get current animation time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Calculate render position for an object
    pub fn calculate_render_rect(
        &self,
        obj: &SceneObject,
        texture: Option<&TextureInfo>,
    ) -> RenderRect {
        let img = match &obj.data {
            SceneObjectData::Image(img) => img,
            _ => return RenderRect::default(),
        };

        // Determine size
        let (tex_width, tex_height) = texture
            .map(|t| (t.width as f32, t.height as f32))
            .unwrap_or((img.size.x, img.size.y));

        let (width, height) = if img.fullscreen {
            (self.width as f32, self.height as f32)
        } else if img.autosize || (img.size.x == 0.0 && img.size.y == 0.0) {
            (tex_width * obj.scale.x, tex_height * obj.scale.y)
        } else {
            (img.size.x * obj.scale.x, img.size.y * obj.scale.y)
        };

        // Calculate position based on alignment
        let (x, y) = self.calculate_aligned_position(
            obj.origin.x,
            obj.origin.y,
            width,
            height,
            img.alignment,
        );

        RenderRect {
            x,
            y,
            width,
            height,
            rotation: obj.angles.z,
            alpha: img.alpha,
            color: img.color,
            blend_mode: img.color_blend_mode,
        }
    }

    /// Calculate position with alignment
    fn calculate_aligned_position(
        &self,
        origin_x: f32,
        origin_y: f32,
        width: f32,
        height: f32,
        alignment: ImageAlignment,
    ) -> (f32, f32) {
        let scene_center_x = self.width as f32 / 2.0;
        let scene_center_y = self.height as f32 / 2.0;

        // Convert from scene coordinates (center origin) to render coordinates (top-left origin)
        let base_x = scene_center_x + origin_x;
        let base_y = scene_center_y - origin_y; // Y is inverted

        // Apply alignment offset
        let (offset_x, offset_y) = match alignment {
            ImageAlignment::Center => (-width / 2.0, -height / 2.0),
            ImageAlignment::TopLeft => (0.0, 0.0),
            ImageAlignment::TopCenter => (-width / 2.0, 0.0),
            ImageAlignment::TopRight => (-width, 0.0),
            ImageAlignment::CenterLeft => (0.0, -height / 2.0),
            ImageAlignment::CenterRight => (-width, -height / 2.0),
            ImageAlignment::BottomLeft => (0.0, -height),
            ImageAlignment::BottomCenter => (-width / 2.0, -height),
            ImageAlignment::BottomRight => (-width, -height),
        };

        (base_x + offset_x, base_y + offset_y)
    }

    /// Get scene background color
    pub fn background_color(&self) -> (f32, f32, f32) {
        let bg = &self.scene.scene.general.clear_color;
        (bg.x, bg.y, bg.z)
    }

    /// Get loaded scene reference
    pub fn scene(&self) -> &LoadedScene {
        &self.scene
    }

    /// Check if scene has any renderable content
    pub fn has_content(&self) -> bool {
        !self.textures.is_empty()
    }
}

/// Rectangle for rendering an object
#[derive(Debug, Clone, Default)]
pub struct RenderRect {
    /// X position (pixels from left)
    pub x: f32,
    /// Y position (pixels from top)
    pub y: f32,
    /// Width in pixels
    pub width: f32,
    /// Height in pixels
    pub height: f32,
    /// Rotation angle in degrees
    pub rotation: f32,
    /// Alpha/opacity (0.0 - 1.0)
    pub alpha: f32,
    /// Tint color
    pub color: Vec3,
    /// Blend mode
    pub blend_mode: BlendMode,
}

impl RenderRect {
    /// Convert to normalized device coordinates (0.0 - 1.0)
    pub fn to_ndc(&self, viewport_width: u32, viewport_height: u32) -> NdcRect {
        NdcRect {
            x: self.x / viewport_width as f32,
            y: self.y / viewport_height as f32,
            width: self.width / viewport_width as f32,
            height: self.height / viewport_height as f32,
            rotation: self.rotation,
            alpha: self.alpha,
            color: self.color,
            blend_mode: self.blend_mode,
        }
    }
}

/// Normalized device coordinates rectangle
#[derive(Debug, Clone, Default)]
pub struct NdcRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub alpha: f32,
    pub color: Vec3,
    pub blend_mode: BlendMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::we::scene::parser::SceneParser;
    use crate::we::scene::pkg::SceneContainer;

    #[test]
    fn test_renderer_load_textures_from_pkg() {
        let home = std::env::var("HOME").unwrap_or_default();
        let pkg_dir = std::path::Path::new(&home)
            .join(".steam/steam/steamapps/workshop/content/431960/3578699777");

        if !pkg_dir.exists() {
            println!("Test PKG not found, skipping");
            return;
        }

        // Parse scene (this handles PKG automatically)
        let loaded = SceneParser::parse(&pkg_dir).expect("Failed to parse scene");

        println!("Scene: {}x{}", loaded.resolution.0, loaded.resolution.1);
        println!("Objects: {}", loaded.scene.objects.len());

        // Create container for texture loading
        let container = SceneContainer::new(&pkg_dir).expect("Failed to create container");

        // Create renderer with container
        let mut renderer = SceneRenderer::with_container(loaded, container);

        // Load resources
        renderer.load_resources().expect("Failed to load resources");

        // Verify textures loaded
        let texture_count = renderer.textures.len();
        println!("Loaded {} textures", texture_count);

        // Print texture info
        for (obj_id, tex) in &renderer.textures {
            println!(
                "  Object {}: {}x{} ({} bytes)",
                obj_id,
                tex.width,
                tex.height,
                tex.data.len()
            );
        }

        // Should have at least some textures
        assert!(texture_count > 0, "Should have loaded some textures");
    }
}
