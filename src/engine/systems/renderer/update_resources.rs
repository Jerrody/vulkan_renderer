use bevy_ecs::system::{Res, ResMut};
use glam::{Mat4, Vec3, Vec4};

use crate::engine::{
    components::camera::Camera,
    resources::{
        DirectionalLight, LightProperties, MemoryBucket, RendererContext, RendererResources,
        SceneData, SwappableBuffer, frame_context,
    },
};

pub fn update_resources(
    render_context: Res<RendererContext>,
    mut renderer_resources: ResMut<RendererResources>,
    mut frame_context: ResMut<frame_context::FrameContext>,
    camera: Res<Camera>,
) {
    let instances_objects_buffer = renderer_resources
        .resources_pool
        .instances_buffer
        .as_ref()
        .unwrap();

    let memory_bucket = &renderer_resources.resources_pool.memory_bucket;
    update_buffer_data(instances_objects_buffer, memory_bucket);

    let camera_position = camera.get_position();
    let view = Mat4::from_scale_rotation_translation(
        Vec3::ONE,
        camera.get_rotation(),
        camera.get_position(),
    )
    .inverse();

    let projection = Mat4::perspective_rh(
        70.0_f32.to_radians(),
        render_context.draw_extent.width as f32 / render_context.draw_extent.height as f32,
        10000.0,
        0.1,
    );

    frame_context.world_matrix = projection * view;

    let scene_data_buffer = renderer_resources
        .resources_pool
        .scene_data_buffer
        .as_mut()
        .unwrap();

    let scene_data = SceneData {
        camera_view_matrix: frame_context.world_matrix.to_cols_array(),
        camera_position,
        _padding: Default::default(),
        light_properties: LightProperties {
            ambient_color: Vec4::new(0.1, 0.1, 0.1, 1.0),
            ambient_strength: 0.1,
            specular_strength: 0.7,
            _padding: Default::default(),
        },
        directional_light: DirectionalLight {
            light_color: Vec3::new(0.72, 0.72, 0.93),
            light_position: Vec3::new(0.1, 0.5, 1.0),
            _padding: Default::default(),
        },
    };
    scene_data_buffer.write_data_to_current_buffer(&scene_data);

    let scene_data_buffer = renderer_resources
        .resources_pool
        .scene_data_buffer
        .as_ref()
        .unwrap();

    let memory_bucket = &renderer_resources.resources_pool.memory_bucket;
    update_buffer_data(scene_data_buffer, memory_bucket);
}

fn update_buffer_data(buffer_to_update: &SwappableBuffer, memory_bucket: &MemoryBucket) {
    let data_to_write = buffer_to_update.get_objects_to_write_as_slice();

    let buffer_to_update_reference = buffer_to_update.get_current_buffer();
    unsafe {
        memory_bucket.transfer_data_to_buffer(
            buffer_to_update_reference,
            data_to_write,
            data_to_write.len(),
        );
    }
}
