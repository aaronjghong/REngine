use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceCreateFlags, InstanceExtensions};
use vulkano::device::{Device, Queue, DeviceExtensions};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::swapchain::{Surface};
use vulkano::device::physical::PhysicalDevice;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

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
    shaders: Arc<shader::Shaders<'a>>,
    window: Arc<Window>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    event_loop: EventLoop<()>,
}

impl<'a> VkApp<'a> {
    pub fn new() -> VkApp<'a> {
        let event_loop = EventLoop::new();
        let required_extensions = Surface::required_extensions(&event_loop);
        let instance = create_instance(required_extensions);
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

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
        let shaders = Arc::new(shader::Shaders::new(device.clone()));
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
