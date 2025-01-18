use vulkano::instance::Instance;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, Queue};
use vulkano::device::QueueFlags;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::swapchain::Surface;
use vulkano::swapchain::SurfaceCapabilities;
use std::sync::Arc;

// Creates a device and a queue
// Note, takes the first physical device it finds and uses the first queue family that supports graphics
// Also, takes the first queue in the first queue family that supports graphics
pub fn create_device(instance: Arc<Instance>, device_extensions: DeviceExtensions, surface: Arc<Surface>) -> (Arc<Device>, Arc<Queue>, u32, Arc<PhysicalDevice>) {
    let (physical_device, queue_family_index) =  instance
        .enumerate_physical_devices()
        .expect("Failed to enumerate physical devices")
        // Filter by extensions
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        // Filter by queue family support
        .filter_map(|p| {
            p.queue_family_properties()
            .iter()
            .enumerate()
            // Find the first queue family that supports graphics and has surface support
            .position(|(i, q)| {
                q.queue_flags.contains(QueueFlags::GRAPHICS) && p.surface_support(i as u32, &surface).unwrap_or(false)
            })
            .map(|q| (p, q as u32))
        })
        // Prioritize discrete GPUs
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("No device found");

    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo{
            queue_create_infos: vec![QueueCreateInfo{
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        }
    ).expect("Failed to create device");

    let queue = queues.next().expect("Failed to get queue");

    (device, queue, queue_family_index, physical_device)
}