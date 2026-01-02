use std::u64;

use bevy_ecs::system::{Res, ResMut};
use vulkanalia::vk::{
    CommandBufferResetFlags, CommandBufferUsageFlags, DeviceV1_0, DeviceV1_3, HasBuilder,
    ImageAspectFlags, ImageLayout, KhrSwapchainExtensionDeviceCommands, PipelineStageFlags2,
    PresentInfoKHR,
};

use crate::engine::{
    resources::{RenderContextResource, VulkanContextResource},
    utils::{
        self, command_buffer_submit_info, image_subresource_range, semaphore_submit_info,
        submit_info, transition_image,
    },
};

pub fn render(
    vulkan_ctx: Res<VulkanContextResource>,
    mut render_context: ResMut<RenderContextResource>,
) {
    let device = &vulkan_ctx.device;

    let frame_data = render_context.get_current_frame_data();

    let fences = [frame_data.render_fence];
    unsafe {
        device.wait_for_fences(&fences, true, u64::MAX).unwrap();
        device.reset_fences(&fences).unwrap();
    }

    let (swapchain_image_index, _) = unsafe {
        device
            .acquire_next_image_khr(
                *vulkan_ctx.swapchain.as_ref(),
                u64::MAX,
                frame_data.swapchain_semaphore,
                Default::default(),
            )
            .unwrap()
    };

    unsafe {
        device
            .reset_command_buffer(
                frame_data.command_buffer,
                CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .unwrap();
    }

    let command_buffer_begin_info =
        utils::create_command_buffer_begin_info(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device
            .begin_command_buffer(frame_data.command_buffer, &command_buffer_begin_info)
            .unwrap();
    }

    let image_index = swapchain_image_index as usize;
    let image = render_context.images[image_index];
    transition_image(
        device,
        frame_data.command_buffer,
        image,
        ImageLayout::UNDEFINED,
        ImageLayout::GENERAL,
    );

    let flash = f32::abs(f32::sin(render_context.frame_number as f32 / 120.0));
    let clear_value = vulkanalia::vk::ClearColorValue {
        float32: [0.0, 0.0, flash, 1.0],
    };

    let clear_range = image_subresource_range(ImageAspectFlags::COLOR);

    let ranges = [clear_range];
    unsafe {
        device.cmd_clear_color_image(
            frame_data.command_buffer,
            image,
            ImageLayout::GENERAL,
            &clear_value,
            &ranges,
        );
    }

    transition_image(
        device,
        frame_data.command_buffer,
        image,
        ImageLayout::GENERAL,
        ImageLayout::PRESENT_SRC_KHR,
    );

    unsafe {
        device
            .end_command_buffer(frame_data.command_buffer)
            .unwrap();
    }

    let command_buffer_submit_infos = [command_buffer_submit_info(frame_data.command_buffer)];

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

    render_context.frame_number += 1;
}
