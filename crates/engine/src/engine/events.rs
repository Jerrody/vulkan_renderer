use std::path::PathBuf;

use bevy_ecs::{entity::Entity, event::Event};

use crate::engine::{
    components::transform::Transform,
    ecs::{materials_pool::MaterialReference, mesh_buffers_pool::MeshBufferReference},
};

#[derive(Event)]
pub struct LoadModelEvent {
    pub path: PathBuf,
    pub parent_entity: Option<Entity>,
}

#[derive(Clone)]
pub struct SpawnEventRecord {
    pub name: String,
    pub parent_index: Option<usize>,
    pub mesh_buffer_reference: Option<MeshBufferReference>,
    pub material_reference: Option<MaterialReference>,
    pub transform: Transform,
}

impl Default for SpawnEventRecord {
    fn default() -> Self {
        Self {
            name: String::default(),
            parent_index: Default::default(),
            mesh_buffer_reference: Default::default(),
            material_reference: None,
            transform: Default::default(),
        }
    }
}

#[derive(Default, Event)]
pub struct SpawnEvent {
    pub spawn_records: Vec<SpawnEventRecord>,
    pub parent_entity: Option<Entity>,
}
