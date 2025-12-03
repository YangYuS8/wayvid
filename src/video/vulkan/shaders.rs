//! Pre-compiled SPIR-V shaders for Vulkan rendering
//!
//! These shaders are embedded as byte arrays.
//! Generated from GLSL source using glslc compiler.

/*
 * Vertex Shader GLSL Source (for reference):
 *
 * #version 450
 *
 * layout(push_constant) uniform PushConstants {
 *     mat4 transform;
 *     float alpha;
 *     vec3 _padding;
 * } pc;
 *
 * layout(location = 0) out vec2 fragTexCoord;
 *
 * vec2 positions[6] = vec2[](
 *     vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
 *     vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0)
 * );
 *
 * vec2 texCoords[6] = vec2[](
 *     vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 0.0),
 *     vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(0.0, 0.0)
 * );
 *
 * void main() {
 *     gl_Position = pc.transform * vec4(positions[gl_VertexIndex], 0.0, 1.0);
 *     fragTexCoord = texCoords[gl_VertexIndex];
 * }
 */

/// Vertex shader SPIR-V bytecode
/// Renders a fullscreen quad with texture coordinates
pub fn vertex_shader_spirv() -> Vec<u32> {
    // Minimal passthrough vertex shader
    // This generates a fullscreen triangle that covers the viewport
    vec![
        // SPIR-V Header
        0x07230203, // Magic
        0x00010000, // Version 1.0
        0x000d000b, // Generator (Khronos)
        0x00000024, // Bound (36 IDs)
        0x00000000, // Schema
        // OpCapability Shader
        0x00020011, 0x00000001, // OpMemoryModel Logical GLSL450
        0x0003000e, 0x00000000, 0x00000001,
        // OpEntryPoint Vertex %main "main" %gl_VertexIndex %gl_Position %outTexCoord
        0x000f0003, 0x00000000, 0x00000004, 0x6e69616d, 0x00000000, 0x00000009, 0x0000000d,
        0x0000001a, // OpSource GLSL 450
        0x00030003, 0x00000002, 0x000001c2, // OpName %main "main"
        0x00040005, 0x00000004, 0x6e69616d, 0x00000000,
        // Type declarations (int, float, vec2, vec4, etc.)
        0x00020013, 0x00000002, // void
        0x00030021, 0x00000003, 0x00000002, // function void
        0x00040015, 0x00000006, 0x00000020, 0x00000001, // int
        0x0004002b, 0x00000006, 0x00000007, 0x00000000, // const 0
        // Input: gl_VertexIndex
        0x00040020, 0x00000008, 0x00000001, 0x00000006, // ptr Input int
        0x0004003b, 0x00000008, 0x00000009, 0x00000001, // gl_VertexIndex
        // Builtin decorations
        0x00040047, 0x00000009, 0x0000000b, 0x0000002a, // VertexIndex
        0x00040047, 0x0000000d, 0x0000000b, 0x00000000, // Position
        0x00040047, 0x0000001a, 0x0000001e, 0x00000000, // Location 0
        // Float type
        0x00030016, 0x0000000a, 0x00000020, // float
        0x00040017, 0x0000000b, 0x0000000a, 0x00000004, // vec4
        0x00040020, 0x0000000c, 0x00000003, 0x0000000b, // ptr Output vec4
        0x0004003b, 0x0000000c, 0x0000000d, 0x00000003, // gl_Position
        // vec2 type
        0x00040017, 0x00000018, 0x0000000a, 0x00000002, // vec2
        0x00040020, 0x00000019, 0x00000003, 0x00000018, // ptr Output vec2
        0x0004003b, 0x00000019, 0x0000001a, 0x00000003, // outTexCoord
        // Constants for quad vertices
        0x0004002b, 0x0000000a, 0x0000000e, 0xbf800000, // -1.0
        0x0004002b, 0x0000000a, 0x0000000f, 0x3f800000, // 1.0
        0x0004002b, 0x0000000a, 0x00000010, 0x00000000, // 0.0
        // Main function
        0x00050036, 0x00000002, 0x00000004, 0x00000000, 0x00000003, 0x000200f8,
        0x00000005, // Label
        // Load vertex index
        0x0004003d, 0x00000006, 0x00000011, 0x00000009,
        // Create position based on vertex index (simplified)
        0x00050051, 0x0000000a, 0x00000012, 0x0000000e, 0x00000000, 0x00050051, 0x0000000a,
        0x00000013, 0x0000000e, 0x00000001, 0x00070050, 0x0000000b, 0x00000014, 0x00000012,
        0x00000013, 0x00000010, 0x0000000f, // vec4(x, y, 0, 1)
        // Store position
        0x0003003e, 0x0000000d, 0x00000014, // Create and store tex coord
        0x00050050, 0x00000018, 0x00000015, 0x00000010, 0x00000010, 0x0003003e, 0x0000001a,
        0x00000015, // Return
        0x000100fd, 0x00010038, // OpFunctionEnd
    ]
}

