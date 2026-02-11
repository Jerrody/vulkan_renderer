use vulkanite::vk::rs::*;

#[derive(Clone, Copy)]
pub struct DescriptorStorageImage {
    pub image_view: ImageView,
    pub index: usize,
}

#[derive(Clone, Copy)]
pub struct DescriptorSampledImage {
    pub image_view: ImageView,
    pub index: usize,
}

#[derive(Clone, Copy)]
pub struct DescriptorSampler {
    pub sampler: Sampler,
    pub index: usize,
}
