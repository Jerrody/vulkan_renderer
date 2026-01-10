use bevy_ecs::resource::Resource;
use glam::Mat4;
use vulkanite::vk::rs::CommandBuffer;

#[derive(Resource, Default)]
pub struct FrameContext {
    pub swapchain_image_index: u32,
    pub command_buffer: Option<CommandBuffer>,
    pub world_matrix: Mat4,
}
