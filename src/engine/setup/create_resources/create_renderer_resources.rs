use bevy_ecs::world::World;
use vma::{Alloc, AllocationOptions, MemoryUsage};
use vulkanalia::vk::{
    DeviceV1_0, Extent3D, Format, ImageAspectFlags, ImageUsageFlags, MemoryPropertyFlags,
};

use crate::engine::{
    Engine,
    resources::{AllocatedImage, RendererContext, RendererResources, VulkanContextResource},
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

        let renderer_resources = RendererResources { draw_image };

        renderer_resources
    }
}
