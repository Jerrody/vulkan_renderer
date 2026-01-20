pub mod allocation;
pub mod model_loader;

use std::slice::{Iter, IterMut};

use bevy_ecs::resource::Resource;
use glam::{Mat4, Vec2, Vec3};
use vma::Allocation;
use vulkanite::{
    Handle,
    vk::{
        DeviceAddress, DeviceSize, Extent3D, Format, ImageSubresourceRange, ShaderStageFlags,
        rs::{Buffer, Image, ImageView, Sampler, ShaderEXT},
    },
};

use crate::engine::{
    descriptors::DescriptorSetHandle, id::Id,
    resources::render_resources::model_loader::ModelLoader,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Meshlet {
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[repr(C)]
pub struct MeshObject {
    pub vertex_buffer_address: DeviceAddress,
    pub vertex_indices_buffer_address: DeviceAddress,
    pub meshlets_buffer_address: DeviceAddress,
    pub local_indices_buffer_address: DeviceAddress,
}

#[repr(C)]
pub struct InstanceObject {
    pub model_matrix: Mat4,
    pub device_address_mesh_object: DeviceAddress,
}

#[repr(C)]
#[derive(Default)]
pub struct GraphicsPushConstant {
    pub view_projection: Mat4,
    pub device_address_instance_object: DeviceAddress,
    pub sampler_index: u32,
    pub texture_image_index: u32,
    pub draw_image_index: u32,
}

pub struct MeshBuffer {
    pub id: Id,
    pub vertex_buffer_id: Id,
    pub vertex_indices_buffer_id: Id,
    pub meshlets_buffer_id: Id,
    pub local_indices_buffer_id: Id,
    pub meshlets_count_id: usize,
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
    pub id: Id,
    pub buffer: Buffer,
    pub allocation: Allocation,
    pub device_address: DeviceAddress,
    pub size: DeviceSize,
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
    pub index: usize,
    pub sampler: Sampler,
}

impl SamplerObject {
    pub fn new(sampler: Sampler) -> Self {
        Self {
            id: Id::new(sampler.as_raw()),
            index: usize::MIN,
            sampler: sampler,
        }
    }
}

#[derive(Default)]
pub struct ResourcesPool {
    pub mesh_buffers: Vec<MeshBuffer>,
    pub storage_buffers: Vec<AllocatedBuffer>,
    pub textures: Vec<AllocatedImage>,
    pub samplers: Vec<SamplerObject>,
}

pub struct InstancesPool {
    pub buffers: Vec<Id>,
}

#[derive(Resource)]
pub struct RendererResources {
    pub depth_image_id: Id,
    pub draw_image_id: Id,
    pub white_image_id: Id,
    pub nearest_sampler_id: Id,
    pub instance_objects_buffers_ids: Vec<Id>,
    pub mesh_objects_buffers_ids: Vec<Id>,
    pub resources_descriptor_set_handle: DescriptorSetHandle,
    pub gradient_compute_shader_object: ShaderObject,
    pub mesh_shader_object: ShaderObject,
    pub fragment_shader_object: ShaderObject,
    pub model_loader: ModelLoader,
    pub resources_pool: ResourcesPool,
    pub is_printed_scene_hierarchy: bool,
}

impl<'a> RendererResources {
    #[must_use]
    pub fn insert_mesh_buffer(&'a mut self, mesh_buffer: MeshBuffer) -> Id {
        let mesh_buffer_id = mesh_buffer.id;

        if !self
            .resources_pool
            .mesh_buffers
            .iter()
            .any(|mesh_buffer| mesh_buffer.id == mesh_buffer_id)
        {
            self.resources_pool.mesh_buffers.push(mesh_buffer);
        }

        return mesh_buffer_id;
    }

    #[must_use]
    pub fn insert_storage_buffer(&'a mut self, allocated_buffer: AllocatedBuffer) -> Id {
        let allocated_buffer_id = allocated_buffer.id;

        if !self
            .resources_pool
            .storage_buffers
            .iter()
            .any(|storage_buffer| storage_buffer.id == allocated_buffer_id)
        {
            self.resources_pool.storage_buffers.push(allocated_buffer);
        }

        return allocated_buffer_id;
    }

    #[must_use]
    pub fn insert_texture(&'a mut self, allocated_image: AllocatedImage) -> Id {
        let allocated_image_id = allocated_image.id;

        if !self
            .resources_pool
            .textures
            .iter()
            .any(|allocated_image| allocated_image.id == allocated_image_id)
        {
            self.resources_pool.textures.push(allocated_image);
        }

        return allocated_image_id;
    }

    #[must_use]
    pub fn insert_sampler(&'a mut self, sampler_object: SamplerObject) -> Id {
        let sampler_object_id = sampler_object.id;

        if !self
            .resources_pool
            .samplers
            .iter()
            .any(|sampler_object| sampler_object.id == sampler_object_id)
        {
            self.resources_pool.samplers.push(sampler_object);
        }

        return sampler_object_id;
    }

    #[must_use]
    pub fn get_mesh_buffers_iter(&'a self) -> Iter<'a, MeshBuffer> {
        self.resources_pool.mesh_buffers.iter()
    }

    pub fn get_storage_buffers_iter(&'a self) -> Iter<'a, AllocatedBuffer> {
        self.resources_pool.storage_buffers.iter()
    }

    #[must_use]
    pub fn get_textures_iter(&'a self) -> Iter<'a, AllocatedImage> {
        self.resources_pool.textures.iter()
    }

    #[must_use]
    pub fn get_samplers_iter(&'a self) -> Iter<'a, SamplerObject> {
        self.resources_pool.samplers.iter()
    }

    #[must_use]
    pub fn get_mesh_buffers_iter_mut(&'a mut self) -> IterMut<'a, MeshBuffer> {
        self.resources_pool.mesh_buffers.iter_mut()
    }

    #[must_use]
    pub fn get_storage_buffers_iter_mut(&'a mut self) -> IterMut<'a, AllocatedBuffer> {
        self.resources_pool.storage_buffers.iter_mut()
    }

    #[must_use]
    pub fn get_textures_iter_mut(&'a mut self) -> IterMut<'a, AllocatedImage> {
        self.resources_pool.textures.iter_mut()
    }

    #[must_use]
    pub fn get_samplers_iter_mut(&'a mut self) -> IterMut<'a, SamplerObject> {
        self.resources_pool.samplers.iter_mut()
    }

    #[must_use]
    pub fn get_mesh_buffer_ref(&'a self, id: Id) -> &'a MeshBuffer {
        self.resources_pool
            .mesh_buffers
            .iter()
            .find(|&mesh_buffer| mesh_buffer.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_storage_buffer_ref(&'a self, id: Id) -> &'a AllocatedBuffer {
        self.resources_pool
            .storage_buffers
            .iter()
            .find(|storage_buffer| storage_buffer.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_texture_ref(&'a self, id: Id) -> &'a AllocatedImage {
        self.resources_pool
            .textures
            .iter()
            .find(|&texture| texture.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_sampler(&'a self, id: Id) -> SamplerObject {
        *self
            .resources_pool
            .samplers
            .iter()
            .find(|&sampler_object| sampler_object.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_mesh_buffer_ref_mut(&'a mut self, id: Id) -> &'a mut MeshBuffer {
        self.resources_pool
            .mesh_buffers
            .iter_mut()
            .find(|mesh_buffer| mesh_buffer.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_storage_buffer_ref_mut(&'a mut self, id: Id) -> &'a mut AllocatedBuffer {
        self.resources_pool
            .storage_buffers
            .iter_mut()
            .find(|storage_buffer| storage_buffer.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_texture_ref_mut(&'a mut self, id: Id) -> &'a mut AllocatedImage {
        self.resources_pool
            .textures
            .iter_mut()
            .find(|texture| texture.id == id)
            .unwrap()
    }

    #[must_use]
    pub fn get_sampler_ref_mut(&'a mut self, id: Id) -> &'a mut SamplerObject {
        self.resources_pool
            .samplers
            .iter_mut()
            .find(|sampler_object| sampler_object.id == id)
            .unwrap()
    }
}
