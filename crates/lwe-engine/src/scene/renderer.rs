use std::collections::BTreeMap;
use std::ffi::CString;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{Context, Result, bail, ensure};

#[cfg(test)]
#[path = "renderer_acceptance.rs"]
mod acceptance;

#[cfg(test)]
#[path = "renderer_yakuza_acceptance.rs"]
mod yakuza_acceptance;

use super::{
    Blend, PreparedLayer, PreparedPass, PreparedScene, PreparedStage, PreparedTexture,
    TEXTURE_SLOTS, TextureBinding,
};

pub(super) const QUAD: [f32; 30] = [
    -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, -1.0, 0.0,
    0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0,
];
pub(super) const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

pub(super) const MAX_SCENE_GPU_BYTES: u64 = 512 * 1024 * 1024;

// A compositor may withhold background callbacks while an output is hidden or
// powered off. Resume from the retained state instead of replaying unbounded
// simulation work or removing a valid wallpaper for a presentation gap.
fn animation_delta(previous: f64, current: f64) -> f64 {
    let elapsed = (current - previous).max(0.0);
    if elapsed > 2.0 { 0.25 } else { elapsed }
}

// An unnamed pass can retain at most one scratch input. Static and named
// textures have separate allocations, so two scratch targets avoid feedback
// throughout a multi-pass layer; a single unnamed pass needs only one.
fn scratch_target_count(layer: &PreparedLayer) -> usize {
    layer
        .stages
        .iter()
        .flat_map(|stage| &stage.passes)
        .filter(|pass| pass.target.is_none())
        .count()
        .clamp(1, 2)
}

fn scratch_layout(layers: &[PreparedLayer]) -> BTreeMap<(i32, i32), usize> {
    let mut layout = BTreeMap::new();
    for layer in layers {
        let dimensions = (layer.size[0].ceil() as i32, layer.size[1].ceil() as i32);
        let count = layout.entry(dimensions).or_insert(0);
        *count = (*count).max(scratch_target_count(layer));
    }
    layout
}

fn needs_full_frame(scene: &PreparedScene) -> bool {
    scene.layers.iter().any(|layer| {
        layer.composite.is_some()
            || layer
                .stages
                .iter()
                .flat_map(|stage| &stage.passes)
                .flat_map(|pass| &pass.textures)
                .any(|binding| matches!(binding, TextureBinding::FullFrame))
    })
}

/// Logical RGBA texture/FBO payload, not driver memory or EGL back buffers.
pub(super) fn estimated_gpu_bytes(scene: &PreparedScene) -> Result<u64> {
    let mut bytes = 4_u64;
    for texture in &scene.textures {
        bytes = bytes
            .checked_add(u64::from(texture.width) * u64::from(texture.height) * 4)
            .context("Scene GPU allocation estimate overflow")?;
    }
    for ((width, height), count) in scratch_layout(&scene.layers) {
        bytes = bytes
            .checked_add(width as u64 * height as u64 * 4 * count as u64)
            .context("Scene shared scratch estimate overflow")?;
    }
    if needs_full_frame(scene) {
        bytes = bytes
            .checked_add(scene.canvas[0].ceil() as u64 * scene.canvas[1].ceil() as u64 * 8)
            .context("Scene full-frame estimate overflow")?;
    }
    for layer in &scene.layers {
        let width = layer.size[0].ceil() as u64;
        let height = layer.size[1].ceil() as u64;
        for stage in &layer.stages {
            for scale in stage.targets.values() {
                let [target_width, target_height] =
                    target_dimensions(width as i32, height as i32, *scale);
                bytes = bytes
                    .checked_add(target_width as u64 * target_height as u64 * 4)
                    .context("Scene GPU allocation estimate overflow")?;
            }
        }
    }
    for layer in &scene.particles {
        bytes = bytes
            .checked_add(
                layer.vertex_capacity as u64
                    * std::mem::size_of::<super::particles::ParticleVertex>() as u64,
            )
            .context("Scene particle GPU allocation estimate overflow")?;
    }
    Ok(bytes)
}

fn target_dimensions(width: i32, height: i32, scale: f32) -> [i32; 2] {
    [
        ((width as f32 / scale).ceil() as i32).max(1),
        ((height as f32 / scale).ceil() as i32).max(1),
    ]
}

pub(super) fn validate_gpu_budget(scene: &PreparedScene) -> Result<u64> {
    let bytes = estimated_gpu_bytes(scene)?;
    if bytes > MAX_SCENE_GPU_BYTES {
        return Err(lwe_scene::SceneAssetError::LimitExceeded {
            resource: "Scene GPU texture and framebuffer payload".into(),
            size: bytes,
            limit: MAX_SCENE_GPU_BYTES,
        }
        .into());
    }
    Ok(bytes)
}

/// GL state touched by Scene drawing is restored even on an early error.
/// GPU resources and this guard must remain on the context-owning thread.
#[derive(Debug, PartialEq)]
pub(super) struct GlState {
    program: i32,
    vao: i32,
    array_buffer: i32,
    unpack_buffer: i32,
    unpack_alignment: i32,
    unpack_layout: [i32; 5],
    polygon_mode: [i32; 2],
    active_texture: i32,
    textures: [i32; TEXTURE_SLOTS],
    samplers: Option<[i32; TEXTURE_SLOTS]>,
    draw_fbo: i32,
    read_fbo: i32,
    viewport: [i32; 4],
    color_mask: [u8; 4],
    depth_mask: u8,
    clear_color: [f32; 4],
    enabled: [bool; 8],
    blend_factors: [i32; 4],
    blend_equations: [i32; 2],
}

const CAPABILITIES: [u32; 8] = [
    gl::BLEND,
    gl::DEPTH_TEST,
    gl::STENCIL_TEST,
    gl::CULL_FACE,
    gl::SCISSOR_TEST,
    gl::FRAMEBUFFER_SRGB,
    gl::RASTERIZER_DISCARD,
    gl::DITHER,
];

impl GlState {
    pub(super) fn capture() -> Self {
        let mut state = Self {
            program: 0,
            vao: 0,
            array_buffer: 0,
            unpack_buffer: 0,
            unpack_alignment: 4,
            unpack_layout: [0; 5],
            polygon_mode: [0; 2],
            active_texture: 0,
            textures: [0; TEXTURE_SLOTS],
            samplers: None,
            draw_fbo: 0,
            read_fbo: 0,
            viewport: [0; 4],
            color_mask: [1; 4],
            depth_mask: 1,
            clear_color: [0.0; 4],
            enabled: [false; 8],
            blend_factors: [0; 4],
            blend_equations: [0; 2],
        };
        // SAFETY: the public initialization/render contract requires a current
        // context, and every pointer below addresses an appropriately sized array.
        unsafe {
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut state.program);
            gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut state.vao);
            gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut state.array_buffer);
            gl::GetIntegerv(gl::PIXEL_UNPACK_BUFFER_BINDING, &mut state.unpack_buffer);
            gl::GetIntegerv(gl::UNPACK_ALIGNMENT, &mut state.unpack_alignment);
            for (index, parameter) in [
                gl::UNPACK_ROW_LENGTH,
                gl::UNPACK_SKIP_PIXELS,
                gl::UNPACK_SKIP_ROWS,
                gl::UNPACK_IMAGE_HEIGHT,
                gl::UNPACK_SKIP_IMAGES,
            ]
            .iter()
            .enumerate()
            {
                gl::GetIntegerv(*parameter, &mut state.unpack_layout[index]);
            }
            gl::GetIntegerv(gl::POLYGON_MODE, state.polygon_mode.as_mut_ptr());
            gl::GetIntegerv(gl::ACTIVE_TEXTURE, &mut state.active_texture);
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut state.draw_fbo);
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut state.read_fbo);
            gl::GetIntegerv(gl::VIEWPORT, state.viewport.as_mut_ptr());
            gl::GetBooleanv(gl::COLOR_WRITEMASK, state.color_mask.as_mut_ptr());
            gl::GetBooleanv(gl::DEPTH_WRITEMASK, &mut state.depth_mask);
            gl::GetFloatv(gl::COLOR_CLEAR_VALUE, state.clear_color.as_mut_ptr());
            for (index, capability) in CAPABILITIES.iter().enumerate() {
                state.enabled[index] = gl::IsEnabled(*capability) == gl::TRUE;
            }
            for (index, parameter) in [
                gl::BLEND_SRC_RGB,
                gl::BLEND_DST_RGB,
                gl::BLEND_SRC_ALPHA,
                gl::BLEND_DST_ALPHA,
            ]
            .iter()
            .enumerate()
            {
                gl::GetIntegerv(*parameter, &mut state.blend_factors[index]);
            }
            gl::GetIntegerv(gl::BLEND_EQUATION_RGB, &mut state.blend_equations[0]);
            gl::GetIntegerv(gl::BLEND_EQUATION_ALPHA, &mut state.blend_equations[1]);
            let mut major = 0;
            let mut minor = 0;
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
            if major > 3 || (major == 3 && minor >= 3) {
                state.samplers = Some([0; TEXTURE_SLOTS]);
            }
            for slot in 0..TEXTURE_SLOTS {
                gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
                gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut state.textures[slot]);
                if let Some(samplers) = state.samplers.as_mut() {
                    gl::GetIntegerv(gl::SAMPLER_BINDING, &mut samplers[slot]);
                }
            }
            gl::ActiveTexture(state.active_texture as u32);
        }
        state
    }

    pub(super) fn begin(&self) {
        // SAFETY: state capture established the current context on this thread.
        unsafe {
            for capability in CAPABILITIES {
                gl::Disable(capability);
            }
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::DepthMask(gl::FALSE);
            gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            for parameter in [
                gl::UNPACK_ROW_LENGTH,
                gl::UNPACK_SKIP_PIXELS,
                gl::UNPACK_SKIP_ROWS,
                gl::UNPACK_IMAGE_HEIGHT,
                gl::UNPACK_SKIP_IMAGES,
            ] {
                gl::PixelStorei(parameter, 0);
            }
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            if self.samplers.is_some() {
                for slot in 0..TEXTURE_SLOTS {
                    gl::BindSampler(slot as u32, 0);
                }
            }
        }
    }
}

