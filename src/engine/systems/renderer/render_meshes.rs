use bevy_ecs::{
    entity::Entity,
    name::Name,
    system::{Query, Res, ResMut},
};
use vulkanite::vk::ShaderStageFlags;

use crate::engine::{
    components::{mesh::Mesh, transform::Parent},
    resources::{FrameContext, GraphicsPushConstant, RendererResources},
};

pub fn render_meshes(
    graphics_entities: Query<&Mesh>,
    entities: Query<(Entity, &Name)>,
    entities_with_parent: Query<&Parent>,
    mut renderer_resources: ResMut<RendererResources>,
    frame_context: Res<FrameContext>,
) {
    let command_buffer = frame_context.command_buffer.unwrap();

    if !renderer_resources.is_printed_scene_hierarchy {
        println!("=====================================");

        for (entity, name) in entities.iter() {
            if let Ok(parent) = entities_with_parent.get(entity) {
                println!("Entity: {} | Name: {} | Parent: {}", entity, name, parent.0);
            } else {
                println!("Entity: {} | Name: {}", entity, name);
            }
        }

        println!("=====================================");
    }

    let instance_objects_buffer_id = renderer_resources.get_current_instance_set_buffer_id();
    let device_address_instance_objects_buffer = renderer_resources
        .get_storage_buffer_ref(instance_objects_buffer_id)
        .device_address;

    let texture_image_index = renderer_resources
        .get_texture_ref(renderer_resources.draw_image_id)
        .index;
    let nearest_sampler_index = renderer_resources
        .get_sampler(renderer_resources.nearest_sampler_id)
        .index;

    let mesh_push_constant = &GraphicsPushConstant {
        view_projection: frame_context.world_matrix,
        device_address_instance_object: device_address_instance_objects_buffer,
        texture_image_index: texture_image_index as _,
        sampler_index: nearest_sampler_index as _,
        ..Default::default()
    };

    let p_mesh_push_constant = mesh_push_constant as *const GraphicsPushConstant;
    command_buffer.push_constants(
        renderer_resources
            .resources_descriptor_set_handle
            .pipeline_layout,
        ShaderStageFlags::MeshEXT
            | ShaderStageFlags::Fragment
            | ShaderStageFlags::Compute
            | ShaderStageFlags::TaskEXT,
        Default::default(),
        size_of::<GraphicsPushConstant>() as u32
            - std::mem::size_of_val(&mesh_push_constant.draw_image_index) as u32,
        p_mesh_push_constant as _,
    );

    command_buffer.draw_mesh_tasks_ext(graphics_entities.iter().len() as _, 1, 1);

    renderer_resources.is_printed_scene_hierarchy = true;
}
