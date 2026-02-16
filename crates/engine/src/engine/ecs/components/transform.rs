use bevy_ecs::{component::Component, entity::Entity};
use glam::{Mat4, Quat, Vec3};

use crate::engine::ecs::{on_add_parent, on_remove_parent};

#[derive(Default, Clone, Copy, Component)]
#[require(GlobalTransform)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub local_scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        local_scale: Vec3::ONE,
    };

    pub fn get_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.local_scale, self.rotation, self.position)
    }
}

#[derive(Default, Component)]
pub struct GlobalTransform(pub Mat4);

#[derive(Clone, Copy, Component)]
#[component(on_add = on_add_parent, on_remove = on_remove_parent)]
pub struct Parent(pub Entity);

#[derive(Default, Component)]
pub struct Children(pub Vec<Entity>);
