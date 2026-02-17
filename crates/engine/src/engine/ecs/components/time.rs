use std::time::Instant;

use bevy_ecs::resource::Resource;

// TODO: Move to the "resources" module.
#[derive(Resource)]
pub struct Time {
    delta_time: f32,
    last_frame: Instant,
}

impl Time {
    pub fn new() -> Self {
        Self {
            delta_time: Default::default(),
            last_frame: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }

    #[inline(always)]
    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let duration = now.duration_since(self.last_frame);

        self.delta_time = duration.as_secs_f32();
        self.last_frame = now;
    }
}
