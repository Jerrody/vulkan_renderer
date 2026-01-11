pub mod allocation;
pub mod model_loader;

use bevy_ecs::resource::Resource;
use glam::{Mat4, Vec2, Vec3};
use vma::Allocation;
use vulkanite::vk::{
    DeviceAddress, Extent3D, Format, ImageSubresourceRange, ShaderStageFlags,
    rs::{Buffer, DescriptorSetLayout, Image, ImageView, PipelineLayout, Sampler, ShaderEXT},
};

use crate::engine::{
    descriptors::AllocatedDescriptorSetBuffer, id::Id,
    resources::render_resources::model_loader::ModelLoader,
};

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

#[derive(Default)]
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
    pub extent: Extent3D,
    pub format: Format,
    pub subresource_range: ImageSubresourceRange,
}

pub struct AllocatedBuffer {
    pub buffer: Buffer,
    pub allocation: Allocation,
    pub device_address: DeviceAddress,
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
    pub white_image: AllocatedImage,
    pub draw_image_descriptor_buffer: AllocatedDescriptorSetBuffer,
    pub gradient_compute_shader_object: ShaderObject,
    pub mesh_shader_object: ShaderObject,
    pub fragment_shader_object: ShaderObject,
    pub model_loader: ModelLoader,
    pub mesh_buffers: Vec<MeshBuffer>,
    pub mesh_pipeline_layout: PipelineLayout,
    pub mesh_push_constant: MeshPushConstant,
    pub nearest_sampler: Sampler,
}

impl<'a> RendererResources {
    pub fn get_mesh_buffer(&'a self, id: Id) -> *const MeshBuffer {
        let found_mesh_buffer = self
            .mesh_buffers
            .iter()
            .find(|&mesh_buffer| mesh_buffer.id == id);

        found_mesh_buffer.unwrap()
    }
}
