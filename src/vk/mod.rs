use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceCreateFlags, InstanceExtensions};
use vulkano::device::{Device, Queue, DeviceExtensions};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::buffer::BufferUsage;
use vulkano::device::physical::PhysicalDevice;
use vulkano::pipeline::graphics::viewport::Viewport;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::{Window, WindowBuilder};
use vulkano::image::Image;
use vulkano::render_pass::{RenderPass, Framebuffer};
use vulkano::buffer::BufferContents;
use vulkano::pipeline::compute::ComputePipeline;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::GraphicsPipeline;
use vulkano::command_buffer::{PrimaryAutoCommandBuffer};
use vulkano::buffer::{Subbuffer, IndexBuffer};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{GpuFuture};
use vulkano::sync::fence::Fence;
use vulkano::sync;
use vulkano::{VulkanError, Validated};



use std::sync::Arc;


#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
pub struct Vert {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}


mod device;
mod buffer;
mod shader;
mod pipeline;
mod image;

pub struct VkApp {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    queue_family_index: u32,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    shaders: shader::Shaders,
    window: Arc<Window>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    event_loop: EventLoop<()>,
    viewport: Viewport,
    graphics_pipeline: Option<Arc<GraphicsPipeline>>,
    compute_pipeline: Option<Arc<ComputePipeline>>,
    command_buffers: Option<Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>>,
    vertex_buffer: Option<Arc<Subbuffer<[Vert]>>>,
    index_buffer: Option<Arc<IndexBuffer>>,
    previous_fence_idx: u32,
}

