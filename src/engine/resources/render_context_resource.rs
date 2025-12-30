use bevy_ecs::resource::Resource;
use vulkanalia::vk::*;

pub struct FrameData {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
    pub fence: Fence,
    pub test_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
    pub present_semaphore: Semaphore,
}

#[derive(Resource)]
pub struct RenderContextResource {
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
    pub frame_overlap: usize,
    pub frames_data: Vec<FrameData>,
}
