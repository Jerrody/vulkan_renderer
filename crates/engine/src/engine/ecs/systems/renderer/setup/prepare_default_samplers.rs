use crate::engine::{
    ecs::{RendererResources, buffers_pool::BuffersMut, samplers_pool::SamplersMut},
    general::renderer::{DescriptorKind, DescriptorSampler, DescriptorSetHandle},
};
use bevy_ecs::system::ResMut;
use vulkanite::vk::*;

pub fn prepare_default_samplers_system(
    mut renderer_resources: ResMut<RendererResources>,
    mut descriptor_set_handle: ResMut<DescriptorSetHandle>,
    buffers_mut: BuffersMut,
    mut samplers_mut: SamplersMut,
) {
    let default_sampler_reference =
        samplers_mut.create_sampler(Filter::Linear, SamplerAddressMode::Repeat, true);
    renderer_resources.default_sampler_reference = default_sampler_reference;

    let sampler = samplers_mut.get(default_sampler_reference).unwrap();
    let sampler_descriptor = DescriptorKind::Sampler(DescriptorSampler {
        sampler: *sampler,
        index: renderer_resources.default_sampler_reference.get_index(),
    });

    descriptor_set_handle.update_binding(&buffers_mut, sampler_descriptor);
}
