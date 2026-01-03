use bevy_ecs::world::{self, World};
use vma::{Alloc, AllocationCreateFlags, AllocationOptions, MemoryUsage};
use vulkanalia::vk::{
    BufferCreateInfo, BufferUsageFlags, DescriptorSetLayoutCreateFlags, DescriptorType, DeviceV1_0,
    ExtDescriptorBufferExtensionDeviceCommands, Extent3D, Format, ImageAspectFlags,
    ImageUsageFlags, MemoryPropertyFlags, ShaderStageFlags, SharingMode,
};

use crate::engine::{
    Engine,
    descriptors::DescriptorSetLayoutBuilder,
    resources::{
        AllocatedBuffer, AllocatedDescriptorBuffer, AllocatedImage, DevicePropertiesResource,
        RendererContext, RendererResources, VulkanContextResource,
    },
    utils::{create_image_info, create_image_view_info},
};

impl Engine {
    pub fn create_renderer_resources(world: &World) -> RendererResources {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();

        let draw_image_extent = Extent3D {
            width: render_context.draw_extent.width,
            height: render_context.draw_extent.height,
            depth: 1,
        };
        let target_draw_image_format = Format::R16G16B16A16_SFLOAT;
        let image_usage_flags = ImageUsageFlags::TRANSFER_SRC
            | ImageUsageFlags::TRANSFER_DST
            | ImageUsageFlags::STORAGE
            | ImageUsageFlags::COLOR_ATTACHMENT;

        let image_create_info = create_image_info(
            target_draw_image_format,
            image_usage_flags,
            draw_image_extent,
        );
        let mut allocation_options = AllocationOptions::default();
        allocation_options.usage = MemoryUsage::Auto;
        allocation_options.required_flags = MemoryPropertyFlags::DEVICE_LOCAL;

        let (allocated_draw_image, allocation) = unsafe {
            vulkan_context
                .allocator
                .create_image(image_create_info, &allocation_options)
                .unwrap()
        };

        let image_view_create_info = create_image_view_info(
            target_draw_image_format,
            allocated_draw_image,
            ImageAspectFlags::COLOR,
        );
        let allocated_image_view = unsafe {
            vulkan_context
                .device
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        };

        let draw_image = AllocatedImage {
            image: allocated_draw_image,
            image_view: allocated_image_view,
            allocation: allocation,
            image_extent: draw_image_extent,
            format: Format::R16G16B16A16_SFLOAT,
        };

        let draw_image_descriptor_buffer = Self::create_descriptors(&world);

        let renderer_resources = RendererResources {
            draw_image,
            draw_image_descriptor_buffer,
        };

        renderer_resources
    }

    fn create_descriptors(world: &World) -> AllocatedDescriptorBuffer {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let device_properties_resource = world
            .get_resource_ref::<DevicePropertiesResource>()
            .unwrap();
        let device = &vulkan_context_resource.device;

        let mut descriptor_set_layout_builder = DescriptorSetLayoutBuilder::new();
        descriptor_set_layout_builder.add_binding(0, DescriptorType::STORAGE_IMAGE);
        let descriptor_set_layout = descriptor_set_layout_builder.build(
            &device,
            ShaderStageFlags::COMPUTE,
            DescriptorSetLayoutCreateFlags::DESCRIPTOR_BUFFER_EXT,
        );

        let descriptor_set_layout_size =
            unsafe { device.get_descriptor_set_layout_size_ext(descriptor_set_layout) };

        let descriptor_buffer_size = Self::aligned_size(
            descriptor_set_layout_size,
            device_properties_resource
                .descriptor_buffer_properties
                .descriptor_buffer_offset_alignment,
        );

        let descriptor_buffer_offset = unsafe {
            device.get_descriptor_set_layout_binding_offset_ext(
                descriptor_set_layout,
                Default::default(),
            )
        };

        let buffer_info = BufferCreateInfo {
            size: descriptor_buffer_size,
            usage: BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::RESOURCE_DESCRIPTOR_BUFFER_EXT,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let allocation_options = AllocationOptions {
            flags: AllocationCreateFlags::MAPPED | AllocationCreateFlags::HOST_ACCESS_RANDOM,
            ..Default::default()
        };

        let (storage_image_descriptor_buffer, allocation) = unsafe {
            vulkan_context_resource
                .allocator
                .create_buffer(buffer_info, &allocation_options)
                .unwrap()
        };

        let allocated_buffer = AllocatedBuffer {
            buffer: storage_image_descriptor_buffer,
            allocation,
        };

        let allocated_descriptor_buffer = AllocatedDescriptorBuffer {
            allocated_buffer,
            descriptor_buffer_offset,
            descriptor_buffer_size,
            descriptor_set_layout,
        };

        allocated_descriptor_buffer
    }

    fn aligned_size(value: u64, alignment: u64) -> u64 {
        (value + alignment - 1) & !(alignment - 1)
    }
}
