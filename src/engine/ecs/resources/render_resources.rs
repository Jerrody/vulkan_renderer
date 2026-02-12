pub mod buffers_pool;
pub mod model_loader;
pub mod samplers_pool;
pub mod textures_pool;

use std::slice::{Iter, IterMut};

use bevy_ecs::resource::Resource;
use bytemuck::{NoUninit, Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use padding_struct::padding_struct;
use vulkanite::vk::{rs::*, *};

use crate::engine::{
    components::material::{MaterialState, MaterialType},
    general::renderer::DescriptorSetHandle,
    id::Id,
    resources::{
        buffers_pool::BufferReference, render_resources::model_loader::ModelLoader,
        samplers_pool::SamplerReference, textures_pool::TextureReference,
    },
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Meshlet {
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MeshObject {
    pub device_address_vertex_buffer: DeviceAddress,
    pub device_address_vertex_indices_buffer: DeviceAddress,
    pub device_address_meshlets_buffer: DeviceAddress,
    pub device_address_local_indices_buffer: DeviceAddress,
}

#[repr(C)]
#[padding_struct]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct InstanceObject {
    pub model_matrix: [f32; 16],
    pub device_address_mesh_object: DeviceAddress,
    pub device_address_material_data: DeviceAddress,
    pub meshlet_count: u32,
    pub material_type: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct GraphicsPushConstant {
    pub device_address_scene_data: DeviceAddress,
    pub device_address_instance_object: DeviceAddress,
    pub draw_image_index: u32,
    pub current_material_type: u32,
}

pub struct MeshBuffer {
    pub id: Id,
    pub mesh_object_device_address: DeviceAddress,
    pub vertex_buffer_reference: BufferReference,
    pub vertex_indices_buffer_reference: BufferReference,
    pub meshlets_buffer_reference: BufferReference,
    pub local_indices_buffer_reference: BufferReference,
    pub meshlets_count: usize,
}

#[derive(Default, Clone, Copy)]
pub struct ShaderObject {
    pub shader: Option<ShaderEXT>,
    pub stage: ShaderStageFlags,
}

impl ShaderObject {
    pub fn new(shader: Option<ShaderEXT>, stage: ShaderStageFlags) -> Self {
        Self { shader, stage }
    }
}

#[repr(C)]
#[padding_struct]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct LightProperties {
    pub ambient_color: Vec4,
    pub ambient_strength: f32,
    pub specular_strength: f32,
}

#[repr(C)]
#[padding_struct]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct DirectionalLight {
    pub light_color: Vec3,
    pub light_position: Vec3,
}

#[repr(C)]
#[padding_struct]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct SceneData {
    pub camera_view_matrix: [f32; 16],
    pub camera_position: Vec3,
    pub light_properties: LightProperties,
    pub directional_light: DirectionalLight,
}

pub struct SwappableBuffer {
    current_buffer_index: usize,
    buffers: Vec<BufferReference>,
    data_to_write: Vec<u8>,
}

impl<'a> SwappableBuffer {
    pub fn new(buffers: Vec<BufferReference>) -> Self {
        Self {
            current_buffer_index: Default::default(),
            buffers,
            data_to_write: Default::default(),
        }
    }

    pub fn next_buffer(&mut self) {
        self.current_buffer_index += 1;
        if self.current_buffer_index >= self.buffers.len() {
            self.current_buffer_index = Default::default();
        }
        self.data_to_write.clear();
    }

    pub fn get_current_buffer(&self) -> BufferReference {
        self.buffers[self.current_buffer_index].clone()
    }

    pub fn get_objects_to_write_as_slice(&'a self) -> &'a [u8] {
        self.data_to_write.as_slice()
    }

    pub fn write_data_to_current_buffer<T: NoUninit>(&mut self, object_to_write: &T) -> usize {
        let object_to_write = bytemuck::bytes_of(object_to_write);
        self.data_to_write.extend_from_slice(object_to_write);

        self.data_to_write.len() - 1
    }
}

#[derive(Clone, Copy)]
struct MaterialLabel {
    pub id: Id,
    pub material_state: MaterialState,
    pub size: usize,
    pub device_address_material_data: DeviceAddress,
}

pub struct MaterialInfo {
    pub material_type: MaterialType,
    pub device_adddress_materail_data: DeviceAddress,
}

#[derive(Default)]
struct MaterialsPool {
    pub materials_data_buffer_reference: BufferReference,
    pub materials_to_write: Vec<u8>,
    pub material_labels: Vec<MaterialLabel>,
}

