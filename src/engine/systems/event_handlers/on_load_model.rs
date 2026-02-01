use asset_importer::{Matrix4x4, node::Node};
use image::ImageReader;
use ktx2_rw::{BasisCompressionParams, Ktx2Texture, VkFormat};
use std::{collections::HashMap, ffi::c_void, io::Cursor};
use vulkanite::vk::{
    BufferCopy, BufferUsageFlags, DeviceAddress, Extent3D, Format, ImageUsageFlags,
};

use bevy_ecs::{
    observer::On,
    system::{Commands, Res, ResMut},
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use meshopt::{
    VertexDataAdapter, build_meshlets, optimize_vertex_cache_in_place, optimize_vertex_fetch,
    optimize_vertex_fetch_remap, remap_index_buffer, remap_vertex_buffer, typed_to_bytes,
};

use crate::engine::{
    Engine,
    components::{
        material::{MaterialData, MaterialState, MaterialType},
        transform::Transform,
    },
    descriptors::{DescriptorKind, DescriptorSampledImage},
    events::{LoadModelEvent, SpawnEvent, SpawnEventRecord},
    id::Id,
    resources::{
        BufferReference, MemoryBucket, MeshBuffer, MeshObject, Meshlet, RendererContext,
        RendererResources, Vertex, VulkanContextResource,
    },
};

struct NodeData {
    pub name: String,
    pub index: usize,
    pub parent_index: Option<usize>,
    pub matrix: Mat4,
    pub mesh_indices: Vec<usize>,
}

impl NodeData {
    pub fn new(
        name: String,
        index: usize,
        parent_index: Option<usize>,
        transformation: Matrix4x4,
        mesh_indices: Vec<usize>,
    ) -> Self {
        let matrix = Self::get_matrix(transformation);

        Self {
            name,
            index,
            parent_index,
            matrix,
            mesh_indices,
        }
    }

    pub fn get_matrix(transformation: Matrix4x4) -> Mat4 {
        let mut matrix = Mat4::default();
        matrix.x_axis = Vec4::new(
            transformation.x_axis.x,
            transformation.x_axis.y,
            transformation.x_axis.z,
            transformation.x_axis.w,
        );
        matrix.y_axis = Vec4::new(
            transformation.y_axis.x,
            transformation.y_axis.y,
            transformation.y_axis.z,
            transformation.y_axis.w,
        );
        matrix.z_axis = Vec4::new(
            transformation.z_axis.x,
            transformation.z_axis.y,
            transformation.z_axis.z,
            transformation.z_axis.w,
        );
        matrix.w_axis = Vec4::new(
            transformation.w_axis.x,
            transformation.w_axis.y,
            transformation.w_axis.z,
            transformation.w_axis.w,
        );

        matrix
    }
}

pub fn on_load_model(
    load_model_event: On<LoadModelEvent>,
    mut commands: Commands,
    vulkan_context: Res<VulkanContextResource>,
    renderer_context_resource: Res<RendererContext>,
    mut renderer_resources: ResMut<RendererResources>,
) {
    let model_loader = &renderer_resources.model_loader;

    let mut nodes = Vec::new();

    let scene = model_loader.load_model(&load_model_event.path.as_os_str().to_str().unwrap());

    let root_node_index = Default::default();
    let root_node = scene.root_node().unwrap();

    nodes.push(NodeData::new(
        root_node.name(),
        root_node_index,
        None,
        root_node.transformation(),
        get_mesh_indices(&root_node, root_node.num_meshes()),
    ));

    let mut stack: Vec<(Node, usize)> = Vec::new();
    stack.push((root_node, root_node_index));

    loop {
        while let Some((parent_node, parent_index_in_array)) = stack.pop() {
            for child_index in (0..parent_node.num_children()).rev() {
                let child_node = parent_node.child(child_index).unwrap();

                let child_index_in_array = nodes.len();
                stack.push((child_node.clone(), child_index_in_array));

                nodes.push(NodeData::new(
                    child_node.name(),
                    child_index_in_array,
                    Some(parent_index_in_array),
                    child_node.transformation(),
                    get_mesh_indices(&child_node, child_node.num_meshes()),
                ));
            }
        }

        if stack.len() == Default::default() {
            break;
        }
    }

    let mut spawn_event = SpawnEvent::default();
    let mut spawn_event_record = SpawnEventRecord::default();

    nodes.iter().for_each(|node_data| {
        let local_matrix = node_data.matrix;

        let (local_scale, rotation, position) = local_matrix.to_scale_rotation_translation();
        let transform = Transform {
            position: position,
            rotation,
            local_scale,
        };
        spawn_event_record.name = node_data.name.clone();
        spawn_event_record.parent_index = node_data.parent_index;
        spawn_event_record.transform = transform;

        spawn_event.spawn_records.push(spawn_event_record.clone());
    });

    let mut uploaded_mesh_buffers: HashMap<usize, (asset_importer::mesh::Mesh, Id)> =
        HashMap::with_capacity(scene.num_meshes());
    let mut uploaded_textures: HashMap<usize, Id> =
        HashMap::with_capacity(uploaded_mesh_buffers.capacity());
    let mut uploaded_materials: HashMap<usize, Id> = HashMap::with_capacity(scene.num_materials());

    renderer_resources.reset_materails_to_write();
    std::fs::create_dir_all("shaders/_outputs").unwrap();
    for node_data in nodes.into_iter() {
        if node_data.mesh_indices.len() > Default::default() {
            let mut mesh_name: String;
            let mut mesh_buffer_id: Id;
            let mut texture_id: Id;
            for &mesh_index in node_data.mesh_indices.iter() {
                texture_id = renderer_resources.fallback_texture_id;
                let mesh = scene.mesh(mesh_index).unwrap();

                let material_index = mesh.material_index();
                let material_id: Id;
                if uploaded_materials.contains_key(&material_index) {
                    material_id = *uploaded_materials.get(&material_index).unwrap();
                } else {
                    let material = scene.material(material_index).unwrap();

                    let alpha_mode = std::str::from_utf8(
                        &material
                            .get_property_raw_ref(c"$mat.gltf.alphaMode", None, 0)
                            .unwrap(),
                    )
                    .unwrap();
                    let mut material_type = MaterialType::Opaque;
                    if alpha_mode.contains("BLEND") {
                        material_type = MaterialType::Transparent;
                    }

                    try_upload_texture(
                        &vulkan_context,
                        &renderer_context_resource,
                        &mut renderer_resources,
                        &scene,
                        &mut uploaded_textures,
                        material.clone(),
                        &mut texture_id,
                        load_model_event.path.file_stem().unwrap().to_str().unwrap(),
                    );

                    let base_color_raw = material.base_color().unwrap();
                    let base_color = Vec4::new(
                        base_color_raw.x,
                        base_color_raw.y,
                        base_color_raw.z,
                        base_color_raw.w,
                    );
                    let texture_index = renderer_resources.get_texture_ref(texture_id).index;
                    let material_data = MaterialData {
                        color: base_color.to_array(),
                        texture_index: texture_index as _,
                        sampler_index: Default::default(),
                    };

                    material_id = renderer_resources.write_material(
                        bytemuck::bytes_of(&material_data),
                        MaterialState { material_type },
                    );
                }

                if uploaded_mesh_buffers.contains_key(&mesh_index) {
                    let already_uploaded_mesh = uploaded_mesh_buffers.get(&mesh_index).unwrap();
                    mesh_name = already_uploaded_mesh.0.name();
                    mesh_buffer_id = already_uploaded_mesh.1;
                } else {
                    let mesh = scene.mesh(mesh_index).unwrap();
                    mesh_name = mesh.name();

                    let mut indices = Vec::with_capacity(mesh.faces().len() * 3);

                    for face in mesh.faces() {
                        for index in face.indices() {
                            indices.push(*index);
                        }
                    }

                    let positions: Vec<Vec3> = mesh
                        .vertices_iter()
                        .map(|v| Vec3::new(v.x, v.y, v.z))
                        .collect();
                    let normals: Vec<Vec3> = mesh
                        .normals()
                        .map(|ns| ns.iter().map(|n| Vec3::new(n.x, n.y, n.z)).collect())
                        .unwrap_or_else(|| vec![Vec3::ZERO; positions.len()]);

                    let uvs: Vec<Vec2> = if mesh.has_texture_coords(0) {
                        mesh.texture_coords_iter(0)
                            .map(|uv| Vec2::new(uv.x, uv.y))
                            .collect()
                    } else {
                        vec![Vec2::ZERO; positions.len()]
                    };

                    let mut vertices = Vec::with_capacity(positions.len());
                    for i in 0..positions.len() {
                        vertices.push(Vertex {
                            position: positions[i].to_array(),
                            normal: normals[i].to_array(),
                            uv: uvs[i].to_array(),
                        });
                    }

                    let remap = optimize_vertex_fetch_remap(&indices, vertices.len());
                    indices = remap_index_buffer(Some(&indices), vertices.len(), &remap);
                    vertices = remap_vertex_buffer(&vertices, vertices.len(), &remap);

                    let position_offset = std::mem::offset_of!(Vertex, position);
                    let vertex_stride = std::mem::size_of::<Vertex>();
                    let vertex_data = typed_to_bytes(&vertices);

                    let vertex_data_adapter =
                        VertexDataAdapter::new(&vertex_data, vertex_stride, position_offset)
                            .unwrap();

                    optimize_vertex_cache_in_place(&mut indices, vertices.len());
                    let vertices = optimize_vertex_fetch(&mut indices, &vertices);

                    let (meshlets, vertex_indices, triangles) =
                        generate_meshlets(&indices, &vertex_data_adapter);

                    let memory_bucket = &mut renderer_resources.resources_pool.memory_bucket;
                    let vertex_bufer_reference = create_and_copy_to_buffer(
                        memory_bucket,
                        vertices.as_ptr() as *const _,
                        vertices.len() * std::mem::size_of::<Vertex>(),
                    );
                    let vertex_indices_buffer_reference = create_and_copy_to_buffer(
                        memory_bucket,
                        vertex_indices.as_ptr() as _,
                        vertex_indices.len() * std::mem::size_of::<u32>(),
                    );
                    let meshlets_buffer_reference = create_and_copy_to_buffer(
                        memory_bucket,
                        meshlets.as_ptr() as _,
                        meshlets.len() * std::mem::size_of::<Meshlet>(),
                    );
                    let local_indices_buffer_reference = create_and_copy_to_buffer(
                        memory_bucket,
                        triangles.as_ptr() as _,
                        triangles.len() * std::mem::size_of::<u8>(),
                    );

                    let mesh_buffer = MeshBuffer {
                        id: Id::new(vertex_bufer_reference.get_buffer_info().device_address),
                        mesh_object_device_address: Default::default(),
                        vertex_buffer: vertex_bufer_reference,
                        vertex_indices_buffer: vertex_indices_buffer_reference,
                        meshlets_buffer: meshlets_buffer_reference,
                        local_indices_buffer: local_indices_buffer_reference,
                        meshlets_count: meshlets.len(),
                    };

                    mesh_buffer_id = renderer_resources.insert_mesh_buffer(mesh_buffer);

                    uploaded_mesh_buffers.insert(mesh_index, (mesh, mesh_buffer_id));
                }

                spawn_event_record.name = mesh_name;
                spawn_event_record.parent_index = Some(node_data.index);
                spawn_event_record.material_id = material_id;
                spawn_event_record.mesh_buffer_id = mesh_buffer_id;
                spawn_event_record.transform = Transform::IDENTITY;

                spawn_event.spawn_records.push(spawn_event_record.clone());
            }
        }
    }

    let mesh_objects_to_write = uploaded_mesh_buffers
        .iter()
        .map(|(_, (_, mesh_buffer_id))| {
            let mesh_buffer = renderer_resources.get_mesh_buffer_ref(*mesh_buffer_id);

            let device_address_vertex_buffer: DeviceAddress =
                mesh_buffer.vertex_buffer.get_buffer_info().device_address;
            let device_address_vertex_indices_buffer: DeviceAddress = mesh_buffer
                .vertex_indices_buffer
                .get_buffer_info()
                .device_address;
            let device_address_meshlets_buffer: DeviceAddress =
                mesh_buffer.meshlets_buffer.get_buffer_info().device_address;
            let device_address_local_indices_buffer: DeviceAddress = mesh_buffer
                .local_indices_buffer
                .get_buffer_info()
                .device_address;

            let mesh_object = MeshObject {
                device_address_vertex_buffer,
                device_address_vertex_indices_buffer,
                device_address_meshlets_buffer,
                device_address_local_indices_buffer,
            };

            mesh_object
        })
        .collect::<Vec<_>>();

    let mesh_object_size = std::mem::size_of::<MeshObject>();
    let mesh_objects_device_address = renderer_resources
        .mesh_objects_buffer_reference
        .get_buffer_info()
        .device_address;
    let mesh_objects_buffer_reference =
        unsafe { &*(&renderer_resources.mesh_objects_buffer_reference as *const _) };
    let memory_bucket: &MemoryBucket =
        unsafe { &*(&renderer_resources.resources_pool.memory_bucket as *const _) };
    let mesh_objects_to_copy_regions = renderer_resources
        .get_mesh_buffers_iter_mut()
        .enumerate()
        .map(|(mesh_object_index, mesh_buffer)| {
            let src_offset = mesh_object_index * mesh_object_size;
            let dst_offset = mesh_object_index * mesh_object_size;

            mesh_buffer.mesh_object_device_address =
                mesh_objects_device_address + dst_offset as u64;

            let region = BufferCopy {
                src_offset: Default::default(),
                dst_offset: dst_offset as _,
                size: mesh_object_size as _,
            };

            let regions = [region];
            unsafe {
                memory_bucket.transfer_data_to_buffer_with_offset(
                    mesh_objects_buffer_reference,
                    mesh_objects_to_write.as_ptr() as *const _,
                    &regions,
                );
            }

            let region = BufferCopy {
                src_offset: src_offset as _,
                dst_offset: dst_offset as _,
                size: mesh_object_size as _,
            };

            region
        })
        .collect::<Vec<BufferCopy>>();

    /*     unsafe {
        renderer_resources
            .resources_pool
            .memory_bucket
            .transfer_data_to_buffer_with_offset(
                &renderer_resources.mesh_objects_buffer_reference,
                mesh_objects_to_write.as_ptr() as *const _,
                &mesh_objects_to_copy_regions,
            );
    } */

    let materials_data_buffer_reference = renderer_resources.get_materials_data_buffer_reference();
    let materials_data_to_write_slice = renderer_resources.get_materials_data_to_write();
    let ptr_materials_data_to_write = materials_data_to_write_slice.as_ptr();
    let materials_data_to_write_len = materials_data_to_write_slice.len();

    unsafe {
        renderer_resources
            .resources_pool
            .memory_bucket
            .transfer_data_to_buffer_raw(
                &materials_data_buffer_reference,
                ptr_materials_data_to_write as *const _,
                materials_data_to_write_len,
            );

        renderer_resources.set_materials_labels_device_addresses(
            materials_data_buffer_reference
                .get_buffer_info()
                .device_address,
        );
    }

    commands.trigger(spawn_event);
}

pub fn create_and_copy_to_buffer(
    memory_bucket: &mut MemoryBucket,
    src: *const c_void,
    size: usize,
) -> BufferReference {
    let buffer_reference = memory_bucket.create_buffer(
        size,
        BufferUsageFlags::TransferDst,
        crate::engine::resources::BufferVisibility::DeviceOnly,
    );

    unsafe {
        memory_bucket.transfer_data_to_buffer_raw(&buffer_reference, src, size);
    }

    buffer_reference
}

fn try_upload_texture(
    vulkan_context: &VulkanContextResource,
    renderer_context: &RendererContext,
    renderer_resources: &mut RendererResources,
    scene: &asset_importer::Scene,
    uploaded_textures: &mut HashMap<usize, Id>,
    material: asset_importer::Material,
    texture_id: &mut Id,
    model_name: &str,
) {
    if material.texture_count(asset_importer::TextureType::BaseColor) > Default::default() {
        let texture_info = material
            .texture(asset_importer::TextureType::BaseColor, Default::default())
            .unwrap();
        let texture_index = texture_info.path[1..].parse::<usize>().unwrap();

        if uploaded_textures.contains_key(&texture_index) {
            *texture_id = *uploaded_textures.get(&texture_index).unwrap();
        } else {
            let texture = scene.texture(texture_index).unwrap();

            let (texture_data, image_extent) =
                try_to_load_cached_texture(model_name, texture.clone());

            let allocated_texture = Engine::allocate_image(
                vulkan_context.device,
                &vulkan_context.allocator,
                Format::Bc1RgbSrgbBlock,
                image_extent,
                ImageUsageFlags::Sampled | ImageUsageFlags::TransferDst,
            );
            vulkan_context.transfer_data_to_image(
                &allocated_texture,
                texture_data.as_ptr() as *const _,
                &mut renderer_resources.resources_pool.memory_bucket,
                &renderer_context.upload_context,
                Some(texture_data.len()),
            );

            *texture_id = renderer_resources.insert_texture(allocated_texture);

            let texture_ref = renderer_resources.get_texture_ref(*texture_id);
            let descriptor_texture = DescriptorKind::SampledImage(DescriptorSampledImage {
                image_view: texture_ref.image_view,
            });
            let texture_resource_index = renderer_resources
                .resources_descriptor_set_handle
                .update_binding(
                    vulkan_context.device,
                    &vulkan_context.allocator,
                    descriptor_texture,
                );
            renderer_resources.get_texture_ref_mut(*texture_id).index =
                texture_resource_index.unwrap();
            println!(
                "Name: {} | Index: {} | Extent: {}x{}x{}",
                texture.filename_str().unwrap(),
                texture_resource_index.unwrap(),
                image_extent.width,
                image_extent.height,
                image_extent.depth,
            );

            uploaded_textures.insert(texture_index, *texture_id);
        }
    }
}

fn try_to_load_cached_texture(
    model_name: &str,
    texture: asset_importer::Texture,
) -> (Vec<u8>, Extent3D) {
    let mut path = std::path::PathBuf::from("intermediate/textures/");
    path.push(model_name);
    std::fs::create_dir_all(&path).unwrap();

    path.push(texture.filename().unwrap());
    let does_exist = std::fs::exists(&path).unwrap();

    let image_extent: Extent3D;
    let mut texture_data: Vec<u8> = Vec::new();
    if does_exist {
        let texture = Ktx2Texture::from_file(&path).unwrap();
        let image_extent_raw = texture.get_metadata("image_extent").unwrap();
        image_extent = unsafe { std::ptr::read(image_extent_raw.as_ptr() as *const Extent3D) };

        texture_data.extend_from_slice(texture.get_image_data(0, 0, 0).unwrap());
    } else {
        let data = texture.data_bytes_ref().unwrap();
        let image = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        let image_rgba = image.to_rgba8();
        let image_bytes = image_rgba.as_raw();
        let image_bytes = (*image_bytes).as_slice();

        image_extent = Extent3D {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };

        let mut texture = Ktx2Texture::create(
            image_extent.width,
            image_extent.height,
            1,
            1,
            1,
            1,
            VkFormat::R8G8B8A8Srgb,
        )
        .unwrap();
        texture.set_image_data(0, 0, 0, image_bytes).unwrap();

        texture
            .compress_basis(
                &BasisCompressionParams::builder()
                    .uastc(false)
                    .compression_level(0)
                    .quality_level(255)
                    .thread_count(8)
                    .build(),
            )
            .unwrap();
        texture
            .transcode_basis(ktx2_rw::TranscodeFormat::Bc1Rgb)
            .unwrap();
        let texture_data_ref = texture.get_image_data(0, 0, 0).unwrap();
        texture_data.extend_from_slice(texture_data_ref);

        // TODO
        texture
            .set_metadata("image_extent", unsafe {
                std::slice::from_raw_parts(
                    (&image_extent as *const Extent3D) as *const u8,
                    std::mem::size_of::<Extent3D>(),
                )
            })
            .unwrap();
        texture.write_to_file(path).unwrap();
    }

    (texture_data, image_extent)
}

fn get_mesh_indices(node: &Node, num_meshes: usize) -> Vec<usize> {
    let mut mesh_indices = Vec::with_capacity(num_meshes);
    if num_meshes > Default::default() {
        for mesh_index in node.mesh_indices() {
            mesh_indices.push(mesh_index);
        }
    }

    mesh_indices
}

fn generate_meshlets(
    indices: &[u32],
    vertices: &VertexDataAdapter,
) -> (Vec<Meshlet>, Vec<u32>, Vec<u8>) {
    let max_vertices = 64;
    let max_triangles = 64;
    let cone_weight = 0.0;

    let raw_meshlets = build_meshlets(indices, vertices, max_vertices, max_triangles, cone_weight);

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
