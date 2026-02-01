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
        DescriptorKind, DescriptorSampledImage, DescriptorSampler, DescriptorSetBuilder,
        DescriptorSetHandle, DescriptorStorageImage,
    },
    id::Id,
    resources::{model_loader::ModelLoader, *},
    utils::*,
};

impl Engine {
    pub fn create_renderer_resources(world: &mut World) -> RendererResources {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();
        let frames_data: *mut FrameData = render_context.frames_data.as_ptr() as *const _ as *mut _;

        let device = vulkan_context.device;
        let allocator = &vulkan_context.allocator;

        let nearest_sampler_create_info = SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            ..Default::default()
        };
        let nearest_sampler_object =
            SamplerObject::new(device.create_sampler(&nearest_sampler_create_info).unwrap());

        let push_constant_range = PushConstantRange {
            stage_flags: ShaderStageFlags::MeshEXT
                | ShaderStageFlags::Fragment
                | ShaderStageFlags::Compute
                | ShaderStageFlags::TaskEXT,
            offset: Default::default(),
            size: std::mem::size_of::<GraphicsPushConstant>() as _,
        };

        let push_constant_ranges = [push_constant_range];

        let resources_descriptor_set_handle =
            Self::create_descriptors(world, &push_constant_ranges);

        let descriptor_set_layouts = [resources_descriptor_set_handle
            .descriptor_set_layout_handle
            .descriptor_set_layout];

