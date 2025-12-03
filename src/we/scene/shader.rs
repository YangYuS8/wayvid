//! OpenGL shader management for scene rendering
//!
//! Provides shader compilation, linking, and uniform management.

// Allow dead code for public API items
#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use std::ffi::CString;
use std::ptr;
use tracing::{debug, error};

/// Vertex shader for scene layers
pub const LAYER_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_texcoord;

out vec2 v_texcoord;

uniform mat4 u_projection;
uniform mat4 u_transform;

void main() {
    gl_Position = u_projection * u_transform * vec4(a_position, 0.0, 1.0);
    v_texcoord = a_texcoord;
}
"#;

/// Fragment shader for scene layers with alpha and color tinting
pub const LAYER_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec2 v_texcoord;
out vec4 FragColor;

uniform sampler2D u_texture;
uniform float u_alpha;
uniform vec3 u_color;
uniform float u_brightness;
uniform int u_blend_mode;

void main() {
    vec4 texColor = texture(u_texture, v_texcoord);
    
    // Apply color tint
    vec3 tintedColor = texColor.rgb * u_color * u_brightness;
    
    // Apply alpha
    float finalAlpha = texColor.a * u_alpha;
    
    FragColor = vec4(tintedColor, finalAlpha);
}
"#;

/// Fragment shader for solid color layers
pub const SOLID_FRAGMENT_SHADER: &str = r#"
#version 330 core

out vec4 FragColor;

uniform vec4 u_solid_color;

void main() {
    FragColor = u_solid_color;
}
"#;

/// OpenGL shader program
pub struct ShaderProgram {
    pub id: u32,
}

impl ShaderProgram {
    /// Create a new shader program from vertex and fragment shader sources
    pub fn new(vertex_source: &str, fragment_source: &str) -> Result<Self> {
        unsafe {
            // Compile vertex shader
            let vertex_shader = Self::compile_shader(gl::VERTEX_SHADER, vertex_source)
                .context("Failed to compile vertex shader")?;

            // Compile fragment shader
            let fragment_shader = Self::compile_shader(gl::FRAGMENT_SHADER, fragment_source)
                .context("Failed to compile fragment shader")?;

            // Link program
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            // Check link status
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut log_len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);
                let mut log = vec![0u8; log_len as usize];
                gl::GetProgramInfoLog(
                    program,
                    log_len,
                    ptr::null_mut(),
                    log.as_mut_ptr() as *mut _,
                );
                let error_msg = String::from_utf8_lossy(&log);
                error!("Shader link error: {}", error_msg);
                return Err(anyhow!("Shader link error: {}", error_msg));
            }

            // Clean up individual shaders (they're now part of the program)
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            debug!("Shader program created: {}", program);
            Ok(Self { id: program })
        }
    }

    /// Compile a shader
    unsafe fn compile_shader(shader_type: u32, source: &str) -> Result<u32> {
        let shader = gl::CreateShader(shader_type);
        let c_source = CString::new(source).unwrap();
        gl::ShaderSource(shader, 1, &c_source.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check compile status
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut log_len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
            let mut log = vec![0u8; log_len as usize];
            gl::GetShaderInfoLog(shader, log_len, ptr::null_mut(), log.as_mut_ptr() as *mut _);
            let error_msg = String::from_utf8_lossy(&log);
            let shader_type_name = if shader_type == gl::VERTEX_SHADER {
                "vertex"
            } else {
                "fragment"
            };
            error!("{} shader compile error: {}", shader_type_name, error_msg);
            return Err(anyhow!(
                "{} shader compile error: {}",
                shader_type_name,
                error_msg
            ));
        }

        Ok(shader)
    }

    /// Use this shader program
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Get uniform location
    pub fn get_uniform_location(&self, name: &str) -> i32 {
        let c_name = CString::new(name).unwrap();
        unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr()) }
    }

    /// Set uniform float
    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(self.get_uniform_location(name), value);
        }
    }

    /// Set uniform int
    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.get_uniform_location(name), value);
        }
    }

    /// Set uniform vec3
    pub fn set_vec3(&self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            gl::Uniform3f(self.get_uniform_location(name), x, y, z);
        }
    }

    /// Set uniform vec4
    pub fn set_vec4(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            gl::Uniform4f(self.get_uniform_location(name), x, y, z, w);
        }
    }

    /// Set uniform mat4
    pub fn set_mat4(&self, name: &str, matrix: &[f32; 16]) {
        unsafe {
            gl::UniformMatrix4fv(
                self.get_uniform_location(name),
                1,
                gl::FALSE,
                matrix.as_ptr(),
            );
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
        debug!("Shader program deleted: {}", self.id);
    }
}

/// Quad mesh for rendering layers
pub struct QuadMesh {
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl QuadMesh {
    /// Create a unit quad (0,0 to 1,1)
    pub fn new() -> Result<Self> {
        // Vertices: position (x, y), texcoord (u, v)
        #[rustfmt::skip]
        let vertices: [f32; 16] = [
            // pos      // tex
            0.0, 0.0,   0.0, 1.0,  // bottom-left (tex flipped for OpenGL)
            1.0, 0.0,   1.0, 1.0,  // bottom-right
            1.0, 1.0,   1.0, 0.0,  // top-right
            0.0, 1.0,   0.0, 0.0,  // top-left
        ];

        let indices: [u32; 6] = [
            0, 1, 2, // first triangle
            2, 3, 0, // second triangle
        ];

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            // Upload vertex data
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Upload index data
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Position attribute (location 0)
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Texcoord attribute (location 1)
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                (2 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);
        }

        debug!("QuadMesh created: VAO={}", vao);
        Ok(Self { vao, vbo, ebo })
    }

    /// Bind and draw the quad
    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for QuadMesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
        debug!("QuadMesh deleted: VAO={}", self.vao);
    }
}

/// Create orthographic projection matrix
pub fn ortho_matrix(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [f32; 16] {
    let mut m = [0.0f32; 16];
    m[0] = 2.0 / (right - left);
    m[5] = 2.0 / (top - bottom);
    m[10] = -2.0 / (far - near);
    m[12] = -(right + left) / (right - left);
    m[13] = -(top + bottom) / (top - bottom);
    m[14] = -(far + near) / (far - near);
    m[15] = 1.0;
    m
}

/// Create translation matrix
pub fn translation_matrix(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

/// Create scale matrix
pub fn scale_matrix(sx: f32, sy: f32, sz: f32) -> [f32; 16] {
    [
        sx, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, sz, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Create rotation matrix (Z-axis, angle in radians)
pub fn rotation_z_matrix(angle: f32) -> [f32; 16] {
    let c = angle.cos();
    let s = angle.sin();
    [
        c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Multiply two 4x4 matrices
pub fn mat4_multiply(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0f32; 16];
    for i in 0..4 {
        for j in 0..4 {
            result[i * 4 + j] = a[i * 4] * b[j]
                + a[i * 4 + 1] * b[4 + j]
                + a[i * 4 + 2] * b[8 + j]
                + a[i * 4 + 3] * b[12 + j];
        }
    }
    result
}

/// Identity matrix
#[allow(dead_code)]
pub fn identity_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
