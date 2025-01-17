use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use std::sync::Arc;

pub mod device;
pub mod buffer;

pub fn create_instance() -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("Failed to load Vulkan library");
    let instance = Instance::new(
        library, 
        InstanceCreateInfo::default()
    ).expect("Failed to create instance");

    instance
}
