use bevy_ecs::system::{Query, Res, ResMut};
use glam::Mat4;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use vulkanite::vk::DeviceAddress;

use crate::engine::{
    components::{material::MaterialType, mesh::Mesh, transform::GlobalTransform},
    ecs::{FrameContext, materials_pool::MaterialsPool, mesh_buffers_pool::MeshBuffers},
    resources::RendererResources,
};

pub struct InstanceDataToWrite {
    pub index: usize,
    pub model_matrix: Mat4,
    pub device_address_mesh_object: DeviceAddress,
    pub device_address_material_data: DeviceAddress,
    pub meshlet_count: usize,
    pub material_type: MaterialType,
}

pub fn collect_instance_objects_system(
    mut frame_context: ResMut<FrameContext>,
    mut materials_pool: Res<MaterialsPool>,
    mut renderer_resources: ResMut<RendererResources>,
    mut mesh_query: Query<(&GlobalTransform, &mut Mesh)>,
    mesh_buffers: MeshBuffers,
) {
    let mut current_instance_data_index = usize::default();
    for (global_transform, mut mesh) in &mut mesh_query {
        let material_info = materials_pool.get_material_info(mesh.material_reference);

        let mesh_buffer = unsafe {
            mesh_buffers
                .get(mesh.mesh_buffer_reference)
                .unwrap_unchecked()
        };

        frame_context
            .instance_objects_to_write
            .push(InstanceDataToWrite {
                index: current_instance_data_index,
                model_matrix: global_transform.0,
                device_address_mesh_object: mesh_buffer.mesh_object_device_address,
                meshlet_count: mesh_buffer.meshlets_count,
                device_address_material_data: material_info.device_adddress_material_data,
                material_type: material_info.material_type,
            });

        mesh.instance_object_index = Some(current_instance_data_index);

        current_instance_data_index += 1;
    }

    frame_context
        .instance_objects_to_write
        .sort_unstable_by_key(|instance_data_to_write| {
            instance_data_to_write.device_address_mesh_object
        });

    mesh_query.par_iter_mut().for_each(|(_, mut mesh)| {
        let mesh_instance_index = unsafe { mesh.instance_object_index.unwrap_unchecked() };
        let instance_object_index = frame_context
            .instance_objects_to_write
            .iter()
            .position(|instance_data_to_write| instance_data_to_write.index == mesh_instance_index);

        mesh.instance_object_index = instance_object_index;
    });

    frame_context
        .instance_objects_to_write
        .drain(..)
        .for_each(|instance_data_to_write| {
            renderer_resources.write_instance_object(
                instance_data_to_write.model_matrix,
                instance_data_to_write.device_address_mesh_object,
                instance_data_to_write.meshlet_count,
                instance_data_to_write.device_address_material_data,
                instance_data_to_write.material_type as u8,
            );
        });
}
