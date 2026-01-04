use std::mem::ManuallyDrop;

use bevy_ecs::world::World;
use vma::{Alloc, AllocationCreateFlags, AllocationCreateInfo, MemoryUsage};
use vulkanite::{
    Handle, include_spirv,
    vk::{self, raw::get_buffer_device_address, rs::*, *},
};

use crate::engine::{
    Engine,
    descriptors::DescriptorSetLayoutBuilder,
    resources::{
        AllocatedBuffer, AllocatedDescriptorBuffer, AllocatedImage, DevicePropertiesResource,
        RendererContext, RendererResources, ShaderObject, VulkanContextResource,
        vulkan_context_resource,
    },
    utils::{create_image_info, create_image_view_info, load_shader},
};

impl Engine {
    pub fn create_renderer_resources(world: &World) -> RendererResources {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();

        let draw_image_extent = Extent3D {
            width: render_context.draw_extent.width,
            height: render_context.draw_extent.height,
            depth: 1,
        };
        let target_draw_image_format = Format::R16G16B16A16Sfloat;
        let image_usage_flags = ImageUsageFlags::TransferSrc
            | ImageUsageFlags::TransferDst
            | ImageUsageFlags::Storage
            | ImageUsageFlags::ColorAttachment;

        let image_create_info = create_image_info(
            target_draw_image_format,
            image_usage_flags,
            draw_image_extent,
        );
        let allocation_info = AllocationCreateInfo {
            usage: MemoryUsage::Auto,
            required_flags: MemoryPropertyFlags::DeviceLocal,
            ..Default::default()
        };

        let (allocated_draw_image, allocation) = unsafe {
            vulkan_context
                .allocator
                .create_image(&image_create_info, &allocation_info)
                .unwrap()
        };

        let allocated_draw_image = rs::Image::from_inner(allocated_draw_image);
        let image_view_create_info = create_image_view_info(
            target_draw_image_format,
            &allocated_draw_image,
            ImageAspectFlags::Color,
        );
        let allocated_image_view = vulkan_context
            .device
            .create_image_view(&image_view_create_info)
            .unwrap();

        let draw_image = AllocatedImage {
            image: allocated_draw_image,
            image_view: allocated_image_view,
            allocation,
            image_extent: draw_image_extent,
            format: Format::R16G16B16A16Sfloat,
        };

        let draw_image_descriptor_buffer = Self::create_descriptors(world, &draw_image);

        let descriptor_layouts = [draw_image_descriptor_buffer.descriptor_set_layout];
        let gradient_compute_shader_object = Self::create_shader(
            &vulkan_context.device,
            ShaderStageFlags::Compute,
            &descriptor_layouts,
        );

        RendererResources {
            draw_image,
            draw_image_descriptor_buffer,
            gradient_compute_shader_object,
        }
    }

