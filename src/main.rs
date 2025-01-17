use vulkano::instance::{Instance};
use vulkano::device::{Device, Queue};
use vulkano::memory::allocator::{StandardMemoryAllocator};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use std::sync::Arc;

mod vk;

struct App<'a> {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    shaders: vk::shader::Shaders<'a>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        let instance = vk::create_instance();
        let (device, queue) = vk::device::create_device(instance.clone());
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let shaders = vk::shader::Shaders::new(device.clone());
        App {
            instance,
            device,
            queue,
            command_buffer_allocator,
            memory_allocator,
            shaders,
        }
    }

    fn run(&self) {
        println!("Running App");
    }
}

fn main() {
    let app = App::new();
    app.run();
}