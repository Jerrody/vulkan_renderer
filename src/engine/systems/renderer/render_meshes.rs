use bevy_ecs::system::Res;
use vulkanite::vk::*;

use crate::engine::resources::{FrameContext, MeshPushConstant, RendererResources};

pub fn render_meshes(renderer_resources: Res<RendererResources>, frame_context: Res<FrameContext>) {
    let command_buffer = frame_context.command_buffer.unwrap();

    let mesh = &renderer_resources.mesh_buffers[2];
    let mesh_push_constant = [MeshPushConstant {
        world_matrix: frame_context.world_matrix,
        vertex_buffer_device_adress: mesh.vertex_buffer.device_address,
        vertex_indices_device_address: mesh.vertex_indices_buffer.device_address,
        meshlets_device_address: mesh.meshlets_buffer.device_address,
        local_indices_device_address: mesh.local_indices_buffer.device_address,
    }];

    command_buffer.push_constants(
        renderer_resources.mesh_pipeline_layout,
        ShaderStageFlags::MeshEXT,
        Default::default(),
        size_of::<MeshPushConstant>() as _,
        mesh_push_constant.as_ptr() as _,
    );

    command_buffer.draw_mesh_tasks_ext(mesh.meshlets_count as _, 1, 1);
}
