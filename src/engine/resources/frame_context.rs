use bevy_ecs::resource::Resource;

#[derive(Resource, Default)]
pub struct FrameContext {
    pub swapchain_image_index: u32,
}