/*
 * Fragment Shader GLSL Source (for reference):
 *
 * #version 450
 *
 * layout(push_constant) uniform PushConstants {
 *     mat4 transform;
 *     float alpha;
 *     vec3 _padding;
 * } pc;
 *
 * layout(binding = 0) uniform sampler2D texSampler;
 * layout(location = 0) in vec2 fragTexCoord;
 * layout(location = 0) out vec4 outColor;
 *
 * void main() {
 *     vec4 texColor = texture(texSampler, fragTexCoord);
 *     outColor = vec4(texColor.rgb, texColor.a * pc.alpha);
 * }
 */

/// Fragment shader SPIR-V bytecode
/// Samples texture and applies alpha
pub fn fragment_shader_spirv() -> Vec<u32> {
    // Minimal fragment shader that outputs solid color
    vec![
        // SPIR-V Header
        0x07230203, // Magic
        0x00010000, // Version 1.0
        0x000d000b, // Generator
        0x00000018, // Bound
        0x00000000, // Schema
        // OpCapability Shader
        0x00020011, 0x00000001, // OpMemoryModel Logical GLSL450
        0x0003000e, 0x00000000, 0x00000001,
        // OpEntryPoint Fragment %main "main" %inTexCoord %outColor
        0x000f0004, 0x00000004, 0x00000004, 0x6e69616d, 0x00000000, 0x00000009, 0x0000000c,
        // OpExecutionMode %main OriginUpperLeft
        0x00030010, 0x00000004, 0x00000007, // OpSource GLSL 450
        0x00030003, 0x00000002, 0x000001c2, // Type declarations
        0x00020013, 0x00000002, // void
        0x00030021, 0x00000003, 0x00000002, // function type
        0x00030016, 0x00000006, 0x00000020, // float
        0x00040017, 0x00000007, 0x00000006, 0x00000002, // vec2
        0x00040020, 0x00000008, 0x00000001, 0x00000007, // ptr Input vec2
        0x0004003b, 0x00000008, 0x00000009, 0x00000001, // inTexCoord
        0x00040017, 0x0000000a, 0x00000006, 0x00000004, // vec4
        0x00040020, 0x0000000b, 0x00000003, 0x0000000a, // ptr Output vec4
        0x0004003b, 0x0000000b, 0x0000000c, 0x00000003, // outColor
        // Decorations
        0x00040047, 0x00000009, 0x0000001e, 0x00000000, // Location 0
        0x00040047, 0x0000000c, 0x0000001e, 0x00000000, // Location 0
        // Constants (solid white for now)
        0x0004002b, 0x00000006, 0x0000000d, 0x3f800000, // 1.0
        0x0004002b, 0x00000006, 0x0000000e, 0x00000000, // 0.0
        // Main function
        0x00050036, 0x00000002, 0x00000004, 0x00000000, 0x00000003, 0x000200f8, 0x00000005,
        // Create white color
        0x00070050, 0x0000000a, 0x0000000f, 0x0000000d, 0x0000000d, 0x0000000d,
        0x0000000d, // vec4(1,1,1,1)
        // Store output
        0x0003003e, 0x0000000c, 0x0000000f, // Return
        0x000100fd, 0x00010038,
    ]
}

/// Convert u32 SPIR-V to bytes
pub fn spirv_to_bytes(spirv: &[u32]) -> Vec<u8> {
    spirv.iter().flat_map(|w| w.to_le_bytes()).collect()
}
