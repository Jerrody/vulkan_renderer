use bevy_ecs::system::{Query, ResMut};

use crate::engine::{
    components::{mesh::Mesh, transform::GlobalTransform},
    resources::{MeshBuffer, RendererResources},
};

pub fn collect_instance_objects(
    mut renderer_resources: ResMut<RendererResources>,
    mut mesh_query: Query<(&GlobalTransform, &mut Mesh)>,
) {
    for (global_transform, mut mesh) in &mut mesh_query {
        let mesh_buffer: &MeshBuffer =
            unsafe { &*(renderer_resources.get_mesh_buffer_ref(mesh.mesh_buffer_id) as *const _) };
        let instance_object_index = renderer_resources
            .write_instance_object(global_transform.0, mesh_buffer.mesh_object_device_address);

        mesh.instance_object_index = Some(instance_object_index);
    }
}
