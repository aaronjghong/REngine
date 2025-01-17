use vulkano::instance::Instance;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, Queue};
use vulkano::device::QueueFlags;
use std::sync::Arc;

// Creates a device and a queue
// Note, takes the first physical device it finds and uses the first queue family that supports graphics
// Also, takes the first queue in the first queue family that supports graphics
pub fn create_device(instance: Arc<Instance>) -> (Arc<Device>, Arc<Queue>) {
    let physical_device =  instance
        .enumerate_physical_devices()
        .expect("Failed to enumerate physical devices")
        .next()
        .expect("No device found");

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .position(|q| q.queue_count > 0 && q.queue_flags.contains(QueueFlags::GRAPHICS))
        .expect("Failed to find a suitable queue family") as u32;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo{
            queue_create_infos: vec![QueueCreateInfo{
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        }
    ).expect("Failed to create device");

    let queue = queues.next().expect("Failed to get queue");

    (device, queue)
}
