use std::sync::Arc;

use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo, PipelineBindPoint};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::graphics::{GraphicsPipeline, GraphicsPipelineCreateInfo};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::buffer::{Buffer, Subbuffer, BufferContents};

use vulkano::render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::command_buffer::{
    RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};

use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::format::{Format, ClearColorValue};

use vulkano::shader::ShaderModule;
use vulkano::device::Device;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;

use crate::vk::shader::Shaders;
use crate::vk::image::create_image_view;
use crate::vk::buffer::PrimaryCommandBufferBuilder;

pub struct Pipe {
    pub pipeline: Option<Arc<dyn Pipeline>>,
    pub layout: Option<Arc<PipelineLayout>>,
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vert {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub fn record_compute_pipeline(mut builder: PrimaryCommandBufferBuilder, pipeline: Arc<ComputePipeline>, set_index: u32, descriptor_set: Arc<PersistentDescriptorSet>, work_group_counts: [u32; 3]) -> PrimaryCommandBufferBuilder {
    builder
        .bind_pipeline_compute(pipeline.clone())
        .unwrap()
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            pipeline.layout().clone(),
            set_index,
            descriptor_set.clone())
        .unwrap()
        .dispatch(work_group_counts)
        .unwrap();
    builder
}

pub fn create_compute_pipeline(device: Arc<Device>, shaders: &Shaders) -> Arc<ComputePipeline> {
    let (pipeline_layout, shader_stages) = create_pipeline_layout(device.clone(), shaders);
    let stage = create_pipeline_stage_from_shader(shaders.compute.clone().unwrap());
    let compute_pipeline = ComputePipeline::new(device.clone(), None, ComputePipelineCreateInfo::stage_layout(stage, pipeline_layout)).expect("Failed to create compute pipeline");
    compute_pipeline
}

pub fn create_pipeline_layout(device: Arc<Device>, shaders: &Shaders) -> (Arc<PipelineLayout>, Vec<PipelineShaderStageCreateInfo>) {
    let mut shader_stages: Vec<PipelineShaderStageCreateInfo> = Vec::new();
    if let Some(vertex) = shaders.vertex.clone() {
        shader_stages.push(create_pipeline_stage_from_shader(vertex));
    }
    if let Some(fragment) = shaders.fragment.clone() {
        shader_stages.push(create_pipeline_stage_from_shader(fragment));
    }
    if let Some(compute) = shaders.compute.clone() {
        shader_stages.push(create_pipeline_stage_from_shader(compute));
    }

    (
        PipelineLayout::new(
            device.clone(), 
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&shader_stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap()
        ).unwrap(),
        shader_stages
    )
}

pub fn create_pipeline_stage_from_shader(shader: Arc<ShaderModule>) -> PipelineShaderStageCreateInfo {
    let stage = PipelineShaderStageCreateInfo::new(shader.entry_point("main").unwrap());
    stage
}

pub fn create_descriptor_set_from_buffer<T: BufferContents>(pipeline_layout: Arc<PipelineLayout>, descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>, set_index: usize, binding_index: usize, buffer: Subbuffer<T>) -> Arc<PersistentDescriptorSet> {
    let layout = pipeline_layout.set_layouts();
    let descriptor_set_layout = layout.get(set_index).unwrap();
    let descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator, 
        descriptor_set_layout.clone(), 
        [WriteDescriptorSet::buffer(binding_index as u32, buffer.clone())], 
        []).unwrap();
    descriptor_set
}

pub fn create_render_pass(device: Arc<Device>, image_format: Format) -> Arc<RenderPass> {
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            c: {
                format: image_format,
                samples: 1,
                load_op: Clear,
                store_op: Store, // Might be more efficient to have store_op: DontCare
            }
        },
        pass: {
            color: [c],
            depth_stencil: {},
        },
    ).expect("Failed to create render pass");
    render_pass
}

pub fn create_framebuffer(render_pass: Arc<RenderPass>, image: Arc<Image>) -> Arc<Framebuffer> {
    let framebuffer = Framebuffer::new(render_pass.clone(), FramebufferCreateInfo{
        attachments: vec![create_image_view(image.clone(), image.format())],
        ..Default::default()
    }).expect("Failed to create framebuffer");
    framebuffer
}

pub fn record_render_pass<T: BufferContents>(mut builder: PrimaryCommandBufferBuilder, render_pass: Arc<RenderPass>, framebuffer: Arc<Framebuffer>, pipeline: Arc<GraphicsPipeline>, set_index: u32, descriptor_set: Arc<PersistentDescriptorSet>, vertex_buffer: Arc<Subbuffer<T>>, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) -> PrimaryCommandBufferBuilder {
    builder
        .begin_render_pass(
            RenderPassBeginInfo{
                clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())], // Only blue for now... 
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo{
                contents: SubpassContents::Inline,
                ..Default::default()    
            },
        )
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone())
        .unwrap()
        .bind_vertex_buffers(0, (*vertex_buffer).clone())
        .unwrap()
        .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), set_index, descriptor_set.clone())
        .unwrap()
        .draw(vertex_count, instance_count, first_vertex, first_instance)
        .unwrap()
        .end_render_pass(SubpassEndInfo::default())
        .unwrap();
    builder
}

pub fn create_graphics_pipeline(device: Arc<Device>, shaders: &Shaders, viewport: Viewport, subpass: Subpass) -> Arc<GraphicsPipeline> {
    let (pipeline_layout, shader_stages) = create_pipeline_layout(device.clone(), shaders);
    let stage = create_pipeline_stage_from_shader(shaders.fragment.clone().unwrap());
    let vertex_shader = shaders.vertex.clone().unwrap();
    let vertex_definition = Vert::per_vertex()
        .definition(&vertex_shader.entry_point("main").unwrap().info().input_interface)
        .unwrap();
    let graphics_pipeline = GraphicsPipeline::new(
        device.clone(), 
        None, 
        GraphicsPipelineCreateInfo{
            stages: shader_stages.into_iter().collect(),
            vertex_input_state: Some(vertex_definition),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState{
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::default()),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(pipeline_layout)
        }).expect("Failed to create graphics pipeline");
    graphics_pipeline
}

