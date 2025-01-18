use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::format::{Format, ClearColorValue};
use vulkano::memory::allocator::{
    StandardMemoryAllocator,
    AllocationCreateInfo,
    MemoryTypeFilter,
};
use vulkano::command_buffer::ClearColorImageInfo;
use vulkano::swapchain::{Swapchain, SwapchainCreateInfo, Surface, CompositeAlphas};
use vulkano::device::Device;
use vulkano::device::physical::PhysicalDevice; 
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