use bevy_ecs::{resource::Resource, system::command};
use vma::Allocator;
use vulkanite::{
    Handle,
    vk::{
        self, BufferImageCopy, BufferUsageFlags, CommandBufferBeginInfo, CommandBufferSubmitInfo,
        CommandBufferUsageFlags, CommandPoolResetFlags, DescriptorType,
        HostImageLayoutTransitionInfo, ImageLayout, ImageSubresourceLayers, SubmitInfo,
        SubmitInfo2, SurfaceFormatKHR,
        rs::{
            DebugUtilsMessengerEXT, Device, Instance, PhysicalDevice, Queue, SurfaceKHR,
            SwapchainKHR,
        },
    },
};

use crate::engine::{
    resources::{AllocatedImage, UploadContext, allocation::create_buffer},
    systems::on_load_model::transfer_data_to_buffer,
};

#[derive(Resource)]
pub struct VulkanContextResource {
    pub instance: Instance,
    pub debug_utils_messenger: Option<DebugUtilsMessengerEXT>,
    pub surface: SurfaceKHR,
    pub device: Device,
    pub physical_device: PhysicalDevice,
    pub allocator: Allocator,
    pub graphics_queue: Queue,
    pub transfer_queue: Queue,
    pub queue_family_index: usize,
    pub swapchain: SwapchainKHR,
    pub surface_format: SurfaceFormatKHR,
}

impl VulkanContextResource {
    pub fn transfer_data_to_image(
        &self,
        allocated_image: &AllocatedImage,
        data_to_copy: *const std::ffi::c_void,
        upload_context: &UploadContext,
    ) {
        let command_buffer = upload_context.command_group.command_buffer;

        let command_buffer_begin_info = CommandBufferBeginInfo {
            flags: CommandBufferUsageFlags::OneTimeSubmit,
            ..Default::default()
        };

        command_buffer.begin(&command_buffer_begin_info).unwrap();

        let image_extent = allocated_image.extent;
        let size = image_extent.depth * image_extent.width * image_extent.height * 4;

        let mut upload_buffer = create_buffer(
            self.device,
            &self.allocator,
            size as usize,
            BufferUsageFlags::TransferSrc,
        );

        unsafe {
            transfer_data_to_buffer(&self.allocator, &mut upload_buffer, data_to_copy, size as _);
        }

        let host_image_layout_transition_info = [HostImageLayoutTransitionInfo {
            image: Some(allocated_image.image.borrow()),
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::General,
            subresource_range: allocated_image.subresource_range,
            ..Default::default()
        }];

        self.device
            .transition_image_layout(&host_image_layout_transition_info)
            .unwrap();

        let buffer_image_copy = [BufferImageCopy {
            image_subresource: ImageSubresourceLayers {
                aspect_mask: allocated_image.subresource_range.aspect_mask,
                mip_level: Default::default(),
                base_array_layer: Default::default(),
                layer_count: 1,
            },
            image_extent,
            ..Default::default()
        }];

        upload_context
            .command_group
            .command_buffer
            .copy_buffer_to_image(
                upload_buffer.buffer,
                allocated_image.image,
                ImageLayout::General,
                &buffer_image_copy,
            );

        command_buffer.end().unwrap();

        let command_buffers = [command_buffer];
        let queue_submits = [SubmitInfo::default().command_buffers(command_buffers.as_slice())];

        self.transfer_queue
            .submit(&queue_submits, Some(upload_context.command_group.fence))
            .unwrap();

        let fences_to_wait = [upload_context.command_group.fence];
        self.device
            .wait_for_fences(fences_to_wait.as_slice(), true, u64::MAX)
            .unwrap();
        self.device.reset_fences(fences_to_wait.as_slice()).unwrap();

        self.device
            .reset_command_pool(
                upload_context.command_group.command_pool,
                CommandPoolResetFlags::ReleaseResources,
            )
            .unwrap();

        unsafe {
            let buffer_raw = vk::raw::Buffer::from_raw(upload_buffer.buffer.as_raw());
            self.allocator
                .destroy_buffer(buffer_raw, &mut upload_buffer.allocation);
        }
    }
}
