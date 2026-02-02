use bevy_ecs::system::ResMut;

use crate::engine::components::time::Time;

pub fn update_time(mut time: ResMut<Time>) {
    time.update();
}
