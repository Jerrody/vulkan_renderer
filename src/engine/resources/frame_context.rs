use bevy_ecs::resource::Resource;
use glam::Mat4;
use vulkanite::vk::rs::CommandBuffer;

use crate::engine::{id::Id, resources::textures_pool::TextureReference};

#[derive(Resource)]
pub struct FrameContext {
    pub swapchain_image_index: u32,
    pub command_buffer: Option<CommandBuffer>,
    pub draw_texture_reference: TextureReference,
    pub depth_texture_reference: TextureReference,
    pub world_matrix: Mat4,
}

impl Default for FrameContext {
    fn default() -> Self {
        Self {
            swapchain_image_index: Default::default(),
            command_buffer: None,
            draw_texture_reference: Default::default(),
            depth_texture_reference: Default::default(),
            world_matrix: Default::default(),
        }
    }
}
