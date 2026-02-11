use vulkanite::vk::{rs::*, *};

#[derive(Clone, Copy)]
pub struct DescriptorStorageBuffer {
    pub address: DeviceAddress,
    pub size: u64,
}

impl DescriptorStorageBuffer {
    pub fn get_descriptor_type(&self) -> DescriptorType {
        DescriptorType::StorageBuffer
    }
}

#[derive(Clone, Copy)]
pub struct DescriptorUniformBuffer {
    pub address: DeviceAddress,
    pub size: u64,
}

impl DescriptorUniformBuffer {
    pub fn get_descriptor_type(&self) -> DescriptorType {
        DescriptorType::UniformBuffer
    }
}

#[derive(Clone, Copy)]
pub struct DescriptorStorageImage {
    pub image_view: ImageView,
    pub index: usize,
}

impl DescriptorStorageImage {
    pub fn get_descriptor_type(&self) -> DescriptorType {
        DescriptorType::StorageImage
    }
}

#[derive(Clone, Copy)]
pub struct DescriptorCombinedImageSampler {
    pub image_view: ImageView,
    pub sampler: Sampler,
}

#[derive(Clone, Copy)]
pub struct DescriptorSampledImage {
    pub image_view: ImageView,
    pub index: usize,
}

impl DescriptorSampledImage {
    pub fn get_descriptor_type(&self) -> DescriptorType {
        DescriptorType::SampledImage
    }
}

#[derive(Clone, Copy)]
pub struct DescriptorSampler {
    pub sampler: Sampler,
}

impl DescriptorSampler {
    pub fn get_descriptor_type(&self) -> DescriptorType {
        DescriptorType::Sampler
    }
}
