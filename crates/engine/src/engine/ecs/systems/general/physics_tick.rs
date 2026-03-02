use bevy_ecs::system::ResMut;

use crate::engine::ecs::physics::PhysicsManager;

pub fn physics_tick_system(mut physics: ResMut<PhysicsManager>) {
    physics.step();
}
