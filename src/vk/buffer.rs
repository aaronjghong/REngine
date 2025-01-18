use vulkano::device::Device;
use vulkano::memory::allocator::{StandardMemoryAllocator, AllocationCreateInfo, MemoryTypeFilter};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer, CommandBufferExecFuture, PrimaryCommandBufferAbstract};
use vulkano::device::Queue;
use vulkano::memory::MemoryPropertyFlags;
use vulkano::sync::{self, GpuFuture};
use std::sync::Arc;

use crate::vk::Vert;

pub type PrimaryCommandBufferBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>;

pub const STAGING_BUFFER_MEMORY_TYPE_FILTER: MemoryTypeFilter = MemoryTypeFilter{
    required_flags: MemoryPropertyFlags::HOST_VISIBLE.union(MemoryPropertyFlags::HOST_COHERENT),
    preferred_flags: MemoryPropertyFlags::empty(),
    not_preferred_flags: MemoryPropertyFlags::HOST_CACHED.union(MemoryPropertyFlags::DEVICE_LOCAL),
};

pub const STREAMING_BUFFER_MEMORY_TYPE_FILTER: MemoryTypeFilter = MemoryTypeFilter{
    required_flags: MemoryPropertyFlags::HOST_VISIBLE,
    preferred_flags: MemoryPropertyFlags::DEVICE_LOCAL,
    not_preferred_flags: MemoryPropertyFlags::HOST_CACHED,
};

pub const UNIFORM_BUFFER_MEMORY_TYPE_FILTER: MemoryTypeFilter = MemoryTypeFilter{
    required_flags: MemoryPropertyFlags::HOST_VISIBLE.union(MemoryPropertyFlags::HOST_COHERENT),
    preferred_flags: MemoryPropertyFlags::DEVICE_LOCAL,
    not_preferred_flags: MemoryPropertyFlags::empty(),
};

pub fn create_buffer(memory_allocator: Arc<StandardMemoryAllocator>, memory_type_filter: MemoryTypeFilter, buffer_usage: BufferUsage) -> Subbuffer<f32> {
    Buffer::new_sized::<f32>(
        memory_allocator.clone(),
        BufferCreateInfo{
            usage: buffer_usage,
            ..Default::default()
        },  
        AllocationCreateInfo{
            memory_type_filter: memory_type_filter,
            ..Default::default()
        },
    ).expect("Failed to create buffer")
}

pub fn create_buffer_from_iter<T>(
    memory_allocator: Arc<StandardMemoryAllocator>, 
    memory_type_filter: MemoryTypeFilter, 
    buffer_usage: BufferUsage, 
    iter: impl ExactSizeIterator<Item = T>
) -> Subbuffer<[T]> 
where 
    T: BufferContents + Copy,
{
    Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo{
            usage: buffer_usage,
            ..Default::default()
        },
        AllocationCreateInfo{
            memory_type_filter: memory_type_filter,
            ..Default::default()
        },
        iter,
    ).expect("Failed to create buffer")
}

pub fn create_vertex_buffer(memory_allocator: Arc<StandardMemoryAllocator>, verts_iter: impl ExactSizeIterator<Item = Vert>) -> Subbuffer<[Vert]> {
    let memory_type_filter = MemoryTypeFilter::PREFER_DEVICE;
    create_buffer_from_iter(memory_allocator, memory_type_filter, BufferUsage::VERTEX_BUFFER, verts_iter)
}

pub fn create_index_buffer(memory_allocator: Arc<StandardMemoryAllocator>, indices_iter: impl ExactSizeIterator<Item = u32>) -> Subbuffer<[u32]> {
    let memory_type_filter = MemoryTypeFilter::PREFER_DEVICE;
    create_buffer_from_iter(memory_allocator, memory_type_filter, BufferUsage::INDEX_BUFFER, indices_iter)
}

//https://docs.rs/vulkano/0.34.0/vulkano/command_buffer/index.html
// Creates a primary command buffer that copies the contents of buffer_src to buffer_dst
pub fn create_command_buffer_builder(command_buffer_allocator: Arc<StandardCommandBufferAllocator>, queue: Arc<Queue>) -> PrimaryCommandBufferBuilder {
    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::MultipleSubmit,
    ).unwrap();

    // We let the App take ownership of the builder so that it can be used to build the command buffer
    builder
}

// pub fn submit(device: Arc<Device>, queue: Arc<Queue>, command_buffer_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>) {
//     let command_buffer = command_buffer_builder.build().unwrap();

//     sync::now(device.clone())
//         .then_execute(queue.clone(), command_buffer)
//         .unwrap()
// }

pub fn build_command_buffer(command_buffer_builder: PrimaryCommandBufferBuilder) -> Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>> {
    command_buffer_builder.build().unwrap()
}

pub fn submit_execute(device: Arc<Device>, queue: Arc<Queue>, command_buffer: Arc<PrimaryAutoCommandBuffer>) {
    sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .flush()
        .unwrap();
}

pub fn submit_execute_wait_fenced(device: Arc<Device>, queue: Arc<Queue>, command_buffer: Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>){
    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();
}
