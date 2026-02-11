use std::collections::HashMap;

use vma::*;
use vulkanite::vk::{rs::*, *};

use crate::engine::{
    descriptors::*,
    resources::buffers_pool::{AllocatedBuffer, BufferInfo, BufferVisibility},
    utils::get_device_address,
};

pub enum DescriptorKind {
    StorageImage(DescriptorStorageImage),
    SampledImage(DescriptorSampledImage),
    Sampler(DescriptorSampler),
}

impl DescriptorKind {
    pub fn get_descriptor_type(&self) -> DescriptorType {
        let descriptor_type = match self {
            DescriptorKind::StorageImage(descriptor_storage_image) => {
                descriptor_storage_image.get_descriptor_type()
            }
            DescriptorKind::SampledImage(descriptor_sampled_image) => {
                descriptor_sampled_image.get_descriptor_type()
            }
            DescriptorKind::Sampler(descriptor_sampler) => descriptor_sampler.get_descriptor_type(),
        };

        descriptor_type
    }
}

struct DescriptorSetLayoutBindingInfo<'a> {
    pub binding: DescriptorSetLayoutBinding<'a>,
    pub flags: DescriptorBindingFlags,
}

#[derive(Default)]
pub struct DescriptorSetBuilder<'a> {
    bindings_infos: Vec<DescriptorSetLayoutBindingInfo<'a>>,
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_binding(
        &mut self,
        descriptor_type: DescriptorType,
        descriptor_count: u32,
        binding_flags: DescriptorBindingFlags,
    ) {
        let next_binding_index = self.bindings_infos.len();
        let binding = DescriptorSetLayoutBinding::default()
            .binding(next_binding_index as _)
            .descriptor_type(descriptor_type)
            .descriptor_count(descriptor_count);

        let binding_info = DescriptorSetLayoutBindingInfo {
            binding,
            flags: binding_flags,
        };

        self.bindings_infos.push(binding_info);
    }

    pub fn clear(&mut self) {
        self.bindings_infos.clear();
    }

    pub fn build(
        &mut self,
        device: Device,
        allocator: &Allocator,
        descriptor_buffer_properties: &PhysicalDeviceDescriptorBufferPropertiesEXT,
        push_constant_ranges: &[PushConstantRange],
        shader_stages: ShaderStageFlags,
    ) -> DescriptorSetHandle {
        let descriptor_set_layout_handle = self.create_descriptor_set_layout(
            device,
            shader_stages,
            DescriptorSetLayoutCreateFlags::DescriptorBufferEXT,
        );

        let mut bindings_infos: HashMap<u32, BindingInfo, ahash::RandomState> =
            HashMap::with_hasher(ahash::RandomState::new());

        self.bindings_infos.iter().enumerate().for_each(
            |(binding_index, descriptor_set_layout_binding_info)| {
                let binding_offset = device.get_descriptor_set_layout_binding_offset_ext(
                    descriptor_set_layout_handle.descriptor_set_layout,
                    binding_index as _,
                );

                let binding_info = BindingInfo {
                    binding_offset,
                    next_empty_slot_index: Default::default(),
                };
                bindings_infos.insert(
                    descriptor_set_layout_binding_info.binding.descriptor_type as _,
                    binding_info,
                );
            },
        );

        let descriptor_buffer_size = Self::get_descriptor_buffer_aligned_size(
            descriptor_set_layout_handle.descriptor_set_layout_size,
            descriptor_buffer_properties.descriptor_buffer_offset_alignment,
        );

        let descriptor_buffer =
            self.create_descriptor_buffer(device, allocator, descriptor_buffer_size);

        let descriptor_set_layouts = [descriptor_set_layout_handle.descriptor_set_layout];
        let pipeline_layout_info = PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_set_layouts.as_slice())
            .push_constant_ranges(push_constant_ranges);
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info)
            .unwrap();

        self.clear();

        let sampled_image_descriptor_size =
            descriptor_buffer_properties.sampled_image_descriptor_size;
        let storage_image_descriptor_size =
            descriptor_buffer_properties.storage_image_descriptor_size;
        let sampler_descriptor_size = descriptor_buffer_properties.sampler_descriptor_size;

        DescriptorSetHandle {
            buffer: descriptor_buffer,
            descriptor_set_layout_handle,
            pipeline_layout,
            bindings_infos,
            descriptors_sizes: DescriptorsSizes {
                sampled_image_descriptor_size,
                sampler_descriptor_size,
                storage_image_descriptor_size,
            },
        }
    }

    fn create_descriptor_buffer(
        &mut self,
        device: Device,
        allocator: &Allocator,
        descriptor_buffer_size: u64,
    ) -> AllocatedBuffer {
        let buffer_info = BufferCreateInfo::default()
            .size(descriptor_buffer_size)
            .usage(
                BufferUsageFlags::ShaderDeviceAddress
                    | BufferUsageFlags::ResourceDescriptorBufferEXT,
            );

        let allocation_info = AllocationCreateInfo {
            flags: AllocationCreateFlags::Mapped | AllocationCreateFlags::HostAccessRandom,
            usage: MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        let (descriptor_buffer_raw, allocation) = unsafe {
            allocator
                .create_buffer(&buffer_info, &allocation_info)
                .unwrap()
        };
        let descriptor_buffer = Buffer::from_inner(descriptor_buffer_raw);

        let buffer_device_address = get_device_address(device, &descriptor_buffer);
        AllocatedBuffer {
            buffer: descriptor_buffer,
            allocation,
            buffer_info: BufferInfo::new(
                buffer_device_address,
                descriptor_buffer_size,
                BufferVisibility::HostVisible,
            ),
        }
    }

    fn create_descriptor_set_layout(
        &mut self,
        device: Device,
        shader_stages: ShaderStageFlags,
        descriptor_set_layout_flags: DescriptorSetLayoutCreateFlags,
    ) -> DescriptorSetLayoutHandle {
        let mut bindings_flags: Vec<DescriptorBindingFlags> =
            Vec::with_capacity(self.bindings_infos.len());

        let bindings: Vec<_> = self
            .bindings_infos
            .iter_mut()
            .map(|binding_info| {
                let binding = &binding_info.binding;

                bindings_flags.push(binding_info.flags);
                let binding = DescriptorSetLayoutBinding {
                    binding: binding.binding,
                    descriptor_type: binding.descriptor_type,
                    descriptor_count: binding_info.binding.descriptor_count,
                    stage_flags: binding.stage_flags | shader_stages,
                    ..Default::default()
                };

                binding
            })
            .collect();

        let descriptor_set_layout_binding_flags_create_info =
            &mut DescriptorSetLayoutBindingFlagsCreateInfo::default()
                .binding_count(bindings_flags.len() as _)
                .binding_flags(&bindings_flags);

        let descriptor_set_layout_info = DescriptorSetLayoutCreateInfo::default()
            .flags(descriptor_set_layout_flags)
            .bindings(&bindings)
            .push_next(descriptor_set_layout_binding_flags_create_info);

        let descriptor_set_layout = device
            .create_descriptor_set_layout(&descriptor_set_layout_info)
            .unwrap();

        let descriptor_set_layout_size =
            device.get_descriptor_set_layout_size_ext(descriptor_set_layout);

        DescriptorSetLayoutHandle {
            descriptor_set_layout,
            descriptor_set_layout_size: descriptor_set_layout_size,
        }
    }

    #[inline(always)]
    fn get_descriptor_buffer_aligned_size(value: u64, alignment: u64) -> u64 {
        (value + alignment - 1) & !(alignment - 1)
    }
}
