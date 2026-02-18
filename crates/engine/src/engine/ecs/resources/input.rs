use std::collections::HashSet;

//use ahash::{AHashSet, HashSet};
use bevy_ecs::resource::Resource;
use winit::keyboard::KeyCode;

#[derive(Resource)]
pub struct Input {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl Input {
    const CAPACITY: usize = KeyCode::ZoomToggle as usize;

    pub(crate) fn new() -> Self {
        Self {
            pressed: HashSet::with_capacity(Self::CAPACITY),
            just_pressed: HashSet::with_capacity(Self::CAPACITY),
            just_released: HashSet::with_capacity(Self::CAPACITY),
        }
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    #[inline(always)]
    pub(crate) fn press(&mut self, key: KeyCode) {
        if !self.pressed.contains(&key) {
            self.just_pressed.insert(key);
        }
        self.pressed.insert(key);
    }

    #[inline(always)]
    pub(crate) fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    #[inline(always)]
    pub(crate) fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
