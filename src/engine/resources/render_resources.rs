pub mod allocation;
pub mod model_loader;

use std::slice::{Iter, IterMut};

use bevy_ecs::resource::Resource;
use glam::Mat4;
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
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
pub struct MeshObject {
    pub device_address_vertex_buffer: DeviceAddress,
    pub device_address_vertex_indices_buffer: DeviceAddress,
    pub device_address_meshlets_buffer: DeviceAddress,
    pub device_address_local_indices_buffer: DeviceAddress,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct InstanceObject {
    pub model_matrix: [f32; 16],
    pub device_address_mesh_object: DeviceAddress,
    pub device_address_material_data: DeviceAddress,
    pub meshlet_count: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct GraphicsPushConstant {
    pub view_projection: [f32; 16],
    pub device_address_instance_object: DeviceAddress,
    pub draw_image_index: u32,
}

pub struct MeshBuffer {
    pub id: Id,
    pub mesh_object_device_address: DeviceAddress,
    pub vertex_buffer_id: Id,
    pub vertex_indices_buffer_id: Id,
    pub meshlets_buffer_id: Id,
    pub local_indices_buffer_id: Id,
    pub meshlets_count: usize,
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

pub struct MeshObjectPool {
    pub mesh_objects_buffer_id: Id,
    pub mesh_buffers_to_write: Vec<Id>,
}

impl Default for MeshObjectPool {
    fn default() -> Self {
        MeshObjectPool {
            mesh_objects_buffer_id: Id::NULL,
            mesh_buffers_to_write: Default::default(),
        }
    }
}

impl<'a> MeshObjectPool {
    pub fn enqueue_mesh_buffer_to_write(&mut self, mesh_buffer_id: Id) {
        self.mesh_buffers_to_write.push(mesh_buffer_id);
    }

    pub fn get_mesh_buffers_to_write_iter(&'a self) -> Iter<'a, Id> {
        self.mesh_buffers_to_write.iter()
    }

    pub fn get_mesh_buffers_to_write_iter_mut(&'a mut self) -> IterMut<'a, Id> {
        self.mesh_buffers_to_write.iter_mut()
    }
}

pub struct InstancesPool {
    pub current_instance_set_index: usize,
    pub instance_sets_buffers_ids: Vec<Id>,
    instance_objects_to_write: Vec<InstanceObject>,
}

impl<'a> InstancesPool {
    pub fn get_current_instance_set_buffer_id(&self) -> Id {
        self.instance_sets_buffers_ids[self.current_instance_set_index]
    }

    pub fn get_instances_objects_to_write_as_slice(&'a self) -> &'a [InstanceObject] {
        self.instance_objects_to_write.as_slice()
    }

    pub fn get_instances_objects_to_write_as_slice_mut(&'a mut self) -> &'a mut [InstanceObject] {
        self.instance_objects_to_write.as_mut_slice()
    }

    pub fn write_instance_object_to_current_instance_set(
        &mut self,
        instance_object: InstanceObject,
    ) -> usize {
        self.instance_objects_to_write.push(instance_object);

        self.instance_objects_to_write.len() - 1
    }
}

impl Default for InstancesPool {
    fn default() -> Self {
        Self {
            current_instance_set_index: usize::MIN,
            instance_sets_buffers_ids: Default::default(),
            instance_objects_to_write: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
struct MaterialLabel {
    pub id: Id,
    pub size: usize,
    pub device_address_material_data: DeviceAddress,
}

struct MaterialsPool {
    pub materials_data_buffer_id: Id,
    pub materials_to_write: Vec<u8>,
    pub material_labels: Vec<MaterialLabel>,
}

impl MaterialsPool {
    pub fn write_material(&mut self, data: &[u8]) -> Id {
        let material_label = MaterialLabel {
            id: Id::new(self.material_labels.len()),
            size: data.len(),
            device_address_material_data: Default::default(),
        };
        let id = material_label.id;

        self.material_labels.push(material_label);
        self.materials_to_write.extend_from_slice(data);

        id
    }

    pub fn reset_materails_to_write(&mut self) {
        self.materials_to_write.clear();
    }

    pub fn get_materials_data_buffer_id(&self) -> Id {
        self.materials_data_buffer_id
    }

    pub fn set_materials_data_buffer_id(&mut self, materials_data_buffer_id: Id) {
        self.materials_data_buffer_id = materials_data_buffer_id;
    }

    pub fn set_materials_labels_device_addresses(
        &mut self,
        mut device_address_materials_data: DeviceAddress,
    ) {
        for material_label_index in 0..self.material_labels.len() {
            let material_label = &mut self.material_labels[material_label_index];
            material_label.device_address_material_data = device_address_materials_data;

            device_address_materials_data += material_label.size as u64;
        }
    }

    pub fn get_materials_data_to_write<'a>(&'a self) -> &'a [u8] {
        &self.materials_to_write.as_slice()
    }

    pub fn get_materials_data_to_write_len(&self) -> usize {
        self.materials_to_write.len()
    }

    pub fn get_material_data_device_address_by_id(&self, material_label_id: Id) -> DeviceAddress {
        let material_label = self
            .material_labels
            .iter()
            .find(|&material_label| material_label.id == material_label_id)
            .unwrap();

        material_label.device_address_material_data
    }
}

impl Default for MaterialsPool {
    fn default() -> Self {
        Self {
            materials_data_buffer_id: Id::NULL,
            materials_to_write: Vec::new(),
            material_labels: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct ResourcesPool {
    pub mesh_buffers: Vec<MeshBuffer>,
    pub storage_buffers: Vec<AllocatedBuffer>,
    pub textures: Vec<AllocatedImage>,
    pub samplers: Vec<SamplerObject>,
    pub instances_pool: InstancesPool,
    pub mesh_objects_pool: MeshObjectPool,
    materials_pool: MaterialsPool,
}

#[derive(Resource)]
pub struct RendererResources {
    pub default_texture_id: Id,
    pub fallback_texture_id: Id,
    pub nearest_sampler_id: Id,
    pub mesh_objects_buffers_ids: Vec<Id>,
    pub resources_descriptor_set_handle: DescriptorSetHandle,
    pub gradient_compute_shader_object: ShaderObject,
    pub task_shader_object: ShaderObject,
    pub mesh_shader_object: ShaderObject,
    pub fragment_shader_object: ShaderObject,
    pub model_loader: ModelLoader,
    pub resources_pool: ResourcesPool,
    pub is_printed_scene_hierarchy: bool,
}

impl<'a> RendererResources {
    pub fn write_material(&mut self, data: &[u8]) -> Id {
        self.resources_pool.materials_pool.write_material(data)
    }

    pub fn reset_materails_to_write(&mut self) {
        self.resources_pool
            .materials_pool
            .reset_materails_to_write();
    }

    pub fn get_materials_data_buffer_id(&self) -> Id {
        self.resources_pool
            .materials_pool
            .get_materials_data_buffer_id()
    }

    pub fn set_materials_data_buffer_id(&mut self, materials_data_buffer_id: Id) {
        self.resources_pool
            .materials_pool
            .set_materials_data_buffer_id(materials_data_buffer_id);
    }

    pub fn set_materials_labels_device_addresses(
        &mut self,
        device_address_materials_data: DeviceAddress,
    ) {
        self.resources_pool
            .materials_pool
            .set_materials_labels_device_addresses(device_address_materials_data);
    }

    pub fn get_materials_data_to_write(&'a self) -> &'a [u8] {
        self.resources_pool
            .materials_pool
            .get_materials_data_to_write()
    }

    pub fn get_materials_data_to_write_len(&self) -> usize {
        self.resources_pool
            .materials_pool
            .get_materials_data_to_write_len()
    }

    pub fn get_material_data_device_address_by_id(&self, material_label_id: Id) -> DeviceAddress {
        self.resources_pool
            .materials_pool
            .get_material_data_device_address_by_id(material_label_id)
    }

    pub fn insert_mesh_objects_buffer_id(&mut self, mesh_objects_buffer_id: Id) {
        self.resources_pool.mesh_objects_pool.mesh_objects_buffer_id = mesh_objects_buffer_id;
    }

    pub fn enqueue_mesh_buffer_to_write(&mut self, mesh_buffer_id: Id) {
        self.resources_pool
            .mesh_objects_pool
            .enqueue_mesh_buffer_to_write(mesh_buffer_id);
    }

    pub fn get_mesh_buffer_to_write_iter(&'a self) -> Iter<'a, Id> {
        self.resources_pool
            .mesh_objects_pool
            .get_mesh_buffers_to_write_iter()
    }

    pub fn insert_instance_set_buffer_id(&mut self, instance_set_buffer_id: Id) {
        self.resources_pool
            .instances_pool
            .instance_sets_buffers_ids
            .push(instance_set_buffer_id);
    }

    pub fn set_and_reset_current_instance_set_by_index(&mut self, index: usize) {
        let instances_pool = &mut self.resources_pool.instances_pool;
        instances_pool.current_instance_set_index = index;
        instances_pool.instance_objects_to_write.clear();
    }

    pub fn get_current_instance_set_buffer_id(&self) -> Id {
        self.resources_pool
            .instances_pool
            .get_current_instance_set_buffer_id()
    }

    pub fn write_instance_object(
        &mut self,
        model_matrix: Mat4,
        device_address_mesh_object: DeviceAddress,
        meshlet_count: usize,
        device_address_material_data: DeviceAddress,
    ) -> usize {
        let instance_object = InstanceObject {
            model_matrix: model_matrix.to_cols_array(),
            device_address_mesh_object,
            meshlet_count: meshlet_count as _,
            device_address_material_data,
        };

        let last_instance_object_index = self
            .resources_pool
            .instances_pool
            .write_instance_object_to_current_instance_set(instance_object);

        last_instance_object_index
    }

    pub fn get_instances_objects_to_write_as_slice(&self) -> &[InstanceObject] {
        self.resources_pool
            .instances_pool
            .get_instances_objects_to_write_as_slice()
    }

    pub fn get_instances_objects_to_write_as_slice_mut(&mut self) -> &mut [InstanceObject] {
        self.resources_pool
            .instances_pool
            .get_instances_objects_to_write_as_slice_mut()
    }

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
