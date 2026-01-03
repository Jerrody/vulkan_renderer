use bevy_ecs::resource::Resource;
use vma::Allocator;
use vulkanite::vk::{
    SurfaceFormatKHR,
    rs::{DebugUtilsMessengerEXT, Device, Instance, PhysicalDevice, Queue, SwapchainKHR},
};

#[derive(Resource)]
pub struct VulkanContextResource {
    pub instance: Instance,
    pub debug_utils_messenger: Option<DebugUtilsMessengerEXT>,
    pub device: Device,
    pub physical_device: PhysicalDevice,
    pub allocator: Allocator,
    pub graphics_queue: Queue,
    pub queue_family_index: usize,
    pub swapchain: SwapchainKHR,
    pub surface_format: SurfaceFormatKHR,
}
