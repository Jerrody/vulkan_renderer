use bevy_ecs::world::World;
use glam::Vec4;
use vulkanite::vk::{rs::*, *};

use crate::engine::{
    Engine,
    descriptors::{
        DescriptorKind, DescriptorSampledImage, DescriptorSampler, DescriptorSetBuilder,
        DescriptorSetHandle, DescriptorStorageImage,
    },
    resources::{
        buffers_pool::{BufferReference, BufferVisibility},
        model_loader::ModelLoader,
        *,
    },
    utils::*,
};

impl Engine {
    pub fn create_renderer_resources(world: &mut World) -> RendererResources {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();
        let frames_data: *mut FrameData = render_context.frames_data.as_ptr() as *const _ as *mut _;

        let device = vulkan_context.device;
        let allocator = &vulkan_context.allocator;

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

        let mesh_shader_path = r"intermediate\shaders\mesh.slang.spv";
        let shaders_info = [
            ShaderInfo {
                path: r"intermediate\shaders\gradient.slang.spv",
                flags: ShaderCreateFlagsEXT::empty(),
                stage: ShaderStageFlags::Compute,
                next_stage: ShaderStageFlags::empty(),
                descriptor_layouts: &descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
            ShaderInfo {
                path: mesh_shader_path,
                flags: ShaderCreateFlagsEXT::LinkStage,
                stage: ShaderStageFlags::TaskEXT,
                next_stage: ShaderStageFlags::MeshEXT,
                descriptor_layouts: &descriptor_set_layouts,
                push_constant_ranges: Some(&push_constant_ranges),
            },
            ShaderInfo {
                path: mesh_shader_path,
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
        let mut resources_pool = ResourcesPool::new(
            device,
            vulkan_context.allocator,
            upload_command_group,
            vulkan_context.transfer_queue,
        );

        let default_sampler_reference = resources_pool.samplers_pool.create_sampler(
            Filter::Linear,
            SamplerAddressMode::Repeat,
            true,
        );

        let mut renderer_resources = RendererResources {
            fallback_texture_reference: Default::default(),
            default_texture_reference: Default::default(),
            default_sampler_reference: default_sampler_reference,
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
        let (checkerboard_texture_reference, _) = renderer_resources.create_texture(
            None,
            false,
            Format::R8G8B8A8Unorm,
            checkerboard_image_extent,
            ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
            false,
        );

        renderer_resources.default_texture_reference = checkerboard_texture_reference;
        let descriptor_checkerboard_image = DescriptorKind::SampledImage(DescriptorSampledImage {
            image_view: renderer_resources
                .get_image(checkerboard_texture_reference)
                .unwrap()
                .image_view,
            index: checkerboard_texture_reference.index,
        });
        renderer_resources
            .resources_descriptor_set_handle
            .update_binding(device, allocator, descriptor_checkerboard_image);

        vulkan_context.transfer_data_to_image(
            &mut renderer_resources,
            checkerboard_texture_reference,
            pixels.as_ptr() as *const _,
            &render_context.upload_context,
            None,
        );

        let white_image_extent = Extent3D {
            width: 1,
            height: 1,
            depth: 1,
        };
        let (white_texture_reference, _) = renderer_resources.create_texture(
            None,
            false,
            Format::R8G8B8A8Srgb,
            white_image_extent,
            ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
            false,
        );
        renderer_resources.fallback_texture_reference = white_texture_reference;

        let white_image_pixels = [Self::pack_unorm_4x8(Vec4::new(1.0, 1.0, 1.0, 1.0))];
        vulkan_context.transfer_data_to_image(
            &mut renderer_resources,
            white_texture_reference,
            white_image_pixels.as_ptr() as *const _,
            &render_context.upload_context,
            None,
        );

        let descriptor_white_image = DescriptorKind::SampledImage(DescriptorSampledImage {
            image_view: renderer_resources
                .get_image(white_texture_reference)
                .unwrap()
                .image_view,
            index: white_texture_reference.index,
        });
        renderer_resources
            .resources_descriptor_set_handle
            .update_binding(device, allocator, descriptor_white_image);

        for frame_data_index in 0..render_context.frame_overlap {
            let frame_data = frames_data.wrapping_add(frame_data_index);

            let draw_image_extent = Extent3D {
                width: render_context.draw_extent.width,
                height: render_context.draw_extent.height,
                depth: 1,
            };

            let (draw_texture_reference, _) = renderer_resources.create_texture(
                None,
                false,
                Format::R16G16B16A16Sfloat,
                draw_image_extent,
                ImageUsageFlags::TransferSrc
                    | ImageUsageFlags::Storage
                    | ImageUsageFlags::ColorAttachment,
                false,
            );

            let (depth_texture_reference, _) = renderer_resources.create_texture(
                None,
                false,
                Format::D32Sfloat,
                draw_image_extent,
                ImageUsageFlags::DepthStencilAttachment,
                false,
            );

            let descriptor_draw_image = DescriptorKind::StorageImage(DescriptorStorageImage {
                image_view: renderer_resources
                    .get_image(draw_texture_reference)
                    .unwrap()
                    .image_view,
                index: draw_texture_reference.index,
            });
            renderer_resources
                .resources_descriptor_set_handle
                .update_binding(device, allocator, descriptor_draw_image);

            unsafe {
                (*frame_data).draw_texture_reference = draw_texture_reference;
                (*frame_data).depth_texture_reference = depth_texture_reference;
            }
        }

        let memory_bucket = &mut renderer_resources.resources_pool.buffers_pool;
        let materials_data_buffer_reference = memory_bucket.create_buffer(
            1024 * 1024 * 64,
            BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
            BufferVisibility::HostVisible,
            Some("Materials Data Buffer"),
        );
        let mut instance_objects_buffers = Vec::with_capacity(render_context.frame_overlap);
        for instances_objects_buffer_index in 0..instance_objects_buffers.capacity() {
            let instance_objects_buffer_reference = memory_bucket.create_buffer(
                std::mem::size_of::<InstanceObject>() * 4096,
                BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
                BufferVisibility::HostVisible,
                Some(
                    std::format!(
                        "Instances Objects Buffer {}",
                        instances_objects_buffer_index
                    )
                    .as_str(),
                ),
            );

            instance_objects_buffers.push(instance_objects_buffer_reference);
        }

        let mut scene_data_buffers = Vec::with_capacity(render_context.frame_overlap);
        for scene_data_buffer_index in 0..scene_data_buffers.capacity() {
            let scene_data_buffer_reference = memory_bucket.create_buffer(
                std::mem::size_of::<SceneData>(),
                BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
                BufferVisibility::HostVisible,
                Some(std::format!("Scene Data Buffer {}", scene_data_buffer_index).as_str()),
            );

            scene_data_buffers.push(scene_data_buffer_reference);
        }

        let mesh_objects_buffer_reference = memory_bucket.create_buffer(
            std::mem::size_of::<MeshObject>() * 8192,
            BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
            BufferVisibility::DeviceOnly,
            Some("Mesh Objects Buffer"),
        );

        renderer_resources.resources_pool.instances_buffer =
            Some(SwappableBuffer::new(instance_objects_buffers));
        renderer_resources.resources_pool.scene_data_buffer =
            Some(SwappableBuffer::new(scene_data_buffers));

        renderer_resources.set_materials_data_buffer_reference(materials_data_buffer_reference);
        renderer_resources.mesh_objects_buffer_reference = mesh_objects_buffer_reference;

        let sampler = renderer_resources
            .default_sampler_reference
            .get_sampler(&renderer_resources.resources_pool.samplers_pool)
            .unwrap();
        let sampler_descriptor = DescriptorKind::Sampler(DescriptorSampler {
            sampler: sampler,
            index: renderer_resources.default_sampler_reference.index,
        });

        renderer_resources
            .resources_descriptor_set_handle
            .update_binding(device, allocator, sampler_descriptor);

        renderer_resources
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

        descriptor_set_builder.build(
            device,
            &vulkan_context_resource.allocator,
            &device_properties_resource.descriptor_buffer_properties,
            push_constants_ranges,
            ShaderStageFlags::Compute
                | ShaderStageFlags::Fragment
                | ShaderStageFlags::MeshEXT
                | ShaderStageFlags::TaskEXT,
        )
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
                ShaderCreateInfoEXT::default()
                    .flags(shader_info.flags)
                    .code(shader_code)
                    .name(Some(c"main"))
                    .stage(shader_info.stage)
                    .next_stage(shader_info.next_stage)
                    .code_type(ShaderCodeTypeEXT::Spirv)
                    .set_layouts(shader_info.descriptor_layouts)
                    .push_constant_ranges(shader_info.push_constant_ranges.unwrap_or_default())
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
