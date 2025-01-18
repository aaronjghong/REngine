use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::format::{Format, ClearColorValue};
use vulkano::memory::allocator::{
    StandardMemoryAllocator,
    AllocationCreateInfo,
    MemoryTypeFilter,
};
use vulkano::command_buffer::ClearColorImageInfo;
use vulkano::swapchain::{self, SwapchainAcquireFuture};
use vulkano::swapchain::{Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, Surface, CompositeAlphas};
use vulkano::{Validated, VulkanError};
use vulkano::device::Device;
use vulkano::device::physical::PhysicalDevice; 
use vulkano::device::Queue;
use vulkano::sync::{self, GpuFuture};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::command_buffer::{PrimaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use std::sync::Arc;
use crate::vk::buffer::PrimaryCommandBufferBuilder;
use winit::window::Window;
pub fn create_image(memory_allocator: Arc<StandardMemoryAllocator>, format: Format, usage: ImageUsage, image_type: ImageType, dimensions: [u32; 3]) -> Arc<Image> {
    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: image_type,
            format: format,
            extent: dimensions,
            usage: usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        }
    ).unwrap();
    image
}

pub fn create_image_view(image: Arc<Image>, format: Format) -> Arc<ImageView> {
    // ImageView::new(image, ImageViewCreateInfo {
    //     format: format,
    //     ..Default::default()
    // }).unwrap()
    
    ImageView::new_default(image).unwrap()
}

// color needs to be in the range [0, 1], normalized to the format
// i.e. R8G8B8A8_UNORM is [0, 255] -> [0.0, 1.0] for each channel
pub fn clear_image(mut builder: PrimaryCommandBufferBuilder, image: Arc<Image>, color: [f32; 4]) -> PrimaryCommandBufferBuilder {
    builder.clear_color_image(
        ClearColorImageInfo {
            clear_value: ClearColorValue::Float(color),
            ..ClearColorImageInfo::image(image.clone())
        }
    ).unwrap();
    builder
}

pub fn create_swapchain(device: Arc<Device>, window: Arc<Window>, surface: Arc<Surface>, physical_device: Arc<PhysicalDevice>) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface_capabilities = physical_device.surface_capabilities(&surface, Default::default()).expect("Failed to get surface capabilities");
    let image_extent = [window.inner_size().width, window.inner_size().height];
    let image_usage = ImageUsage::COLOR_ATTACHMENT;
    let image_format = physical_device.surface_formats(&surface, Default::default()).unwrap()[0].0;
    let composite_alpha = surface_capabilities.supported_composite_alpha.into_iter().next().unwrap();
    Swapchain::new(device, surface, SwapchainCreateInfo {
        min_image_count: surface_capabilities.min_image_count + 1,
        image_format,
        image_extent: image_extent,
        image_usage: image_usage,
        composite_alpha,
        ..Default::default()
    }).unwrap()
}

pub fn recreate_swapchain(swapchain: Arc<Swapchain>, dimensions: [u32; 2]) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let (swapchain, swapchain_images) = swapchain.recreate(SwapchainCreateInfo {
        image_extent: dimensions,
        ..swapchain.create_info()
     }).unwrap();
    (swapchain, swapchain_images)
}

pub fn obtain_next_swapchain_image(swapchain: Arc<Swapchain>) -> (Option<(u32, SwapchainAcquireFuture)>, bool) {
    let (image_idx, suboptimal, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap){
        Ok(result) => result,
        Err(VulkanError::OutOfDate) => {
            println!("Swapchain out of date");
            return (None, true);
        },
        Err(e) => panic!("Failed to acquire swapchain image: {}", e),
    };

    if suboptimal {
        return (None, true);
    }
    (Some((image_idx, acquire_future)), false)
}

pub fn present_swapchain_image(device: Arc<Device>, swapchain: Arc<Swapchain>, queue: Arc<Queue>, command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>, image_idx: u32, swapchain_image_future: SwapchainAcquireFuture) -> bool {
    let execution_future = sync::now(device.clone())
        .join(swapchain_image_future)
        .then_execute(queue.clone(), command_buffers[image_idx as usize].clone())
        .unwrap()
        .then_swapchain_present(
            queue.clone(),
            SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_idx)
        )
        .then_signal_fence_and_flush();

    match execution_future.map_err(Validated::unwrap) {
        Ok(future) => {
            future.wait(None).unwrap();
            return false;
        },
        Err(VulkanError::OutOfDate) => {
            return true;
        },
        Err(e) => panic!("Failed to present swapchain image: {}", e),
    }
}

pub fn present_swapchain_image_with_fence(
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    queue: Arc<Queue>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
    image_idx: u32,
    swapchain_image_future: SwapchainAcquireFuture,
    previous_future: Box<dyn GpuFuture>,
    mut fences: Vec<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture>>>>>
) -> bool {
    let mut recreate_swapchain = false;
    let future = previous_future
        .join(swapchain_image_future)
        .then_execute(queue.clone(), command_buffers[image_idx as usize].clone())
        .unwrap()
        .then_swapchain_present(
            queue.clone(),
            SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_idx)
        );
    
    // Box the future chain before creating the fence
    let boxed_future = Box::new(future) as Box<dyn GpuFuture>;
    let execution_future = boxed_future.then_signal_fence_and_flush();

    fences[image_idx as usize] = match execution_future.map_err(Validated::unwrap) {
        Ok(v) => Some(Arc::new(v)),
        Err(VulkanError::OutOfDate) => {
            recreate_swapchain = true;
            None
        },
        Err(e) => {
            println!("Failed to present swapchain image: {}", e);
            None
        }
    };
    recreate_swapchain
}
