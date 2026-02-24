use bevy_ecs::component::Component;

use crate::engine::{
    components::transform::Transform,
    ecs::{materials_pool::MaterialReference, mesh_buffers_pool::MeshBufferReference},
};

#[derive(Component, Clone, Copy)]
#[require(Transform)]
pub struct Mesh {
    pub(crate) mesh_buffer_reference: MeshBufferReference,
    pub(crate) material_reference: MaterialReference,
}
