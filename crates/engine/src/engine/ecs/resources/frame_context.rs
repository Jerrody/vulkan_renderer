use bevy_ecs::resource::Resource;
use glam::Mat4;
use vulkanite::vk::rs::CommandBuffer;

use crate::engine::{
    ecs::collect_instance_objects::InstanceDataToWrite, resources::textures_pool::TextureReference,
};

#[derive(Resource)]
pub struct FrameContext {
    pub swapchain_image_index: u32,
    pub command_buffer: Option<CommandBuffer>,
    pub draw_texture_reference: TextureReference,
    pub depth_texture_reference: TextureReference,
    pub world_matrix: Mat4,
    pub instance_objects_to_write: Vec<InstanceDataToWrite>,
}

impl Default for FrameContext {
    fn default() -> Self {
        Self {
            swapchain_image_index: Default::default(),
            command_buffer: Default::default(),
            draw_texture_reference: Default::default(),
            depth_texture_reference: Default::default(),
            world_matrix: Default::default(),
            instance_objects_to_write: Vec::with_capacity(2_000),
        }
    }
}