impl Drop for GlState {
    fn drop(&mut self) {
        // SAFETY: guards never leave their context-owning thread or outlive it.
        unsafe {
            gl::UseProgram(self.program as u32);
            gl::BindVertexArray(self.vao as u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.array_buffer as u32);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, self.unpack_buffer as u32);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, self.unpack_alignment);
            for (index, parameter) in [
                gl::UNPACK_ROW_LENGTH,
                gl::UNPACK_SKIP_PIXELS,
                gl::UNPACK_SKIP_ROWS,
                gl::UNPACK_IMAGE_HEIGHT,
                gl::UNPACK_SKIP_IMAGES,
            ]
            .iter()
            .enumerate()
            {
                gl::PixelStorei(*parameter, self.unpack_layout[index]);
            }
            if self.polygon_mode[0] == self.polygon_mode[1] {
                gl::PolygonMode(gl::FRONT_AND_BACK, self.polygon_mode[0] as u32);
            } else {
                gl::PolygonMode(gl::FRONT, self.polygon_mode[0] as u32);
                gl::PolygonMode(gl::BACK, self.polygon_mode[1] as u32);
            }
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.draw_fbo as u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.read_fbo as u32);
            gl::Viewport(
                self.viewport[0],
                self.viewport[1],
                self.viewport[2],
                self.viewport[3],
            );
            gl::ColorMask(
                self.color_mask[0],
                self.color_mask[1],
                self.color_mask[2],
                self.color_mask[3],
            );
            gl::DepthMask(self.depth_mask);
            gl::ClearColor(
                self.clear_color[0],
                self.clear_color[1],
                self.clear_color[2],
                self.clear_color[3],
            );
            gl::BlendFuncSeparate(
                self.blend_factors[0] as u32,
                self.blend_factors[1] as u32,
                self.blend_factors[2] as u32,
                self.blend_factors[3] as u32,
            );
            gl::BlendEquationSeparate(
                self.blend_equations[0] as u32,
                self.blend_equations[1] as u32,
            );
            for (index, capability) in CAPABILITIES.iter().enumerate() {
                if self.enabled[index] {
                    gl::Enable(*capability);
                } else {
                    gl::Disable(*capability);
                }
            }
            for slot in 0..TEXTURE_SLOTS {
                gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
                gl::BindTexture(gl::TEXTURE_2D, self.textures[slot] as u32);
                if let Some(samplers) = self.samplers {
                    gl::BindSampler(slot as u32, samplers[slot] as u32);
                }
            }
            gl::ActiveTexture(self.active_texture as u32);
        }
    }
}

struct GpuTexture {
    id: u32,
    width: i32,
    height: i32,
    image_width: i32,
    image_height: i32,
    source_flags: u32,
}

impl GpuTexture {
    fn new(
        width: i32,
        height: i32,
        image_width: i32,
        image_height: i32,
        source_flags: u32,
    ) -> Self {
        Self {
            id: 0,
            width,
            height,
            image_width,
            image_height,
            source_flags,
        }
    }

    fn upload(texture: &PreparedTexture) -> Result<Self> {
        ensure!(
            texture.width > 0 && texture.height > 0,
            "Invalid zero texture size"
        );
        let mut max_size = 0;
        // SAFETY: the caller initialized GL and holds a current context.
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_size);
        }
        ensure!(
            texture.width <= max_size as u32 && texture.height <= max_size as u32,
            "Scene texture exceeds this GPU's maximum dimensions"
        );
        let row = texture.width as usize * 4;
        ensure!(
            texture.rgba.len() == row * texture.height as usize,
            "Scene texture RGBA size mismatch"
        );
        // All uploaded images use GL's bottom-up origin. Pass outputs then share
        // the same UV convention, avoiding alternate-pass vertical flips.
        let mut pixels = Vec::with_capacity(texture.rgba.len());
        for line in texture.rgba.chunks_exact(row).rev() {
            pixels.extend_from_slice(line);
        }
        let mut result = Self::new(
            texture.width as i32,
            texture.height as i32,
            texture.image_width as i32,
            texture.image_height as i32,
            texture.source_flags,
        );
        // SAFETY: pixels contains width*height*4 bytes; unpack PBO is disabled.
        unsafe {
            gl::GenTextures(1, &mut result.id);
            gl::BindTexture(gl::TEXTURE_2D, result.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                result.width,
                result.height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
            texture_parameters(false);
        }
        check_gl("uploading Scene texture")?;
        Ok(result)
    }
    fn resolution(&self) -> [f32; 4] {
        [
            self.width as f32,
            self.height as f32,
            self.image_width as f32,
            self.image_height as f32,
        ]
    }
}
impl Drop for GpuTexture {
    fn drop(&mut self) {
        if self.id != 0 {
            unsafe {
                gl::DeleteTextures(1, &self.id);
            }
        }
    }
}

unsafe fn texture_parameters(repeat: bool) {
    // SAFETY: caller has a texture bound in the current context.
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        let wrap = if repeat {
            gl::REPEAT
        } else {
            gl::CLAMP_TO_EDGE
        };
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as i32);
    }
}

fn pass_texture_repeat(label: &str, slot: usize, texture: &GpuTexture) -> bool {
    if super::shader::is_legacy_image(label) {
        texture.source_flags & 2 == 0
    } else {
        (matches!(label, "effects/shine_downsample2" | "effects/waterripple") && slot == 2)
            || (label == "effects/scroll" && slot == 0)
    }
}

pub(super) struct RenderTarget {
    texture: GpuTexture,
    fbo: u32,
}
impl RenderTarget {
    pub(super) fn new(width: i32, height: i32) -> Result<Self> {
        let mut max_size = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_size);
        }
        ensure!(
            width > 0 && height > 0 && width <= max_size && height <= max_size,
            "Scene render target exceeds GPU dimensions"
        );
        let mut result = Self {
            texture: GpuTexture::new(width, height, width, height, 2),
            fbo: 0,
        };
        // SAFETY: dimensions were bounded; a null client pointer allocates storage.
        unsafe {
            gl::GenTextures(1, &mut result.texture.id);
            gl::BindTexture(gl::TEXTURE_2D, result.texture.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            texture_parameters(false);
            gl::GenFramebuffers(1, &mut result.fbo);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, result.fbo);
            gl::FramebufferTexture2D(
                gl::DRAW_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                result.texture.id,
                0,
            );
            ensure!(
                gl::CheckFramebufferStatus(gl::DRAW_FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE,
                "Scene framebuffer is incomplete"
            );
        }
        check_gl("allocating Scene framebuffer")?;
        Ok(result)
    }

    pub(super) fn framebuffer(&self) -> u32 {
        self.fbo
    }

    pub(super) fn texture_id(&self) -> u32 {
        self.texture.id
    }
}
impl Drop for RenderTarget {
    fn drop(&mut self) {
        if self.fbo != 0 {
            unsafe {
                gl::DeleteFramebuffers(1, &self.fbo);
            }
        }
    }
}

pub(super) struct Program {
    id: u32,
    locations: BTreeMap<String, i32>,
}
impl Program {
    pub(super) fn compile(shader: &super::shader::Shader, label: &str) -> Result<Self> {
        let vertex = ShaderObject::compile(gl::VERTEX_SHADER, &shader.vertex, label)?;
        let fragment = ShaderObject::compile(gl::FRAGMENT_SHADER, &shader.fragment, label)?;
        let mut result = Self {
            id: unsafe { gl::CreateProgram() },
            locations: BTreeMap::new(),
        };
        ensure!(result.id != 0, "Cannot create Scene shader program");
        // SAFETY: shader/program names are owned by these RAII wrappers.
        unsafe {
            gl::AttachShader(result.id, vertex.0);
            gl::AttachShader(result.id, fragment.0);
            gl::BindAttribLocation(result.id, 0, c"a_Position".as_ptr());
            gl::BindAttribLocation(result.id, 1, c"a_TexCoord".as_ptr());
            gl::BindAttribLocation(result.id, 2, c"a_Color".as_ptr());
            gl::BindAttribLocation(result.id, 3, c"a_TexCoordNext".as_ptr());
            gl::BindAttribLocation(result.id, 4, c"a_TexCoordBlend".as_ptr());
            gl::LinkProgram(result.id);
            let mut linked = 0;
            gl::GetProgramiv(result.id, gl::LINK_STATUS, &mut linked);
            if linked == 0 {
                bail!(
                    "Scene shader {label} link failed: {}",
                    program_log(result.id)
                );
            }
        }
        let mut names = vec![
            "g_ModelViewProjectionMatrix".to_owned(),
            "g_ModelMatrix".to_owned(),
            "g_Color4".to_owned(),
            "g_Time".to_owned(),
            "g_BlendMode".to_owned(),
        ];
        for slot in 0..TEXTURE_SLOTS {
            names.push(format!("g_Texture{slot}"));
            names.push(format!("g_Texture{slot}Resolution"));
        }
        names.extend(shader.uniforms.iter().map(|uniform| uniform.name.clone()));
        for name in names {
            let cname = CString::new(name.as_str())?;
            result.locations.insert(name, unsafe {
                gl::GetUniformLocation(result.id, cname.as_ptr())
            });
        }
        Ok(result)
    }
    pub(super) fn id(&self) -> u32 {
        self.id
    }

