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
use vulkano::pipeline::graphics::vertex_input::Vertex;
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

pub struct VkApp<'a> {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    queue_family_index: u32,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    shaders: shader::Shaders<'a>,
    window: Arc<Window>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    event_loop: EventLoop<()>,
    viewport: Viewport,
}

impl<'a> VkApp<'a> {
    pub fn new() -> VkApp<'a> {
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
        }
    }

    pub fn run(self) {
        println!("Running App");
        self.event_loop.run(|event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => ()
            }
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
            Vert { position: [0.0, 0.5, 0.0]},
            Vert { position: [-0.5, -0.5, 0.0]},
            Vert { position: [0.5, -0.5, 0.0]},
        ];

        let vertex_buffer = buffer::create_buffer_from_iter(
            self.memory_allocator.clone(), 
            buffer::UNIFORM_BUFFER_MEMORY_TYPE_FILTER, 
            BufferUsage::VERTEX_BUFFER, 
            vertices.into_iter()
        );

        let vertex_buffer = Arc::new(vertex_buffer);

        self.shaders.load_shader_from_file("vert.vs", "vertex");
        self.shaders.load_shader_from_file("frag.fs", "fragment"); 

        let pipeline = pipeline::create_graphics_pipeline(self.device.clone(), &self.shaders, self.viewport.clone(), self.render_pass.clone());
        let command_buffer_builder = buffer::create_command_buffer_builder(self.command_buffer_allocator.clone(), self.queue.clone());
        let command_buffer_builder = pipeline::record_render_pass(command_buffer_builder, self.render_pass.clone(), self.framebuffers[0].clone(), pipeline, 0, vertex_buffer, 3, 1, 0, 0);
        let command_buffer = buffer::build_command_buffer(command_buffer_builder);
        buffer::submit_execute_wait_fenced(self.device.clone(), self.queue.clone(), command_buffer);
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
