use bevy_ecs::resource::Resource;
use vma::Allocation;
use vulkanalia::vk::{Extent3D, Format, Image, ImageView};

pub struct AllocatedImage {
    pub image: Image,
    pub image_view: ImageView,
    pub allocation: Allocation,
    pub image_extent: Extent3D,
    pub format: Format,
}

#[derive(Resource)]
pub struct RendererResources {
    pub draw_image: AllocatedImage,
}
