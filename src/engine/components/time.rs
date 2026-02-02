use std::time::Instant;

use bevy_ecs::{component::Component, resource::Resource};

#[derive(Resource)]
pub struct Time {
    delta_time: f32,
    last_frame: f32,
}

impl Time {
    pub fn new() -> Self {
        let last_time = Instant::now();

        Self {
            delta_time: Default::default(),
            last_frame: last_time.elapsed().as_secs_f32(),
        }
    }

    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now().elapsed().as_secs_f32();

        self.delta_time = now - self.last_frame;
        self.last_frame = now;
    }
}
