use bevy_ecs::system::{Res, ResMut};
use vulkanalia::vk::{CommandBufferResetFlags, DeviceV1_0, KhrSwapchainExtensionDeviceCommands};

use crate::engine::resources::{FrameContext, RenderContextResource, VulkanContextResource};

pub fn prepare_frame(
    vulkan_ctx: Res<VulkanContextResource>,
    render_ctx: Res<RenderContextResource>,
    mut frame_ctx: ResMut<FrameContext>,
) {
    let device = &vulkan_ctx.device;
    let frame_data = render_ctx.get_current_frame_data();
    let fences = [frame_data.render_fence];

    unsafe {
        device.wait_for_fences(&fences, true, u64::MAX).unwrap();
        device.reset_fences(&fences).unwrap();

        let (swapchain_image_index, _) = device
            .acquire_next_image_khr(
                *vulkan_ctx.swapchain.as_ref(),
                u64::MAX,
                frame_data.swapchain_semaphore,
                Default::default(),
            )
            .unwrap();
        frame_ctx.swapchain_image_index = swapchain_image_index;

        device
            .reset_command_buffer(
                frame_data.command_buffer,
                CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .unwrap();
    }
}
