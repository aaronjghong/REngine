use vulkano::device::Device;
use vulkano::memory::allocator::{StandardMemoryAllocator, AllocationCreateInfo, MemoryTypeFilter};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer};
use vulkano::device::Queue;
use vulkano::memory::MemoryPropertyFlags;
use vulkano::sync::{self, GpuFuture};
use std::sync::Arc;

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

pub fn create_buffer(memory_allocator: Arc<StandardMemoryAllocator>, memory_type_filter: MemoryTypeFilter, buffer_usage: BufferUsage, size: u64) -> Subbuffer<f32> {
    Buffer::new_sized::<f32>(
        memory_allocator.clone(),
        BufferCreateInfo{
            usage: buffer_usage,
            size: size,
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
    size: u64, 
    iter: impl ExactSizeIterator<Item = T>
) -> Subbuffer<[T]> 
where 
    T: BufferContents + Copy,
{
    Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo{
            usage: buffer_usage,
            size: size,
            ..Default::default()
        },
        AllocationCreateInfo{
            memory_type_filter: memory_type_filter,
            ..Default::default()
        },
        iter,
    ).expect("Failed to create buffer")
}

pub fn create_vertex_buffer(memory_allocator: Arc<StandardMemoryAllocator>, size: u64, verts_iter: impl ExactSizeIterator<Item = f32>) -> Subbuffer<[f32]> {
    let memory_type_filter = MemoryTypeFilter::PREFER_DEVICE;
    create_buffer_from_iter(memory_allocator, memory_type_filter, BufferUsage::VERTEX_BUFFER, size, verts_iter)
}

pub fn create_index_buffer(memory_allocator: Arc<StandardMemoryAllocator>, size: u64, indices_iter: impl ExactSizeIterator<Item = u32>) -> Subbuffer<[u32]> {
    let memory_type_filter = MemoryTypeFilter::PREFER_DEVICE;
    create_buffer_from_iter(memory_allocator, memory_type_filter, BufferUsage::INDEX_BUFFER, size, indices_iter)
}

//https://docs.rs/vulkano/0.34.0/vulkano/command_buffer/index.html
// Creates a primary command buffer that copies the contents of buffer_src to buffer_dst
pub fn create_command_buffer_builder(command_buffer_allocator: Arc<StandardCommandBufferAllocator>, queue: Arc<Queue>) -> Arc<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>> {
    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::MultipleSubmit,
    ).unwrap();

    // We let the App take ownership of the builder so that it can be used to build the command buffer
    Arc::new(builder)
}

pub fn submit(queue: Arc<Queue>, command_buffer_builder: Arc<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>>) {
    let command_buffer = command_buffer_builder.build().unwrap();

    sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
}

pub fn submit_execute(queue: Arc<Queue>, command_buffer_builder: Arc<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>>) {
    let command_buffer = command_buffer_builder.build().unwrap();

    sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .flush()
        .unwrap();
}

pub fn submit_execute_wait_fenced(queue: Arc<Queue>, command_buffer_builder: Arc<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>>) -> GpuFuture<()> {
    let command_buffer = command_buffer_builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();
    
}
