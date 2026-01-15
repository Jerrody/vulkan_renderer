use bevy_ecs::component::Component;
use glam::Mat4;

#[derive(Default, Component)]
pub struct Transform {
    pub mat4: Mat4,
}
