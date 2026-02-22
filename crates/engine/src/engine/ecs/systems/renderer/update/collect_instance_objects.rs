use bevy_ecs::system::{Query, Res, ResMut};

use crate::engine::{
    components::{mesh::Mesh, transform::GlobalTransform},
    ecs::{materials_pool::MaterialsPool, mesh_buffers_pool::MeshBuffers},
    resources::RendererResources,
};

// TODO: Take into account if GlobalTransform really changed or not and update if necessary.
pub fn collect_instance_objects_system(
    materials_pool: Res<MaterialsPool>,
    mut renderer_resources: ResMut<RendererResources>,
    mesh_query: Query<(&GlobalTransform, &Mesh)>,
    mesh_buffers: MeshBuffers,
) {
    for (global_transform, mesh) in mesh_query.iter() {
        let material_info = materials_pool.get_material_info(mesh.material_reference);

        let mesh_buffer = unsafe {
            mesh_buffers
                .get(mesh.mesh_buffer_reference)
                .unwrap_unchecked()
        };

        renderer_resources.write_instance_object(
            global_transform.0,
            mesh_buffer.mesh_object_device_address,
            mesh_buffer.meshlets_count,
            material_info.device_adddress_material_data,
            material_info.material_type as u8,
        );
    }
}
