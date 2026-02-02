use bevy_ecs::system::{Res, ResMut};

use crate::engine::components::{camera::Camera, time::Time};

pub fn update_camera(mut camera: ResMut<Camera>, time: Res<Time>) {
    camera.update(time.get_delta_time());
}
