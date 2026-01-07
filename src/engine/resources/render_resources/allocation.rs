use vma::{Alloc, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage};
use vulkanite::vk::BufferCreateInfo;
use vulkanite::vk::{rs::*, *};

use crate::engine::resources::AllocatedBuffer;

pub fn create_buffer(
    allocator: &Allocator,
    allocation_size: usize,
    usage: BufferUsageFlags,
) -> AllocatedBuffer {
    let buffer_create_info = BufferCreateInfo {
        size: allocation_size as _,
        usage: usage | BufferUsageFlags::ShaderDeviceAddress,
        sharing_mode: vulkanite::vk::SharingMode::Exclusive,
        ..Default::default()
    };

    let allocation_create_info = AllocationCreateInfo {
        flags: AllocationCreateFlags::Mapped | AllocationCreateFlags::HostAccessRandom,
        usage: MemoryUsage::AutoPreferDevice,
        ..Default::default()
    };

    let (buffer, allocation) = unsafe {
        allocator
            .create_buffer(&buffer_create_info, &allocation_create_info)
            .unwrap()
    };
    let buffer = Buffer::from_inner(buffer);

    AllocatedBuffer { buffer, allocation }
}
