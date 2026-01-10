pub mod allocation;
pub mod model_loader;

use bevy_ecs::resource::Resource;
use glam::{Mat4, Vec2, Vec3};
use vma::Allocation;
use vulkanite::vk::{
    DeviceAddress, Extent3D, Format, ShaderStageFlags,
    rs::{Buffer, DescriptorSetLayout, Image, ImageView, PipelineLayout, ShaderEXT},
};

use crate::engine::{id::Id, resources::render_resources::model_loader::ModelLoader};

#[derive(Clone, Copy)]
#[repr(C, align(4))]
pub struct Meshlet {
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
}

#[derive(Default, Clone, Copy)]
#[repr(C, align(4))]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct MeshBuffer {
    pub id: Id,
    pub vertex_buffer: AllocatedBuffer,
    pub vertex_indices_buffer: AllocatedBuffer,
    pub meshlets_buffer: AllocatedBuffer,
    pub local_indices_buffer: AllocatedBuffer,
    pub meshlets_count: usize,
}

#[repr(C, align(4))]
pub struct MeshPushConstant {
    pub world_matrix: Mat4,
    pub meshlets_device_address: DeviceAddress,
    pub vertex_buffer_device_adress: DeviceAddress,
    pub vertex_indices_device_address: DeviceAddress,
    pub local_indices_device_address: DeviceAddress,
}

pub struct AllocatedImage {
    pub image: Image,
    pub image_view: ImageView,
    pub allocation: Allocation,
    pub image_extent: Extent3D,
    pub format: Format,
}

pub struct AllocatedBuffer {
    pub buffer: Buffer,
    pub allocation: Allocation,
    pub device_address: DeviceAddress,
}

pub struct AllocatedDescriptorBuffer {
    pub allocated_descriptor_buffer: AllocatedBuffer,
    pub descriptor_buffer_offset: u64,
    pub descriptor_buffer_size: u64,
    pub descriptor_set_layout: DescriptorSetLayout,
    pub address: DeviceAddress,
    pub pipeline_layout: PipelineLayout,
}

#[derive(Clone, Copy)]
pub struct ShaderObject {
    pub shader: ShaderEXT,
    pub stage: ShaderStageFlags,
}

impl ShaderObject {
    pub fn new(shader: ShaderEXT, stage: ShaderStageFlags) -> Self {
        Self { shader, stage }
    }
}

#[derive(Resource)]
pub struct RendererResources {
    pub draw_image: AllocatedImage,
    pub depth_image: AllocatedImage,
    pub draw_image_descriptor_buffer: AllocatedDescriptorBuffer,
    pub gradient_compute_shader_object: ShaderObject,
    pub mesh_shader_object: ShaderObject,
    pub fragment_shader_object: ShaderObject,
    pub model_loader: ModelLoader,
    pub mesh_buffers: Vec<MeshBuffer>,
    pub mesh_pipeline_layout: PipelineLayout,
}
