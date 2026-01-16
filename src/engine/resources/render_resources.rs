pub mod allocation;
pub mod model_loader;

use ahash::HashMap;
use asset_importer::utils::mesh;
use bevy_ecs::resource::Resource;
use glam::{Mat4, Vec2, Vec3};
use vma::Allocation;
use vulkanite::vk::{
    DeviceAddress, Extent3D, Format, ImageSubresourceRange, ShaderStageFlags,
    rs::{Buffer, Image, ImageView, PipelineLayout, Sampler, ShaderEXT},
};

use crate::engine::{
    descriptors::DescriptorSetHandle, id::Id,
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
    pub index: usize,
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
    pub id: Id,
    pub index: usize,
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

#[derive(Clone, Copy)]
pub struct SamplerObject {
    pub id: Id,
    pub sampler: Sampler,
}

#[derive(Default)]
pub struct ResourcesPool {
    pub mesh_buffers: HashMap<Id, MeshBuffer>,
    pub textures: HashMap<Id, AllocatedImage>,
    pub samplers: HashMap<Id, SamplerObject>,
}

#[derive(Resource)]
pub struct RendererResources {
    pub depth_image: AllocatedImage,
    pub draw_image_id: Id,
    pub resources_descriptor_set_handle: DescriptorSetHandle,
    pub gradient_compute_shader_object: ShaderObject,
    pub mesh_shader_object: ShaderObject,
    pub fragment_shader_object: ShaderObject,
    pub model_loader: ModelLoader,
    pub resources_pool: ResourcesPool,
    pub mesh_pipeline_layout: PipelineLayout,
    pub mesh_push_constant: MeshPushConstant,
    pub nearest_sampler: Sampler,
}

impl<'a> RendererResources {
    pub fn insert_mesh_buffer(&'a mut self, mesh_buffer: MeshBuffer) -> Id {
        let mesh_buffer_id = mesh_buffer.id;

        let id = match self
            .resources_pool
            .mesh_buffers
            .insert(mesh_buffer.id, mesh_buffer)
        {
            Some(already_presented_mesh_buffer) => already_presented_mesh_buffer.id,
            None => mesh_buffer_id,
        };

        return id;
    }

    pub fn insert_texture(&'a mut self, allocated_image: AllocatedImage) -> Id {
        let allocated_image_id = allocated_image.id;

        let id = match self
            .resources_pool
            .textures
            .insert(allocated_image.id, allocated_image)
        {
            Some(already_presented_allocated_image) => already_presented_allocated_image.id,
            None => allocated_image_id,
        };

        return id;
    }

    pub fn insert_sampler(&'a mut self, sampler_object: SamplerObject) -> Id {
        let id = match self
            .resources_pool
            .samplers
            .insert(sampler_object.id, sampler_object)
        {
            Some(already_presented_sampler_object) => already_presented_sampler_object.id,
            None => sampler_object.id,
        };

        return id;
    }

    pub fn get_mesh_buffer(&'a self, id: Id) -> *const MeshBuffer {
        self.resources_pool.mesh_buffers.get(&id).unwrap()
    }

    pub fn get_texture(&'a self, id: Id) -> *const AllocatedImage {
        self.resources_pool.textures.get(&id).unwrap()
    }

    pub fn get_sampler(&'a self, id: Id) -> SamplerObject {
        *self.resources_pool.samplers.get(&id).unwrap()
    }
}