        let mesh_shader_path = r"shaders\_outputs\mesh.slang.spv";
        let shaders_info = [
            ShaderInfo {
                path: r"shaders\_outputs\gradient.slang.spv",
                flags: ShaderCreateFlagsEXT::empty(),
                stage: ShaderStageFlags::Compute,
                next_stage: ShaderStageFlags::empty(),
                descriptor_layouts: &descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
            ShaderInfo {
                path: &mesh_shader_path,
                flags: ShaderCreateFlagsEXT::LinkStage,
                stage: ShaderStageFlags::TaskEXT,
                next_stage: ShaderStageFlags::MeshEXT,
                descriptor_layouts: &descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
            ShaderInfo {
                path: &mesh_shader_path,
                flags: ShaderCreateFlagsEXT::LinkStage,
                stage: ShaderStageFlags::MeshEXT,
                next_stage: ShaderStageFlags::Fragment,
                descriptor_layouts: &descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
            ShaderInfo {
                path: mesh_shader_path,
                flags: ShaderCreateFlagsEXT::LinkStage,
                stage: ShaderStageFlags::Fragment,
                next_stage: ShaderStageFlags::empty(),
                descriptor_layouts: &descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
        ];

        let created_shaders = Self::create_shaders(&vulkan_context.device, &shaders_info);

        let model_loader = ModelLoader::new();

        let upload_command_group = render_context.upload_context.command_group;
        let resources_pool = ResourcesPool::new(
            device,
            vulkan_context.allocator,
            upload_command_group,
            vulkan_context.transfer_queue,
        );
        let mut renderer_resources = RendererResources {
            fallback_texture_id: Id::NULL,
            default_texture_id: Id::NULL,
            nearest_sampler_id: Id::NULL,
            mesh_objects_buffer_reference: BufferReference::default(),
            resources_descriptor_set_handle,
            gradient_compute_shader_object: created_shaders[0],
            task_shader_object: created_shaders[1],
            mesh_shader_object: created_shaders[2],
            fragment_shader_object: created_shaders[3],
            model_loader,
            resources_pool,
            is_printed_scene_hierarchy: true,
        };

        let magenta = &Self::pack_unorm_4x8(Vec4::new(1.0, 0.0, 1.0, 1.0));
        let black = &Self::pack_unorm_4x8(Vec4::new(0.0, 0.0, 0.0, 0.0));
        let mut pixels: Vec<u32> = vec![0; 16 * 16];
        for x in 0..16 {
            for y in 0..16 {
                pixels[y * 16 + x] = if (x % 2) ^ (y % 2) == 0 {
                    *magenta
                } else {
                    *black
                };
            }
        }

        let checkerboard_image_extent = Extent3D {
            width: 16,
            height: 16,
            depth: 1,
        };
        let checkerboard_image = Self::allocate_image(
            device,
            &allocator,
            Format::R8G8B8A8Unorm,
            checkerboard_image_extent,
            ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
        );

        vulkan_context.transfer_data_to_image(
            &checkerboard_image,
            pixels.as_ptr() as *const _,
            &mut renderer_resources.resources_pool.memory_bucket,
            &render_context.upload_context,
            None,
        );

        let white_image_extent = Extent3D {
            width: 1,
            height: 1,
            depth: 1,
        };
        let white_image = Self::allocate_image(
            device,
            &allocator,
            Format::R8G8B8A8Srgb,
            white_image_extent,
            ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
        );

        let white_image_pixels = [Self::pack_unorm_4x8(Vec4::new(1.0, 1.0, 1.0, 1.0))];
        vulkan_context.transfer_data_to_image(
            &white_image,
            white_image_pixels.as_ptr() as *const _,
            &mut renderer_resources.resources_pool.memory_bucket,
            &render_context.upload_context,
            None,
        );

        for frame_data_index in 0..render_context.frame_overlap {
            let frame_data = frames_data.wrapping_add(frame_data_index);

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

            let depth_image = Self::allocate_image(
                device,
                allocator,
                Format::D32Sfloat,
                draw_image_extent,
                ImageUsageFlags::DepthStencilAttachment,
            );

            let draw_image_id = renderer_resources.insert_texture(draw_image);

            let draw_image_ref = renderer_resources.get_texture_ref(draw_image_id);
            let descriptor_draw_image = DescriptorKind::StorageImage(DescriptorStorageImage {
                image_view: draw_image_ref.image_view,
            });
            let draw_image_index = renderer_resources
                .resources_descriptor_set_handle
                .update_binding(device, allocator, descriptor_draw_image);
            renderer_resources.get_texture_ref_mut(draw_image_id).index = draw_image_index.unwrap();

            let depth_image_id = renderer_resources.insert_texture(depth_image);

            unsafe {
                (*frame_data).draw_image_id = draw_image_id;
                (*frame_data).depth_image_id = depth_image_id;
            }
        }

        let memory_bucket = &mut renderer_resources.resources_pool.memory_bucket;
        let materials_data_buffer_reference = memory_bucket.create_buffer(
            1024 * 1024 * 64,
            BufferUsageFlags::StorageBuffer
                | BufferUsageFlags::ShaderDeviceAddress
                | BufferUsageFlags::TransferDst,
            BufferVisibility::HostVisible,
        );
        let mut instance_objects_buffers = Vec::with_capacity(render_context.frame_overlap);
        for _ in 0..instance_objects_buffers.capacity() {
            let instance_objects_buffer_reference = memory_bucket.create_buffer(
                std::mem::size_of::<InstanceObject>() * 4096,
                BufferUsageFlags::StorageBuffer
                    | BufferUsageFlags::ShaderDeviceAddress
                    | BufferUsageFlags::TransferDst,
                BufferVisibility::HostVisible,
            );

            instance_objects_buffers.push(instance_objects_buffer_reference);
        }

        let mesh_objects_buffer_reference = memory_bucket.create_buffer(
            std::mem::size_of::<MeshObject>() * 8192,
            BufferUsageFlags::StorageBuffer
                | BufferUsageFlags::ShaderDeviceAddress
                | BufferUsageFlags::TransferDst,
            BufferVisibility::HostVisible,
        );

        renderer_resources.resources_pool.instances_buffer =
            Some(SwappableBuffer::new(instance_objects_buffers));

        renderer_resources.set_materials_data_buffer_reference(materials_data_buffer_reference);
        renderer_resources.mesh_objects_buffer_reference = mesh_objects_buffer_reference;

        renderer_resources.fallback_texture_id = renderer_resources.insert_texture(white_image);
        renderer_resources.default_texture_id =
            renderer_resources.insert_texture(checkerboard_image);
        renderer_resources.nearest_sampler_id =
            renderer_resources.insert_sampler(nearest_sampler_object);

        // TODO: Need to make this mess more ergonomic and simpler.

        let checkerboard_image_ref =
            renderer_resources.get_texture_ref(renderer_resources.default_texture_id);
        let descriptor_checkerboard_image = DescriptorKind::SampledImage(DescriptorSampledImage {
            image_view: checkerboard_image_ref.image_view,
        });
        let checkerboard_image_index = renderer_resources
            .resources_descriptor_set_handle
            .update_binding(device, allocator, descriptor_checkerboard_image);
        renderer_resources
            .get_texture_ref_mut(renderer_resources.default_texture_id)
            .index = checkerboard_image_index.unwrap();

        let white_image_ref =
            renderer_resources.get_texture_ref(renderer_resources.fallback_texture_id);
        let descriptor_white_image = DescriptorKind::SampledImage(DescriptorSampledImage {
            image_view: white_image_ref.image_view,
        });
        let white_image_index = renderer_resources
            .resources_descriptor_set_handle
            .update_binding(device, allocator, descriptor_white_image);
        renderer_resources
            .get_texture_ref_mut(renderer_resources.fallback_texture_id)
            .index = white_image_index.unwrap();

        let sampler_object = renderer_resources.get_sampler(renderer_resources.nearest_sampler_id);
        let sampler_descriptor = DescriptorKind::Sampler(DescriptorSampler {
            sampler: sampler_object.sampler,
        });
        let sampler_object_index = renderer_resources
            .resources_descriptor_set_handle
            .update_binding(device, allocator, sampler_descriptor);
        renderer_resources
            .get_sampler_ref_mut(renderer_resources.nearest_sampler_id)
            .index = sampler_object_index.unwrap();

        renderer_resources
    }

    pub fn allocate_image(
        device: Device,
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
            id: Id::new(image.as_raw()),
            index: usize::MIN,
            image,
            image_view,
            allocation,
            extent,
            format,
            subresource_range: image_view_create_info.subresource_range,
        }
    }

    fn create_descriptors(
        world: &World,
        push_constants_ranges: &[PushConstantRange],
    ) -> DescriptorSetHandle {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let device_properties_resource = world
            .get_resource_ref::<DevicePropertiesResource>()
            .unwrap();
        let device = vulkan_context_resource.device;

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
            DescriptorBindingFlags::PartiallyBound
                | DescriptorBindingFlags::VariableDescriptorCount,
        );

        let resources_descriptor_set_handle = descriptor_set_builder.build(
            device,
            &vulkan_context_resource.allocator,
            &device_properties_resource.descriptor_buffer_properties,
            push_constants_ranges,
            ShaderStageFlags::Compute
                | ShaderStageFlags::Fragment
                | ShaderStageFlags::MeshEXT
                | ShaderStageFlags::TaskEXT,
        );

        resources_descriptor_set_handle
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