    pub(super) fn location(&self, name: &str) -> i32 {
        self.locations.get(name).copied().unwrap_or(-1)
    }
    pub(super) fn constants(&self, values: &BTreeMap<String, Vec<f32>>) -> Result<()> {
        for (name, values) in values {
            let location = self.location(name);
            if location < 0 {
                continue;
            }
            // SAFETY: uniform locations belong to the active program; component
            // counts were validated during CPU preparation.
            unsafe {
                match values.len() {
                    1 => gl::Uniform1f(location, values[0]),
                    2 => gl::Uniform2fv(location, 1, values.as_ptr()),
                    3 => gl::Uniform3fv(location, 1, values.as_ptr()),
                    4 => gl::Uniform4fv(location, 1, values.as_ptr()),
                    _ => bail!("Unsupported Scene uniform component count for {name}"),
                }
            }
        }
        Ok(())
    }
    pub(super) fn matrix(&self, value: &[f32; 16]) {
        unsafe {
            gl::UniformMatrix4fv(
                self.location("g_ModelViewProjectionMatrix"),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
            gl::UniformMatrix4fv(
                self.location("g_ModelMatrix"),
                1,
                gl::FALSE,
                IDENTITY.as_ptr(),
            );
        }
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        if self.id != 0 {
            unsafe {
                gl::DeleteProgram(self.id);
            }
        }
    }
}

struct ShaderObject(u32);
impl ShaderObject {
    fn compile(kind: u32, source: &str, label: &str) -> Result<Self> {
        let source = CString::new(source).context("Scene shader contains a NUL byte")?;
        let shader = Self(unsafe { gl::CreateShader(kind) });
        ensure!(shader.0 != 0, "Cannot create Scene shader");
        unsafe {
            gl::ShaderSource(shader.0, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader.0);
            let mut compiled = 0;
            gl::GetShaderiv(shader.0, gl::COMPILE_STATUS, &mut compiled);
            if compiled == 0 {
                bail!(
                    "Scene shader {label} compilation failed: {}",
                    shader_log(shader.0)
                );
            }
        }
        Ok(shader)
    }
}
impl Drop for ShaderObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}
unsafe fn shader_log(id: u32) -> String {
    let mut size = 0;
    unsafe {
        gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut size);
    }
    let mut bytes = vec![0u8; size.max(1) as usize];
    unsafe {
        gl::GetShaderInfoLog(id, size, std::ptr::null_mut(), bytes.as_mut_ptr().cast());
    }
    String::from_utf8_lossy(&bytes)
        .trim_end_matches('\0')
        .to_owned()
}
unsafe fn program_log(id: u32) -> String {
    let mut size = 0;
    unsafe {
        gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut size);
    }
    let mut bytes = vec![0u8; size.max(1) as usize];
    unsafe {
        gl::GetProgramInfoLog(id, size, std::ptr::null_mut(), bytes.as_mut_ptr().cast());
    }
    String::from_utf8_lossy(&bytes)
        .trim_end_matches('\0')
        .to_owned()
}

fn check_gl(stage: &str) -> Result<()> {
    let error = unsafe { gl::GetError() };
    ensure!(
        error == gl::NO_ERROR,
        "OpenGL error 0x{error:x} while {stage}"
    );
    Ok(())
}

struct GpuPass {
    program: Program,
    textures: Vec<TextureBinding>,
    constants: BTreeMap<String, Vec<f32>>,
    target: Option<String>,
    blend: Blend,
    label: String,
}
impl GpuPass {
    fn new(pass: &PreparedPass) -> Result<Self> {
        let program = Program::compile(&pass.shader, &pass.label)?;
        ensure!(
            program.location("g_PointerPosition") < 0,
            "Scene shader needs an unavailable pointer input host"
        );
        for (slot, binding) in pass.textures.iter().enumerate() {
            ensure!(
                !matches!(binding, TextureBinding::Black)
                    || program.location(&format!("g_Texture{slot}")) < 0,
                "Scene shader {} requires unbound sampler g_Texture{slot}",
                pass.label
            );
        }
        Ok(Self {
            program,
            textures: pass.textures.clone(),
            constants: pass.constants.clone(),
            target: pass.target.clone(),
            blend: pass.blend,
            label: pass.label.clone(),
        })
    }
}
struct GpuStage {
    passes: Vec<GpuPass>,
    targets: BTreeMap<String, RenderTarget>,
}
impl GpuStage {
    fn new(stage: &PreparedStage, width: i32, height: i32) -> Result<Self> {
        let mut targets = BTreeMap::new();
        for (name, scale) in &stage.targets {
            let [target_width, target_height] = target_dimensions(width, height, *scale);
            targets.insert(
                name.clone(),
                RenderTarget::new(target_width, target_height)?,
            );
        }
        Ok(Self {
            passes: stage
                .passes
                .iter()
                .map(GpuPass::new)
                .collect::<Result<_>>()?,
            targets,
        })
    }
}
struct GpuLayer {
    size: [f32; 2],
    origin: [f32; 2],
    scale: [f32; 2],
    angle: f32,
    tint: [f32; 4],
    blend: Blend,
    stages: Vec<GpuStage>,
    composite: Option<Program>,
}
impl GpuLayer {
    fn new(layer: &PreparedLayer) -> Result<Self> {
        let width = layer.size[0].ceil() as i32;
        let height = layer.size[1].ceil() as i32;
        let stages = layer
            .stages
            .iter()
            .map(|stage| GpuStage::new(stage, width, height))
            .collect::<Result<_>>()?;
        Ok(Self {
            size: layer.size,
            origin: layer.origin,
            scale: layer.scale,
            angle: layer.angle,
            tint: layer.tint,
            blend: layer.blend,
            stages,
            composite: layer
                .composite
                .as_ref()
                .map(|shader| Program::compile(shader, "LWE authored color blend"))
                .transpose()?,
        })
    }
}

struct GpuParticleLayer {
    runtime: super::particles::ParticleRuntime,
    materials: Vec<GpuPass>,
    vao: u32,
    vbo: u32,
    capacity: usize,
}

impl GpuParticleLayer {
    fn new(layer: &super::PreparedParticleLayer, seed: u64) -> Result<Self> {
        let mut runtime = super::particles::ParticleRuntime::new(&layer.simulation, seed)
            .with_context(|| {
                format!(
                    "Initializing particle system {}",
                    layer.simulation.systems[0].path
                )
            })?;
        runtime.configure_material_aspects(&layer.aspects)?;
        runtime.configure_material_animations(&layer.animations)?;
        let mut result = Self {
            runtime,
            materials: layer
                .materials
                .iter()
                .map(GpuPass::new)
                .collect::<Result<_>>()?,
            vao: 0,
            vbo: 0,
            capacity: layer.vertex_capacity,
        };
        // SAFETY: initialization requires a current context. The bounded,
        // repr(C) ParticleVertex supplies bounded geometry and two atlas UVs.
        unsafe {
            gl::GenVertexArrays(1, &mut result.vao);
            gl::GenBuffers(1, &mut result.vbo);
            ensure!(
                result.vao != 0 && result.vbo != 0,
                "Cannot allocate Scene particle geometry"
            );
            gl::BindVertexArray(result.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, result.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (result.capacity * std::mem::size_of::<super::particles::ParticleVertex>())
                    as isize,
                std::ptr::null(),
                gl::STREAM_DRAW,
            );
            for (index, components, offset) in
                [(0, 2, 0), (1, 2, 8), (2, 4, 28), (3, 2, 16), (4, 1, 24)]
            {
                gl::EnableVertexAttribArray(index);
                gl::VertexAttribPointer(
                    index,
                    components,
                    gl::FLOAT,
                    gl::FALSE,
                    std::mem::size_of::<super::particles::ParticleVertex>() as i32,
                    offset as *const _,
                );
            }
        }
        check_gl("allocating Scene particle geometry")?;
        Ok(result)
    }

