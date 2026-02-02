use std::collections::HashMap;

use bevy_ecs::resource::Resource;
use dolly::prelude::*;
use glam::{EulerRot, Quat, Vec2, Vec3};
use winit::{event::ElementState, keyboard::KeyCode};

#[derive(Resource)]
pub struct Camera {
    move_speed: f32,
    rotation_speed: f32,
    camera_rig: CameraRig,
    keyboard_state: HashMap<KeyCode, ElementState>,
    mouse_delta: Vec2,
}

impl Camera {
    pub fn new(move_speed: f32, rotation_speed: f32) -> Self {
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
            move_speed,
            rotation_speed,
            camera_rig,
            keyboard_state,
            mouse_delta: Vec2::default(),
        }
    }

    pub fn get_position(&self) -> Vec3 {
        let position = self.camera_rig.driver::<Position>().position;

        Vec3::new(position.x, position.y, position.z)
    }

    pub fn get_rotation(&self) -> Quat {
        let yaw_pitch = self.camera_rig.driver::<YawPitch>();

        let rotation = Quat::from_euler(
            EulerRot::YXZ,
            yaw_pitch.yaw_degrees.to_radians(),
            yaw_pitch.pitch_degrees.to_radians(),
            Default::default(),
        );

        rotation
    }

    pub fn process_keycode(&mut self, key_code: KeyCode, new_state: ElementState) {
        if let Some(state) = self.keyboard_state.get_mut(&key_code) {
            *state = new_state;
        }
    }

    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta = Vec2::new(delta_x, delta_y).normalize();

        self.camera_rig.driver_mut::<YawPitch>().rotate_yaw_pitch(
            self.rotation_speed * self.mouse_delta.x,
            self.rotation_speed * self.mouse_delta.y,
        );
    }

    pub fn update(&mut self, delta_time: f32) {
        let camera_position = self.camera_rig.driver_mut::<Position>();
        let mut original_position = camera_position.position;

        let move_speed = self.move_speed;
        self.keyboard_state.iter().for_each(|(key_code, state)| {
            let is_pressed = state.is_pressed();

            if is_pressed {
                match key_code {
                    KeyCode::KeyW => {
                        original_position.z += move_speed;
                    }
                    KeyCode::KeyA => {
                        original_position.x += move_speed;
                    }
                    KeyCode::KeyS => {
                        original_position.z -= move_speed;
                    }
                    KeyCode::KeyD => {
                        original_position.x -= move_speed;
                    }
                    KeyCode::KeyE => {
                        original_position.y += move_speed;
                    }
                    KeyCode::KeyQ => {
                        original_position.y -= move_speed;
                    }
                    _ => {}
                }
            }
        });

        //println!("{}", self.mouse_delta);
        camera_position.position = original_position;

        self.camera_rig.update(delta_time);
    }
}
