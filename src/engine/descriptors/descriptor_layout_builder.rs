use std::sync::Arc;

use vulkanalia::vk::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags,
    DescriptorSetLayoutCreateInfo, DescriptorType, DeviceV1_0, ShaderStageFlags,
};

#[derive(Default)]
pub struct DescriptorLayoutBuilder {
    bindings: Vec<DescriptorSetLayoutBinding>,
}

impl DescriptorLayoutBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_binding(&mut self, binding: u32, descriptor_type: DescriptorType) {
        let binding = DescriptorSetLayoutBinding {
            binding,
            descriptor_type,
            descriptor_count: 1,
            ..Default::default()
        };

        self.bindings.push(binding);
    }

    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn build(
        &mut self,
        device: &Arc<vulkanalia_bootstrap::Device>,
        shader_stages: ShaderStageFlags,
        descriptor_set_layout_flags: DescriptorSetLayoutCreateFlags,
    ) -> DescriptorSetLayout {
        self.bindings.iter_mut().for_each(|binding| {
            binding.stage_flags |= shader_stages;
        });

        let descriptor_set_layout_info = DescriptorSetLayoutCreateInfo {
            flags: descriptor_set_layout_flags,
            binding_count: self.bindings.len() as _,
            bindings: self.bindings.as_ptr(),
            ..Default::default()
        };

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&descriptor_set_layout_info, None)
                .unwrap()
        };

        self.clear();

        descriptor_set_layout
    }
}
