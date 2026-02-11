use bevy_ecs::{name::Name, observer::On, system::Commands};
use glam::{Quat, Vec3};

use crate::engine::{
    components::{
        mesh::Mesh,
        transform::{GlobalTransform, Parent, Transform},
    },
    events::SpawnEvent,
    id::Id,
};

pub fn on_spawn_mesh(spawn_event: On<SpawnEvent>, mut commands: Commands) {
    let scene_transform = Transform {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        local_scale: Vec3::ONE,
    };
    let scene_global_transform = GlobalTransform(scene_transform.get_matrix());

    let scene_entity_id = commands
        .spawn((Name::new("Scene"), scene_global_transform, scene_transform))
        .id();

    let mut spawned_entities = Vec::with_capacity(spawn_event.spawn_records.len());

    for spawn_event_record in spawn_event.spawn_records.iter() {
        let basic_components = (
            GlobalTransform(spawn_event_record.transform.get_matrix()),
            spawn_event_record.transform,
        );
        let mut spawned_entity = commands.spawn(basic_components);
        spawned_entities.push(spawned_entity.id());
        let mut name = Name::new(std::format!(
            "Entity ID: {}",
            spawn_event_record.name.as_str()
        ));

        let mesh_buffer_id = spawn_event_record.mesh_buffer_id;
        if mesh_buffer_id != Id::NULL {
            let mesh = Mesh {
                instance_object_index: None,
                mesh_buffer_id,
                material_id: spawn_event_record.material_id,
            };
            name.set(std::format!(
                "Mesh ID: {:?}",
                spawn_event_record.name.as_str()
            ));

            spawned_entity.insert(mesh);
        }

        let parent = if let Some(parent_index) = spawn_event_record.parent_index {
            Parent(spawned_entities[parent_index])
        } else {
            Parent(scene_entity_id)
        };

        spawned_entity.insert((name, parent));
    }
}
