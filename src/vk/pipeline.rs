use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo, PipelineBindPoint};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::buffer::{Buffer, Subbuffer};
use vulkano::buffer::BufferContents;
use vulkano::shader::ShaderModule;
use vulkano::device::Device;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo};
use vulkano::command_buffer::{
    RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::image::Image;
use vulkano::format::Format;
use vulkano::image::view::ImageView;

use std::sync::Arc;

use crate::vk::shader::Shaders;
use crate::vk::image::create_image_view;
use crate::vk::buffer::PrimaryCommandBufferBuilder;


pub struct Pipe {
    pub pipeline: Option<Arc<dyn Pipeline>>,
    pub layout: Option<Arc<PipelineLayout>>,
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
    let pipeline_layout = create_pipeline_layout(device.clone(), shaders);
    let stage = create_pipeline_stage_from_shader(shaders.compute.clone().unwrap());
    let compute_pipeline = ComputePipeline::new(device.clone(), None, ComputePipelineCreateInfo::stage_layout(stage, pipeline_layout)).expect("Failed to create compute pipeline");
    compute_pipeline
}

pub fn create_pipeline_layout(device: Arc<Device>, shaders: &Shaders) -> Arc<PipelineLayout> {
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

    PipelineLayout::new(
        device.clone(), 
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&shader_stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap()
    ).unwrap()
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

