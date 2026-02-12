use bevy_ecs::system::{Res, ResMut};
use glam::Vec4;
use vulkanite::vk::{rs::*, *};

use crate::engine::{
    ecs::{
        InstanceObject, MeshObject, RendererContext, RendererResources, SceneData, ShaderObject,
        SwappableBuffer, VulkanContextResource,
        buffers_pool::{BufferVisibility, BuffersMut},
        samplers_pool::SamplersMut,
        textures_pool::TexturesMut,
    },
    general::renderer::{DescriptorKind, DescriptorSampledImage, DescriptorStorageImage},
    utils::{ShaderInfo, load_shader},
};

pub fn prepare_shaders_system(
    vulkan_ctx_resource: Res<VulkanContextResource>,
    mut render_context: ResMut<RendererContext>,
    mut renderer_resources: ResMut<RendererResources>,
    mut samplers_mut: SamplersMut,
    mut textures_mut: TexturesMut,
    mut buffers_mut: BuffersMut,
) {
    let device = vulkan_ctx_resource.device;
    let allocator = vulkan_ctx_resource.allocator;

    let descriptor_set_handle = renderer_resources
        .resources_descriptor_set_handle
        .as_ref()
        .unwrap();

    let descriptor_set_layouts = [descriptor_set_handle
        .descriptor_set_layout_handle
        .descriptor_set_layout];
    let push_constant_ranges = descriptor_set_handle.push_contant_ranges.as_slice();

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

    let created_shaders = create_shaders(device, &shaders_info);

    renderer_resources.gradient_compute_shader_object = created_shaders[0];
    renderer_resources.task_shader_object = created_shaders[1];
    renderer_resources.mesh_shader_object = created_shaders[2];
    renderer_resources.fragment_shader_object = created_shaders[3];

    let magenta = &pack_unorm_4x8(Vec4::new(1.0, 0.0, 1.0, 1.0));
    let black = &pack_unorm_4x8(Vec4::new(0.0, 0.0, 0.0, 0.0));
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
    let (checkerboard_texture_reference, _) = textures_mut.create_texture(
        None,
        false,
        Format::R8G8B8A8Unorm,
        checkerboard_image_extent,
        ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
        false,
    );

    renderer_resources.default_texture_reference = checkerboard_texture_reference;
    let descriptor_checkerboard_image = DescriptorKind::SampledImage(DescriptorSampledImage {
        image_view: textures_mut
            .get(checkerboard_texture_reference)
            .unwrap()
            .image_view,
        index: checkerboard_texture_reference.index,
    });
    renderer_resources
        .resources_descriptor_set_handle
        .as_mut()
        .unwrap()
        .update_binding(device, allocator, descriptor_checkerboard_image);

    vulkan_ctx_resource.transfer_data_to_image(
        textures_mut.get(checkerboard_texture_reference).unwrap(),
        &mut buffers_mut,
        pixels.as_ptr() as *const _,
        &render_context.upload_context,
        None,
    );

    let white_image_extent = Extent3D {
        width: 1,
        height: 1,
        depth: 1,
    };
    let (white_texture_reference, _) = textures_mut.create_texture(
        None,
        false,
        Format::R8G8B8A8Srgb,
        white_image_extent,
        ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
        false,
    );
    renderer_resources.fallback_texture_reference = white_texture_reference;

    let white_image_pixels = [pack_unorm_4x8(Vec4::new(1.0, 1.0, 1.0, 1.0))];
    vulkan_ctx_resource.transfer_data_to_image(
        textures_mut.get(white_texture_reference).unwrap(),
        &mut buffers_mut,
        white_image_pixels.as_ptr() as *const _,
        &render_context.upload_context,
        None,
    );

    let descriptor_white_image = DescriptorKind::SampledImage(DescriptorSampledImage {
        image_view: textures_mut
            .get(white_texture_reference)
            .unwrap()
            .image_view,
        index: white_texture_reference.index,
    });
    renderer_resources
        .resources_descriptor_set_handle
        .as_mut()
        .unwrap()
        .update_binding(device, allocator, descriptor_white_image);

    let draw_extent = render_context.draw_extent;
    render_context
        .frames_data
        .iter_mut()
        .for_each(|frame_data| {
            let draw_image_extent = Extent3D {
                width: draw_extent.width,
                height: draw_extent.height,
                depth: 1,
            };

            let (draw_texture_reference, _) = textures_mut.create_texture(
                None,
                false,
                Format::R16G16B16A16Sfloat,
                draw_image_extent,
                ImageUsageFlags::TransferSrc
                    | ImageUsageFlags::Storage
                    | ImageUsageFlags::ColorAttachment,
                false,
            );

            let (depth_texture_reference, _) = textures_mut.create_texture(
                None,
                false,
                Format::D32Sfloat,
                draw_image_extent,
                ImageUsageFlags::DepthStencilAttachment,
                false,
            );

            let descriptor_draw_image = DescriptorKind::StorageImage(DescriptorStorageImage {
                image_view: textures_mut.get(draw_texture_reference).unwrap().image_view,
                index: draw_texture_reference.index,
            });
            renderer_resources
                .resources_descriptor_set_handle
                .as_mut()
                .unwrap()
                .update_binding(device, allocator, descriptor_draw_image);

            frame_data.draw_texture_reference = draw_texture_reference;
            frame_data.depth_texture_reference = depth_texture_reference;
        });

    let materials_data_buffer_reference = buffers_mut.create(
        1024 * 1024 * 64,
        BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
        BufferVisibility::HostVisible,
        Some("Materials Data Buffer".to_string()),
    );
    let mut instance_objects_buffers = Vec::with_capacity(render_context.frame_overlap);
    for instances_objects_buffer_index in 0..instance_objects_buffers.capacity() {
        let instance_objects_buffer_reference = buffers_mut.create(
            std::mem::size_of::<InstanceObject>() * 4096,
            BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
            BufferVisibility::HostVisible,
            Some(std::format!(
                "Instances Objects Buffer {}",
                instances_objects_buffer_index
            )),
        );

        instance_objects_buffers.push(instance_objects_buffer_reference);
    }

    let mut scene_data_buffers = Vec::with_capacity(render_context.frame_overlap);
    for scene_data_buffer_index in 0..scene_data_buffers.capacity() {
        let scene_data_buffer_reference = buffers_mut.create(
            std::mem::size_of::<SceneData>(),
            BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
            BufferVisibility::HostVisible,
            Some(std::format!(
                "Scene Data Buffer {}",
                scene_data_buffer_index
            )),
        );

        scene_data_buffers.push(scene_data_buffer_reference);
    }

    let mesh_objects_buffer_reference = buffers_mut.create(
        std::mem::size_of::<MeshObject>() * 8192,
        BufferUsageFlags::ShaderDeviceAddress | BufferUsageFlags::TransferDst,
        BufferVisibility::DeviceOnly,
        Some("Mesh Objects Buffer".to_string()),
    );

    renderer_resources.resources_pool.instances_buffer =
        Some(SwappableBuffer::new(instance_objects_buffers));
    renderer_resources.resources_pool.scene_data_buffer =
        Some(SwappableBuffer::new(scene_data_buffers));

    renderer_resources.set_materials_data_buffer_reference(materials_data_buffer_reference);
    renderer_resources.mesh_objects_buffer_reference = mesh_objects_buffer_reference;
}

fn create_shaders(device: Device, shader_infos: &[ShaderInfo]) -> Vec<ShaderObject> {
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
        .map(|(shader, shader_info)| ShaderObject::new(Some(shader), shader_info.stage))
        .collect()
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
