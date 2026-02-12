use bevy_ecs::system::{Res, ResMut};
use vulkanite::vk::{rs::*, *};

use crate::engine::{
    ecs::{
        DevicePropertiesResource, GraphicsPushConstant, RendererResources, VulkanContextResource,
    },
    general::renderer::{DescriptorSetBuilder, DescriptorSetHandle},
};

pub fn prepare_descriptors_system(
    vulkan_ctx_resource: Res<VulkanContextResource>,
    mut renderer_resources: ResMut<RendererResources>,
    device_properties_resource: Res<DevicePropertiesResource>,
) {
    let device = vulkan_ctx_resource.device;
    let allocator = vulkan_ctx_resource.allocator;

    let push_constant_range = PushConstantRange {
        stage_flags: ShaderStageFlags::MeshEXT
            | ShaderStageFlags::Fragment
            | ShaderStageFlags::Compute
            | ShaderStageFlags::TaskEXT,
        offset: Default::default(),
        size: std::mem::size_of::<GraphicsPushConstant>() as _,
    };

    let push_constant_ranges = [push_constant_range];

    let resources_descriptor_set_handle = create_descriptors(
        device,
        allocator,
        &device_properties_resource,
        &push_constant_ranges,
    );

    renderer_resources.resources_descriptor_set_handle = Some(resources_descriptor_set_handle);
}

fn create_descriptors(
    device: Device,
    allocator: vma::Allocator,
    device_properties_resource: &DevicePropertiesResource,
    push_constant_ranges: &[PushConstantRange],
) -> DescriptorSetHandle {
    let mut descriptor_set_builder = DescriptorSetBuilder::new();

    // Samplers
    descriptor_set_builder.add_binding(
        DescriptorType::Sampler,
        16,
        DescriptorBindingFlags::PartiallyBound,
    );
    // Storage Images (aka Draw Image)
    descriptor_set_builder.add_binding(
        DescriptorType::StorageImage,
        128,
        DescriptorBindingFlags::PartiallyBound,
    );
    // Sampled Images (aka Textures), we can resize count of descriptors, we pre-alllocate N descriptors,
    // but we specify that count as unbound (aka variable)
    descriptor_set_builder.add_binding(
        DescriptorType::SampledImage,
        10_240,
        DescriptorBindingFlags::PartiallyBound | DescriptorBindingFlags::VariableDescriptorCount,
    );

    descriptor_set_builder.build(
        device,
        allocator,
        &device_properties_resource.descriptor_buffer_properties,
        push_constant_ranges,
        ShaderStageFlags::Compute
            | ShaderStageFlags::Fragment
            | ShaderStageFlags::MeshEXT
            | ShaderStageFlags::TaskEXT,
    )
}