    fn draw(
        &mut self,
        seconds: f64,
        canvas: [f32; 2],
        width: i32,
        height: i32,
        target: u32,
        textures: &[GpuTexture],
    ) -> Result<()> {
        let frame = self.runtime.advance(seconds)?;
        let matrix = particle_matrix(canvas, width, height);
        for batch in frame.batches {
            ensure!(
                batch.vertices.len() <= self.capacity,
                "Scene particle geometry exceeds its GPU allocation"
            );
            if batch.vertices.is_empty() {
                continue;
            }
            let pass = self
                .materials
                .get(batch.material)
                .context("Particle material index unavailable")?;
            let TextureBinding::Static(index) = pass.textures[0] else {
                bail!("Particle primary texture is not static")
            };
            let texture = textures
                .get(index)
                .context("Particle texture unavailable")?;
            // SAFETY: the validated vertex slice supplies exactly the stated
            // bounded byte count; this VBO retains its allocated capacity.
            unsafe {
                gl::BindVertexArray(self.vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    std::mem::size_of_val(batch.vertices.as_slice()) as isize,
                    batch.vertices.as_ptr().cast(),
                );
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target);
                gl::Viewport(0, 0, width, height);
                gl::UseProgram(pass.program.id);
                set_blend(pass.blend);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                texture_parameters(false);
                gl::Uniform1i(pass.program.location("g_Texture0"), 0);
            }
            pass.program.matrix(&matrix);
            pass.program.constants(&pass.constants)?;
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, batch.vertices.len() as i32);
            }
        }
        check_gl("drawing Scene particles")
    }
}

impl Drop for GpuParticleLayer {
    fn drop(&mut self) {
        // SAFETY: the owning SceneRenderer drops before its current GL context.
        unsafe {
            if self.vbo != 0 {
                gl::DeleteBuffers(1, &self.vbo);
            }
            if self.vao != 0 {
                gl::DeleteVertexArrays(1, &self.vao);
            }
        }
    }
}

fn particle_matrix(canvas: [f32; 2], width: i32, height: i32) -> [f32; 16] {
    let output_ratio = width as f32 / height as f32;
    let canvas_ratio = canvas[0] / canvas[1];
    let visible = if output_ratio > canvas_ratio {
        [canvas[0], canvas[0] / output_ratio]
    } else {
        [canvas[1] * output_ratio, canvas[1]]
    };
    [
        2.0 / visible[0],
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / visible[1],
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        -canvas[0] / visible[0],
        -canvas[1] / visible[1],
        0.0,
        1.0,
    ]
}

/// Not Send/Sync: GL initialization, rendering and destruction stay on the
/// context-owning thread. The associated PreparedScene is entirely CPU-only.
pub struct SceneRenderer {
    canvas: [f32; 2],
    clear: [f32; 4],
    textures: Vec<GpuTexture>,
    layers: Vec<GpuLayer>,
    particles: Vec<GpuParticleLayer>,
    draw_order: Vec<super::DrawObject>,
    black: Option<GpuTexture>,
    copy: Option<Program>,
    vao: u32,
    vbo: u32,
    animated: bool,
    paused: bool,
    drawn: bool,
    last_size: [i32; 2],
    time: f64,
    last_input: Option<f64>,
    rebase_time: bool,
    gpu_payload_bytes: u64,
    prepared_bloom: Option<Arc<super::postprocess::PreparedBloom>>,
    bloom: Option<super::postprocess::BloomRenderer>,
    scratch: BTreeMap<(i32, i32), Vec<RenderTarget>>,
    full_frame: Vec<RenderTarget>,
    spectrum: Option<super::spectrum::Spectrum>,
    analysis_active: bool,
    last_spectrum: super::spectrum::SpectrumSnapshot,
    scripts: Option<super::dynamic_script::ScriptWorker>,
    script_layers: BTreeMap<usize, usize>,
    hidden_script_layers: std::collections::BTreeSet<usize>,
    scripts_ready: bool,
    _context_thread: PhantomData<Rc<()>>,
}

