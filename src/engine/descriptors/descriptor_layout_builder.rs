use vulkanite::vk::{rs::*, *};

#[derive(Default)]
pub struct DescriptorSetLayoutBuilder<'a> {
    bindings: Vec<DescriptorSetLayoutBinding<'a>>,
}

impl<'a> DescriptorSetLayoutBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_binding(&mut self, binding: u32, descriptor_type: DescriptorType) {
        let binding = DescriptorSetLayoutBinding::default()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(1);

        self.bindings.push(binding);
    }

    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn build(
        &mut self,
        device: &Device,
        shader_stages: ShaderStageFlags,
        descriptor_set_layout_flags: DescriptorSetLayoutCreateFlags,
    ) -> DescriptorSetLayout {
        self.bindings.iter_mut().for_each(|binding| {
            binding.stage_flags |= shader_stages;
        });

        let mut descriptor_set_layout_info =
            DescriptorSetLayoutCreateInfo::default().flags(descriptor_set_layout_flags);
        descriptor_set_layout_info = descriptor_set_layout_info.bindings(&self.bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&descriptor_set_layout_info)
                .unwrap()
        };

        self.clear();

        descriptor_set_layout
    }
}
