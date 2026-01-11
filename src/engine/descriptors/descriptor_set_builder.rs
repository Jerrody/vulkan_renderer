use std::mem::ManuallyDrop;

use bevy_ecs::resource::Resource;
use vma::*;
use vulkanite::{
    Handle,
    vk::{rs::*, *},
};

use crate::engine::{descriptors::*, resources::AllocatedBuffer, utils::get_device_address};

pub enum DescriptorKind {
    UniformBuffer(DescriptorUniformBuffer),
    StorageImage(DescriptorStorageImage),
    CombinedImageSampler(DescriptorCombinedImageSampler),
    SampledImage(DescriptorSampledImage),
    Sampler(DescriptorSampler),
}

struct DescriptorSetLayoutBindingInfo<'a> {
    pub binding: DescriptorSetLayoutBinding<'a>,
    pub descriptor_kind: DescriptorKind,
    pub binding_offset: u64,
}

#[derive(Resource, Default)]
pub struct DescriptorSetBuilder<'a> {
    bindings_infos: Vec<DescriptorSetLayoutBindingInfo<'a>>,
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_binding(&mut self, descriptor_kind: DescriptorKind) {
        let mut sampler: Option<Sampler> = None;
        let mut image_view: Option<ImageView> = None;

        let descriptor_type = match descriptor_kind {
            DescriptorKind::UniformBuffer(descriptor_uniform_buffer) => {
                descriptor_uniform_buffer.get_descriptor_type()
            }
            DescriptorKind::StorageImage(descriptor_storage_image) => {
                descriptor_storage_image.get_descriptor_type()
            }
            DescriptorKind::CombinedImageSampler(descriptor_combined_image_sampler) => {
                image_view = Some(descriptor_combined_image_sampler.image_view);
                sampler = Some(descriptor_combined_image_sampler.sampler);

                descriptor_combined_image_sampler.get_descriptor_type()
            }
            DescriptorKind::SampledImage(_) | DescriptorKind::Sampler(_) => panic!(
                "Should be specified descriptor CombinedImageSampler instead of descriptor Sampled Image or/and Sampler!"
            ),
        };

        if descriptor_type == DescriptorType::CombinedImageSampler {
            for i in 0..2 {
                let mut descriptor_type = DescriptorType::Sampler;
                if i > Default::default() {
                    descriptor_type = DescriptorType::SampledImage;
                }

                let next_binding_index = self.bindings_infos.len();
                let binding = DescriptorSetLayoutBinding::default()
                    .binding(next_binding_index as _)
                    .descriptor_type(descriptor_type)
                    .descriptor_count(1);

                let binding_info: DescriptorSetLayoutBindingInfo;
                if descriptor_type == DescriptorType::SampledImage {
                    let descriptor_kind = DescriptorSampledImage {
                        image_view: image_view.unwrap(),
                    };

                    binding_info = DescriptorSetLayoutBindingInfo {
                        binding,
                        binding_offset: Default::default(),
                        descriptor_kind: DescriptorKind::SampledImage(descriptor_kind),
                    };
                } else {
                    let descriptor_kind = DescriptorSampler {
                        sampler: sampler.unwrap(),
                    };

                    binding_info = DescriptorSetLayoutBindingInfo {
                        binding,
                        binding_offset: Default::default(),
                        descriptor_kind: DescriptorKind::Sampler(descriptor_kind),
                    };
                }

                self.bindings_infos.push(binding_info);
            }
        } else {
            let next_binding_index = self.bindings_infos.len();
            let binding = DescriptorSetLayoutBinding::default()
                .binding(next_binding_index as _)
                .descriptor_type(descriptor_type)
                .descriptor_count(1);

            let binding_info = DescriptorSetLayoutBindingInfo {
                binding,
                descriptor_kind,
                binding_offset: Default::default(),
            };

            self.bindings_infos.push(binding_info);
        }
    }

    pub fn clear(&mut self) {
        self.bindings_infos.clear();
    }

    pub fn build(
        &mut self,
        device: Device,
        allocator: &Allocator,
        descriptor_buffer_properties: &PhysicalDeviceDescriptorBufferPropertiesEXT,
        shader_stages: ShaderStageFlags,
    ) -> DescriptorSetHandle {
        let descriptor_set_layout_handle = self.create_descriptor_set_layout(
            device,
            shader_stages,
            DescriptorSetLayoutCreateFlags::DescriptorBufferEXT,
        );

        self.bindings_infos.iter_mut().enumerate().for_each(
            |(binding_info_index, binding_info)| {
                let binding_offset = device.get_descriptor_set_layout_binding_offset_ext(
                    descriptor_set_layout_handle.descriptor_set_layout,
                    binding_info_index as _,
                );

                binding_info.binding_offset = binding_offset;
            },
        );

        let descriptor_buffer_size = Self::aligned_size(
            descriptor_set_layout_handle.descriptor_set_layout_size,
            descriptor_buffer_properties.descriptor_buffer_offset_alignment,
        );

        let mut descriptor_buffer =
            self.create_descriptor_buffer(device, allocator, descriptor_buffer_size);
        let descriptor_buffer_allocation = &mut descriptor_buffer.allocation;

        self.bindings_infos
            .iter()
            .enumerate()
            .for_each(|(binding_index, binding_info)| {
                let mut descriptor_data = DescriptorDataEXT::default();

                match binding_info.descriptor_kind {
                    DescriptorKind::UniformBuffer(descriptor_uniform_buffer) => {
                        let uniform_descriptor_address_info = DescriptorAddressInfoEXT {
                            address: descriptor_uniform_buffer.address,
                            range: descriptor_uniform_buffer.size,
                            format: Format::Undefined,
                            ..Default::default()
                        };
                        let descriptor_size =
                            descriptor_buffer_properties.uniform_buffer_descriptor_size;

                        let p_uniform_descriptor_address_info =
                            ManuallyDrop::new(&uniform_descriptor_address_info as *const _ as _);
                        descriptor_data.p_uniform_buffer = p_uniform_descriptor_address_info;

                        Self::get_descriptor(
                            device,
                            allocator,
                            descriptor_buffer_allocation,
                            descriptor_uniform_buffer.get_descriptor_type(),
                            descriptor_data,
                            descriptor_size,
                            binding_index,
                            binding_info.binding_offset,
                            descriptor_buffer_size,
                        );

                        drop(p_uniform_descriptor_address_info);
                    }
                    DescriptorKind::StorageImage(descriptor_storage_image) => {
                        let storage_image_descriptor_info = DescriptorImageInfo {
                            image_view: Some(descriptor_storage_image.image_view.borrow()),
                            image_layout: ImageLayout::General,
                            ..Default::default()
                        };
                        let descriptor_size =
                            descriptor_buffer_properties.storage_image_descriptor_size;

                        let p_storage_image_descriptor_info =
                            ManuallyDrop::new(&storage_image_descriptor_info as *const _ as _);
                        descriptor_data.p_storage_image = p_storage_image_descriptor_info;

                        Self::get_descriptor(
                            device,
                            allocator,
                            descriptor_buffer_allocation,
                            descriptor_storage_image.get_descriptor_type(),
                            descriptor_data,
                            descriptor_size,
                            binding_index,
                            binding_info.binding_offset,
                            descriptor_buffer_size,
                        );

                        drop(p_storage_image_descriptor_info);
                    }
                    DescriptorKind::SampledImage(descriptor_sampled_image) => {
                        let sampled_image_descriptor_info = DescriptorImageInfo {
                            image_view: Some(descriptor_sampled_image.image_view.borrow()),
                            image_layout: ImageLayout::General,
                            ..Default::default()
                        };
                        let descriptor_size =
                            descriptor_buffer_properties.sampled_image_descriptor_size;

                        let p_sampled_image_descriptor_info =
                            ManuallyDrop::new(&sampled_image_descriptor_info as *const _ as _);
                        descriptor_data.p_sampled_image = p_sampled_image_descriptor_info;

                        Self::get_descriptor(
                            device,
                            allocator,
                            descriptor_buffer_allocation,
                            descriptor_sampled_image.get_descriptor_type(),
                            descriptor_data,
                            descriptor_size,
                            binding_index,
                            binding_info.binding_offset,
                            descriptor_buffer_size,
                        );

                        drop(p_sampled_image_descriptor_info);
                    }
                    DescriptorKind::Sampler(descriptor_sampler) => {
                        let descriptor_size = descriptor_buffer_properties.sampler_descriptor_size;

                        let p_sampler =
                            ManuallyDrop::new(&descriptor_sampler.sampler as *const _ as _);
                        descriptor_data.p_sampler = p_sampler;

                        Self::get_descriptor(
                            device,
                            allocator,
                            descriptor_buffer_allocation,
                            descriptor_sampler.get_descriptor_type(),
                            descriptor_data,
                            descriptor_size,
                            binding_index,
                            binding_info.binding_offset,
                            descriptor_buffer_size,
                        );

                        drop(p_sampler);
                    }
                    DescriptorKind::CombinedImageSampler(_) => panic!(
                        "Descriptor Combined Image Sampler should be presented in bindings infos!"
                    ),
                };
            });

        let descriptor_set_layouts = [descriptor_set_layout_handle.descriptor_set_layout];
        let pipeline_layout_info =
            PipelineLayoutCreateInfo::default().set_layouts(descriptor_set_layouts.as_slice());
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info)
            .unwrap();

        self.clear();

        DescriptorSetHandle {
            buffer: descriptor_buffer,
            descriptor_set_layout_handle,
            pipeline_layout,
        }
    }

    fn get_descriptor(
        device: Device,
        allocator: &Allocator,
        allocation: &mut Allocation,
        descriptor_type: DescriptorType,
        descriptor_data: DescriptorDataEXT<'_>,
        descriptor_size: usize,
        binding_index: usize,
        descriptor_binding_offset: u64,
        descriptor_buffer_size: u64,
    ) {
        let descriptor_get_info = DescriptorGetInfoEXT {
            ty: descriptor_type,
            data: descriptor_data,
            ..Default::default()
        };

        let descriptor_buffer_address_with_offset = unsafe {
            let mut descriptor_buffer_address = allocator.map_memory(allocation).unwrap();

            descriptor_buffer_address =
                descriptor_buffer_address.add(descriptor_binding_offset as _);
            descriptor_buffer_address =
                descriptor_buffer_address.add(binding_index * descriptor_buffer_size as usize);

            descriptor_buffer_address
        };

        device.get_descriptor_ext(
            &descriptor_get_info,
            descriptor_size,
            descriptor_buffer_address_with_offset as _,
        );

        unsafe {
            allocator.unmap_memory(allocation);
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

        AllocatedBuffer {
            buffer: descriptor_buffer,
            allocation,
            device_address: get_device_address(device, &descriptor_buffer),
        }
    }

    fn create_descriptor_set_layout(
        &mut self,
        device: Device,
        shader_stages: ShaderStageFlags,
        descriptor_set_layout_flags: DescriptorSetLayoutCreateFlags,
    ) -> DescriptorSetLayoutHandle {
        let bindings: Vec<_> = self
            .bindings_infos
            .iter_mut()
            .map(|binding_info| {
                let binding = &binding_info.binding;

                let binding = DescriptorSetLayoutBinding {
                    binding: binding.binding,
                    descriptor_type: binding.descriptor_type,
                    descriptor_count: 1,
                    stage_flags: binding.stage_flags | shader_stages,
                    ..Default::default()
                };

                binding
            })
            .collect();

        let descriptor_set_layout_info = DescriptorSetLayoutCreateInfo {
            flags: descriptor_set_layout_flags,
            binding_count: bindings.len() as _,
            p_bindings: bindings.as_ptr() as *const _,
            ..Default::default()
        };

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
    fn aligned_size(value: u64, alignment: u64) -> u64 {
        (value + alignment - 1) & !(alignment - 1)
    }
}
