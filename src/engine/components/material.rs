use glam::Vec4;
use vulkanite::vk::DeviceAddress;

use crate::engine::id::Id;

pub struct MaterialState {
    pub depth_test: bool,
    pub depth_write: bool,
}

#[repr(C)]
pub struct MaterialData {
    pub color: Vec4,
    pub texture_index: u32,
    pub sampler_index: u32,
}

pub struct Material {
    pub id: Id,
    pub ptr_data: DeviceAddress,
    pub state: MaterialState,
}
