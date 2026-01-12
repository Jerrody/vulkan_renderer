use bevy_ecs::world::World;
use glam::Vec4;
use vma::{Alloc, AllocationCreateInfo, Allocator, MemoryUsage};
use vulkanite::{
    Handle,
    vk::{rs::*, *},
};

use crate::engine::{
    Engine,
    descriptors::{
        DescriptorKind, DescriptorSetBuilder, DescriptorSetHandle, DescriptorStorageImage,
        descriptor_set_builder,
    },
    resources::{model_loader::ModelLoader, *},
    utils::*,
};

impl Engine {
    pub fn create_renderer_resources(world: &World) -> RendererResources {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();

        let device = &vulkan_context.device;
        let allocator = &vulkan_context.allocator;

        let draw_image_extent = Extent3D {
            width: render_context.draw_extent.width,
            height: render_context.draw_extent.height,
            depth: 1,
        };

        let draw_image = Self::allocate_image(
            device,
            allocator,
            Format::R16G16B16A16Sfloat,
            draw_image_extent,
            ImageUsageFlags::TransferSrc
                | ImageUsageFlags::Storage
                | ImageUsageFlags::ColorAttachment,
        );

        let white_color = &Self::pack_unorm_4x8(Vec4::new(1.0, 1.0, 1.0, 1.0));
        let white_image_extent = Extent3D {
            width: 1,
            height: 1,
            depth: 1,
        };
        let white_image = Self::allocate_image(
            device,
            &allocator,
            Format::R8G8B8A8Unorm,
            white_image_extent,
            ImageUsageFlags::Sampled | ImageUsageFlags::HostTransfer,
        );
        Self::transfer_data_to_image(device, &white_image, white_color as *const _ as _);

        let depth_image = Self::allocate_image(
            device,
            allocator,
            Format::D32Sfloat,
            draw_image_extent,
            ImageUsageFlags::DepthStencilAttachment,
        );

        let draw_image_descriptor_set_handle = Self::create_descriptors(world, &draw_image);

        let gradient_descriptor_layouts = [draw_image_descriptor_set_handle
            .descriptor_set_layout_handle
            .descriptor_set_layout];

        let triangle_descriptor_set_layouts = [];

        let push_constant_ranges = [PushConstantRange {
            stage_flags: ShaderStageFlags::MeshEXT,
            offset: Default::default(),
            size: size_of::<MeshPushConstant>() as _,
        }];
        let mesh_pipeline_layout_create_info = PipelineLayoutCreateInfo::default()
            .push_constant_ranges(push_constant_ranges.as_slice());

        let mesh_pipeline_layout = vulkan_context
            .device
            .create_pipeline_layout(&mesh_pipeline_layout_create_info)
            .unwrap();

        let mesh_shader_path = r"shaders\output\mesh.slang.spv";
        let shaders_info = [
            ShaderInfo {
                path: r"shaders\output\gradient.slang.spv",
                flags: ShaderCreateFlagsEXT::empty(),
                stage: ShaderStageFlags::Compute,
                next_stage: ShaderStageFlags::empty(),
                descriptor_layouts: &gradient_descriptor_layouts,
                push_constant_ranges: None,
            },
            ShaderInfo {
                path: &mesh_shader_path,
                flags: ShaderCreateFlagsEXT::LinkStage | ShaderCreateFlagsEXT::NoTaskShader,
                stage: ShaderStageFlags::MeshEXT,
                next_stage: ShaderStageFlags::Fragment,
                descriptor_layouts: &triangle_descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
            ShaderInfo {
                path: mesh_shader_path,
                flags: ShaderCreateFlagsEXT::LinkStage,
                stage: ShaderStageFlags::Fragment,
                next_stage: ShaderStageFlags::empty(),
                descriptor_layouts: &triangle_descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
        ];

        let created_shaders = Self::create_shaders(&vulkan_context.device, &shaders_info);

        let model_loader = ModelLoader::new();

        let nearest_sampler_create_info = SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            ..Default::default()
        };
        let nearest_sampler = device.create_sampler(&nearest_sampler_create_info).unwrap();

        RendererResources {
            draw_image,
            depth_image,
            white_image,
            draw_image_descriptor_buffer: draw_image_descriptor_set_handle,
            gradient_compute_shader_object: created_shaders[0],
            mesh_shader_object: created_shaders[1],
            fragment_shader_object: created_shaders[2],
            model_loader,
            mesh_buffers: Default::default(),
            mesh_pipeline_layout,
            mesh_push_constant: Default::default(),
            nearest_sampler,
        }
    }

    fn allocate_image(
        device: &Device,
        allocator: &Allocator,
        format: Format,
        extent: Extent3D,
        usage_flags: ImageUsageFlags,
    ) -> AllocatedImage {
        let mut aspect_flags = ImageAspectFlags::Color;
        if format == Format::D32Sfloat {
            aspect_flags = ImageAspectFlags::Depth;
        }

        let allocation_info = AllocationCreateInfo {
            usage: MemoryUsage::Auto,
            required_flags: MemoryPropertyFlags::DeviceLocal,
            ..Default::default()
        };

        let image_create_info =
            create_image_info(format, usage_flags, extent, ImageLayout::Undefined);
        let (allocated_image, allocation) = unsafe {
            allocator
                .create_image(&image_create_info, &allocation_info)
                .unwrap()
        };

        let image = rs::Image::from_inner(allocated_image);
        let image_view_create_info = create_image_view_info(format, &image, aspect_flags);
        let image_view = device.create_image_view(&image_view_create_info).unwrap();

        AllocatedImage {
            image,
            image_view,
            allocation,
            extent,
            format,
            subresource_range: image_view_create_info.subresource_range,
        }
    }

    fn transfer_data_to_image(
        device: &Device,
        allocated_image: &AllocatedImage,
        data: *const std::ffi::c_void,
    ) {
        let host_image_layout_transition_info = [HostImageLayoutTransitionInfo {
            image: Some(allocated_image.image.borrow()),
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::General,
            subresource_range: allocated_image.subresource_range,
            ..Default::default()
        }];

        device
            .transition_image_layout(&host_image_layout_transition_info)
            .unwrap();

        let memory_to_image_copy = MemoryToImageCopy {
            p_host_pointer: data,
            image_subresource: ImageSubresourceLayers {
                aspect_mask: allocated_image.subresource_range.aspect_mask,
                mip_level: Default::default(),
                base_array_layer: Default::default(),
                layer_count: 1,
            },
            image_extent: allocated_image.extent,
            ..Default::default()
        };

        let regions = [memory_to_image_copy];
        let copy_memory_to_image_info = CopyMemoryToImageInfo {
            flags: HostImageCopyFlags::Memcpy,
            dst_image: Some(allocated_image.image.borrow()),
            dst_image_layout: ImageLayout::General,
            region_count: regions.len() as _,
            p_regions: regions.as_ptr() as *const _,
            ..Default::default()
        };

        device
            .copy_memory_to_image(&copy_memory_to_image_info)
            .unwrap();
    }

    fn create_descriptors(world: &World, draw_image: &AllocatedImage) -> DescriptorSetHandle {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let device_properties_resource = world
            .get_resource_ref::<DevicePropertiesResource>()
            .unwrap();
        let device = vulkan_context_resource.device;

        let mut descriptor_set_builder = DescriptorSetBuilder::new();

        let descriptor_storage_image = DescriptorStorageImage {
            image_view: draw_image.image_view,
        };

        descriptor_set_builder.add_binding(DescriptorKind::StorageImage(descriptor_storage_image));
        let storage_image_descriptor_set_handle = descriptor_set_builder.build(
            device,
            &vulkan_context_resource.allocator,
            &device_properties_resource.descriptor_buffer_properties,
            ShaderStageFlags::Compute,
        );

        storage_image_descriptor_set_handle
    }

    fn create_shaders(device: &Device, shader_infos: &[ShaderInfo]) -> Vec<ShaderObject> {
        let shader_codes: Vec<Vec<u8>> = shader_infos
            .iter()
            .map(|shader_info| load_shader(shader_info.path))
            .collect();

        let shader_create_infos: Vec<_> = shader_infos
            .iter()
            .zip(shader_codes.as_slice())
            .map(|(shader_info, shader_code)| {
                let shader_info = ShaderCreateInfoEXT::default()
                    .flags(shader_info.flags)
                    .code(shader_code)
                    .name(Some(c"main"))
                    .stage(shader_info.stage)
                    .next_stage(shader_info.next_stage)
                    .code_type(ShaderCodeTypeEXT::Spirv)
                    .set_layouts(shader_info.descriptor_layouts)
                    .push_constant_ranges(shader_info.push_constant_ranges.unwrap_or_default());

                shader_info
            })
            .collect();

        let (_status, shaders): (_, Vec<ShaderEXT>) =
            device.create_shaders_ext(&shader_create_infos).unwrap();

        shaders
            .into_iter()
            .zip(shader_infos.iter().as_slice())
            .map(|(shader, shader_info)| ShaderObject::new(shader, shader_info.stage))
            .collect()
    }

    #[allow(unused)]
    fn create_shader(device: &Device, shader_info: ShaderInfo) -> ShaderObject {
        let shader_code = load_shader(shader_info.path);

        let shader_create_info = ShaderCreateInfoEXT::default()
            .flags(shader_info.flags)
            .code(&shader_code)
            .name(Some(c"main"))
            .stage(shader_info.stage)
            .next_stage(shader_info.next_stage)
            .code_type(ShaderCodeTypeEXT::Spirv)
            .set_layouts(shader_info.descriptor_layouts);

        let shader_infos = [shader_create_info];
        let (_status, shaders): (_, Vec<ShaderEXT>) =
            device.create_shaders_ext(&shader_infos).unwrap();

        let shader = shaders[0];

        ShaderObject::new(shader, shader_info.stage)
    }

    pub fn pack_unorm_4x8(v: Vec4) -> u32 {
        let v = v.clamp(Vec4::ZERO, Vec4::ONE) * 255.0;

        // 3. Round to nearest integer and cast to u8
        // Note: using arrays + map is often cleaner than manual bit shifting
        let [x, y, z, w] = v.to_array().map(|c| c.round() as u8);

        // 4. Pack into u32 using Little Endian (x is LSB, w is MSB)
        // This matches the GLSL behavior:
        // Bits 0-7:   x
        // Bits 8-15:  y
        // Bits 16-23: z
        // Bits 24-31: w
        u32::from_le_bytes([x, y, z, w])
    }
}
