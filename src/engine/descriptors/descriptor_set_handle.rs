use std::mem::ManuallyDrop;

use ahash::HashMap;
use vma::Allocator;

use vulkanite::{
    Handle,
    vk::{rs::*, *},
};

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

#[derive(Clone, Copy)]
pub struct DescriptorsSizes {
    pub uniform_buffer_descriptor_size: usize,
    pub sampled_image_descriptor_size: usize,
    pub sampler_descriptor_size: usize,
    pub storage_image_descriptor_size: usize,
    pub storage_buffer_descriptor_size: usize,
}

#[derive(Clone, Copy)]
pub struct BindingInfo {
    pub binding_offset: DeviceSize,
    // TODO: Pick next free slot index for simplicity, in reality, we should take free slot based on slot occupancy.
    pub next_empty_slot_index: usize,
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
        device: Device,
        allocator: &Allocator,
        descriptor_kind: DescriptorKind,
    ) {
        let descriptor_type = descriptor_kind.get_descriptor_type();

        let descriptors_sizes = self.descriptors_sizes;
        let descriptor_size = match descriptor_type {
            DescriptorType::UniformBuffer => descriptors_sizes.uniform_buffer_descriptor_size,
            DescriptorType::SampledImage => descriptors_sizes.sampled_image_descriptor_size,
            DescriptorType::StorageImage => descriptors_sizes.storage_image_descriptor_size,
            DescriptorType::Sampler => descriptors_sizes.sampled_image_descriptor_size,
            unsupported_descriptor_type => panic!(
                "Unsupported Descriptor Type found: {:?}",
                unsupported_descriptor_type
            ),
        };

        let descriptor_type_raw = descriptor_type as u32;
        let binding_info = self.bindings_infos[&descriptor_type_raw];

        let base_binding_offset = binding_info.binding_offset;
        let binding_offset = base_binding_offset
            + (binding_info.next_empty_slot_index as u64 * descriptor_size as u64);

        let allocation = &mut self.buffer.allocation;
        let descriptor_buffer_address = unsafe { allocator.map_memory(allocation).unwrap() };

        let target_descriptor_buffer_address =
            unsafe { descriptor_buffer_address.add(binding_offset as usize) };

        let mut descriptor_data = DescriptorDataEXT::default();
        let mut descriptor_get_info = DescriptorGetInfoEXT::default();

        match descriptor_kind {
            DescriptorKind::UniformBuffer(descriptor_uniform_buffer) => {
                let uniform_buffer_descriptor_address_info = DescriptorAddressInfoEXT {
                    address: descriptor_uniform_buffer.address,
                    range: descriptor_uniform_buffer.size,
                    format: Format::Undefined,
                    ..Default::default()
                };

                let mut p_uniform_descriptor_address_info =
                    ManuallyDrop::new(&uniform_buffer_descriptor_address_info as *const _ as _);
                descriptor_data.p_uniform_buffer = p_uniform_descriptor_address_info;

                descriptor_get_info.ty = DescriptorType::UniformBuffer;
                descriptor_get_info.data = descriptor_data;

                device.get_descriptor_ext(
                    &descriptor_get_info,
                    descriptor_size,
                    target_descriptor_buffer_address as _,
                );

                unsafe {
                    ManuallyDrop::drop(&mut p_uniform_descriptor_address_info);
                }
            }
            DescriptorKind::StorageImage(descriptor_storage_image) => {
                let storage_image_descriptor_info = DescriptorImageInfo {
                    image_view: Some(descriptor_storage_image.image_view.borrow()),
                    image_layout: ImageLayout::General,
                    ..Default::default()
                };

                let mut p_storage_image_descriptor_info =
                    ManuallyDrop::new(&storage_image_descriptor_info as *const _ as _);
                descriptor_data.p_storage_image = p_storage_image_descriptor_info;

                descriptor_get_info.ty = DescriptorType::StorageImage;
                descriptor_get_info.data = descriptor_data;

                device.get_descriptor_ext(
                    &descriptor_get_info,
                    descriptor_size,
                    target_descriptor_buffer_address as _,
                );

                unsafe {
                    ManuallyDrop::drop(&mut p_storage_image_descriptor_info);
                }
            }
            DescriptorKind::SampledImage(descriptor_sampled_image) => {
                let sampled_image_descriptor_info = DescriptorImageInfo {
                    image_view: Some(descriptor_sampled_image.image_view.borrow()),
                    image_layout: ImageLayout::General,
                    ..Default::default()
                };

                let mut p_sampled_image_descriptor_info =
                    ManuallyDrop::new(&sampled_image_descriptor_info as *const _ as _);
                descriptor_data.p_sampled_image = p_sampled_image_descriptor_info;

                descriptor_get_info.ty = DescriptorType::SampledImage;
                descriptor_get_info.data = descriptor_data;

                device.get_descriptor_ext(
                    &descriptor_get_info,
                    descriptor_size,
                    target_descriptor_buffer_address as _,
                );

                unsafe {
                    ManuallyDrop::drop(&mut p_sampled_image_descriptor_info);
                }
            }
            DescriptorKind::Sampler(descriptor_sampler) => {
                let mut p_sampler = ManuallyDrop::new(&descriptor_sampler.sampler as *const _ as _);
                descriptor_data.p_sampler = p_sampler;

                descriptor_get_info.ty = DescriptorType::Sampler;
                descriptor_get_info.data = descriptor_data;

                device.get_descriptor_ext(
                    &descriptor_get_info,
                    descriptor_size,
                    target_descriptor_buffer_address as _,
                );

                unsafe {
                    ManuallyDrop::drop(&mut p_sampler);
                }
            }
        };

        unsafe {
            allocator.unmap_memory(&mut *allocation);
        }
    }
}
