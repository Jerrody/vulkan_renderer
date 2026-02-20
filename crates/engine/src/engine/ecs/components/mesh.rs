use bevy_ecs::{component::Component, name::Name};

use crate::engine::{
    components::transform::Transform,
    ecs::{materials_pool::MaterialReference, mesh_buffers_pool::MeshBufferReference},
};

#[derive(Component)]
#[require(Transform, Name)]
pub struct Mesh {
    pub(crate) instance_object_index: Option<usize>,
    pub(crate) mesh_buffer_reference: MeshBufferReference,
    pub(crate) material_reference: MaterialReference,
}
