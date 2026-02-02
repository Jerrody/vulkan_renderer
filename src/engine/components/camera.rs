use std::collections::HashMap;

use bevy_ecs::resource::Resource;
use dolly::prelude::*;
use glam::Vec3;
use winit::{event::ElementState, keyboard::KeyCode};

#[derive(Resource)]
pub struct Camera {
    speed: f32,
    camera_rig: CameraRig,
    keyboard_state: HashMap<KeyCode, ElementState>,
}

impl Camera {
    pub fn new(speed: f32) -> Self {
        let camera_rig = CameraRig::builder()
            .with(Position::new(Vec3::new(-85.45, 0.0, 2.52).to_array()))
            .with(YawPitch::default())
            .with(Smooth::new_position_rotation(1.0, 1.0))
            .build();

        let mut keyboard_state = HashMap::new();
        keyboard_state.insert(KeyCode::KeyW, ElementState::Released);
        keyboard_state.insert(KeyCode::KeyA, ElementState::Released);
        keyboard_state.insert(KeyCode::KeyS, ElementState::Released);
        keyboard_state.insert(KeyCode::KeyD, ElementState::Released);
        keyboard_state.insert(KeyCode::KeyE, ElementState::Released);
        keyboard_state.insert(KeyCode::KeyQ, ElementState::Released);

        Self {
            speed,
            camera_rig,
            keyboard_state,
        }
    }

    pub fn get_position(&self) -> Vec3 {
        let position = self.camera_rig.driver::<Position>().position;

        Vec3::new(position.x, position.y, position.z)
    }

    pub fn process_keycode(&mut self, key_code: KeyCode, new_state: ElementState) {
        if let Some(state) = self.keyboard_state.get_mut(&key_code) {
            *state = new_state;
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let camera_position = self.camera_rig.driver_mut::<Position>();
        let mut original_position = camera_position.position;

        let speed = self.speed;
        self.keyboard_state.iter().for_each(|(key_code, state)| {
            let is_pressed = state.is_pressed();

            if is_pressed {
                match key_code {
                    KeyCode::KeyW => {
                        original_position.z += speed;
                    }
                    KeyCode::KeyA => {
                        original_position.x += speed;
                    }
                    KeyCode::KeyS => {
                        original_position.z -= speed;
                    }
                    KeyCode::KeyD => {
                        original_position.x -= speed;
                    }
                    KeyCode::KeyE => {
                        original_position.y += speed;
                    }
                    KeyCode::KeyQ => {
                        original_position.y -= speed;
                    }
                    _ => {}
                }
            }
        });

        camera_position.position = original_position;
        self.camera_rig.update(delta_time);
    }
}
