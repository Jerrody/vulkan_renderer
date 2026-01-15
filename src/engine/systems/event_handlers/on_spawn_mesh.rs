use bevy_ecs::{name::Name, observer::On, system::Commands};
use uuid::Uuid;

use crate::engine::{components::mesh::Mesh, events::SpawnMeshEvent, id::Id};

pub fn on_spawn_mesh(spawn_mesh_event: On<SpawnMeshEvent>, mut commands: Commands) {
    let mesh_buffer_id = spawn_mesh_event.mesh_buffer_id;

    let mesh = Mesh {
        id: Id::new(Uuid::new_v4()),
        buffer_id: mesh_buffer_id,
        material_id: Id::NULL,
    };
    let entity_id = std::format!("entity_{}", mesh.id.value());

    commands.spawn((mesh, Name::new(entity_id)));
}