impl MaterialsPool {
    pub fn write_material(&mut self, data: &[u8], material_state: MaterialState) -> Id {
        let material_label = MaterialLabel {
            id: Id::new(self.material_labels.len()),
            size: data.len(),
            material_state,
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

    pub fn get_materials_data_buffer_reference(&self) -> BufferReference {
        self.materials_data_buffer_reference.clone()
    }

    pub fn set_materials_data_buffer_reference(
        &mut self,
        materials_data_buffer_reference: BufferReference,
    ) {
        self.materials_data_buffer_reference = materials_data_buffer_reference;
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

    pub fn get_material_info_device_address_by_id(&self, material_label_id: Id) -> MaterialInfo {
        let material_label = self
            .material_labels
            .iter()
            .find(|&material_label| material_label.id == material_label_id)
            .unwrap();

        MaterialInfo {
            material_type: material_label.material_state.material_type,
            device_adddress_materail_data: material_label.device_address_material_data,
        }
    }
}

pub struct ResourcesPool {
    pub mesh_buffers: Vec<MeshBuffer>,
    pub instances_buffer: Option<SwappableBuffer>,
    pub scene_data_buffer: Option<SwappableBuffer>,
    materials_pool: MaterialsPool,
}

impl ResourcesPool {
    pub fn new() -> Self {
        Self {
            mesh_buffers: Default::default(),
            instances_buffer: Default::default(),
            scene_data_buffer: Default::default(),
            materials_pool: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct RendererResources {
    pub default_texture_reference: TextureReference,
    pub fallback_texture_reference: TextureReference,
    pub default_sampler_reference: SamplerReference,
    pub mesh_objects_buffer_reference: BufferReference,
    pub resources_descriptor_set_handle: Option<DescriptorSetHandle>,
    pub gradient_compute_shader_object: ShaderObject,
    pub task_shader_object: ShaderObject,
    pub mesh_shader_object: ShaderObject,
    pub fragment_shader_object: ShaderObject,
    pub model_loader: ModelLoader,
    pub resources_pool: ResourcesPool,
    pub is_printed_scene_hierarchy: bool,
}

impl<'a> RendererResources {
    pub fn write_material(&mut self, data: &[u8], material_state: MaterialState) -> Id {
        self.resources_pool
            .materials_pool
            .write_material(data, material_state)
    }

    pub fn reset_materails_to_write(&mut self) {
        self.resources_pool
            .materials_pool
            .reset_materails_to_write();
    }

    pub fn get_materials_data_buffer_reference(&self) -> BufferReference {
        self.resources_pool
            .materials_pool
            .get_materials_data_buffer_reference()
    }

    pub fn set_materials_data_buffer_reference(
        &mut self,
        materials_data_buffer_reference: BufferReference,
    ) {
        self.resources_pool
            .materials_pool
            .set_materials_data_buffer_reference(materials_data_buffer_reference);
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

    pub fn get_material_data_device_address_by_id(&self, material_label_id: Id) -> MaterialInfo {
        self.resources_pool
            .materials_pool
            .get_material_info_device_address_by_id(material_label_id)
    }

    pub fn write_instance_object(
        &mut self,
        model_matrix: Mat4,
        device_address_mesh_object: DeviceAddress,
        meshlet_count: usize,
        device_address_material_data: DeviceAddress,
        material_type: u8,
    ) -> usize {
        let instance_object = InstanceObject {
            model_matrix: model_matrix.to_cols_array(),
            device_address_mesh_object,
            meshlet_count: meshlet_count as _,
            device_address_material_data,
            material_type,
            ..Default::default()
        };

        let last_instance_object_index = self
            .resources_pool
            .instances_buffer
            .as_mut()
            .unwrap()
            .write_data_to_current_buffer(&instance_object);

        last_instance_object_index
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

        mesh_buffer_id
    }

    #[must_use]
    pub fn get_mesh_buffers_iter(&'a self) -> Iter<'a, MeshBuffer> {
        self.resources_pool.mesh_buffers.iter()
    }

    #[must_use]
    pub fn get_mesh_buffers_iter_mut(&'a mut self) -> IterMut<'a, MeshBuffer> {
        self.resources_pool.mesh_buffers.iter_mut()
    }

    #[must_use]
    pub fn get_mesh_buffer_ref(&'a self, id: Id) -> &'a MeshBuffer {
        self.resources_pool
            .mesh_buffers
            .iter()
            .find(|&mesh_buffer| mesh_buffer.id == id)
            .unwrap()
    }
}
