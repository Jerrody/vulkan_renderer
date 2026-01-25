use std::collections::HashMap;

use bevy_ecs::system::{Query, ResMut};
use glam::Mat4;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator};
use vulkanite::vk::DeviceAddress;

use crate::engine::{
    components::{mesh::Mesh, transform::GlobalTransform},
    resources::{MeshBuffer, RendererResources},
};

struct InstanceDataToWrite {
    pub index: usize,
    pub model_matrix: Mat4,
    pub device_address_mesh_object: DeviceAddress,
    pub meshlet_count: usize,
}

pub fn collect_instance_objects(
    mut renderer_resources: ResMut<RendererResources>,
    mut mesh_query: Query<(&GlobalTransform, &mut Mesh)>,
) {
    let mut instances_data_to_write: Vec<InstanceDataToWrite> =
        Vec::with_capacity(mesh_query.iter().len());

    let mesh_buffers_iter = renderer_resources.get_mesh_buffers_iter();
    let mut mesh_buffers_map = HashMap::with_capacity(mesh_buffers_iter.len());
    mesh_buffers_iter.for_each(|mesh_buffer| {
        mesh_buffers_map.insert(mesh_buffer.id, mesh_buffer);
    });

    let mut current_instance_data_index = usize::default();
    for (global_transform, mut mesh) in &mut mesh_query {
        let mesh_buffer = unsafe {
            mesh_buffers_map
                .get(&mesh.mesh_buffer_id)
                .unwrap_unchecked()
        };
        instances_data_to_write.push(InstanceDataToWrite {
            index: current_instance_data_index,
            model_matrix: global_transform.0,
            device_address_mesh_object: mesh_buffer.mesh_object_device_address,
            meshlet_count: mesh_buffer.meshlets_count,
        });

        mesh.instance_object_index = Some(current_instance_data_index);

        current_instance_data_index += 1;
    }

    instances_data_to_write.sort_unstable_by_key(|instance_data_to_write| {
        instance_data_to_write.device_address_mesh_object
    });

    for (_, mut mesh) in &mut mesh_query {
        let mesh_instance_index = unsafe { mesh.instance_object_index.unwrap_unchecked() };
        let instance_object_index = instances_data_to_write
            .iter()
            .position(|instance_data_to_write| instance_data_to_write.index == mesh_instance_index);

        mesh.instance_object_index = instance_object_index;
    }

    instances_data_to_write
        .drain(..)
        .for_each(|instance_data_to_write| {
            renderer_resources.write_instance_object(
                instance_data_to_write.model_matrix,
                instance_data_to_write.device_address_mesh_object,
                instance_data_to_write.meshlet_count,
            );
        });
}
