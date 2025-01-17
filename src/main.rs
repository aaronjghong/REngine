use vulkano::instance::{Instance};
use vulkano::device::{Device, Queue};
use vulkano::memory::allocator::{StandardMemoryAllocator};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use std::sync::Arc;

mod vk;

struct App {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
}

impl App {
    fn new() -> App {
        let instance = vk::create_instance();
        let (device, queue) = vk::device::create_device(instance.clone());
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        App {
            instance,
            device,
            queue,
            command_buffer_allocator,
            memory_allocator,
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