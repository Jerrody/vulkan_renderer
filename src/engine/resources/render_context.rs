use bevy_ecs::resource::Resource;
use vulkanalia::vk::{CommandBuffer, CommandPool, Extent2D, Fence, Image, ImageView, Semaphore};

pub struct FrameData {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
    pub render_fence: Fence,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
}

#[derive(Resource)]
pub struct RendererContext {
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
    pub frame_overlap: usize,
    pub frames_data: Vec<FrameData>,
    pub frame_number: usize,
    pub draw_extent: Extent2D,
}

impl RendererContext {
    pub fn get_current_frame_data(&self) -> &FrameData {
        &self.frames_data[self.frame_number % self.frame_overlap]
    }
}
