use std::{
    collections::HashMap,
    ffi::{CString, c_void},
    str::FromStr as _,
    sync::{Arc, Weak},
};

use vma::{
    Alloc as _, Allocation, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage,
};
use vulkanite::{
    Handle,
    vk::{rs::*, *},
};

use crate::engine::{id::Id, resources::CommandGroup};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BufferVisibility {
    #[default]
    Unspecified,
    HostVisible,
    DeviceOnly,
}

pub struct AllocatedBuffer {
    pub id: Id,
    pub buffer: Buffer,
    pub allocation: Allocation,
    pub buffer_info: BufferInfo,
}

#[derive(Default, Clone)]
pub struct BufferReference {
    buffer_id: Id,
    weak_ptr: Weak<AllocatedBuffer>,
    buffer_info: BufferInfo,
}

#[derive(Default, Clone, Copy)]
pub struct BufferInfo {
    pub device_address: DeviceAddress,
    pub size: DeviceSize,
    pub buffer_visibility: BufferVisibility,
}

impl BufferInfo {
    pub fn new(
        device_address: DeviceAddress,
        size: DeviceSize,
        buffer_visibility: BufferVisibility,
    ) -> Self {
        Self {
            device_address,
            size,
            buffer_visibility,
        }
    }
}

impl BufferReference {
    pub fn new(
        buffer_id: Id,
        allocated_buffer: Weak<AllocatedBuffer>,
        device_address: DeviceAddress,
        size: DeviceSize,
        buffer_visibility: BufferVisibility,
    ) -> Self {
        Self {
            buffer_id,
            weak_ptr: allocated_buffer,
            buffer_info: BufferInfo::new(device_address, size, buffer_visibility),
        }
    }

    pub fn get_buffer<'a>(&'a self) -> Option<&'a AllocatedBuffer> {
        let mut allocated_buffer = None;

        if !self.weak_ptr.strong_count() != Default::default() {
            let allocated_buffer_ref = unsafe { &*(self.weak_ptr.as_ptr()) };

            if allocated_buffer_ref.id == self.buffer_id {
                allocated_buffer = Some(allocated_buffer_ref);
            }
        }

        allocated_buffer
    }

    #[inline(always)]
    pub fn get_buffer_info(&self) -> BufferInfo {
        self.buffer_info
    }
}

pub struct MemoryBucket {
    device: Device,
    allocator: Allocator,
    buffers: Vec<Arc<AllocatedBuffer>>,
    buffers_map: HashMap<Id, usize>,
    staging_buffer_reference: BufferReference,
    upload_command_group: CommandGroup,
    transfer_queue: Queue,
}

impl MemoryBucket {
    pub fn new(
        device: Device,
        allocator: Allocator,
        upload_command_group: CommandGroup,
        transfer_queue: Queue,
    ) -> Self {
        let mut memory_bucket = Self {
            device,
            allocator,
            buffers: Vec::with_capacity(1024),
            buffers_map: HashMap::with_capacity(1024),
            staging_buffer_reference: Default::default(),
            upload_command_group,
            transfer_queue,
        };

        // Pre-allocate 64 MB for transfers.
        let staging_buffer_reference = memory_bucket.create_buffer(
            1024 * 1024 * 64,
            BufferUsageFlags::TransferSrc,
            BufferVisibility::HostVisible,
            Some("Staging Buffer"),
        );
        memory_bucket.staging_buffer_reference = staging_buffer_reference;

        memory_bucket
    }

