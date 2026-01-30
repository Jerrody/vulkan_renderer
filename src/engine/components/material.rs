use bytemuck::{Pod, Zeroable};
use vulkanite::vk::DeviceAddress;

use crate::engine::id::Id;

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum MaterialType {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Clone, Copy)]
pub struct MaterialState {
    pub material_type: MaterialType,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialData {
    pub color: [f32; 4],
    pub texture_index: u32,
    pub sampler_index: u32,
}

pub struct Material {
    pub id: Id,
    pub ptr_data: DeviceAddress,
    pub state: MaterialState,
}
