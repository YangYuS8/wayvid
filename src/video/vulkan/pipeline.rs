//! Vulkan render pipeline
//!
//! Graphics pipeline for rendering textured quads (wallpapers).

use super::VulkanDevice;
use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;
use tracing::debug;

/// Render pipeline for textured quad rendering
pub struct RenderPipeline {
    /// Reference to device
    device: Arc<VulkanDevice>,
    /// Render pass
    render_pass: vk::RenderPass,
    /// Pipeline layout
    pipeline_layout: vk::PipelineLayout,
    /// Graphics pipeline
    pipeline: vk::Pipeline,
    /// Descriptor set layout
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub fn new(device: Arc<VulkanDevice>, format: vk::Format) -> Result<Self> {
        // Create render pass
        let render_pass = Self::create_render_pass(&device, format)?;

        // Create descriptor set layout
        let descriptor_set_layout = Self::create_descriptor_set_layout(&device)?;

        // Create pipeline layout
        let pipeline_layout = Self::create_pipeline_layout(&device, descriptor_set_layout)?;

        // Create graphics pipeline
        let pipeline = Self::create_graphics_pipeline(&device, render_pass, pipeline_layout)?;

        Ok(Self {
            device,
            render_pass,
            pipeline_layout,
            pipeline,
            descriptor_set_layout,
        })
    }

    /// Create render pass
    fn create_render_pass(device: &VulkanDevice, format: vk::Format) -> Result<vk::RenderPass> {
        let attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&attachment_ref));

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        unsafe { device.handle().create_render_pass(&create_info, None) }
            .context("Failed to create render pass")
    }

    /// Create descriptor set layout
    fn create_descriptor_set_layout(device: &VulkanDevice) -> Result<vk::DescriptorSetLayout> {
        // Combined image sampler for texture
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let create_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));

        unsafe {
            device
                .handle()
                .create_descriptor_set_layout(&create_info, None)
        }
        .context("Failed to create descriptor set layout")
    }

    /// Create pipeline layout
    fn create_pipeline_layout(
        device: &VulkanDevice,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        // Push constants for transform and alpha
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<PushConstants>() as u32);

        let create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        unsafe { device.handle().create_pipeline_layout(&create_info, None) }
            .context("Failed to create pipeline layout")
    }

    /// Create graphics pipeline
    fn create_graphics_pipeline(
        device: &VulkanDevice,
        render_pass: vk::RenderPass,
        pipeline_layout: vk::PipelineLayout,
    ) -> Result<vk::Pipeline> {
        // Use generated SPIR-V shaders
        use super::shaders::{fragment_shader_spirv, spirv_to_bytes, vertex_shader_spirv};

        let vert_spirv = spirv_to_bytes(&vertex_shader_spirv());
        let frag_spirv = spirv_to_bytes(&fragment_shader_spirv());

        let vert_shader = Self::create_shader_module(device, &vert_spirv)?;
        let frag_shader = Self::create_shader_module(device, &frag_spirv)?;

        let main_name = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader)
                .name(&main_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader)
                .name(&main_name),
        ];

        // Vertex input (fullscreen quad, no vertex buffer needed)
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();

        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // Viewport state (dynamic)
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        // Rasterization
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Color blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        // Dynamic state
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        // Create pipeline
        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device.handle().create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&create_info),
                None,
            )
        }
        .map_err(|e| anyhow::anyhow!("Failed to create graphics pipeline: {:?}", e.1))?[0];

        // Clean up shader modules
        unsafe {
            device.handle().destroy_shader_module(vert_shader, None);
            device.handle().destroy_shader_module(frag_shader, None);
        }

        Ok(pipeline)
    }

    /// Create shader module from SPIR-V
    fn create_shader_module(device: &VulkanDevice, code: &[u8]) -> Result<vk::ShaderModule> {
        // Ensure alignment
        let code_u32: Vec<u32> = code
            .chunks(4)
            .map(|chunk| {
                let mut bytes = [0u8; 4];
                bytes[..chunk.len()].copy_from_slice(chunk);
                u32::from_le_bytes(bytes)
            })
            .collect();

        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_u32);

        unsafe { device.handle().create_shader_module(&create_info, None) }
            .context("Failed to create shader module")
    }

    /// Get render pass
    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    /// Get pipeline
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    /// Get pipeline layout
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    /// Get descriptor set layout
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for RenderPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_pipeline(self.pipeline, None);
            self.device
                .handle()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .handle()
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device
                .handle()
                .destroy_render_pass(self.render_pass, None);
        }
        debug!("Render pipeline destroyed");
    }
}

/// Push constants for transform
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PushConstants {
    /// Transform matrix (4x4)
    pub transform: [[f32; 4]; 4],
    /// Alpha/opacity
    pub alpha: f32,
    /// Padding
    pub _padding: [f32; 3],
}