impl SceneRenderer {
    pub(super) fn initialize(scene: &PreparedScene) -> Result<Self> {
        let gpu_payload_bytes = validate_gpu_budget(scene)?;
        ensure!(
            gl::GetString::is_loaded() && !unsafe { gl::GetString(gl::VERSION) }.is_null(),
            "Scene initialization requires a current OpenGL context"
        );
        let guard = GlState::capture();
        guard.begin();
        let mut renderer = Self {
            canvas: scene.canvas,
            clear: scene.clear,
            textures: Vec::new(),
            layers: Vec::new(),
            particles: Vec::new(),
            draw_order: scene.draw_order.clone(),
            black: None,
            copy: None,
            vao: 0,
            vbo: 0,
            animated: scene.summary().animated,
            paused: false,
            drawn: false,
            last_size: [0; 2],
            time: 0.0,
            last_input: None,
            rebase_time: false,
            gpu_payload_bytes,
            prepared_bloom: scene.bloom.clone(),
            bloom: None,
            scratch: BTreeMap::new(),
            full_frame: Vec::new(),
            spectrum: None,
            analysis_active: false,
            last_spectrum: super::spectrum::SpectrumSnapshot::default(),
            scripts: scene
                .scripts
                .as_ref()
                .map(|plan| super::dynamic_script::ScriptWorker::spawn(plan))
                .transpose()?,
            script_layers: scene.script_layers.clone(),
            hidden_script_layers: std::collections::BTreeSet::new(),
            scripts_ready: false,
            _context_thread: PhantomData,
        };
        renderer.textures = scene
            .textures
            .iter()
            .map(GpuTexture::upload)
            .collect::<Result<_>>()?;
        renderer.black = Some(GpuTexture::upload(&PreparedTexture {
            width: 1,
            height: 1,
            image_width: 1,
            image_height: 1,
            source_format: 0,
            source_flags: 2,
            animation: None,
            rgba: vec![0, 0, 0, 0],
        })?);
        renderer.copy = Some(Program::compile(
            &super::shader::image(),
            "LWE image composite",
        )?);
        unsafe {
            gl::GenVertexArrays(1, &mut renderer.vao);
            gl::BindVertexArray(renderer.vao);
            gl::GenBuffers(1, &mut renderer.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, renderer.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&QUAD) as isize,
                QUAD.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 20, std::ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 20, (12usize) as *const _);
        }
        renderer.layers = scene
            .layers
            .iter()
            .map(GpuLayer::new)
            .collect::<Result<_>>()?;
        for (dimensions, count) in scratch_layout(&scene.layers) {
            renderer.scratch.insert(
                dimensions,
                (0..count)
                    .map(|_| RenderTarget::new(dimensions.0, dimensions.1))
                    .collect::<Result<_>>()?,
            );
        }
        if needs_full_frame(scene) {
            renderer.full_frame = (0..2)
                .map(|_| {
                    RenderTarget::new(scene.canvas[0].ceil() as i32, scene.canvas[1].ceil() as i32)
                })
                .collect::<Result<_>>()?;
        }
        if scene
            .layers
            .iter()
            .flat_map(|layer| &layer.stages)
            .flat_map(|stage| &stage.passes)
            .flat_map(|pass| &pass.shader.uniforms)
            .any(super::shader::is_audio_spectrum)
            || scene
                .scripts
                .as_ref()
                .is_some_and(|scripts| scripts.uses_audio())
        {
            renderer.spectrum = Some(super::spectrum::Spectrum::new()?);
        }
        renderer.particles = scene
            .particles
            .iter()
            .enumerate()
            .map(|(index, layer)| {
                GpuParticleLayer::new(layer, 0x9e3779b97f4a7c15_u64.wrapping_mul(index as u64 + 1))
            })
            .collect::<Result<_>>()?;
        check_gl("initializing Scene renderer")?;
        Ok(renderer)
    }

    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(spectrum) = &self.spectrum {
            spectrum.set_paused(true);
        }
    }

    pub(crate) fn activate_analysis(&mut self, paused: bool) {
        self.analysis_active = true;
        if let Some(spectrum) = &self.spectrum {
            spectrum.set_paused(paused);
        }
    }

    pub(crate) fn deactivate_analysis(&mut self) {
        self.analysis_active = false;
        if let Some(spectrum) = &self.spectrum {
            spectrum.set_paused(true);
        }
    }

    /// Resident logical payload estimated from owned textures and framebuffers.
    pub fn gpu_payload_bytes(&self) -> u64 {
        self.gpu_payload_bytes
            + self
                .bloom
                .as_ref()
                .map(|bloom| bloom.payload_bytes())
                .unwrap_or_default()
    }

    /// New framebuffer payload must coexist with the retained old layout until
    /// every allocation succeeds, so report its full size rather than a delta.
    pub(crate) fn additional_gpu_bytes(&self, width: i32, height: i32) -> Result<u64> {
        if self
            .bloom
            .as_ref()
            .is_some_and(|bloom| bloom.dimensions() == [width, height])
        {
            return Ok(0);
        }
        self.prepared_bloom
            .as_ref()
            .map(|bloom| bloom.estimated_bytes(width, height))
            .transpose()
            .map(Option::unwrap_or_default)
    }
    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            self.rebase_time = true;
            if self.analysis_active
                && let Some(spectrum) = &self.spectrum
            {
                spectrum.set_paused(false);
            }
        }
    }
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        ensure!(width > 0 && height > 0, "Invalid Scene output dimensions");
        let additional = self.additional_gpu_bytes(width, height)?;
        ensure!(
            self.gpu_payload_bytes()
                .checked_add(additional)
                .is_some_and(|bytes| bytes <= MAX_SCENE_GPU_BYTES),
            "Scene framebuffer resize exceeds GPU payload budget"
        );
        if additional != 0
            && let Some(prepared) = &self.prepared_bloom
        {
            let replacement =
                super::postprocess::BloomRenderer::initialize(prepared, width, height)?;
            self.bloom = Some(replacement);
        }
        if self.last_size != [width, height] {
            self.drawn = false;
        }
        Ok(())
    }
    pub fn update(&mut self, time_seconds: f64) -> Result<()> {
        ensure!(
            time_seconds.is_finite() && time_seconds >= 0.0,
            "Invalid Scene animation time"
        );
        if !self.paused {
            if let Some(previous) = self.last_input {
                if !self.rebase_time {
                    self.time += animation_delta(previous, time_seconds);
                }
            } else {
                self.time = time_seconds;
            }
        }
        self.last_input = Some(time_seconds);
        self.rebase_time = false;
        Ok(())
    }

    /// Draws to the caller's current draw framebuffer. A true result means a
    /// complete valid frame was drawn; EGL swap/first-frame ACK remain host-owned.
    pub fn render(&mut self, width: i32, height: i32, time_seconds: f64) -> Result<bool> {
        self.render_internal(width, height, time_seconds, false, None)
    }

    /// Re-presents the retained scene after a rejected candidate overwrote the
    /// output. Paused scenes keep their animation time but still draw this frame.
    pub fn render_forced(&mut self, width: i32, height: i32, time_seconds: f64) -> Result<bool> {
        self.render_internal(width, height, time_seconds, true, None)
    }

    fn render_internal(
        &mut self,
        width: i32,
        height: i32,
        time_seconds: f64,
        force: bool,
        spectrum_override: Option<super::spectrum::SpectrumSnapshot>,
    ) -> Result<bool> {
        self.resize(width, height)?;
        self.update(time_seconds)?;
        if !force
            && (self.paused || (self.drawn && !self.animated && self.last_size == [width, height]))
        {
            return Ok(false);
        }
        let guard = GlState::capture();
        guard.begin();
        let final_target = guard.draw_fbo as u32;
        let output_target = self
            .bloom
            .as_ref()
            .map(|bloom| bloom.scene_framebuffer())
            .unwrap_or(final_target);
        let composition_size = if self.full_frame.is_empty() {
            [width, height]
        } else {
            [self.canvas[0].ceil() as i32, self.canvas[1].ceil() as i32]
        };
        let target = self
            .full_frame
            .first()
            .map(|frame| frame.fbo)
            .unwrap_or(output_target);
        if let Some(spectrum) = &self.spectrum {
            spectrum.set_paused(self.paused || !self.analysis_active);
        }
        if let Some(spectrum) = spectrum_override {
            self.last_spectrum = spectrum;
        } else if self.analysis_active && !self.paused {
            self.last_spectrum = self
                .spectrum
                .as_ref()
                .map(|spectrum| spectrum.snapshot())
                .unwrap_or_default();
        }
        let spectrum = self.last_spectrum;
        if let Some(worker) = &self.scripts
            && (!self.paused || !self.scripts_ready)
        {
            if let Some(states) = worker.poll_frame(self.time, spectrum.script_average)? {
                self.scripts_ready = true;
                for state in states {
                    let Some(index) = self.script_layers.get(&state.handle).copied() else {
                        continue;
                    };
                    ensure!(
                        state.angles[0] == 0.0 && state.angles[1] == 0.0,
                        "Script attempted unsupported 3D rotation"
                    );
                    ensure!(
                        state.origin[2] == 0.0,
                        "Script attempted unsupported image depth"
                    );
                    let layer = &mut self.layers[index];
                    layer.origin = [state.origin[0], state.origin[1]];
                    layer.scale = [state.scale[0], state.scale[1]];
                    layer.angle = state.angles[2];
                    layer.tint = [state.color[0], state.color[1], state.color[2], state.alpha];
                    if state.visible {
                        self.hidden_script_layers.remove(&index);
                    } else {
                        self.hidden_script_layers.insert(index);
                    }
                }
            } else if !self.scripts_ready {
                // A pending VM must not produce a first-frame acknowledgement.
                return Ok(false);
            }
        }
        let black = self
            .black
            .as_ref()
            .context("Scene renderer has been cleaned up")?;
        let copy = self
            .copy
            .as_ref()
            .context("Scene renderer has been cleaned up")?;
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target);
            gl::Viewport(0, 0, composition_size[0], composition_size[1]);
            gl::ClearColor(self.clear[0], self.clear[1], self.clear[2], self.clear[3]);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        for object in &self.draw_order {
            let super::DrawObject::Image(index) = *object else {
                let super::DrawObject::Particles(index) = *object else {
                    unreachable!()
                };
                self.particles[index].draw(
                    self.time,
                    self.canvas,
                    composition_size[0],
                    composition_size[1],
                    target,
                    &self.textures,
                )?;
                // Particle geometry uses a separate VAO. Restore the fullscreen
                // quad before subsequent image/effect passes.
                unsafe {
                    gl::BindVertexArray(self.vao);
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                }
                continue;
            };
            let layer = &self.layers[index];
            if self.hidden_script_layers.contains(&index)
                || layer.scale[0] == 0.0
                || layer.scale[1] == 0.0
            {
                continue;
            }
            if self.full_frame.len() == 2
                && (layer.composite.is_some()
                    || layer
                        .stages
                        .iter()
                        .flat_map(|stage| &stage.passes)
                        .flat_map(|pass| &pass.textures)
                        .any(|binding| matches!(binding, TextureBinding::FullFrame)))
            {
                unsafe {
                    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, target);
                    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.full_frame[1].fbo);
                    gl::BlitFramebuffer(
                        0,
                        0,
                        composition_size[0],
                        composition_size[1],
                        0,
                        0,
                        composition_size[0],
                        composition_size[1],
                        gl::COLOR_BUFFER_BIT,
                        gl::NEAREST,
                    );
                }
            }
            let scratch = self
                .scratch
                .get(&(layer.size[0].ceil() as i32, layer.size[1].ceil() as i32))
                .context("Scene shared scratch targets unavailable")?;
            let output = render_layer(
                layer,
                &self.textures,
                black,
                self.time as f32,
                scratch,
                self.full_frame.get(1).map(|frame| &frame.texture),
                &spectrum,
            )?;
            let composite = layer.composite.as_ref().unwrap_or(copy);
            unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target);
                gl::Viewport(0, 0, composition_size[0], composition_size[1]);
                gl::UseProgram(composite.id);
                set_blend(layer.blend);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, output.id);
                gl::Uniform1i(composite.location("g_Texture0"), 0);
                gl::Uniform4fv(
                    composite.location("g_Texture0Resolution"),
                    1,
                    output.resolution().as_ptr(),
                );
                gl::Uniform4f(composite.location("g_Color4"), 1.0, 1.0, 1.0, 1.0);
                if let Some(frame) = self.full_frame.get(1) {
                    gl::ActiveTexture(gl::TEXTURE4);
                    gl::BindTexture(gl::TEXTURE_2D, frame.texture.id);
                    texture_parameters(false);
                    gl::Uniform1i(composite.location("g_Texture4"), 4);
                }
            }
            composite.matrix(&layer_matrix(
                self.canvas,
                layer,
                composition_size[0],
                composition_size[1],
            ));
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
        }
        if let Some(frame) = self.full_frame.first() {
            unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, output_target);
                gl::Viewport(0, 0, width, height);
                gl::Disable(gl::BLEND);
                gl::UseProgram(copy.id);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, frame.texture.id);
                texture_parameters(false);
                gl::Uniform1i(copy.location("g_Texture0"), 0);
                gl::Uniform4fv(
                    copy.location("g_Texture0Resolution"),
                    1,
                    frame.texture.resolution().as_ptr(),
                );
                gl::Uniform4f(copy.location("g_Color4"), 1.0, 1.0, 1.0, 1.0);
                gl::Uniform1i(copy.location("g_BlendMode"), 0);
            }
            copy.matrix(&cover_matrix(self.canvas, width, height));
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
        }
        if let Some(bloom) = &self.bloom {
            bloom.composite_to(final_target)?;
        }
        check_gl("rendering Scene frame")?;
        self.drawn = true;
        self.last_size = [width, height];
        Ok(true)
    }

    /// The host must make this renderer's context current before calling this
    /// method or dropping it. Cleanup is idempotent and precedes EGL destruction.
    pub fn cleanup(&mut self) {
        self.scripts = None;
        self.script_layers.clear();
        self.hidden_script_layers.clear();
        self.scripts_ready = false;
        self.bloom = None;
        self.spectrum = None;
        self.full_frame.clear();
        self.scratch.clear();
        self.particles.clear();
        self.layers.clear();
        self.textures.clear();
        self.copy = None;
        self.black = None;
        unsafe {
            if self.vbo != 0 {
                gl::DeleteBuffers(1, &self.vbo);
                self.vbo = 0;
            }
            if self.vao != 0 {
                gl::DeleteVertexArrays(1, &self.vao);
                self.vao = 0;
            }
        }
        self.drawn = false;
        self.gpu_payload_bytes = 0;
    }
}
impl Drop for SceneRenderer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn layer_matrix(canvas: [f32; 2], layer: &GpuLayer, width: i32, height: i32) -> [f32; 16] {
    // Cover preserves wallpaper aspect on differently shaped monitor outputs.
    let output_ratio = width as f32 / height as f32;
    let canvas_ratio = canvas[0] / canvas[1];
    let visible = if output_ratio > canvas_ratio {
        [canvas[0], canvas[0] / output_ratio]
    } else {
        [canvas[1] * output_ratio, canvas[1]]
    };
    let (sin, cos) = layer.angle.sin_cos();
    let sx = layer.size[0] * layer.scale[0] / visible[0];
    let sy = layer.size[1] * layer.scale[1] / visible[1];
    [
        cos * sx,
        sin * layer.size[0] * layer.scale[0] / visible[1],
        0.0,
        0.0,
        -sin * layer.size[1] * layer.scale[1] / visible[0],
        cos * sy,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        2.0 * (layer.origin[0] - canvas[0] / 2.0) / visible[0],
        2.0 * (layer.origin[1] - canvas[1] / 2.0) / visible[1],
        0.0,
        1.0,
    ]
}

