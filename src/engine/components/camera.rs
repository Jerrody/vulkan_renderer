use bevy_ecs::component::Component;
use dolly::prelude::*;
use winit::keyboard::KeyCode;

#[derive(Component)]
pub struct Camera {
    camera_rig: CameraRig,
}

impl Camera {
    pub fn new(camera_rig: CameraRig) -> Self {
        Self { camera_rig }
    }

    pub fn process_keycode(&mut self, key_code: KeyCode) {
        let camera_position = self.camera_rig.driver_mut::<Position>();
        let mut original_position = camera_position.position;
        match key_code {
            KeyCode::KeyW => {
                original_position.z += 1.0;
            }
            KeyCode::KeyA => {
                original_position.z += 1.0;
            }
            KeyCode::KeyS => {
                original_position.z += 1.0;
            }
            KeyCode::KeyD => {
                original_position.z += 1.0;
            }
            KeyCode::KeyE => {
                original_position.z += 1.0;
            }
            KeyCode::KeyQ => {
                original_position.z += 1.0;
            }
            _ => {}
        }
    }
}
