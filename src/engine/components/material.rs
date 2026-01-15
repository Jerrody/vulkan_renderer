use bevy_ecs::component::Component;
use glam::Vec4;

use crate::engine::id::Id;

#[derive(Component)]
pub struct Material {
    pub id: Id,
    pub data: MaterialData,
    pub state: MaterialState,
}

#[repr(C)]
pub struct MaterialData {
    pub color: Vec4,
    pub metallic: Vec4,
    pub duffuse_texture_index: u64,
}

pub struct MaterialState {
    pub depth_test: bool,
    pub depth_write: bool,
}
