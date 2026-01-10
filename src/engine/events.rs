use bevy_ecs::event::Event;

use crate::engine::id::Id;

#[derive(Event)]
pub struct LoadModelEvent {
    pub path: String,
}

#[derive(Event)]
pub struct SpawnMeshEvent {
    pub mesh_buffer_id: Id,
}
