use bevy_ecs::resource::Resource;
use vulkanite::vk::{
    Extent2D,
    rs::{CommandBuffer, CommandPool, Fence, Image, ImageView, Semaphore},
};

use crate::engine::id::Id;

pub struct FrameData {
    pub command_group: CommandGroup,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
    pub draw_image_id: Id,
    pub depth_image_id: Id,
}

pub struct CommandGroup {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
    pub fence: Fence,
}

pub struct UploadContext {
    pub command_group: CommandGroup,
}

#[derive(Resource)]
pub struct RendererContext {
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
    pub frame_overlap: usize,
    pub frames_data: Vec<FrameData>,
    pub upload_context: UploadContext,
    pub frame_number: usize,
    pub draw_extent: Extent2D,
}

impl RendererContext {
    pub fn get_current_frame_data(&self) -> &FrameData {
        &self.frames_data[self.frame_number % self.frame_overlap]
    }
}
