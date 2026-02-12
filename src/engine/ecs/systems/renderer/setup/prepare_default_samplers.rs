use crate::engine::{
    ecs::{RendererResources, VulkanContextResource, samplers_pool::SamplersMut},
    general::renderer::{DescriptorKind, DescriptorSampler},
};
use bevy_ecs::system::{Res, ResMut};
use vulkanite::vk::*;

pub fn prepare_default_samplers_system(
    vulkan_context_resource: Res<VulkanContextResource>,
    mut renderer_resources: ResMut<RendererResources>,
    mut samplers_mut: SamplersMut,
) {
    let device = vulkan_context_resource.device;
    let allocator = vulkan_context_resource.allocator;

    let default_sampler_reference =
        samplers_mut.create_sampler(Filter::Linear, SamplerAddressMode::Repeat, true);

    let sampler = samplers_mut.get(default_sampler_reference).unwrap();
    let sampler_descriptor = DescriptorKind::Sampler(DescriptorSampler {
        sampler: sampler,
        index: renderer_resources.default_sampler_reference.index,
    });

    renderer_resources
        .resources_descriptor_set_handle
        .as_mut()
        .unwrap()
        .update_binding(device, allocator, sampler_descriptor);
}
