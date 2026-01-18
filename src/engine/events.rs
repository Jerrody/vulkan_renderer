use bevy_ecs::event::Event;

use crate::engine::{components::transform::Transform, id::Id};

#[derive(Event)]
pub struct LoadModelEvent {
    pub path: String,
}

#[derive(Clone)]
pub struct SpawnEventRecord {
    pub name: String,
    pub parent_index: Option<usize>,
    pub mesh_buffer_id: Id,
    pub transform: Transform,
}

impl Default for SpawnEventRecord {
    fn default() -> Self {
        Self {
            name: String::default(),
            parent_index: Default::default(),
            mesh_buffer_id: Id::NULL,
            transform: Default::default(),
        }
    }
}

#[derive(Default, Event)]
pub struct SpawnEvent {
    pub spawn_records: Vec<SpawnEventRecord>,
}
