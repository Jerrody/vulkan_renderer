use bevy_ecs::{component::Component, entity::Entity};
use glam::{Mat4, Quat, Vec3};

use crate::engine::systems::{on_add_parent, on_remove_parent};

#[derive(Default, Component)]
#[require(GlobalTransform)]
pub struct Transform {
    pub position: Vec3,
}

impl Transform {
    pub fn get_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(Quat::IDENTITY, self.position)
    }
}

#[derive(Default, Component)]
pub struct GlobalTransform(pub Mat4);

#[derive(Component)]
#[component(on_add = on_add_parent, on_remove = on_remove_parent)]
pub struct Parent(pub Entity);

#[derive(Default, Component)]
pub struct Children(pub Vec<Entity>);
