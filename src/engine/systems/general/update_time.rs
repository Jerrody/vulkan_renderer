use bevy_ecs::system::ResMut;

use crate::engine::components::time::Time;

pub fn update_time_system(mut time: ResMut<Time>) {
    time.update();
}
