use bevy_ecs::system::ResMut;

use crate::engine::Audio;

pub fn check_audio_state_system(mut audio: ResMut<Audio>) {
    audio.check_audio_state();
}
