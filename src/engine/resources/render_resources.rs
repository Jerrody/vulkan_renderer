pub mod model_loader;

use std::{
    ffi::c_void,
    slice::{Iter, IterMut},
    sync::{Arc, Weak},
};

use ahash::{HashMap, HashMapExt};
use bevy_ecs::resource::Resource;
use bytemuck::{NoUninit, Pod, Zeroable};
use glam::Mat4;
use image::buffer;
use vma::{Alloc, Allocation, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage};
use vulkanite::{
    Handle,
    vk::{rs::*, *},
};

use crate::engine::{
    components::material::{MaterialState, MaterialType},
    descriptors::DescriptorSetHandle,
    id::Id,
    resources::{CommandGroup, render_resources::model_loader::ModelLoader},
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
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct InstanceObject {
    pub model_matrix: [f32; 16],
    pub device_address_mesh_object: DeviceAddress,
    pub device_address_material_data: DeviceAddress,
    pub meshlet_count: u32,
    pub material_type: u8,
    pub _pad: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct GraphicsPushConstant {
    pub view_projection: [f32; 16],
    pub device_address_instance_object: DeviceAddress,
    pub draw_image_index: u32,
    pub current_material_type: u8,
    pub _pad: [u8; 3],
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
    pub buffer_info: BufferInfo,
}

#[derive(Default, Clone)]
pub struct BufferReference {
    buffer_id: Id,
    weak_ptr: Weak<AllocatedBuffer>,
    buffer_info: BufferInfo,
}

#[derive(Default, Clone, Copy)]
pub struct BufferInfo {
    pub device_address: DeviceAddress,
    pub size: DeviceSize,
    pub buffer_visibility: BufferVisibility,
}

impl BufferInfo {
    pub fn new(
        device_address: DeviceAddress,
        size: DeviceSize,
        buffer_visibility: BufferVisibility,
    ) -> Self {
        Self {
            device_address,
            size,
            buffer_visibility,
        }
    }
}

impl BufferReference {
    pub fn new(
        buffer_id: Id,
        allocated_buffer: Weak<AllocatedBuffer>,
        device_address: DeviceAddress,
        size: DeviceSize,
        buffer_visibility: BufferVisibility,
    ) -> Self {
        Self {
            buffer_id,
            weak_ptr: allocated_buffer,
            buffer_info: BufferInfo::new(device_address, size, buffer_visibility),
        }
    }

    pub fn get_buffer<'a>(&'a self) -> Option<&'a AllocatedBuffer> {
        let mut allocated_buffer = None;

        if !self.weak_ptr.strong_count() != Default::default() {
            let allocated_buffer_ref = unsafe { &*(self.weak_ptr.as_ptr()) };

            if allocated_buffer_ref.id == self.buffer_id {
                allocated_buffer = Some(allocated_buffer_ref);
            }
        }

        allocated_buffer
    }

    #[inline(always)]
    pub fn get_buffer_id(&self) -> Id {
        let allocated_buffer = self.get_buffer();
        match allocated_buffer {
            Some(allocated_buffer) => allocated_buffer.id,
            None => Id::NULL,
        }
    }

    #[inline(always)]
    pub fn get_buffer_info(&self) -> BufferInfo {
        self.buffer_info
    }
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

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SceneData {
    pub camera_position: [f32; 16],
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BufferVisibility {
    #[default]
    Unspecified,
    HostVisible,
    DeviceOnly,
}

pub struct MemoryBucket {
    device: Device,
    allocator: Allocator,
    buffers: Vec<Arc<AllocatedBuffer>>,
    buffers_map: HashMap<Id, usize>,
    staging_buffer_reference: BufferReference,
    upload_command_group: CommandGroup,
    transfer_queue: Queue,
}

impl MemoryBucket {
    pub fn new(
        device: Device,
        allocator: Allocator,
        upload_command_group: CommandGroup,
        transfer_queue: Queue,
    ) -> Self {
        let mut memory_bucket = Self {
            device,
            allocator,
            buffers: Vec::with_capacity(1024),
            buffers_map: HashMap::with_capacity(1024),
            staging_buffer_reference: Default::default(),
            upload_command_group,
            transfer_queue,
        };

        // Pre-allocate 64 MB for transfers.
        let staging_buffer_reference = memory_bucket.create_buffer(
            1024 * 1024 * 64,
            BufferUsageFlags::TransferSrc,
            BufferVisibility::HostVisible,
        );
        memory_bucket.staging_buffer_reference = staging_buffer_reference;

        memory_bucket
    }

    pub fn create_buffer(
        &mut self,
        allocation_size: usize,
        usage: BufferUsageFlags,
        buffer_visibility: BufferVisibility,
    ) -> BufferReference {
        let buffer_kind_usage = if allocation_size < 1024 * 64 {
            BufferUsageFlags::UniformBuffer
        } else {
            BufferUsageFlags::StorageBuffer
        };

        let buffer_create_info = BufferCreateInfo {
            size: allocation_size as _,
            usage: usage | buffer_kind_usage | BufferUsageFlags::ShaderDeviceAddress,
            sharing_mode: vulkanite::vk::SharingMode::Exclusive,
            ..Default::default()
        };

        if buffer_visibility == BufferVisibility::Unspecified {
            panic!("Trying to create a buffer with unspecified visibility!");
        }

        let allocation_flags = match buffer_visibility {
            BufferVisibility::HostVisible => {
                AllocationCreateFlags::Mapped
                    | AllocationCreateFlags::HostAccessSequentialWrite
                    | AllocationCreateFlags::StrategyMinMemory
            }
            BufferVisibility::DeviceOnly => AllocationCreateFlags::StrategyMinMemory,
            BufferVisibility::Unspecified => unreachable!(),
        };

        let preferred_flags = match buffer_visibility {
            BufferVisibility::HostVisible => MemoryPropertyFlags::HostCoherent,
            BufferVisibility::DeviceOnly => MemoryPropertyFlags::empty(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        let allocation_create_info = AllocationCreateInfo {
            flags: allocation_flags,
            usage: MemoryUsage::Auto,
            required_flags: MemoryPropertyFlags::DeviceLocal,
            preferred_flags: preferred_flags,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            self.allocator
                .create_buffer(&buffer_create_info, &allocation_create_info)
                .unwrap()
        };
        let buffer = Buffer::from_inner(buffer);
        let device_address = unsafe { self.get_device_address(buffer) };

        let buffer_info = BufferInfo::new(device_address, allocation_size as _, buffer_visibility);
        let allocated_buffer = AllocatedBuffer {
            id: Id::new(device_address),
            buffer,
            allocation,
            buffer_info,
        };
        let allocated_buffer_size = allocated_buffer.buffer_info.size;
        let allocated_buffer_id = allocated_buffer.id;
        let weak_ptr_allocated_buffer = self.insert_buffer(allocated_buffer);

        let allocated_buffer_reference = BufferReference::new(
            allocated_buffer_id,
            weak_ptr_allocated_buffer,
            device_address,
            allocated_buffer_size,
            buffer_visibility,
        );

        allocated_buffer_reference
    }

    fn insert_buffer(&mut self, allocated_buffer: AllocatedBuffer) -> Weak<AllocatedBuffer> {
        let allocated_buffer_id = allocated_buffer.id;
        let allocated_buffer = Arc::new(allocated_buffer);
        let weak_ptr_allocated_buffer = Arc::downgrade(&allocated_buffer);
        self.buffers.push(allocated_buffer);
        let buffer_index = self.buffers.len() - 1;

        if let Some(already_presented_buffer_index) =
            self.buffers_map.insert(allocated_buffer_id, buffer_index)
        {
            panic!("Memory Bucket already has buffer by index: {already_presented_buffer_index}");
        }

        weak_ptr_allocated_buffer
    }

    unsafe fn get_device_address(&self, buffer: Buffer) -> DeviceAddress {
        let buffer_device_address = BufferDeviceAddressInfo::default().buffer(&buffer);

        self.device.get_buffer_address(&buffer_device_address)
    }

    pub unsafe fn transfer_data_to_buffer(
        &mut self,
        buffer_reference: BufferReference,
        src: &[u8],
        size: usize,
    ) {
        let allocated_buffer = buffer_reference.get_buffer().unwrap();

        let buffer_visibility = allocated_buffer.buffer_info.buffer_visibility;
        let target_buffer = match buffer_visibility {
            BufferVisibility::HostVisible => allocated_buffer,
            BufferVisibility::DeviceOnly => self.staging_buffer_reference.get_buffer().unwrap(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        unsafe {
            let p_mapped_memory = self.allocator.map_memory(target_buffer.allocation).unwrap();

            std::ptr::copy_nonoverlapping(src.as_ptr(), p_mapped_memory as _, size);

            self.allocator.unmap_memory(target_buffer.allocation);
        }

        if buffer_visibility == BufferVisibility::DeviceOnly {
            let regions_to_copy = [BufferCopy {
                size: size as _,
                ..Default::default()
            }];
            unsafe {
                self.copy_buffer_to_buffer(
                    target_buffer.buffer,
                    allocated_buffer.buffer,
                    &regions_to_copy,
                )
            }
        }
    }

    pub fn get_staging_buffer_reference<'a>(&self) -> &BufferReference {
        &self.staging_buffer_reference
    }

    pub unsafe fn transfer_data_to_buffer_raw(
        &mut self,
        buffer_reference: &BufferReference,
        src: *const c_void,
        size: usize,
    ) {
        let allocated_buffer = buffer_reference.get_buffer().unwrap();

        let buffer_visibility = allocated_buffer.buffer_info.buffer_visibility;
        let target_buffer = match buffer_visibility {
            BufferVisibility::HostVisible => allocated_buffer,
            BufferVisibility::DeviceOnly => self.staging_buffer_reference.get_buffer().unwrap(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        unsafe {
            let p_mapped_memory = self.allocator.map_memory(target_buffer.allocation).unwrap();

            std::ptr::copy_nonoverlapping(src, p_mapped_memory as _, size);

            self.allocator.unmap_memory(target_buffer.allocation);
        }

        if buffer_visibility == BufferVisibility::DeviceOnly {
            let regions_to_copy = [BufferCopy {
                size: size as _,
                ..Default::default()
            }];
            unsafe {
                self.copy_buffer_to_buffer(
                    target_buffer.buffer,
                    allocated_buffer.buffer,
                    &regions_to_copy,
                )
            }
        }
    }

    pub unsafe fn transfer_data_to_buffer_with_offset(
        &self,
        buffer_reference: &BufferReference,
        src: *const c_void,
        regions_to_copy: &[BufferCopy],
    ) {
        let allocated_buffer = buffer_reference.get_buffer().unwrap();

        let buffer_visibility = allocated_buffer.buffer_info.buffer_visibility;
        let target_buffer = match buffer_visibility {
            BufferVisibility::HostVisible => allocated_buffer,
            BufferVisibility::DeviceOnly => self.staging_buffer_reference.get_buffer().unwrap(),
            BufferVisibility::Unspecified => unreachable!(),
        };

        unsafe {
            let ptr_mapped_memory = self.allocator.map_memory(target_buffer.allocation).unwrap();

            for &buffer_copy in regions_to_copy {
                let src_with_offset = src.add(buffer_copy.src_offset as usize);

                let ptr_mapped_memory_with_offset =
                    ptr_mapped_memory.add(buffer_copy.dst_offset as usize);

                std::ptr::copy_nonoverlapping(
                    src_with_offset,
                    ptr_mapped_memory_with_offset as _,
                    buffer_copy.size as usize,
                );
            }

            self.allocator.unmap_memory(target_buffer.allocation);
        }

        if buffer_visibility == BufferVisibility::DeviceOnly {
            unsafe {
                self.copy_buffer_to_buffer(
                    target_buffer.buffer,
                    allocated_buffer.buffer,
                    &regions_to_copy,
                )
            }
        }
    }

    unsafe fn copy_buffer_to_buffer(
        &self,
        src_buffer: Buffer,
        dst_buffer: Buffer,
        regions_to_copy: &[BufferCopy],
    ) {
        let command_buffer = self.upload_command_group.command_buffer;

        let command_buffer_begin_info = CommandBufferBeginInfo {
            flags: CommandBufferUsageFlags::OneTimeSubmit,
            ..Default::default()
        };

        command_buffer.begin(&command_buffer_begin_info).unwrap();

        self.upload_command_group.command_buffer.copy_buffer(
            src_buffer,
            dst_buffer,
            regions_to_copy,
        );

        command_buffer.end().unwrap();

        let command_buffers = [command_buffer];
        let queue_submits = [SubmitInfo::default().command_buffers(command_buffers.as_slice())];

        self.transfer_queue
            .submit(&queue_submits, Some(self.upload_command_group.fence))
            .unwrap();

        let fences_to_wait = [self.upload_command_group.fence];
        self.device
            .wait_for_fences(fences_to_wait.as_slice(), true, u64::MAX)
            .unwrap();
        self.device.reset_fences(fences_to_wait.as_slice()).unwrap();

        self.device
            .reset_command_pool(
                self.upload_command_group.command_pool,
                CommandPoolResetFlags::ReleaseResources,
            )
            .unwrap();
    }

    pub unsafe fn free_allocations(&mut self) {
        self.buffers.drain(..).for_each(|allocated_buffer| unsafe {
            let mut allocation = allocated_buffer.allocation;
            self.allocator
                .destroy_buffer(*allocated_buffer.buffer, &mut allocation);
        });
    }
}

pub struct SwappableBuffer {
    current_buffer_index: usize,
    buffers: Vec<BufferReference>,
    objects_to_write: Vec<u8>,
}

impl<'a> SwappableBuffer {
    pub fn new(buffers: Vec<BufferReference>) -> Self {
        Self {
            current_buffer_index: Default::default(),
            buffers,
            objects_to_write: Default::default(),
        }
    }

    pub fn next_buffer(&mut self) {
        self.current_buffer_index += 1;
        if self.current_buffer_index >= self.buffers.len() {
            self.current_buffer_index = Default::default();
        }
    }

    pub fn get_current_buffer(&self) -> BufferReference {
        self.buffers[self.current_buffer_index].clone()
    }

    pub fn get_objects_to_write_as_slice(&'a self) -> &'a [u8] {
        self.objects_to_write.as_slice()
    }

    pub fn get_objects_to_write_as_slice_mut(&'a mut self) -> &'a mut [u8] {
        self.objects_to_write.as_mut_slice()
    }

    pub fn write_object_to_current_buffer<T: NoUninit>(&mut self, object_to_write: &T) -> usize {
        let object_to_write = bytemuck::bytes_of(object_to_write);
        self.objects_to_write.extend_from_slice(object_to_write);

        self.objects_to_write.len() - 1
    }

    pub fn clear_objects_to_write(&mut self) {
        self.objects_to_write.clear();
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

    pub fn get_materials_data_to_write_len(&self) -> usize {
        self.materials_to_write.len()
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
    pub memory_bucket: MemoryBucket,
    pub mesh_buffers: Vec<MeshBuffer>,
    pub textures: Vec<AllocatedImage>,
    pub samplers: Vec<SamplerObject>,
    pub instances_buffer: Option<SwappableBuffer>,
    pub scene_data_buffer: Option<SwappableBuffer>,
    materials_pool: MaterialsPool,
}

impl ResourcesPool {
    pub fn new(
        device: Device,
        allocator: Allocator,
        upload_command_group: CommandGroup,
        transfer_queue: Queue,
    ) -> Self {
        Self {
            memory_bucket: MemoryBucket::new(
                device,
                allocator,
                upload_command_group,
                transfer_queue,
            ),
            mesh_buffers: Default::default(),
            textures: Default::default(),
            samplers: Default::default(),
            instances_buffer: Default::default(),
            scene_data_buffer: Default::default(),
            materials_pool: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct RendererResources {
    pub default_texture_id: Id,
    pub fallback_texture_id: Id,
    pub nearest_sampler_id: Id,
    pub mesh_objects_buffer_reference: BufferReference,
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

    pub fn get_materials_data_to_write_len(&self) -> usize {
        self.resources_pool
            .materials_pool
            .get_materials_data_to_write_len()
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
            .write_object_to_current_buffer(&instance_object);

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

        return mesh_buffer_id;
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
