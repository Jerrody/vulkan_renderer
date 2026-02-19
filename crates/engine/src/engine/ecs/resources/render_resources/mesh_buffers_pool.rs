use bevy_ecs::{
    resource::Resource,
    system::{Res, ResMut, SystemParam},
};
use slotmap::{Key, SlotMap};
use vulkanite::vk::DeviceAddress;

use crate::engine::ecs::{MeshBufferKey, buffers_pool::BufferReference};

#[derive(Default)]
pub struct MeshBuffer {
    pub mesh_object_device_address: DeviceAddress,
    pub vertex_buffer_reference: BufferReference,
    pub vertex_indices_buffer_reference: BufferReference,
    pub meshlets_buffer_reference: BufferReference,
    pub local_indices_buffer_reference: BufferReference,
    pub meshlets_count: usize,
}

#[derive(Clone, Copy)]
pub struct MeshBufferReference {
    key: MeshBufferKey,
}

impl MeshBufferReference {
    pub fn get_index(&self) -> u32 {
        self.key.data().get_key() - 1
    }
}

#[derive(SystemParam)]
pub struct MeshBuffers<'w> {
    mesh_buffers_pool: Res<'w, MeshBuffersPool>,
}

impl<'w> MeshBuffers<'w> {
    #[inline(always)]
    pub fn get(&self, mesh_buffer_reference: MeshBufferReference) -> Option<&MeshBuffer> {
        self.mesh_buffers_pool
            .get_mesh_buffer(mesh_buffer_reference)
    }
}

#[derive(SystemParam)]
pub struct MeshBuffersMut<'w> {
    mesh_buffers_pool: ResMut<'w, MeshBuffersPool>,
}

impl<'w> MeshBuffersMut<'w> {
    #[inline(always)]
    pub fn get(&self, mesh_buffer_reference: MeshBufferReference) -> Option<&MeshBuffer> {
        self.mesh_buffers_pool
            .get_mesh_buffer(mesh_buffer_reference)
    }

    #[inline(always)]
    pub fn get_mut(
        &mut self,
        mesh_buffer_reference: MeshBufferReference,
    ) -> Option<&mut MeshBuffer> {
        self.mesh_buffers_pool
            .get_mut_mesh_buffer(mesh_buffer_reference)
    }

    #[inline(always)]
    pub fn insert_mesh_buffer(&mut self, mesh_buffer: MeshBuffer) -> MeshBufferReference {
        self.mesh_buffers_pool.insert_mesh_buffer(mesh_buffer)
    }
}

#[derive(Resource)]
pub struct MeshBuffersPool {
    slots: SlotMap<MeshBufferKey, MeshBuffer>,
}

impl MeshBuffersPool {
    pub fn new(pre_allocated_count: usize) -> Self {
        Self {
            slots: SlotMap::with_capacity_and_key(pre_allocated_count),
        }
    }

    fn insert_mesh_buffer(&mut self, mesh_buffer: MeshBuffer) -> MeshBufferReference {
        let mesh_buffer_key = self.slots.insert(mesh_buffer);

        MeshBufferReference {
            key: mesh_buffer_key,
        }
    }

    fn get_mesh_buffer(&self, mesh_buffer_reference: MeshBufferReference) -> Option<&MeshBuffer> {
        self.slots.get(mesh_buffer_reference.key)
    }

    fn get_mut_mesh_buffer(
        &mut self,
        mesh_buffer_reference: MeshBufferReference,
    ) -> Option<&mut MeshBuffer> {
        self.slots.get_mut(mesh_buffer_reference.key)
    }
}
