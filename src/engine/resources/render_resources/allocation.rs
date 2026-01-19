use vma::{Alloc, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage};
use vulkanite::vk::BufferCreateInfo;
use vulkanite::vk::{rs::*, *};

use crate::engine::id::Id;
use crate::engine::resources::AllocatedBuffer;
use crate::engine::utils::get_device_address;

pub fn create_buffer(
    device: rs::Device,
    allocator: &Allocator,
    allocation_size: usize,
    usage: BufferUsageFlags,
) -> AllocatedBuffer {
    let buffer_create_info = BufferCreateInfo {
        size: allocation_size as _,
        usage: usage | BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::StorageBuffer,
        sharing_mode: vulkanite::vk::SharingMode::Exclusive,
        ..Default::default()
    };

    let allocation_create_info = AllocationCreateInfo {
        flags: AllocationCreateFlags::Mapped | AllocationCreateFlags::HostAccessSequentialWrite,
        usage: MemoryUsage::AutoPreferDevice,
        required_flags: MemoryPropertyFlags::DeviceLocal,
        ..Default::default()
    };

    let (buffer, allocation) = unsafe {
        allocator
            .create_buffer(&buffer_create_info, &allocation_create_info)
            .unwrap()
    };
    let buffer = Buffer::from_inner(buffer);

    let device_address = get_device_address(device, &buffer);

    AllocatedBuffer {
        id: Id::new(device_address),
        buffer,
        allocation,
        device_address,
        size: allocation_size as _,
    }
}
