use bevy_ecs::{component::Component, name::Name};

use crate::engine::{components::transform::Transform, id::Id};

#[derive(Component)]
#[require(Transform, Name)]
pub struct Mesh {
    pub id: Id,
    pub instance_object_index: Option<usize>,
    pub mesh_buffer_id: Id,
    pub texture_id: Id,
}
