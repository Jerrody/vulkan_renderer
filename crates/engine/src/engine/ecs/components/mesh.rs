use bevy_ecs::{component::Component, name::Name};

use crate::engine::{
    components::transform::Transform, ecs::mesh_buffers_pool::MeshBufferReference, id::Id,
};

#[derive(Component)]
#[require(Transform, Name)]
pub struct Mesh {
    pub instance_object_index: Option<usize>,
    pub mesh_buffer_reference: MeshBufferReference,
    pub material_id: Id,
}
