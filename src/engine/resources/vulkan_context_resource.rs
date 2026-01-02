use std::sync::Arc;

use bevy_ecs::resource::Resource;
use vulkanalia::vk::Queue;

pub struct QueueData {
    pub index: usize,
    pub queue: Queue,
}

impl QueueData {
    pub fn new(index: usize, queue: Queue) -> Self {
        QueueData { index, queue }
    }
}

#[derive(Resource)]
pub struct VulkanContextResource {
    pub instance: Arc<vulkanalia_bootstrap::Instance>,
    pub device: Arc<vulkanalia_bootstrap::Device>,
    pub graphics_queue_data: QueueData,
    pub swapchain: vulkanalia_bootstrap::Swapchain,
}
