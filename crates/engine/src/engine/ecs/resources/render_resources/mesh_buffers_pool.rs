use bevy_ecs::{
    resource::Resource,
    system::{Res, ResMut, SystemParam},
};
use vulkanite::vk::DeviceAddress;

use crate::engine::ecs::buffers_pool::BufferReference;

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
    index: u32,
}

#[derive(SystemParam)]
pub struct MeshBuffers<'w> {
    mesh_buffers_pool: Res<'w, MeshBuffersPool>,
}

impl<'w> MeshBuffers<'w> {
    #[inline(always)]
    pub fn get(&self, mesh_buffer_reference: MeshBufferReference) -> &MeshBuffer {
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
    pub fn get(&self, mesh_buffer_reference: MeshBufferReference) -> &MeshBuffer {
        self.mesh_buffers_pool
            .get_mesh_buffer(mesh_buffer_reference)
    }

    #[inline(always)]
    pub fn get_mut(&mut self, mesh_buffer_reference: MeshBufferReference) -> &mut MeshBuffer {
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
    slots: Vec<MeshBuffer>,
    free_indices: Vec<u32>,
}

impl MeshBuffersPool {
    pub fn new(pre_allocated_count: u32) -> Self {
        let slots = (0..pre_allocated_count)
            .map(|_| Default::default())
            .collect();

        Self {
            slots,
            free_indices: (0..pre_allocated_count).rev().collect(),
        }
    }

    fn insert_mesh_buffer(&mut self, mesh_buffer: MeshBuffer) -> MeshBufferReference {
        let free_index = self.free_indices.pop().unwrap();

        self.slots[free_index as usize] = mesh_buffer;

        MeshBufferReference { index: free_index }
    }

    fn get_mesh_buffer(&self, mesh_buffer_reference: MeshBufferReference) -> &MeshBuffer {
        &self.slots[mesh_buffer_reference.index as usize]
    }

    fn get_mut_mesh_buffer(
        &mut self,
        mesh_buffer_reference: MeshBufferReference,
    ) -> &mut MeshBuffer {
        &mut self.slots[mesh_buffer_reference.index as usize]
    }
}
