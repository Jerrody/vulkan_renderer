use bevy_ecs::system::{Res, ResMut};
use vulkanalia::vk::{
    DeviceV1_3, HasBuilder, KhrSwapchainExtensionDeviceCommands, PipelineStageFlags2,
    PresentInfoKHR,
};

use crate::engine::{
    resources::{FrameContext, RendererContext, VulkanContextResource},
    utils::{command_buffer_submit_info, semaphore_submit_info, submit_info},
};

pub fn present(
    vulkan_ctx: Res<VulkanContextResource>,
    mut render_ctx: ResMut<RendererContext>,
    frame_ctx: Res<FrameContext>,
) {
    let device = &vulkan_ctx.device;
    let frame_data = render_ctx.get_current_frame_data();
    let command_buffer = frame_data.command_buffer;
    let swapchain_image_index = frame_ctx.swapchain_image_index;

    let command_buffer_submit_infos = [command_buffer_submit_info(command_buffer)];

    let wait_semaphore_submit_infos = [semaphore_submit_info(
        PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        frame_data.swapchain_semaphore,
    )];
    let signal_semaphore_submit_infos = [semaphore_submit_info(
        PipelineStageFlags2::ALL_GRAPHICS,
        frame_data.render_semaphore,
    )];

    let submit_info = submit_info(
        &command_buffer_submit_infos,
        &wait_semaphore_submit_infos,
        &signal_semaphore_submit_infos,
    );

    let submit_infos = [submit_info];
    unsafe {
        device
            .queue_submit2(
                vulkan_ctx.graphics_queue_data.queue,
                &submit_infos,
                frame_data.render_fence,
            )
            .unwrap();
    }

    let swapchains = [*vulkan_ctx.swapchain.as_ref()];
    let wait_semaphores = [frame_data.render_semaphore];
    let image_indicies = [swapchain_image_index];

    let present_info = PresentInfoKHR::builder()
        .swapchains(&swapchains)
        .wait_semaphores(&wait_semaphores)
        .image_indices(&image_indicies)
        .build();

    unsafe {
        device
            .queue_present_khr(vulkan_ctx.graphics_queue_data.queue, &present_info)
            .unwrap();
    }

    render_ctx.frame_number += 1;
}
