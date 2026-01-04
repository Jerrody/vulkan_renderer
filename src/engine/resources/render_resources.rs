use bevy_ecs::resource::Resource;
use vma::Allocation;
use vulkanite::vk::{
    DeviceAddress, Extent3D, Format, ShaderStageFlags,
    rs::{Buffer, DescriptorSetLayout, Image, ImageView, PipelineLayout, ShaderEXT},
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
    pub allocated_descriptor_buffer: AllocatedBuffer,
    pub descriptor_buffer_offset: u64,
    pub descriptor_buffer_size: u64,
    pub descriptor_set_layout: DescriptorSetLayout,
    pub address: DeviceAddress,
    pub pipeline_layout: PipelineLayout,
}

#[derive(Clone, Copy)]
pub struct ShaderObject {
    pub shader: ShaderEXT,
    pub stage: ShaderStageFlags,
}

impl ShaderObject {
    pub fn new(shader: ShaderEXT, stage: ShaderStageFlags) -> Self {
        Self { shader, stage }
    }
}

#[derive(Resource)]
pub struct RendererResources {
    pub draw_image: AllocatedImage,
    pub draw_image_descriptor_buffer: AllocatedDescriptorBuffer,
    pub gradient_compute_shader_object: ShaderObject,
}
