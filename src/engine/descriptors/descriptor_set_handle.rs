use std::mem::ManuallyDrop;

use ahash::HashMap;
use vma::{Allocation, Allocator};

use vulkanite::vk::{rs::*, *};

use crate::engine::{descriptors::DescriptorKind, resources::AllocatedBuffer};

pub struct DescriptorSetLayoutHandle {
    pub descriptor_set_layout: DescriptorSetLayout,
    pub descriptor_set_layout_size: u64,
}

pub struct DescriptorSetBinding {
    pub descriptor_binding_type: DescriptorType,
    pub index: usize,
    pub offset: usize,
}

pub struct DescriptorsSizes {
    pub uniform_buffer_descriptor_size: usize,
    pub sampled_image_descriptor_size: usize,
    pub sampler_descriptor_size: usize,
    pub storage_image_descriptor_size: usize,
    pub storage_buffer_descriptor_size: usize,
}

pub struct BindingInfo {
    pub binding_index: usize,
    pub binding_offset: DeviceSize,
}

pub struct DescriptorSetHandle {
    pub buffer: AllocatedBuffer,
    pub descriptor_set_layout_handle: DescriptorSetLayoutHandle,
    pub bindings_infos: HashMap<u32, BindingInfo>,
    pub pipeline_layout: PipelineLayout,
    pub descriptors_sizes: DescriptorsSizes,
}

impl DescriptorSetHandle {
    pub fn update_binding(
        &mut self,
        binding_index: u32,
        element_index: u32,
        descriptor_kind: DescriptorKind,
    ) {
        let mut allocation = &mut self.buffer.allocation;
        let mut descriptor_data = DescriptorDataEXT::default();

        match descriptor_kind {
            DescriptorKind::UniformBuffer(descriptor_uniform_buffer) => {
                let uniform_descriptor_address_info = DescriptorAddressInfoEXT {
                    address: descriptor_uniform_buffer.address,
                    range: descriptor_uniform_buffer.size,
                    format: Format::Undefined,
                    ..Default::default()
                };

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
                    binding_info.binding_offset,
                );

                drop(p_uniform_descriptor_address_info);
            }
            DescriptorKind::StorageImage(descriptor_storage_image) => {
                let storage_image_descriptor_info = DescriptorImageInfo {
                    image_view: Some(descriptor_storage_image.image_view.borrow()),
                    image_layout: ImageLayout::General,
                    ..Default::default()
                };

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
                    binding_info.binding_offset,
                );

                drop(p_storage_image_descriptor_info);
            }
            DescriptorKind::SampledImage(descriptor_sampled_image) => {
                let sampled_image_descriptor_info = DescriptorImageInfo {
                    image_view: Some(descriptor_sampled_image.image_view.borrow()),
                    image_layout: ImageLayout::General,
                    ..Default::default()
                };

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
                    binding_info.binding_offset,
                );

                drop(p_sampled_image_descriptor_info);
            }
            DescriptorKind::Sampler(descriptor_sampler) => {
                let p_sampler = ManuallyDrop::new(&descriptor_sampler.sampler as *const _ as _);
                descriptor_data.p_sampler = p_sampler;

                self.get_descriptor(
                    device,
                    allocator,
                    descriptor_buffer_allocation,
                    descriptor_sampler.get_descriptor_type(),
                    descriptor_data,
                    descriptor_size,
                    binding_info.binding_offset,
                );

                drop(p_sampler);
            }
        };
    }

    fn get_descriptor(
        &self,
        device: Device,
        allocator: &Allocator,
        allocation: &mut Allocation,
        descriptor_type: DescriptorType,
        descriptor_data: DescriptorDataEXT<'_>,
        descriptor_size: usize,
        descriptor_binding_offset: u64,
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
}
