use std::sync::Arc;

use bevy_ecs::world::World;
use vulkanalia::vk::{
    CommandBufferAllocateInfo, CommandPoolCreateFlags, CommandPoolCreateInfo, DeviceV1_0, Extent2D,
    FenceCreateFlags, FenceCreateInfo, HasBuilder, SemaphoreCreateInfo,
};
use winit::window::Window;

use crate::engine::{
    Engine,
    resources::{FrameData, RendererContext, VulkanContextResource},
};

impl Engine {
    pub(crate) fn create_renderer_context(
        window: &Arc<dyn Window>,
        world: &World,
    ) -> RendererContext {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let swapchain = &vulkan_context_resource.swapchain;

        let images = swapchain.get_images().unwrap();
        let image_views = swapchain.get_image_views().unwrap();
        let frame_overlap = image_views.len();

        let command_pool_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();

        let device = &vulkan_context_resource.device;
        let frames_data = (0..frame_overlap)
            .map(|_| unsafe {
                let command_pool = device
                    .create_command_pool(&command_pool_info, None)
                    .unwrap();

                let command_buffer_allocate_info = CommandBufferAllocateInfo::builder()
                    .command_pool(command_pool)
                    .command_buffer_count(1)
                    .build();
                let command_buffer = device
                    .allocate_command_buffers(&command_buffer_allocate_info)
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone();

                let render_fence = device
                    .create_fence(
                        &FenceCreateInfo::builder().flags(FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .unwrap();

                let semaphore_create_info = SemaphoreCreateInfo::default();
                let swapchain_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();
                let render_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();

                FrameData {
                    command_pool,
                    command_buffer,
                    render_fence,
                    swapchain_semaphore,
                    render_semaphore,
                }
            })
            .collect();

        let surface_size = window.surface_size();
        let draw_extent = Extent2D {
            width: surface_size.width,
            height: surface_size.height,
        };
        let render_context_resource = RendererContext {
            images,
            image_views,
            frame_overlap,
            draw_extent,
            frames_data,
            frame_number: Default::default(),
        };

        render_context_resource
    }
}
