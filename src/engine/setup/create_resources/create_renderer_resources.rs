use std::{mem::ManuallyDrop, os::raw::c_void};

use bevy_ecs::world::World;
use glam::{Vec2, Vec3};
use meshopt::{
    VertexDataAdapter, build_meshlets, optimize_overdraw_in_place, optimize_vertex_cache,
    optimize_vertex_cache_in_place, optimize_vertex_fetch, optimize_vertex_fetch_remap,
    remap_index_buffer, remap_vertex_buffer, typed_to_bytes,
};
use vma::{Alloc, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage};
use vulkanite::vk::{rs::*, *};

use crate::engine::{
    Engine,
    descriptors::DescriptorSetLayoutBuilder,
    resources::{
        AllocatedBuffer, AllocatedDescriptorBuffer, AllocatedImage, DevicePropertiesResource,
        MeshBuffer, MeshPushConstant, Meshlet, RendererContext, RendererResources, ShaderObject,
        Vertex, VulkanContextResource, allocation::create_buffer, model_loader::ModelLoader,
    },
    utils::{
        ShaderInfo, create_image_info, create_image_view_info, get_device_address, load_shader,
    },
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

        let gradient_descriptor_layouts = [draw_image_descriptor_buffer.descriptor_set_layout];

        /* let gradient_shader_info = ShaderInfo {
                   path: r"shaders\output\gradient.slang.spv",
                   flags: ShaderCreateFlagsEXT::empty(),
                   stage: ShaderStageFlags::Compute,
                   next_stage: ShaderStageFlags::empty(),
                   descriptor_layouts: &descriptor_layouts,
               };
        */
        /*         let gradient_compute_shader_object =
        Self::create_shader(&vulkan_context.device, gradient_shader_info); */

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

        let meshes = model_loader.load_model(r"assets/basicmesh.glb");

        let mut mesh_buffers = Vec::new();
        for mesh in meshes {
            let mut indices = Vec::new();
            for face in mesh.faces() {
                for index in face.indices() {
                    indices.push(*index);
                }
            }

            let mut vertices = mesh
                .vertices_iter()
                .zip(
                    mesh.normals()
                        .unwrap()
                        .into_iter()
                        .zip(mesh.texture_coords_iter(Default::default())),
                )
                .into_iter()
                .map(|(position, (normal, uv))| {
                    let position = Vec3::new(position.x, position.y, position.z);
                    let normal = Vec3::new(normal.x, normal.y, normal.z);
                    let uv = Vec2::new(uv.x, uv.y);

                    Vertex {
                        position,
                        normal,
                        uv,
                    }
                })
                .collect::<Vec<Vertex>>();

            let remap = optimize_vertex_fetch_remap(&indices, vertices.len());
            indices = remap_index_buffer(Some(&indices), vertices.len(), &remap);
            vertices = remap_vertex_buffer(&vertices, vertices.len(), &remap);

            let position_offset = std::mem::offset_of!(Vertex, position);
            let vertex_stride = std::mem::size_of::<Vertex>();
            let vertex_data = typed_to_bytes(&vertices);

            let vertex_data_adapter =
                VertexDataAdapter::new(&vertex_data, vertex_stride, position_offset).unwrap();

            optimize_vertex_cache_in_place(&mut indices, vertices.len());
            optimize_vertex_fetch(&mut indices, &vertices);

            let (meshlets, vertex_indices, triangles) =
                Self::generate_meshlets(&indices, &vertex_data_adapter);

            let device = &vulkan_context.device;
            let allocator = &vulkan_context.allocator;

            let vertex_buffer = Self::create_buffer_and_update::<Vertex>(
                device,
                allocator,
                vertices.as_ptr() as _,
                vertices.len(),
            );
            let vertex_indices_buffer = Self::create_buffer_and_update::<u32>(
                device,
                allocator,
                vertex_indices.as_ptr() as _,
                vertex_indices.len(),
            );
            let meshlets_buffer = Self::create_buffer_and_update::<Meshlet>(
                device,
                allocator,
                meshlets.as_ptr() as _,
                meshlets.len(),
            );
            let local_indices_buffer = Self::create_buffer_and_update::<u8>(
                device,
                allocator,
                triangles.as_ptr() as _,
                triangles.len(),
            );

            let mesh_buffer = MeshBuffer {
                vertex_buffer: vertex_buffer,
                vertex_indices_buffer,
                meshlets_buffer,
                local_indices_buffer,
                meshlets_count: meshlets.len(),
            };

            mesh_buffers.push(mesh_buffer);
        }

        RendererResources {
            draw_image,
            draw_image_descriptor_buffer,
            gradient_compute_shader_object: created_shaders[0],
            mesh_shader_object: created_shaders[1],
            fragment_shader_object: created_shaders[2],
            model_loader,
            mesh_buffers,
            mesh_pipeline_layout,
        }
    }

    fn create_buffer_and_update<T>(
        device: &Device,
        allocator: &Allocator,
        data: *const c_void,
        len: usize,
    ) -> AllocatedBuffer
    where
        T: Sized,
    {
        let allocation_size = len * std::mem::size_of::<T>();
        let mut allocated_buffer = create_buffer(
            device,
            allocator,
            allocation_size,
            BufferUsageFlags::TransferDst,
        );
        unsafe {
            Self::transfer_data(&allocator, &mut allocated_buffer, data, allocation_size);
        }

        allocated_buffer
    }

    fn generate_meshlets(
        indices: &[u32],
        vertices: &VertexDataAdapter,
    ) -> (Vec<Meshlet>, Vec<u32>, Vec<u8>) {
        let max_vertices = 64;
        let max_triangles = 124;
        let cone_weight = 0.0;

        let raw_meshlets =
            build_meshlets(indices, vertices, max_vertices, max_triangles, cone_weight);

        let mut meshlets = Vec::new();

        for raw_meshlet in raw_meshlets.meshlets.iter() {
            meshlets.push(Meshlet {
                vertex_offset: raw_meshlet.vertex_offset as _,
                triangle_offset: raw_meshlet.triangle_offset as _,
                vertex_count: raw_meshlet.vertex_count as _,
                triangle_count: raw_meshlet.triangle_count as _,
            });
        }

        (meshlets, raw_meshlets.vertices, raw_meshlets.triangles)
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
            device.get_descriptor_set_layout_size_ext(descriptor_set_layout);

        let descriptor_buffer_size = Self::aligned_size(
            descriptor_set_layout_size,
            device_properties_resource
                .descriptor_buffer_properties
                .descriptor_buffer_offset_alignment,
        );

        let descriptor_buffer_offset = device.get_descriptor_set_layout_binding_offset_ext(
            descriptor_set_layout,
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
            device_address: Default::default(),
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

        let descriptor_set_layouts = [descriptor_set_layout];
        let pipeline_layout_info =
            PipelineLayoutCreateInfo::default().set_layouts(descriptor_set_layouts.as_slice());
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info)
            .unwrap();

        let allocated_descriptor_buffer_address =
            get_device_address(device, &allocated_descriptor_buffer.buffer);

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

    unsafe fn transfer_data(
        allocator: &Allocator,
        allocated_buffer: &mut AllocatedBuffer,
        src: *const c_void,
        size: usize,
    ) {
        unsafe {
            let p_mapped_memory = allocator
                .map_memory(&mut allocated_buffer.allocation)
                .unwrap();
            std::ptr::copy_nonoverlapping(src, p_mapped_memory as _, size);

            allocator.unmap_memory(&mut allocated_buffer.allocation);
        }
    }
}
