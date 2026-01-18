use bevy_ecs::system::{Query, Res, ResMut};
use vulkanite::vk::*;

use crate::engine::{
    components::mesh::Mesh,
    resources::{FrameContext, MeshPushConstant, RendererResources},
};

pub fn render_meshes(
    meshes: Query<&Mesh>,
    mut renderer_resources: ResMut<RendererResources>,
    frame_context: Res<FrameContext>,
) {
    let command_buffer = frame_context.command_buffer.unwrap();

    let mesh_pipeline_layout = renderer_resources
        .resources_descriptor_set_handle
        .pipeline_layout;
    meshes.iter().for_each(|mesh| {
        let mesh_buffer = renderer_resources.get_mesh_buffer_ref(mesh.buffer_id);

        let texture_image_index = renderer_resources
            .get_texture_ref(renderer_resources.draw_image_id)
            .index;
        let nearest_sampler_index = renderer_resources
            .get_sampler(renderer_resources.nearest_sampler_id)
            .index;

        let mesh_push_constant = &MeshPushConstant {
            world_matrix: frame_context.world_matrix,
            vertex_buffer_device_adress: mesh_buffer.vertex_buffer.device_address,
            vertex_indices_device_address: mesh_buffer.vertex_indices_buffer.device_address,
            meshlets_device_address: mesh_buffer.meshlets_buffer.device_address,
            local_indices_device_address: mesh_buffer.local_indices_buffer.device_address,
            texture_image_index: texture_image_index as _,
            sampler_index: nearest_sampler_index as _,
            ..Default::default()
        };

        let p_mesh_push_constant = mesh_push_constant as *const MeshPushConstant;
        command_buffer.push_constants(
            mesh_pipeline_layout,
            ShaderStageFlags::MeshEXT | ShaderStageFlags::Fragment | ShaderStageFlags::Compute,
            Default::default(),
            size_of::<MeshPushConstant>() as u32
                - std::mem::size_of_val(&mesh_push_constant.draw_image_index) as u32,
            p_mesh_push_constant as _,
        );

        command_buffer.draw_mesh_tasks_ext(mesh_buffer.meshlets_count as _, 1, 1);
    });
}
