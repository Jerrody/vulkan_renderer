use bevy_ecs::system::{Res, ResMut};
use vulkanalia::vk::{CommandBufferUsageFlags, DeviceV1_0, ImageAspectFlags, ImageLayout};

use crate::engine::{
    resources::{FrameContext, RenderContextResource, VulkanContextResource},
    utils::{self, image_subresource_range, transition_image},
};

pub fn render(
    vulkan_ctx: Res<VulkanContextResource>,
    render_context: ResMut<RenderContextResource>,
    frame_context: Res<FrameContext>,
) {
    let device = &vulkan_ctx.device;
    let frame_data = render_context.get_current_frame_data();

    let command_buffer = frame_data.command_buffer;

    let command_buffer_begin_info =
        utils::create_command_buffer_begin_info(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .unwrap();
    }

    let image_index = frame_context.swapchain_image_index as usize;
    let image = render_context.images[image_index];
    transition_image(
        device,
        command_buffer,
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
            command_buffer,
            image,
            ImageLayout::GENERAL,
            &clear_value,
            &ranges,
        );
    }

    transition_image(
        device,
        command_buffer,
        image,
        ImageLayout::GENERAL,
        ImageLayout::PRESENT_SRC_KHR,
    );

    unsafe {
        device.end_command_buffer(command_buffer).unwrap();
    }
}
