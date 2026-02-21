use bevy_ecs::system::{Query, Res, ResMut};
use glam::Mat4;
use vulkanite::vk::DeviceAddress;

use crate::engine::{
    components::{material::MaterialType, mesh::Mesh, transform::GlobalTransform},
    ecs::{FrameContext, materials_pool::MaterialsPool, mesh_buffers_pool::MeshBuffers},
    resources::RendererResources,
};

pub struct InstanceDataToWrite {
    pub model_matrix: Mat4,
    pub device_address_mesh_object: DeviceAddress,
    pub device_address_material_data: DeviceAddress,
    pub meshlet_count: usize,
    pub material_type: MaterialType,
}

pub fn collect_instance_objects_system(
    mut frame_context: ResMut<FrameContext>,
    materials_pool: Res<MaterialsPool>,
    mut renderer_resources: ResMut<RendererResources>,
    mesh_query: Query<(&GlobalTransform, &Mesh)>,
    mesh_buffers: MeshBuffers,
) {
    for (global_transform, mesh) in mesh_query {
        let material_info = materials_pool.get_material_info(mesh.material_reference);

        let mesh_buffer = unsafe {
            mesh_buffers
                .get(mesh.mesh_buffer_reference)
                .unwrap_unchecked()
        };

        let instance_data_to_write = InstanceDataToWrite {
            model_matrix: global_transform.0,
            device_address_mesh_object: mesh_buffer.mesh_object_device_address,
            meshlet_count: mesh_buffer.meshlets_count,
            device_address_material_data: material_info.device_adddress_material_data,
            material_type: material_info.material_type,
        };

        frame_context
            .instance_objects_to_write
            .push(instance_data_to_write);
    }

    /*     frame_context
           .instance_objects_to_write
           .sort_unstable_by_key(|instance_data_to_write| {
               instance_data_to_write.device_address_mesh_object
           });
    */
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
