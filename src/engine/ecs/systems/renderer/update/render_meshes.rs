use bevy_ecs::{
    entity::Entity,
    name::Name,
    system::{Query, Res, ResMut},
};
use vulkanite::vk::{Bool32, ColorBlendEquationEXT, ShaderStageFlags};

use crate::engine::{
    components::{material::MaterialType, mesh::Mesh, transform::Parent},
    resources::{FrameContext, GraphicsPushConstant, RendererResources},
};

pub fn render_meshes_system(
    graphics_entities: Query<&Mesh>,
    entities: Query<(Entity, &Name)>,
    entities_with_parent: Query<&Parent>,
    mut renderer_resources: ResMut<RendererResources>,
    frame_context: Res<FrameContext>,
) {
    let command_buffer = frame_context.command_buffer.unwrap();

    if !renderer_resources.is_printed_scene_hierarchy {
        println!("=====================================");

        for (entity, name) in entities.iter() {
            if let Ok(parent) = entities_with_parent.get(entity) {
                println!("Entity: {} | Name: {} | Parent: {}", entity, name, parent.0);
            } else {
                println!("Entity: {} | Name: {}", entity, name);
            }
        }

        println!("=====================================");
    }

    let color_blend_equation = [ColorBlendEquationEXT {
        src_color_blend_factor: vulkanite::vk::BlendFactor::One,
        dst_color_blend_factor: vulkanite::vk::BlendFactor::One,
        color_blend_op: vulkanite::vk::BlendOp::Add,
        src_alpha_blend_factor: vulkanite::vk::BlendFactor::One,
        dst_alpha_blend_factor: vulkanite::vk::BlendFactor::Zero,
        alpha_blend_op: vulkanite::vk::BlendOp::Add,
    }];
    command_buffer.set_color_blend_equation_ext(Default::default(), &color_blend_equation);

    let meshes_len = graphics_entities.iter().len();
    for material_type in 0..2 {
        let is_draw_transparent_materials =
            material_type as u32 == MaterialType::Transparent as u32;
        let blend_enables = [Bool32::from(is_draw_transparent_materials)];

        command_buffer.set_depth_write_enable(!is_draw_transparent_materials);

        command_buffer.set_color_blend_enable_ext(Default::default(), blend_enables.as_slice());

        let push_constants = GraphicsPushConstant {
            current_material_type: material_type as _,
            ..Default::default()
        };
        command_buffer.push_constants(
            renderer_resources
                .resources_descriptor_set_handle
                .as_ref()
                .unwrap()
                .pipeline_layout,
            ShaderStageFlags::Fragment
                | ShaderStageFlags::TaskEXT
                | ShaderStageFlags::MeshEXT
                | ShaderStageFlags::Compute,
            std::mem::offset_of!(GraphicsPushConstant, current_material_type) as _,
            std::mem::size_of::<u32>() as _,
            &push_constants.current_material_type as *const _ as _,
        );

        command_buffer.draw_mesh_tasks_ext(meshes_len as _, 1, 1);
    }

    renderer_resources.is_printed_scene_hierarchy = true;
}
