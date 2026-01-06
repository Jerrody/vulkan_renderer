use bevy_ecs::system::{Res, ResMut};
use vulkanite::vk::{
    rs::{CommandBuffer, Image},
    *,
};

use crate::engine::{
    resources::{FrameContext, RendererContext, RendererResources},
    utils::{self, copy_image_to_image, image_subresource_range, transition_image},
};

pub fn render(
    render_context: ResMut<RendererContext>,
    renderer_resources: Res<RendererResources>,
    frame_context: Res<FrameContext>,
) {
    let frame_data = render_context.get_current_frame_data();

    let command_buffer = frame_data.command_buffer;

    let command_buffer_begin_info =
        utils::create_command_buffer_begin_info(CommandBufferUsageFlags::OneTimeSubmit);

    command_buffer.begin(&command_buffer_begin_info).unwrap();

    let image_index = frame_context.swapchain_image_index as usize;
    let swapchain_image = render_context.images[image_index];
    let draw_image = renderer_resources.draw_image.image;
    transition_image(
        command_buffer,
        draw_image,
        ImageLayout::Undefined,
        ImageLayout::General,
    );
    transition_image(
        command_buffer,
        swapchain_image,
        ImageLayout::Undefined,
        ImageLayout::General,
    );

    let gradient_compute_shader_object = renderer_resources.gradient_compute_shader_object;

    let stages = [gradient_compute_shader_object.stage];
    let shaders = [gradient_compute_shader_object.shader];

    command_buffer.bind_shaders_ext(stages.as_slice(), shaders.as_slice());

    let descriptor_binding_info = DescriptorBufferBindingInfoEXT::default()
        .usage(BufferUsageFlags::ResourceDescriptorBufferEXT)
        .address(renderer_resources.draw_image_descriptor_buffer.address);
    let descriptor_binding_infos = [descriptor_binding_info];
    command_buffer.bind_descriptor_buffers_ext(&descriptor_binding_infos);

    let buffer_indices = [0];
    let offsets = [0];
    command_buffer.set_descriptor_buffer_offsets_ext(
        PipelineBindPoint::Compute,
        renderer_resources
            .draw_image_descriptor_buffer
            .pipeline_layout,
        Default::default(),
        &buffer_indices,
        &offsets,
    );

    command_buffer.dispatch(
        f32::ceil(render_context.draw_extent.width as f32 / 16.0) as _,
        f32::ceil(render_context.draw_extent.height as f32 / 16.0) as _,
        1,
    );

    //draw_background(&render_context, command_buffer, &draw_image);

    let draw_image_extent = renderer_resources.draw_image.image_extent;

    copy_image_to_image(
        command_buffer,
        draw_image,
        swapchain_image,
        Extent2D {
            width: draw_image_extent.width,
            height: draw_image_extent.height,
        },
        render_context.draw_extent,
    );

    transition_image(
        command_buffer,
        swapchain_image,
        ImageLayout::General,
        ImageLayout::PresentSrcKHR,
    );

    command_buffer.end().unwrap();
}

fn draw_background(
    render_context: &ResMut<RendererContext>,
    command_buffer: CommandBuffer,
    draw_image: &Image,
) {
    let flash = f32::abs(f32::sin(render_context.frame_number as f32 / 120.0));
    let clear_value = ClearColorValue {
        float32: [0.0, 0.0, flash, 1.0],
    };

    let clear_range = image_subresource_range(ImageAspectFlags::Color);

    let ranges = [clear_range];
    command_buffer.clear_color_image(*draw_image, ImageLayout::General, &clear_value, &ranges);
}
