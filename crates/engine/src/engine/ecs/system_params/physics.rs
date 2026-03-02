use bevy_ecs::{
    entity::Entity,
    system::{Commands, Res, ResMut, SystemParam},
};
use rapier3d::{
    math::Vec3,
    prelude::{RigidBodyBuilder, RigidBodyHandle},
};

use crate::engine::{
    Mesh, Transform,
    ecs::{
        mesh_buffers_pool::{MeshBufferReference, MeshBuffersPool},
        physics::{PhysicsManager, RigidBody},
    },
};

#[derive(SystemParam)]
pub struct Physics<'w, 's> {
    commands: Commands<'w, 's>,
    pub(crate) physics_manager: ResMut<'w, PhysicsManager>,
    mesh_buffers_pool: Res<'w, MeshBuffersPool>,
}

impl<'w, 's> Physics<'w, 's> {
    pub fn create_mesh_collider_from_mesh(
        &mut self,
        target_entity: Entity,
        mesh: Mesh,
        rigid_body: RigidBody,
    ) {
        let mesh_buffer_reference = mesh.mesh_buffer_reference;
        let mesh_buffer = self
            .mesh_buffers_pool
            .get_mesh_buffer(mesh_buffer_reference)
            .unwrap();

        let collider = self
            .physics_manager
            .create_mesh_collider(&mesh_buffer.mesh_data, rigid_body.rigid_body_handle);

        let mut entity_commands = self.commands.entity(target_entity);
        entity_commands.insert(collider);
    }

    pub fn create_rigid_body(&mut self, target_entity: Entity, transform: &Transform) -> RigidBody {
        let rigid_body = self
            .physics_manager
            .create_rigid_body(transform.local_position.to_array(), None);

        let mut entity_commands = self.commands.entity(target_entity);
        entity_commands.insert(rigid_body);

        rigid_body
    }
}
