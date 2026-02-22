use bevy_ecs::resource::Resource;
use glam::Mat4;
use vulkanite::vk::rs::CommandBuffer;

use crate::engine::resources::textures_pool::TextureReference;

#[derive(Resource)]
#[derive(Default)]
pub struct FrameContext {
    pub swapchain_image_index: u32,
    pub command_buffer: Option<CommandBuffer>,
    pub draw_texture_reference: TextureReference,
    pub depth_texture_reference: TextureReference,
    pub world_matrix: Mat4,
}

