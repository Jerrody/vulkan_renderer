use bevy_ecs::resource::Resource;
use vma::Allocation;
use vulkanite::vk::{
    Extent3D, Format,
    rs::{Buffer, DescriptorSetLayout, Image, ImageView, ShaderEXT},
};

pub struct AllocatedImage {
    pub image: Image,
    pub image_view: ImageView,
    pub allocation: Allocation,
    pub image_extent: Extent3D,
    pub format: Format,
}

pub struct AllocatedBuffer {
    pub buffer: Buffer,
    pub allocation: Allocation,
}

pub struct AllocatedDescriptorBuffer {
    pub allocated_buffer: AllocatedBuffer,
    pub descriptor_buffer_offset: u64,
    pub descriptor_buffer_size: u64,
    pub descriptor_set_layout: DescriptorSetLayout,
}

#[derive(Resource)]
pub struct RendererResources {
    pub draw_image: AllocatedImage,
    pub draw_image_descriptor_buffer: AllocatedDescriptorBuffer,
    pub gradient_shader: ShaderEXT,
}
