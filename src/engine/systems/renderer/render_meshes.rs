use bevy_ecs::{
    entity::Entity,
    name::Name,
    system::{Query, Res, ResMut},
};

use crate::engine::{
    components::{mesh::Mesh, transform::Parent},
    resources::{FrameContext, RendererResources},
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

    command_buffer.draw_mesh_tasks_ext(graphics_entities.iter().len() as _, 1, 1);

    renderer_resources.is_printed_scene_hierarchy = true;
}
