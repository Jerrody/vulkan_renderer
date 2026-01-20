pub mod allocation;
pub mod model_loader;

use ahash::HashMap;
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

#[derive(Clone, Copy)]
#[repr(C)]
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

pub struct MeshBuffer {
    pub vertex_buffer: Id,
    pub vertex_indices_buffer: Id,
    pub meshlets_buffer: Id,
    pub local_indices_buffer: Id,
    pub meshlets_count: usize,
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
    pub mesh_buffers: HashMap<Id, MeshBuffer>,
    pub storage_buffers: HashMap<Id, AllocatedBuffer>,
    pub textures: HashMap<Id, AllocatedImage>,
    pub samplers: HashMap<Id, SamplerObject>,
}

#[derive(Resource)]
pub struct RendererResources {
    pub depth_image_id: Id,
    pub draw_image_id: Id,
    pub white_image_id: Id,
    pub nearest_sampler_id: Id,
    pub insatance_objects_buffers_ids: Vec<Id>,
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
        let mesh_buffer_id = Id::new(mesh_buffer.vertex_buffer.value());

        let id = match self
            .resources_pool
            .mesh_buffers
            .insert(mesh_buffer_id, mesh_buffer)
        {
            Some(_) => mesh_buffer_id,
            None => mesh_buffer_id,
        };

        return id;
    }

    #[must_use]
    pub fn insert_storage_buffer(&'a mut self, allocated_buffer: AllocatedBuffer) -> Id {
        let allocated_buffer_id = allocated_buffer.id;

        let id = match self
            .resources_pool
            .storage_buffers
            .insert(allocated_buffer.id, allocated_buffer)
        {
            Some(already_presented_storage_buffer) => already_presented_storage_buffer.id,
            None => allocated_buffer_id,
        };

        return id;
    }

    #[must_use]
    pub fn get_mesh_buffers_iter(&'a self) -> std::collections::hash_map::Iter<'a, Id, MeshBuffer> {
        self.resources_pool.mesh_buffers.iter()
    }

    pub fn get_storage_buffers_iter(
        &'a self,
    ) -> std::collections::hash_map::Iter<'a, Id, AllocatedBuffer> {
        self.resources_pool.storage_buffers.iter()
    }

    #[must_use]
    pub fn get_textures_iter(&'a self) -> std::collections::hash_map::Iter<'a, Id, AllocatedImage> {
        self.resources_pool.textures.iter()
    }

    #[must_use]
    pub fn get_samplers_iter(&'a self) -> std::collections::hash_map::Iter<'a, Id, SamplerObject> {
        self.resources_pool.samplers.iter()
    }

    #[must_use]
    pub fn get_mesh_buffers_iter_mut(
        &'a mut self,
    ) -> std::collections::hash_map::IterMut<'a, Id, MeshBuffer> {
        self.resources_pool.mesh_buffers.iter_mut()
    }

    #[must_use]
    pub fn get_storage_buffers_iter_mut(
        &'a mut self,
    ) -> std::collections::hash_map::IterMut<'a, Id, AllocatedBuffer> {
        self.resources_pool.storage_buffers.iter_mut()
    }

    #[must_use]
    pub fn get_textures_iter_mut(
        &'a mut self,
    ) -> std::collections::hash_map::IterMut<'a, Id, AllocatedImage> {
        self.resources_pool.textures.iter_mut()
    }

    #[must_use]
    pub fn get_samplers_iter_mut(
        &'a mut self,
    ) -> std::collections::hash_map::IterMut<'a, Id, SamplerObject> {
        self.resources_pool.samplers.iter_mut()
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
    pub fn get_mesh_buffer_ref(&'a self, id: Id) -> &'a MeshBuffer {
        unsafe { &*(self.resources_pool.mesh_buffers.get(&id).unwrap() as *const _) }
    }

    #[must_use]
    pub fn get_storage_buffer_ref(&'a self, id: Id) -> &'a AllocatedBuffer {
        unsafe { &*(self.resources_pool.storage_buffers.get(&id).unwrap() as *const _) }
    }

    #[must_use]
    pub fn get_texture_ref(&'a self, id: Id) -> &'a AllocatedImage {
        unsafe { &*(self.resources_pool.textures.get(&id).unwrap() as *const _) }
    }

    #[must_use]
    pub fn get_sampler(&'a self, id: Id) -> SamplerObject {
        *self.resources_pool.samplers.get(&id).unwrap()
    }

    #[must_use]
    pub fn get_mesh_buffer_ref_mut(&'a mut self, id: Id) -> &'a mut MeshBuffer {
        unsafe { &mut *(self.resources_pool.mesh_buffers.get_mut(&id).unwrap() as *mut _) }
    }

    #[must_use]
    pub fn get_storage_buffer_ref_mut(&'a mut self, id: Id) -> &'a mut AllocatedBuffer {
        unsafe { &mut *(self.resources_pool.storage_buffers.get_mut(&id).unwrap() as *mut _) }
    }

    #[must_use]
    pub fn get_texture_ref_mut(&'a mut self, id: Id) -> &'a mut AllocatedImage {
        unsafe { &mut *(self.resources_pool.textures.get_mut(&id).unwrap() as *mut _) }
    }

    #[must_use]
    pub fn get_sampler_ref_mut(&'a mut self, id: Id) -> &'a mut SamplerObject {
        unsafe { &mut *(self.resources_pool.samplers.get_mut(&id).unwrap() as *mut _) }
    }
}