    pub fn create_buffer(
        &mut self,
        allocation_size: usize,
        usage: BufferUsageFlags,
        buffer_visibility: BufferVisibility,
        name: Option<&str>,
    ) -> BufferReference {
        let buffer_kind_usage = if allocation_size < 1024 * 64 {
            BufferUsageFlags::UniformBuffer
        } else {
            BufferUsageFlags::StorageBuffer
        };

        let buffer_create_info = BufferCreateInfo {
            size: allocation_size as _,
            usage: usage | buffer_kind_usage | BufferUsageFlags::ShaderDeviceAddress,
            sharing_mode: vulkanite::vk::SharingMode::Exclusive,
            ..Default::default()
        };

        if buffer_visibility == BufferVisibility::Unspecified {
            panic!("Trying to create a buffer with unspecified visibility!");
        }

        let allocation_flags = match buffer_visibility {
            BufferVisibility::HostVisible => {
                AllocationCreateFlags::Mapped
                    | AllocationCreateFlags::HostAccessSequentialWrite
                    | AllocationCreateFlags::StrategyMinMemory
            }
            BufferVisibility::DeviceOnly => AllocationCreateFlags::StrategyMinMemory,
            BufferVisibility::Unspecified => unreachable!(),
        };

        let preferred_flags = match buffer_visibility {
            BufferVisibility::HostVisible => MemoryPropertyFlags::HostCoherent,
            BufferVisibility::DeviceOnly => MemoryPropertyFlags::empty(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        let allocation_create_info = AllocationCreateInfo {
            flags: allocation_flags,
            usage: MemoryUsage::Auto,
            required_flags: MemoryPropertyFlags::DeviceLocal,
            preferred_flags: preferred_flags,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            self.allocator
                .create_buffer(&buffer_create_info, &allocation_create_info)
                .unwrap()
        };
        let buffer = Buffer::from_inner(buffer);
        let device_address = unsafe { self.get_device_address(buffer) };

        if let Some(name) = name {
            let name = CString::from_str(name).unwrap();
            let debug_utils_object_name = DebugUtilsObjectNameInfoEXT {
                object_type: ObjectType::Buffer,
                object_handle: buffer.as_raw().get(),
                p_object_name: name.as_ptr() as *const _,
                ..Default::default()
            };

            self.device
                .set_debug_utils_object_name_ext(&debug_utils_object_name)
                .unwrap();
        }

        let buffer_info = BufferInfo::new(device_address, allocation_size as _, buffer_visibility);
        let allocated_buffer = AllocatedBuffer {
            id: Id::new(device_address),
            buffer,
            allocation,
            buffer_info,
        };
        let allocated_buffer_size = allocated_buffer.buffer_info.size;
        let allocated_buffer_id = allocated_buffer.id;
        let weak_ptr_allocated_buffer = self.insert_buffer(allocated_buffer);

        let allocated_buffer_reference = BufferReference::new(
            allocated_buffer_id,
            weak_ptr_allocated_buffer,
            device_address,
            allocated_buffer_size,
            buffer_visibility,
        );

        allocated_buffer_reference
    }

    fn insert_buffer(&mut self, allocated_buffer: AllocatedBuffer) -> Weak<AllocatedBuffer> {
        let allocated_buffer_id = allocated_buffer.id;
        let allocated_buffer = Arc::new(allocated_buffer);
        let weak_ptr_allocated_buffer = Arc::downgrade(&allocated_buffer);
        self.buffers.push(allocated_buffer);
        let buffer_index = self.buffers.len() - 1;

        if let Some(already_presented_buffer_index) =
            self.buffers_map.insert(allocated_buffer_id, buffer_index)
        {
            panic!("Memory Bucket already has buffer by index: {already_presented_buffer_index}");
        }

        weak_ptr_allocated_buffer
    }

    unsafe fn get_device_address(&self, buffer: Buffer) -> DeviceAddress {
        let buffer_device_address = BufferDeviceAddressInfo::default().buffer(&buffer);

        self.device.get_buffer_address(&buffer_device_address)
    }

    pub unsafe fn transfer_data_to_buffer(
        &self,
        buffer_reference: BufferReference,
        src: &[u8],
        size: usize,
    ) {
        let allocated_buffer = buffer_reference.get_buffer().unwrap();

        let buffer_visibility = allocated_buffer.buffer_info.buffer_visibility;
        let target_buffer = match buffer_visibility {
            BufferVisibility::HostVisible => allocated_buffer,
            BufferVisibility::DeviceOnly => self.staging_buffer_reference.get_buffer().unwrap(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        unsafe {
            let p_mapped_memory = self.allocator.map_memory(target_buffer.allocation).unwrap();

            std::ptr::copy_nonoverlapping(src.as_ptr(), p_mapped_memory as _, size);

            self.allocator.unmap_memory(target_buffer.allocation);
        }

        if buffer_visibility == BufferVisibility::DeviceOnly {
            let regions_to_copy = [BufferCopy {
                size: size as _,
                ..Default::default()
            }];
            unsafe {
                self.copy_buffer_to_buffer(
                    target_buffer.buffer,
                    allocated_buffer.buffer,
                    &regions_to_copy,
                )
            }
        }
    }

    pub fn get_staging_buffer_reference<'a>(&self) -> &BufferReference {
        &self.staging_buffer_reference
    }

    pub unsafe fn transfer_data_to_buffer_raw(
        &mut self,
        buffer_reference: &BufferReference,
        src: *const c_void,
        size: usize,
    ) {
        let allocated_buffer = buffer_reference.get_buffer().unwrap();

        let buffer_visibility = allocated_buffer.buffer_info.buffer_visibility;
        let target_buffer = match buffer_visibility {
            BufferVisibility::HostVisible => allocated_buffer,
            BufferVisibility::DeviceOnly => self.staging_buffer_reference.get_buffer().unwrap(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        unsafe {
            let p_mapped_memory = self.allocator.map_memory(target_buffer.allocation).unwrap();

            std::ptr::copy_nonoverlapping(src, p_mapped_memory as _, size);

            self.allocator.unmap_memory(target_buffer.allocation);
        }

        if buffer_visibility == BufferVisibility::DeviceOnly {
            let regions_to_copy = [BufferCopy {
                size: size as _,
                ..Default::default()
            }];
            unsafe {
                self.copy_buffer_to_buffer(
                    target_buffer.buffer,
                    allocated_buffer.buffer,
                    &regions_to_copy,
                )
            }
        }
    }

    pub unsafe fn transfer_data_to_buffer_with_offset(
        &self,
        buffer_reference: &BufferReference,
        src: *const c_void,
        regions_to_copy: &[BufferCopy],
    ) {
        let allocated_buffer = buffer_reference.get_buffer().unwrap();

        let buffer_visibility = allocated_buffer.buffer_info.buffer_visibility;
        let target_buffer = match buffer_visibility {
            BufferVisibility::HostVisible => allocated_buffer,
            BufferVisibility::DeviceOnly => self.staging_buffer_reference.get_buffer().unwrap(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        unsafe {
            let ptr_mapped_memory = self.allocator.map_memory(target_buffer.allocation).unwrap();

            for &buffer_copy in regions_to_copy {
                let src_with_offset = src.add(buffer_copy.src_offset as usize);

                let ptr_mapped_memory_with_offset =
                    ptr_mapped_memory.add(buffer_copy.dst_offset as usize);

                std::ptr::copy_nonoverlapping(
                    src_with_offset,
                    ptr_mapped_memory_with_offset as _,
                    buffer_copy.size as usize,
                );
            }

            self.allocator.unmap_memory(target_buffer.allocation);
        }

        if buffer_visibility == BufferVisibility::DeviceOnly {
            unsafe {
                self.copy_buffer_to_buffer(
                    target_buffer.buffer,
                    allocated_buffer.buffer,
                    &regions_to_copy,
                )
            }
        }
    }

    unsafe fn copy_buffer_to_buffer(
        &self,
        src_buffer: Buffer,
        dst_buffer: Buffer,
        regions_to_copy: &[BufferCopy],
    ) {
        let command_buffer = self.upload_command_group.command_buffer;

        let command_buffer_begin_info = CommandBufferBeginInfo {
            flags: CommandBufferUsageFlags::OneTimeSubmit,
            ..Default::default()
        };

        command_buffer.begin(&command_buffer_begin_info).unwrap();

        self.upload_command_group.command_buffer.copy_buffer(
            src_buffer,
            dst_buffer,
            regions_to_copy,
        );

        command_buffer.end().unwrap();

        let command_buffers = [command_buffer];
        let queue_submits = [SubmitInfo::default().command_buffers(command_buffers.as_slice())];

        self.transfer_queue
            .submit(&queue_submits, Some(self.upload_command_group.fence))
            .unwrap();

        let fences_to_wait = [self.upload_command_group.fence];
        self.device
            .wait_for_fences(fences_to_wait.as_slice(), true, u64::MAX)
            .unwrap();
        self.device.reset_fences(fences_to_wait.as_slice()).unwrap();

        self.device
            .reset_command_pool(
                self.upload_command_group.command_pool,
                CommandPoolResetFlags::ReleaseResources,
            )
            .unwrap();
    }

    pub unsafe fn free_allocations(&mut self) {
        self.buffers.drain(..).for_each(|allocated_buffer| unsafe {
            let mut allocation = allocated_buffer.allocation;
            self.allocator
                .destroy_buffer(*allocated_buffer.buffer, &mut allocation);
        });
    }
}
