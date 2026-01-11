use vulkanite::vk::{DescriptorType, DeviceAddress, rs::*};

use crate::engine::resources::AllocatedBuffer;

pub struct DescriptorSetLayoutHandle {
    pub descriptor_set_layout: DescriptorSetLayout,
    pub descriptor_set_layout_size: u64,
}

pub struct DescriptorSetBinding {
    pub descriptor_binding_type: DescriptorType,
    pub index: usize,
    pub offset: usize,
}

pub struct DescriptorSetHandle {
    pub buffer: AllocatedBuffer,
    pub descriptor_set_layout_handle: DescriptorSetLayoutHandle,
    pub pipeline_layout: PipelineLayout,
}