    fn create_descriptors(world: &World, draw_image: &AllocatedImage) -> AllocatedDescriptorBuffer {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let device_properties_resource = world
            .get_resource_ref::<DevicePropertiesResource>()
            .unwrap();
        let device = &vulkan_context_resource.device;

        let mut descriptor_set_layout_builder = DescriptorSetLayoutBuilder::new();
        descriptor_set_layout_builder.add_binding(0, DescriptorType::StorageImage);
        let descriptor_set_layout = descriptor_set_layout_builder.build(
            device,
            ShaderStageFlags::Compute,
            DescriptorSetLayoutCreateFlags::DescriptorBufferEXT,
        );

        let descriptor_set_layout_size =
            device.get_descriptor_set_layout_size_ext(&descriptor_set_layout);

        let descriptor_buffer_size = Self::aligned_size(
            descriptor_set_layout_size,
            device_properties_resource
                .descriptor_buffer_properties
                .descriptor_buffer_offset_alignment,
        );

        let descriptor_buffer_offset = device.get_descriptor_set_layout_binding_offset_ext(
            &descriptor_set_layout,
            Default::default(),
        );

        let buffer_info = BufferCreateInfo::default()
            .size(descriptor_buffer_size)
            .usage(
                BufferUsageFlags::ShaderDeviceAddress
                    | BufferUsageFlags::ResourceDescriptorBufferEXT,
            );

        let allocation_info = AllocationCreateInfo {
            flags: AllocationCreateFlags::Mapped | AllocationCreateFlags::HostAccessRandom,
            usage: MemoryUsage::Auto,
            ..Default::default()
        };

        let (storage_image_descriptor_buffer, allocation) = unsafe {
            vulkan_context_resource
                .allocator
                .create_buffer(&buffer_info, &allocation_info)
                .unwrap()
        };
        let storage_image_descriptor_buffer = Buffer::from_inner(storage_image_descriptor_buffer);

        let allocated_descriptor_buffer = AllocatedBuffer {
            buffer: storage_image_descriptor_buffer,
            allocation,
        };

        let draw_image_descriptor_image_info = DescriptorImageInfo::default()
            .image_layout(ImageLayout::General)
            .image_view(Some(&draw_image.image_view));

        let descriptor_size = device_properties_resource
            .descriptor_buffer_properties
            .storage_image_descriptor_size;

        let mut draw_image_descriptor_get_info =
            DescriptorGetInfoEXT::default().ty(DescriptorType::StorageImage);

        let p_draw_image_descriptor_image_info =
            ManuallyDrop::new(&draw_image_descriptor_image_info as *const _ as _);
        draw_image_descriptor_get_info.data.p_storage_image = p_draw_image_descriptor_image_info;

        let mut allocation = allocated_descriptor_buffer.allocation;
        let descriptor_buffer_address = unsafe {
            vulkan_context_resource
                .allocator
                .map_memory(&mut allocation)
                .unwrap()
        };
        device.get_descriptor_ext(
            &draw_image_descriptor_get_info,
            descriptor_size,
            descriptor_buffer_address as _,
        );
        unsafe {
            vulkan_context_resource
                .allocator
                .unmap_memory(&mut allocation);
        }

        let descriptor_set_layouts: [vk::raw::DescriptorSetLayout; 1] = unsafe {
            [vk::raw::DescriptorSetLayout::from_raw(
                descriptor_set_layout.as_raw(),
            )]
        };
        let pipeline_layout_info =
            PipelineLayoutCreateInfo::default().set_layouts(descriptor_set_layouts.as_slice());
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info)
            .unwrap();

        let allocated_descriptor_buffer_address =
            Self::get_device_address(&device, &allocated_descriptor_buffer.buffer);
        AllocatedDescriptorBuffer {
            allocated_descriptor_buffer,
            descriptor_buffer_offset,
            descriptor_buffer_size,
            descriptor_set_layout,
            address: allocated_descriptor_buffer_address,
            pipeline_layout,
        }
    }

    fn aligned_size(value: u64, alignment: u64) -> u64 {
        (value + alignment - 1) & !(alignment - 1)
    }

    fn get_device_address(device: &Device, buffer: &Buffer) -> DeviceAddress {
        let buffer_device_address = BufferDeviceAddressInfo::default().buffer(buffer);

        let buffer_address = device.get_buffer_address(&buffer_device_address);

        buffer_address
    }

    fn create_shader(
        device: &Device,
        stage: ShaderStageFlags,
        descriptor_set_layout: &[DescriptorSetLayout],
    ) -> ShaderObject {
        let shader_code = load_shader(r"shaders\output\gradient.slang.spv");

        let shader_info = ShaderCreateInfoEXT::default()
            .code(&shader_code)
            .name(Some(c"main"))
            .stage(stage)
            .code_type(ShaderCodeTypeEXT::Spirv)
            .set_layouts(descriptor_set_layout);

        let shader_infos = [shader_info];
        let (_status, shaders): (_, Vec<ShaderEXT>) =
            device.create_shaders_ext(&shader_infos).unwrap();

        let shader = shaders[0];

        let shader_object = ShaderObject::new(shader, stage);

        shader_object
    }
}