fn cover_matrix(canvas: [f32; 2], width: i32, height: i32) -> [f32; 16] {
    let output_ratio = width as f32 / height as f32;
    let canvas_ratio = canvas[0] / canvas[1];
    let mut matrix = IDENTITY;
    if output_ratio > canvas_ratio {
        matrix[5] = output_ratio / canvas_ratio;
    } else {
        matrix[0] = canvas_ratio / output_ratio;
    }
    matrix
}

unsafe fn set_blend(blend: Blend) {
    unsafe {
        match blend {
            Blend::Replace => gl::Disable(gl::BLEND),
            Blend::Alpha => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(
                    gl::SRC_ALPHA,
                    gl::ONE_MINUS_SRC_ALPHA,
                    gl::ONE,
                    gl::ONE_MINUS_SRC_ALPHA,
                );
            }
            Blend::Add => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE, gl::ONE, gl::ONE);
            }
        }
    }
}

fn render_layer<'a>(
    layer: &'a GpuLayer,
    textures: &[GpuTexture],
    black: &GpuTexture,
    time: f32,
    scratch: &'a [RenderTarget],
    full_frame: Option<&GpuTexture>,
    spectrum: &super::spectrum::SpectrumSnapshot,
) -> Result<&'a GpuTexture> {
    let mut current: Option<&GpuTexture> = None;
    for (stage_index, stage) in layer.stages.iter().enumerate() {
        let stage_input = current;
        for pass in &stage.passes {
            let input = if stage_index == 0 {
                current
            } else {
                stage_input
            };
            let resolved = pass
                .textures
                .iter()
                .map(|binding| match binding {
                    TextureBinding::Static(index) => textures
                        .get(*index)
                        .context("Scene texture index unavailable"),
                    TextureBinding::Input => {
                        if stage_index == 0 {
                            Ok(input.unwrap_or(black))
                        } else {
                            input.context("Scene pass has no previous image")
                        }
                    }
                    TextureBinding::Named(name) => stage
                        .targets
                        .get(name)
                        .map(|target| &target.texture)
                        .context("Scene named target unavailable"),
                    TextureBinding::Black => Ok(black),
                    TextureBinding::FullFrame => {
                        full_frame.context("Scene full-frame texture unavailable")
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            let target = if let Some(name) = &pass.target {
                stage
                    .targets
                    .get(name)
                    .context("Scene output target unavailable")?
            } else {
                scratch
                    .iter()
                    .find(|target| {
                        input.is_none_or(|input| input.id != target.texture.id)
                            && !resolved
                                .iter()
                                .any(|texture| texture.id == target.texture.id)
                    })
                    .context("Scene scratch framebuffer feedback loop")?
            };
            ensure!(
                !resolved
                    .iter()
                    .any(|texture| texture.id == target.texture.id),
                "Scene render pass samples its output"
            );
            unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target.fbo);
                gl::Viewport(0, 0, target.texture.width, target.texture.height);
                gl::Disable(gl::BLEND);
                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::UseProgram(pass.program.id);
                if stage_index != 0 {
                    set_blend(pass.blend);
                }
                for (slot, texture) in resolved.iter().enumerate() {
                    if pass.program.location(&format!("g_Texture{slot}")) < 0 {
                        continue;
                    }
                    gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
                    gl::BindTexture(gl::TEXTURE_2D, texture.id);
                    texture_parameters(pass_texture_repeat(&pass.label, slot, texture));
                    gl::Uniform1i(
                        pass.program.location(&format!("g_Texture{slot}")),
                        slot as i32,
                    );
                    let resolution = texture.resolution();
                    gl::Uniform4fv(
                        pass.program.location(&format!("g_Texture{slot}Resolution")),
                        1,
                        resolution.as_ptr(),
                    );
                }
                gl::Uniform1f(pass.program.location("g_Time"), time);
                gl::Uniform1fv(
                    pass.program.location("g_AudioSpectrum16Left"),
                    16,
                    spectrum.left.as_ptr(),
                );
                gl::Uniform1fv(
                    pass.program.location("g_AudioSpectrum16Right"),
                    16,
                    spectrum.right.as_ptr(),
                );
                gl::Uniform2f(
                    pass.program.location("g_TexelSize"),
                    1.0 / resolved[0].width as f32,
                    1.0 / resolved[0].height as f32,
                );
                gl::Uniform2f(pass.program.location("g_PointerPosition"), 0.0, 0.0);
                gl::UniformMatrix4fv(
                    pass.program.location("g_ModelViewProjectionMatrixInverse"),
                    1,
                    gl::FALSE,
                    IDENTITY.as_ptr(),
                );
                gl::Uniform1i(
                    pass.program.location("g_BlendMode"),
                    match pass.blend {
                        Blend::Replace => 0,
                        Blend::Alpha => 1,
                        Blend::Add => 2,
                    },
                );
                let tint = if stage_index == 0 {
                    layer.tint
                } else {
                    [1.0; 4]
                };
                gl::Uniform4fv(pass.program.location("g_Color4"), 1, tint.as_ptr());
            }
            pass.program.matrix(&IDENTITY);
            pass.program.constants(&pass.constants)?;
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
            check_gl(&format!("drawing Scene pass {}", pass.label))?;
            if pass.target.is_none() {
                current = Some(&target.texture);
            }
        }
    }
    current.context("Scene image produced no frame")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_estimate_and_allocation_share_float_rounding() {
        assert_eq!(target_dimensions(100, 100, 100.0 / 7.0), [7, 7]);
        assert_eq!(target_dimensions(101, 103, 2.0), [51, 52]);
    }

    #[test]
    fn legacy_sampler_flags_do_not_change_modern_effect_sampling() {
        let repeat = GpuTexture::new(4, 4, 2, 2, 0);
        let clamp = GpuTexture::new(4, 4, 2, 2, 2);
        for texture in [&repeat, &clamp] {
            assert!(!pass_texture_repeat("effects/iris", 0, texture));
            assert!(!pass_texture_repeat("effects/waterripple", 0, texture));
            assert!(pass_texture_repeat("effects/waterripple", 2, texture));
            assert!(pass_texture_repeat("effects/shine_downsample2", 2, texture));
            assert!(!pass_texture_repeat("genericparticle", 0, texture));
        }
        for label in ["genericimage", "flowimage", "swayimage", "flag"] {
            for slot in 0..TEXTURE_SLOTS {
                assert!(pass_texture_repeat(label, slot, &repeat));
                assert!(!pass_texture_repeat(label, slot, &clamp));
            }
        }
        assert_eq!(clamp.resolution(), [4.0, 4.0, 2.0, 2.0]);
    }

    #[test]
    #[ignore = "requires an explicit EGL/OpenGL offscreen acceptance run"]
    fn offscreen_legacy_sampling_preserves_padding_y_math_and_wrap() -> Result<()> {
        use khronos_egl as egl;
        let instance = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required()? };
        let display = unsafe { instance.get_display(egl::DEFAULT_DISPLAY) }
            .context("No offscreen EGL display")?;
        instance.initialize(display)?;
        instance.bind_api(egl::OPENGL_API)?;
        let config = instance
            .choose_first_config(
                display,
                &[
                    egl::SURFACE_TYPE,
                    egl::PBUFFER_BIT,
                    egl::RENDERABLE_TYPE,
                    egl::OPENGL_BIT,
                    egl::RED_SIZE,
                    8,
                    egl::GREEN_SIZE,
                    8,
                    egl::BLUE_SIZE,
                    8,
                    egl::ALPHA_SIZE,
                    8,
                    egl::NONE,
                ],
            )?
            .context("No EGL configuration")?;
        let context = instance.create_context(
            display,
            config,
            None,
            &[
                egl::CONTEXT_MAJOR_VERSION,
                3,
                egl::CONTEXT_MINOR_VERSION,
                0,
                egl::NONE,
            ],
        )?;
        let surface = instance.create_pbuffer_surface(
            display,
            config,
            &[egl::WIDTH, 2, egl::HEIGHT, 2, egl::NONE],
        )?;
        instance.make_current(display, Some(surface), Some(surface), Some(context))?;
        gl::load_with(|name| {
            instance
                .get_proc_address(name)
                .map(|function| function as *const std::ffi::c_void)
                .unwrap_or(std::ptr::null())
        });

        // These authored shader inputs are independent fixture code. Loading
        // them through the production adapter exercises its actual vertex and
        // sampling wrappers, rather than reproducing those wrappers in a test.
        struct Fixture(std::path::PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let fixture = Fixture(
            std::env::temp_dir().join(format!("lwe-legacy-gl-{}-{nonce}", std::process::id())),
        );
        std::fs::create_dir(&fixture.0)?;
        std::fs::create_dir(fixture.0.join("shaders"))?;
        std::fs::write(
            fixture.0.join("shaders/genericimage.vert"),
            "attribute vec3 a_Position;\nattribute vec2 a_TexCoord;\nuniform mat4 g_ModelViewProjectionMatrix;\nvarying vec2 v_TexCoord;\nvoid main(){gl_Position=g_ModelViewProjectionMatrix*vec4(a_Position,1.0);v_TexCoord=a_TexCoord;}\n",
        )?;
        let assets = lwe_scene::SceneAssets::open(&fixture.0, None)?;
        let mut rgba = [255, 0, 255, 255].repeat(16);
        for (index, color) in [
            (0, [255, 0, 0, 255]),
            (1, [255, 255, 0, 255]),
            (4, [0, 0, 255, 255]),
            (5, [0, 255, 0, 255]),
        ] {
            rgba[index * 4..index * 4 + 4].copy_from_slice(&color);
        }
        let orientation = [
            0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 255, 0, 255,
        ];
        let tinted = [0, 0, 96, 255, 0, 32, 0, 255, 64, 0, 0, 255, 64, 32, 0, 255];
        let tint = [0.5, 0.25, 0.75, 0.5];
        let mut cases = vec![
            ("v_TexCoord", 2, [1.0; 4], orientation),
            (
                "vec2(v_TexCoord.x,v_TexCoord.y*0.5+0.125)",
                2,
                [1.0; 4],
                [
                    64, 0, 191, 255, 64, 255, 0, 255, 191, 0, 64, 255, 191, 255, 0, 255,
                ],
            ),
            ("v_TexCoord+vec2(1.0,0.0)", 0, [1.0; 4], orientation),
            (
                "v_TexCoord+vec2(1.0,0.0)",
                2,
                [1.0; 4],
                [255, 0, 255, 255].repeat(4).try_into().unwrap(),
            ),
            ("v_TexCoord", 2, tint, tinted),
            (
                "negative odd power",
                2,
                [1.0; 4],
                [
                    124, 0, 255, 255, 124, 255, 0, 255, 96, 0, 0, 255, 96, 255, 0, 255,
                ],
            ),
        ];
        let original_assets_path = std::env::var_os("LWE_SCENE_TEST_ASSETS");
        if original_assets_path.is_some() {
            cases.push(("original genericimage", 2, [1.0; 4], orientation));
            cases.push(("original genericimage", 2, tint, tinted));
            cases.push((
                "original genericimage negative scroll",
                0,
                [1.0; 4],
                [
                    255, 0, 255, 255, 0, 0, 255, 255, 255, 0, 255, 255, 255, 0, 0, 255,
                ],
            ));
        }
        for (expression, flags, tint, expected) in cases {
            let negative_scroll = expression == "original genericimage negative scroll";
            let (shader, constants) = if expression.starts_with("original genericimage") {
                // An empty project root prevents the authored fixture shaders
                // from shadowing the installed original assets in this case.
                let original_project = fixture.0.join("original");
                std::fs::create_dir_all(&original_project)?;
                let original_assets = lwe_scene::SceneAssets::open(
                    &original_project,
                    original_assets_path.as_deref().map(std::path::Path::new),
                )?;
                let shader = super::super::shader::load_legacy_image(
                    &original_assets,
                    "genericimage",
                    &BTreeMap::new(),
                    &std::collections::BTreeSet::from([0]),
                )?;
                let mut constants = shader
                    .uniforms
                    .iter()
                    .filter_map(|uniform| {
                        uniform.default.as_ref().map(|value| {
                            super::super::numbers(value).map(|value| (uniform.name.clone(), value))
                        })
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                if negative_scroll {
                    constants.insert("g_ScrollX".to_owned(), vec![-0.5]);
                }
                (shader, constants)
            } else {
                let body = if expression == "negative odd power" {
                    // The signed cubic is the same operation used by the flag
                    // family, with a varying input that is negative here. A
                    // constant-folded pow would not exercise the driver path.
                    "vec4 color=texSample2D(g_Texture0,v_TexCoord);color.r=0.5+pow(v_TexCoord.y-0.625,3.0);gl_FragColor=color;".to_owned()
                } else {
                    format!("gl_FragColor=texSample2D(g_Texture0,{expression});")
                };
                std::fs::write(
                    fixture.0.join("shaders/genericimage.frag"),
                    format!(
                        "uniform sampler2D g_Texture0;\nvarying vec2 v_TexCoord;\nvoid main(){{{body}}}\n"
                    ),
                )?;
                (
                    super::super::shader::load_legacy_image(
                        &assets,
                        "genericimage",
                        &BTreeMap::new(),
                        &std::collections::BTreeSet::from([0]),
                    )?,
                    BTreeMap::new(),
                )
            };
            let prepared = PreparedScene {
                scripts: None,
                script_layers: BTreeMap::new(),
                audio: Vec::new(),
                bloom: None,
                particles: Vec::new(),
                draw_order: vec![super::super::DrawObject::Image(0)],
                canvas: [2.0, 2.0],
                clear: [0.0, 0.0, 0.0, 1.0],
                textures: vec![PreparedTexture {
                    width: 4,
                    height: 4,
                    image_width: 2,
                    image_height: 2,
                    source_format: 0,
                    source_flags: flags,
                    animation: None,
                    rgba: rgba.clone(),
                }],
                layers: vec![PreparedLayer {
                    size: [2.0, 2.0],
                    origin: [1.0, 1.0],
                    scale: [1.0; 2],
                    angle: 0.0,
                    tint,
                    blend: Blend::Alpha,
                    composite: None,
                    stages: vec![PreparedStage {
                        targets: BTreeMap::new(),
                        passes: vec![PreparedPass {
                            label: "genericimage".into(),
                            shader,
                            textures: vec![TextureBinding::Static(0)],
                            constants,
                            target: None,
                            blend: Blend::Replace,
                        }],
                    }],
                }],
                summary: super::super::SceneSummary {
                    layer_count: 1,
                    effect_count: 0,
                    pass_count: 1,
                    animated: negative_scroll,
                },
            };
            let mut renderer = prepared.initialize()?;
            assert_eq!(renderer.textures[0].source_flags, flags);
            assert_eq!(renderer.scratch[&(2, 2)][0].texture.source_flags, 2);
            assert_eq!(
                renderer.gpu_payload_bytes(),
                prepared.estimated_gpu_bytes()?
            );
            assert!(renderer.render(2, 2, 0.0)?);
            if negative_scroll {
                assert!(renderer.render(2, 2, 1.0)?);
            }
            let mut pixels = [0_u8; 16];
            unsafe {
                gl::ReadPixels(
                    0,
                    0,
                    2,
                    2,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    pixels.as_mut_ptr().cast(),
                );
            }
            for (actual, expected) in pixels.into_iter().zip(expected) {
                assert!(
                    actual.abs_diff(expected) <= 1,
                    "legacy padding/Y/wrap/tint result {pixels:?} for {expression}, flags={flags}, tint={tint:?}"
                );
            }
            renderer.cleanup();
            assert_eq!(renderer.gpu_payload_bytes(), 0);
        }
        check_gl("offscreen legacy sampling")?;
        instance.make_current(display, None, None, None)?;
        instance.destroy_surface(display, surface)?;
        instance.destroy_context(display, context)?;
        instance.terminate(display)?;
        Ok(())
    }

    #[test]
    #[ignore = "requires an explicit EGL/OpenGL offscreen acceptance run"]
    fn offscreen_pixels_and_gl_state_are_preserved() -> Result<()> {
        use khronos_egl as egl;
        let instance = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required()? };
        let display = unsafe { instance.get_display(egl::DEFAULT_DISPLAY) }
            .context("No offscreen EGL display")?;
        instance.initialize(display)?;
        instance.bind_api(egl::OPENGL_API)?;
        let config = instance
            .choose_first_config(
                display,
                &[
                    egl::SURFACE_TYPE,
                    egl::PBUFFER_BIT,
                    egl::RENDERABLE_TYPE,
                    egl::OPENGL_BIT,
                    egl::RED_SIZE,
                    8,
                    egl::GREEN_SIZE,
                    8,
                    egl::BLUE_SIZE,
                    8,
                    egl::ALPHA_SIZE,
                    8,
                    egl::NONE,
                ],
            )?
            .context("No EGL configuration")?;
        let context = instance.create_context(
            display,
            config,
            None,
            &[
                egl::CONTEXT_MAJOR_VERSION,
                3,
                egl::CONTEXT_MINOR_VERSION,
                0,
                egl::NONE,
            ],
        )?;
        let surface = instance.create_pbuffer_surface(
            display,
            config,
            &[egl::WIDTH, 2, egl::HEIGHT, 2, egl::NONE],
        )?;
        instance.make_current(display, Some(surface), Some(surface), Some(context))?;
        gl::load_with(|name| {
            instance
                .get_proc_address(name)
                .map(|function| function as *const std::ffi::c_void)
                .unwrap_or(std::ptr::null())
        });
        let prepared = PreparedScene {
            scripts: None,
            script_layers: BTreeMap::new(),
            audio: Vec::new(),
            bloom: None,
            particles: Vec::new(),
            draw_order: vec![super::super::DrawObject::Image(0)],
            canvas: [2.0, 2.0],
            clear: [0.0, 0.0, 0.0, 1.0],
            textures: vec![PreparedTexture {
                width: 2,
                height: 2,
                image_width: 2,
                image_height: 2,
                source_format: 0,
                source_flags: 2,
                animation: None,
                rgba: vec![
                    255, 0, 0, 255, 255, 255, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255,
                ],
            }],
            layers: vec![PreparedLayer {
                size: [2.0, 2.0],
                origin: [1.0, 1.0],
                scale: [1.0; 2],
                angle: 0.0,
                tint: [1.0; 4],
                blend: Blend::Alpha,
                composite: None,
                stages: vec![PreparedStage {
                    targets: BTreeMap::new(),
                    passes: vec![PreparedPass {
                        label: "authored orientation fixture".into(),
                        shader: super::super::shader::image(),
                        textures: vec![TextureBinding::Static(0)],
                        constants: BTreeMap::new(),
                        target: None,
                        blend: Blend::Replace,
                    }],
                }],
            }],
            summary: super::super::SceneSummary {
                layer_count: 1,
                effect_count: 0,
                pass_count: 1,
                animated: false,
            },
        };
        // Caller-owned state deliberately conflicts with Scene rendering. In
        // particular a nonzero unpack PBO/row stride must not corrupt uploads.
        let mut caller_vao = 0;
        let mut caller_buffer = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut caller_vao);
            gl::BindVertexArray(caller_vao);
            gl::GenBuffers(1, &mut caller_buffer);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, caller_buffer);
            gl::BufferData(
                gl::PIXEL_UNPACK_BUFFER,
                64,
                std::ptr::null(),
                gl::STATIC_DRAW,
            );
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 8);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 3);
            gl::PixelStorei(gl::UNPACK_SKIP_PIXELS, 1);
            gl::ActiveTexture(gl::TEXTURE0 + 5);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(1, 1, 1, 1);
            gl::ColorMask(gl::FALSE, gl::TRUE, gl::FALSE, gl::TRUE);
            gl::Enable(gl::BLEND);
            gl::BlendFuncSeparate(gl::ONE, gl::DST_ALPHA, gl::ZERO, gl::ONE);
            gl::BlendEquationSeparate(gl::FUNC_REVERSE_SUBTRACT, gl::FUNC_SUBTRACT);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        }
        let before = GlState::capture();
        let mut renderer = prepared.initialize()?;
        assert_eq!(renderer.scratch[&(2, 2)].len(), 1);
        assert_eq!(
            renderer.gpu_payload_bytes(),
            prepared.estimated_gpu_bytes()?
        );
        assert_eq!(
            before,
            GlState::capture(),
            "initialization changed caller GL state"
        );
        assert!(renderer.render(2, 2, 0.0)?);
        let after = GlState::capture();
        assert_eq!(before, after, "rendering changed caller GL state");
        drop(after);
        drop(before);
        let mut pixels = [0_u8; 16];
        unsafe {
            gl::ReadPixels(
                0,
                0,
                2,
                2,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast(),
            );
        }
        assert_eq!(
            pixels,
            [
                0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 255, 0, 255
            ],
            "pixels lost orientation or caller state interfered with drawing"
        );
        assert!(
            !renderer.render(2, 2, 1.0)?,
            "static scene redrew without invalidation"
        );
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::ClearColor(0.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        assert!(
            !renderer.render(2, 2, 2.0)?,
            "normal static rendering must remain idle"
        );
        renderer.pause();
        assert!(
            renderer.render_forced(2, 2, 3.0)?,
            "rollback must redraw even a paused static scene"
        );
        unsafe {
            gl::ReadPixels(
                0,
                0,
                2,
                2,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast(),
            );
        }
        assert_eq!(
            pixels,
            [
                0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 255, 0, 255
            ],
            "rollback retained the failed candidate's pixels"
        );
        renderer.cleanup();
        drop(renderer);
        let mut composite = prepared;
        composite.clear = [0.25, 0.5, 0.75, 1.0];
        composite.textures[0].rgba = [255, 0, 0, 128].repeat(4);
        composite.textures.push(PreparedTexture {
            width: 2,
            height: 2,
            image_width: 2,
            image_height: 2,
            source_format: 0,
            source_flags: 2,
            animation: None,
            rgba: [0, 0, 255, 128].repeat(4),
        });
        composite.layers[0].stages[0].passes = (0..2)
            .map(|index| PreparedPass {
                label: "authored alpha material pass".into(),
                shader: super::super::shader::image_material(),
                textures: vec![TextureBinding::Static(index), TextureBinding::Input],
                constants: BTreeMap::new(),
                target: None,
                blend: Blend::Alpha,
            })
            .collect();
        let mut renderer = composite.initialize()?;
        assert_eq!(renderer.scratch[&(2, 2)].len(), 2);
        assert!(renderer.render(2, 2, 0.0)?);
        unsafe {
            gl::ReadPixels(
                0,
                0,
                2,
                2,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast(),
            );
        }
        for pixel in pixels.as_chunks::<4>().0 {
            assert!(
                pixel[0].abs_diff(80) <= 2
                    && pixel[1].abs_diff(32) <= 2
                    && pixel[2].abs_diff(176) <= 2
                    && pixel[3] == 255,
                "material pass alpha was lost or multiplied twice: {pixel:?}"
            );
        }
        renderer.cleanup();
        assert_eq!(renderer.gpu_payload_bytes(), 0);
        drop(renderer);

        // Reusing two scratch targets must retain each effect's original input
        // across multiple unnamed writes, including a half-size named target.
        composite.textures[0].rgba = [51, 0, 0, 255].repeat(4);
        composite.layers[0].stages[0].passes.truncate(1);
        composite.layers[0].stages[0].passes[0].shader = super::super::shader::image();
        composite.layers[0].stages[0].passes[0].textures = vec![TextureBinding::Static(0)];
        let effect_pass = |target: Option<&str>, expression: &str, bindings| {
            let mut shader = super::super::shader::image();
            shader.fragment = format!(
                "#version 130\nin vec2 v_UV; out vec4 lwe_FragColor; uniform sampler2D g_Texture0; uniform sampler2D g_Texture1; void main(){{lwe_FragColor={expression};}}"
            );
            PreparedPass {
                label: "authored retained effect input".into(),
                shader,
                textures: bindings,
                constants: BTreeMap::new(),
                target: target.map(str::to_owned),
                blend: Blend::Replace,
            }
        };
        composite.layers[0].stages.push(PreparedStage {
            targets: BTreeMap::from([("saved".into(), 2.0)]),
            passes: vec![
                effect_pass(
                    Some("saved"),
                    "texture(g_Texture0,v_UV)",
                    vec![TextureBinding::Input],
                ),
                effect_pass(
                    None,
                    "texture(g_Texture0,v_UV)+vec4(0.1,0,0,0)",
                    vec![TextureBinding::Input],
                ),
                effect_pass(
                    None,
                    "vec4(texture(g_Texture0,v_UV).r+texture(g_Texture1,v_UV).r,0,0,1)",
                    vec![TextureBinding::Input, TextureBinding::Named("saved".into())],
                ),
            ],
        });
        composite.layers[0].stages.push(PreparedStage {
            targets: BTreeMap::new(),
            passes: vec![effect_pass(
                None,
                "texture(g_Texture0,v_UV)",
                vec![TextureBinding::Input],
            )],
        });
        let mut renderer = composite.initialize()?;
        assert_eq!(renderer.scratch[&(2, 2)].len(), 2);
        assert!(renderer.render(2, 2, 0.0)?);
        unsafe {
            gl::ReadPixels(
                0,
                0,
                2,
                2,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast(),
            );
        }
        for pixel in pixels.as_chunks::<4>().0 {
            assert_eq!(
                pixel,
                &[102, 0, 0, 255],
                "effect input or named target was overwritten"
            );
        }
        renderer.cleanup();
        drop(renderer);
        unsafe {
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::DeleteBuffers(1, &caller_buffer);
            gl::DeleteVertexArrays(1, &caller_vao);
        }
        check_gl("offscreen state acceptance")?;
        instance.make_current(display, None, None, None)?;
        instance.destroy_surface(display, surface)?;
        instance.destroy_context(display, context)?;
        instance.terminate(display)?;
        Ok(())
    }

    #[test]
    fn aspect_cover_and_image_transform_are_centered() {
        let layer = GpuLayer {
            size: [1920.0, 1080.0],
            origin: [960.0, 540.0],
            scale: [1.0; 2],
            angle: 0.0,
            tint: [1.0; 4],
            blend: Blend::Alpha,
            stages: Vec::new(),
            composite: None,
        };
        assert_eq!(layer_matrix([1920.0, 1080.0], &layer, 1920, 1080), IDENTITY);
        let matrix = layer_matrix([1920.0, 1080.0], &layer, 1080, 1920);
        assert!(matrix[0] > 1.0);
        assert_eq!(matrix[5], 1.0);
        assert_eq!([matrix[12], matrix[13]], [0.0, 0.0]);
    }
}
#[test]
fn long_compositor_gaps_resume_without_unbounded_particle_catchup() {
    assert_eq!(animation_delta(12.0, 72.0), 0.25);
    assert_eq!(animation_delta(12.0, 11.0), 0.0);
    assert_eq!(animation_delta(12.0, 13.0), 1.0);
    assert!((animation_delta(12.0, 12.033) - 0.033).abs() < 1e-9);
}
