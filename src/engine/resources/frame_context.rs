use bevy_ecs::resource::Resource;
use glam::Mat4;
use vulkanite::vk::rs::CommandBuffer;

use crate::engine::id::Id;

#[derive(Resource)]
pub struct FrameContext {
    pub swapchain_image_index: u32,
    pub command_buffer: Option<CommandBuffer>,
    pub draw_image_id: Id,
    pub depth_image_id: Id,
    pub world_matrix: Mat4,
}

impl Default for FrameContext {
    fn default() -> Self {
        Self {
            swapchain_image_index: Default::default(),
            command_buffer: None,
            draw_image_id: Id::NULL,
            depth_image_id: Id::NULL,
            world_matrix: Default::default(),
        }
    }
}