impl VkApp {
    pub fn new() -> VkApp {
        let event_loop = EventLoop::new();
        let required_extensions = Surface::required_extensions(&event_loop);
        let instance = create_instance(required_extensions);
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let mut viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };
    

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };        
        let (device, queue, queue_family_index, physical_device) = device::create_device(instance.clone(), device_extensions, surface.clone());
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));
        let mut shaders = shader::Shaders::new(device.clone());

        let (swapchain, swapchain_images) = image::create_swapchain(device.clone(), window.clone(), surface.clone(), physical_device.clone());
        let render_pass = pipeline::create_render_pass(device.clone(), swapchain.image_format());
        let framebuffers = pipeline::create_framebuffers(render_pass.clone(), swapchain_images.clone());


        VkApp {
            instance,
            device,
            queue,
            queue_family_index,
            command_buffer_allocator,
            memory_allocator,
            descriptor_set_allocator,
            shaders,
            window,
            surface,
            physical_device,
            viewport,
            swapchain,
            swapchain_images,
            render_pass,
            framebuffers,
            event_loop,
            graphics_pipeline: None,
            compute_pipeline: None,
            command_buffers: None,
            vertex_buffer: None,
            index_buffer: None,
            previous_fence_idx: 0,
        }
    }

    pub fn run(mut self) {
        println!("Running App");
        let mut window_resized = false;
        let mut recreate_swapchain = false;
        let mut fences: Vec<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture>>>>> = vec![None; self.swapchain_images.len()];


        self.event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    window_resized = true;
                    println!("Window resized to {}x{}", new_size.width, new_size.height);
                }
            }
            Event::MainEventsCleared => {
                let (image_result, needs_recreate) = image::obtain_next_swapchain_image(self.swapchain.clone());
                
                if window_resized || recreate_swapchain || needs_recreate {
                    recreate_swapchain = false;
                    let new_dimensions = self.window.inner_size();
                    
                    if new_dimensions.width > 0 && new_dimensions.height > 0 {
                        let (swapchain, swapchain_images) = image::recreate_swapchain(
                            self.swapchain.clone(),
                            new_dimensions.into()
                        );
                        self.swapchain = swapchain;
                        self.swapchain_images = swapchain_images;
                        self.framebuffers = pipeline::create_framebuffers(
                            self.render_pass.clone(),
                            self.swapchain_images.clone()
                        );
                        
                        if window_resized {
                            window_resized = false;
                            self.viewport.extent = new_dimensions.into();
                            
                            let pipeline = pipeline::create_graphics_pipeline(
                                self.device.clone(),
                                &self.shaders,
                                self.viewport.clone(),
                                self.render_pass.clone()
                            );
                            self.graphics_pipeline = Some(pipeline);
                            
                            if let Some(vertex_buffer) = &self.vertex_buffer {
                                let mut new_command_buffers = Vec::new();
                                for framebuffer in &self.framebuffers {
                                    let command_buffer_builder = buffer::create_command_buffer_builder(
                                        self.command_buffer_allocator.clone(),
                                        self.queue.clone()
                                    );
                                    let command_buffer_builder = pipeline::record_render_pass(
                                        command_buffer_builder,
                                        self.render_pass.clone(),
                                        framebuffer.clone(),
                                        self.graphics_pipeline.as_ref().unwrap().clone(),
                                        0,
                                        vertex_buffer.clone(),
                                        self.index_buffer.as_ref().unwrap().clone(),
                                        self.index_buffer.as_ref().unwrap().len() as u32,
                                        self.index_buffer.as_ref().unwrap().len() as u32 / 3,
                                        0,
                                        0
                                    );
                                    let command_buffer = buffer::build_command_buffer(command_buffer_builder);
                                    new_command_buffers.push(command_buffer);
                                }
                                self.command_buffers = Some(new_command_buffers);
                            }
                        }
                        
                        // Re-acquire image after recreating swapchain
                        let (new_image_result, _) = image::obtain_next_swapchain_image(self.swapchain.clone());
                        if let Some((image_idx, swapchain_future)) = new_image_result {
                            if let Some(fence) = &fences[image_idx as usize] {
                                fence.wait(None).unwrap();
                            }
                            let previous_future = match &fences[self.previous_fence_idx as usize] {
                                Some(fence) => Box::new(fence.clone()) as Box<dyn GpuFuture>,
                                None => {
                                    let mut now = sync::now(self.device.clone());
                                    now.cleanup_finished();
                                    Box::new(now) as Box<dyn GpuFuture>
                                }
                            };
                            recreate_swapchain = image::present_swapchain_image_with_fence(
                                self.device.clone(),
                                self.swapchain.clone(),
                                self.queue.clone(),
                                self.command_buffers.as_ref().unwrap().clone(),
                                image_idx,
                                swapchain_future,
                                previous_future,
                                fences.clone(),
                            );
                            self.previous_fence_idx = image_idx;
                        }
                    }
                } else if let Some((image_idx, swapchain_future)) = image_result {
                    // Normal presentation path (no resize)
                    if let Some(fence) = &fences[image_idx as usize] {
                        fence.wait(None).unwrap();
                    }
                    let previous_future = match &fences[self.previous_fence_idx as usize] {
                        Some(fence) => Box::new(fence.clone()) as Box<dyn GpuFuture>,
                        None => {
                            let mut now = sync::now(self.device.clone());
                            now.cleanup_finished();
                            Box::new(now) as Box<dyn GpuFuture>
                        }
                    };
                    recreate_swapchain = image::present_swapchain_image_with_fence(
                        self.device.clone(),
                        self.swapchain.clone(),
                        self.queue.clone(),
                        self.command_buffers.as_ref().unwrap().clone(),
                        image_idx,
                        swapchain_future,
                        previous_future,
                        fences.clone(),
                    );
                    self.previous_fence_idx = image_idx;
                }
            }
            _ => (),
        });
    }

    // pub fn fractal_sample(&self) {
    //     let image = image::create_image(self.memory_allocator.clone(), Format::R8G8B8A8_UNORM, ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC, ImageType::Dim2d, [1024, 1024, 1]);
    //     let image_view = image::create_image_view(image.clone(), Format::R8G8B8A8_UNORM);
    //     let command_buffer = self.command_buffer_allocator.new_command_buffer(QueueFamily::Graphics).unwrap();
    //     let command_buffer = image::clear_image(command_buffer, image.clone(), [0.0, 0.0, 0.0, 1.0]);
    // }

    pub fn triangle_sample(&mut self) {
        let vertices = vec![
            Vert { position: [-0.5, -0.5, 0.0]}, // UL
            Vert { position: [0.5, -0.5, 0.0]}, // UR
            Vert { position: [0.5, 0.5, 0.0]}, // BR
            Vert { position: [-0.5, 0.5, 0.0]}, // BL
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            2, 3, 0,
        ];

        let num_vertices = indices.len() as u32/3;

        let vertex_buffer = buffer::create_buffer_from_iter(
            self.memory_allocator.clone(), 
            buffer::UNIFORM_BUFFER_MEMORY_TYPE_FILTER, 
            BufferUsage::VERTEX_BUFFER, 
            vertices.into_iter()
        );

        let index_buffer = buffer::create_index_buffer(
            self.memory_allocator.clone(), 
            indices.into_iter()
        );

        let vertex_buffer = Arc::new(vertex_buffer);

        self.shaders.load_shader_from_file("shaders/vert.vs", "vertex");
        self.shaders.load_shader_from_file("shaders/frag.fs", "fragment"); 

        let pipeline = pipeline::create_graphics_pipeline(self.device.clone(), &self.shaders, self.viewport.clone(), self.render_pass.clone());
        self.graphics_pipeline = Some(pipeline);

        let mut new_command_buffers = Vec::new();
        for framebuffer in &self.framebuffers {
            let command_buffer_builder = buffer::create_command_buffer_builder(
                self.command_buffer_allocator.clone(),
                self.queue.clone()
            );
            let command_buffer_builder = pipeline::record_render_pass(
                command_buffer_builder,
                self.render_pass.clone(),
                framebuffer.clone(),
                self.graphics_pipeline.as_ref().unwrap().clone(),
                0,
                vertex_buffer.clone(),
                index_buffer.clone(),
                num_vertices,
                num_vertices / 3,
                0,
                0
            );
            let command_buffer = buffer::build_command_buffer(command_buffer_builder);
            new_command_buffers.push(command_buffer);
        }

        // buffer::submit_execute_wait_fenced(self.device.clone(), self.queue.clone(), command_buffer.clone()); The present call runs then_execute which executes the command buffer

        self.vertex_buffer = Some(vertex_buffer.clone());
        self.index_buffer = Some(index_buffer.clone());
        self.command_buffers = Some(new_command_buffers);
    }   
}


fn create_instance(required_extensions: InstanceExtensions) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("Failed to load Vulkan library");
    let instance = Instance::new(
        library, 
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            ..Default::default()
        }
    ).expect("Failed to create instance");

    instance
}
