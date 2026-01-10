use bevy_ecs::system::Res;

use crate::engine::{
    resources::{FrameContext, RendererContext, RendererResources},
    utils::{copy_image_to_image, transition_image},
};
use vulkanite::vk::*;

pub fn end_rendering(
    renderer_context: Res<RendererContext>,
    renderer_resources: Res<RendererResources>,
    frame_context: Res<FrameContext>,
) {
    let command_buffer = frame_context.command_buffer.unwrap();

    let swapchain_image = renderer_context.images[frame_context.swapchain_image_index as usize];

    let draw_image = renderer_resources.draw_image.image;

    let draw_image_extent3d = renderer_resources.draw_image.image_extent;
    let draw_image_extent2d = Extent2D {
        width: draw_image_extent3d.width,
        height: draw_image_extent3d.height,
    };

    command_buffer.end_rendering();

    copy_image_to_image(
        command_buffer,
        draw_image,
        swapchain_image,
        draw_image_extent2d,
        renderer_context.draw_extent,
    );

    transition_image(
        command_buffer,
        swapchain_image,
        ImageLayout::General,
        ImageLayout::PresentSrcKHR,
        ImageAspectFlags::Color,
    );

    command_buffer.end().unwrap();
}
