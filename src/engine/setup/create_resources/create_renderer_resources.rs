use bevy_ecs::world::World;
use vma::{Alloc, AllocationCreateFlags, AllocationCreateInfo, MemoryUsage};
use vulkanite::vk::{rs::*, *};

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
        let target_draw_image_format = Format::R16G16B16A16Sfloat;
        let image_usage_flags = ImageUsageFlags::TransferSrc
            | ImageUsageFlags::TransferDst
            | ImageUsageFlags::Storage
            | ImageUsageFlags::ColorAttachment;

        let image_create_info = create_image_info(
            target_draw_image_format,
            image_usage_flags,
            draw_image_extent,
        );
        let mut allocation_info = AllocationCreateInfo::default();
        allocation_info.usage = MemoryUsage::Auto;
        allocation_info.required_flags = MemoryPropertyFlags::DeviceLocal;

        let (allocated_draw_image, allocation) = unsafe {
            vulkan_context
                .allocator
                .create_image(&image_create_info, &allocation_info)
                .unwrap()
        };

        let allocated_draw_image = rs::Image::from_inner(allocated_draw_image);
        let image_view_create_info = create_image_view_info(
            target_draw_image_format,
            &allocated_draw_image,
            ImageAspectFlags::Color,
        );
        let allocated_image_view = unsafe {
            vulkan_context
                .device
                .create_image_view(&image_view_create_info)
                .unwrap()
        };

        let draw_image = AllocatedImage {
            image: allocated_draw_image,
            image_view: allocated_image_view,
            allocation: allocation,
            image_extent: draw_image_extent,
            format: Format::R16G16B16A16Sfloat,
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
        descriptor_set_layout_builder.add_binding(0, DescriptorType::StorageImage);
        let descriptor_set_layout = descriptor_set_layout_builder.build(
            &device,
            ShaderStageFlags::Compute,
            DescriptorSetLayoutCreateFlags::DescriptorBufferEXT,
        );

        let descriptor_set_layout_size =
            unsafe { device.get_descriptor_set_layout_size_ext(&descriptor_set_layout) };

        let descriptor_buffer_size = Self::aligned_size(
            descriptor_set_layout_size,
            device_properties_resource
                .descriptor_buffer_properties
                .descriptor_buffer_offset_alignment,
        );

        let descriptor_buffer_offset = unsafe {
            device.get_descriptor_set_layout_binding_offset_ext(
                &descriptor_set_layout,
                Default::default(),
            )
        };

        let buffer_info = BufferCreateInfo::default()
            .size(descriptor_buffer_size)
            .usage(
                BufferUsageFlags::ShaderDeviceAddress
                    | BufferUsageFlags::ResourceDescriptorBufferEXT,
            );

        let allocation_info = AllocationCreateInfo {
            flags: AllocationCreateFlags::Mapped | AllocationCreateFlags::HostAccessRandom,
            usage: MemoryUsage::Auto,
            ..Default::default()
        };

        let (storage_image_descriptor_buffer, allocation) = unsafe {
            vulkan_context_resource
                .allocator
                .create_buffer(&buffer_info, &allocation_info)
                .unwrap()
        };
        let storage_image_descriptor_buffer = Buffer::from_inner(storage_image_descriptor_buffer);

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
