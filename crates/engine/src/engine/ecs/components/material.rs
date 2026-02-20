use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use vulkanite::vk::DeviceAddress;

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
pub struct MaterialProperties {
    pub base_color: [f32; 4],
    pub metallic_value: f32,
    pub roughness_value: f32,
}

impl MaterialProperties {
    pub fn new(base_color: Vec4, metallic_value: f32, roughness_value: f32) -> Self {
        Self {
            base_color: base_color.to_array(),
            metallic_value,
            roughness_value,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialTextures {
    pub albedo_texture_index: u32,
    pub metallic_texture_index: u32,
    pub roughness_texture_index: u32,
}

impl MaterialTextures {
    pub fn new(
        albedo_texture_index: u32,
        metallic_texture_index: u32,
        roughness_texture_index: u32,
    ) -> Self {
        Self {
            albedo_texture_index,
            metallic_texture_index,
            roughness_texture_index,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialData {
    pub material_properties: MaterialProperties,
    pub material_textures: MaterialTextures,
    pub sampler_index: u32,
}

pub struct Material {
    pub ptr_data: DeviceAddress,
    pub state: MaterialState,
}
