use crate::engine::{Engine, descriptors::DescriptorSetBuilder};

impl Engine {
    pub fn create_descriptor_set_builder_resource<'a>() -> DescriptorSetBuilder<'a> {
        DescriptorSetBuilder::new()
    }
}
