use std::num::NonZero;

use bevy_ecs::system::{Res, ResMut};
use glam::Mat4;
use vulkanite::{
    Handle,
    vk::{
        self,
        raw::ShaderEXT,
        rs::{CommandBuffer, Image},
        *,
    },
};

use crate::engine::{
    resources::{
        FrameContext, MeshPushConstant, RendererContext, RendererResources, VulkanContextResource,
        vulkan_context_resource,
    },
    utils::{self, copy_image_to_image, image_subresource_range, transition_image},
};

pub fn render(
    vulkan_context_resource: Res<VulkanContextResource>,
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

    let draw_image_extent3d = renderer_resources.draw_image.image_extent;
    let draw_image_extent2d = Extent2D {
        width: draw_image_extent3d.width,
        height: draw_image_extent3d.height,
    };

    draw_gradient(
        renderer_resources.as_ref(),
        command_buffer,
        draw_image_extent2d,
    );

    let color_attachment_infos = [RenderingAttachmentInfo {
        image_view: Some(renderer_resources.draw_image.image_view.borrow()),
        image_layout: ImageLayout::General,
        resolve_mode: ResolveModeFlags::None,
        load_op: AttachmentLoadOp::Load,
        store_op: AttachmentStoreOp::Store,
        ..Default::default()
    }];

    let rendering_info = RenderingInfo {
        render_area: Rect2D {
            offset: Default::default(),
            extent: draw_image_extent2d,
        },
        layer_count: 1,
        color_attachment_count: color_attachment_infos.len() as _,
        p_color_attachments: color_attachment_infos.as_ptr(),
        ..Default::default()
    };

    command_buffer.begin_rendering(&rendering_info);

    let viewports = Viewport {
        width: draw_image_extent2d.width as _,
        height: draw_image_extent2d.height as _,
        min_depth: 0.0,
        max_depth: 1.0,
        ..Default::default()
    };
    let scissors = Rect2D {
        extent: draw_image_extent2d,
        ..Default::default()
    };

    command_buffer.set_viewport_with_count(&viewports);
    command_buffer.set_scissor_with_count(&scissors);
    command_buffer.set_cull_mode(CullModeFlags::Back);
    command_buffer.set_front_face(FrontFace::Clockwise);
    command_buffer.set_depth_test_enable_ext(false);
    command_buffer.set_depth_write_enable(false);
    command_buffer.set_primitive_restart_enable(false);
    command_buffer.set_rasterizer_discard_enable(false);
    command_buffer.set_primitive_topology(PrimitiveTopology::TriangleList);
    command_buffer.set_polygon_mode_ext(PolygonMode::Fill);
    command_buffer.set_rasterization_samples_ext(SampleCountFlags::Count1);
    command_buffer.set_alpha_to_coverage_enable_ext(false);
    command_buffer.set_depth_bias_enable(false);
    command_buffer.set_stencil_test_enable(false);
    command_buffer.set_sample_mask_ext(SampleCountFlags::Count1, &[SampleMask::MAX]);

    let blend_enables = [Bool32::False];
    command_buffer.set_color_blend_enable_ext(Default::default(), blend_enables.as_slice());

    let color_component_flags = [ColorComponentFlags::all()];
    command_buffer.set_color_write_mask_ext(Default::default(), &color_component_flags);

    let first_mesh = &renderer_resources.mesh_buffers[0];
    let mesh_push_constant = [MeshPushConstant {
        world_matrix: Mat4::IDENTITY,
        vertex_buffer_device_adress: first_mesh.vertex_buffer_device_address,
    }];

    command_buffer.push_constants(
        renderer_resources.mesh_pipeline_layout,
        ShaderStageFlags::MeshEXT,
        Default::default(),
        size_of::<MeshPushConstant>() as _,
        mesh_push_constant.as_ptr() as _,
    );

    let vertex_bindings_descriptions = [];
    let vertex_attributes = [];
    command_buffer.set_vertex_input_ext(&vertex_bindings_descriptions, &vertex_attributes);

    let shader_stages = [ShaderStageFlags::Vertex];
    use vulkanite::Dispatcher;

    unsafe {
        let dispatcher = vulkan_context_resource.device.get_dispatcher();
        let vulkan_command = dispatcher
            .get_command_dispatcher()
            .cmd_bind_shaders_ext
            .get();
        vulkan_command(
            Some(command_buffer.borrow()),
            1,
            shader_stages.as_slice().as_ptr().cast(),
            std::ptr::null(),
        );
    }

    let shader_stages = [
        renderer_resources.mesh_shader_object.stage,
        renderer_resources.fragment_shader_object.stage,
    ];
    let shaders = [
        *renderer_resources.mesh_shader_object.shader,
        *renderer_resources.fragment_shader_object.shader,
    ];

    command_buffer.bind_shaders_ext(shader_stages.as_slice(), shaders.as_slice());

    command_buffer.draw_mesh_tasks_ext(36, 1, 1);

    command_buffer.end_rendering();

    copy_image_to_image(
        command_buffer,
        draw_image,
        swapchain_image,
        draw_image_extent2d,
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

fn draw_triangle(renderer_resources: &RendererResources, command_buffer: CommandBuffer) {
    let vertex_bindings_descriptions = [];
    let vertex_attributes = [];
    command_buffer.set_vertex_input_ext(&vertex_bindings_descriptions, &vertex_attributes);

    let shader_stages = [
        renderer_resources.mesh_shader_object.stage,
        renderer_resources.fragment_shader_object.stage,
    ];
    let shaders = [
        renderer_resources.mesh_shader_object.shader,
        renderer_resources.fragment_shader_object.shader,
    ];

    command_buffer.bind_shaders_ext(shader_stages.as_slice(), shaders.as_slice());

    command_buffer.draw(3, 1, Default::default(), Default::default());
}

fn draw_gradient(
    renderer_resources: &RendererResources,
    command_buffer: CommandBuffer,
    draw_extent: Extent2D,
) {
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
        f32::ceil(draw_extent.width as f32 / 16.0) as _,
        f32::ceil(draw_extent.height as f32 / 16.0) as _,
        1,
    );
}

#[allow(unused)]
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
