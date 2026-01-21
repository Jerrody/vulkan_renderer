use asset_importer::{Matrix4x4, node::Node};
use std::ffi::c_void;
use uuid::Uuid;
use vulkanite::vk::{BufferUsageFlags, DeviceAddress};

use bevy_ecs::{
    observer::On,
    system::{Commands, Res, ResMut},
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use meshopt::{
    VertexDataAdapter, build_meshlets, optimize_vertex_cache_in_place, optimize_vertex_fetch,
    optimize_vertex_fetch_remap, remap_index_buffer, remap_vertex_buffer, typed_to_bytes,
};
use vma::Allocator;
use vulkanite::vk::rs::Device;

use crate::engine::{
    components::transform::Transform,
    events::{LoadModelEvent, SpawnEvent, SpawnEventRecord},
    id::Id,
    resources::{
        AllocatedBuffer, MeshBuffer, MeshObject, MeshObjectPool, Meshlet, RendererResources,
        Vertex, VulkanContextResource, allocation::create_buffer,
    },
    utils::get_device_address,
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
    mut renderer_resources: ResMut<RendererResources>,
) {
    let device = vulkan_context.device;
    let allocator = &vulkan_context.allocator;
    let model_loader = &renderer_resources.model_loader;

    let mut nodes = Vec::new();

    let scene = model_loader.load_model(&load_model_event.path);
    let root_node = scene.root_node().unwrap();

    nodes.push(NodeData::new(
        root_node.name(),
        Default::default(),
        None,
        root_node.transformation(),
        get_mesh_indices(&root_node, root_node.num_meshes()),
    ));

    let mut stack = Vec::new();
    stack.push(root_node);

    loop {
        while let Some(parent_node) = stack.pop() {
            let parent_index = nodes.len() - 1;
            for (child_index, child_node) in parent_node.children().enumerate() {
                stack.push(child_node.clone());

                nodes.push(NodeData::new(
                    child_node.name(),
                    parent_index + child_index + 1,
                    Some(parent_index),
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

    let mut mesh_index_offset = usize::default();
    for node_data in nodes.into_iter() {
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

        if node_data.mesh_indices.len() > Default::default() {
            for &mesh_index in node_data.mesh_indices.iter() {
                let mesh = scene.mesh(mesh_index);
                if let Some(mesh) = mesh {
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
                        VertexDataAdapter::new(&vertex_data, vertex_stride, position_offset)
                            .unwrap();

                    optimize_vertex_cache_in_place(&mut indices, vertices.len());
                    optimize_vertex_fetch(&mut indices, &vertices);

                    let (meshlets, vertex_indices, triangles) =
                        generate_meshlets(&indices, &vertex_data_adapter);

                    let vertex_buffer = create_buffer_and_update::<Vertex>(
                        device,
                        allocator,
                        vertices.as_ptr() as _,
                        vertices.len(),
                    );
                    let vertex_indices_buffer = create_buffer_and_update::<u32>(
                        device,
                        allocator,
                        vertex_indices.as_ptr() as _,
                        vertex_indices.len(),
                    );
                    let meshlets_buffer = create_buffer_and_update::<Meshlet>(
                        device,
                        allocator,
                        meshlets.as_ptr() as _,
                        meshlets.len(),
                    );
                    let local_indices_buffer = create_buffer_and_update::<u8>(
                        device,
                        allocator,
                        triangles.as_ptr() as _,
                        triangles.len(),
                    );

                    let vertex_buffer_id = renderer_resources.insert_storage_buffer(vertex_buffer);
                    let vertex_indices_buffer_id =
                        renderer_resources.insert_storage_buffer(vertex_indices_buffer);
                    let meshlets_buffer_id =
                        renderer_resources.insert_storage_buffer(meshlets_buffer);
                    let local_indices_buffer_id =
                        renderer_resources.insert_storage_buffer(local_indices_buffer);

                    let mesh_buffer = MeshBuffer {
                        id: Id::new(vertex_buffer_id),
                        mesh_object_device_address: Id::NULL.value(),
                        vertex_buffer_id,
                        vertex_indices_buffer_id,
                        meshlets_buffer_id,
                        local_indices_buffer_id,
                        meshlets_count: meshlets.len(),
                    };

                    let mesh_buffer_id = renderer_resources.insert_mesh_buffer(mesh_buffer);
                    renderer_resources.enqueue_mesh_buffer_to_write(mesh_buffer_id);

                    spawn_event_record.name = mesh.name();
                    spawn_event_record.parent_index = Some(node_data.index + mesh_index_offset);
                    spawn_event_record.mesh_buffer_id = mesh_buffer_id;
                    spawn_event_record.transform = transform;

                    spawn_event.spawn_records.push(spawn_event_record.clone());
                }
            }

            mesh_index_offset = node_data.mesh_indices.len();
        } else {
            mesh_index_offset = Default::default();
        }

        spawn_event_record.mesh_buffer_id = Id::NULL;
    }

    let mesh_objects_to_write = renderer_resources
        .get_mesh_buffer_to_write_iter()
        .map(|&mesh_buffer_to_write_id| {
            let mesh_buffer = renderer_resources.get_mesh_buffer_ref(mesh_buffer_to_write_id);

            let device_address_vertex_buffer: DeviceAddress = get_device_address(
                vulkan_context.device,
                &renderer_resources
                    .get_storage_buffer_ref(mesh_buffer.vertex_buffer_id)
                    .buffer,
            );
            let device_address_vertex_indices_buffer: DeviceAddress = get_device_address(
                vulkan_context.device,
                &renderer_resources
                    .get_storage_buffer_ref(mesh_buffer.vertex_indices_buffer_id)
                    .buffer,
            );
            let device_address_meshlets_buffer: DeviceAddress = get_device_address(
                vulkan_context.device,
                &renderer_resources
                    .get_storage_buffer_ref(mesh_buffer.meshlets_buffer_id)
                    .buffer,
            );
            let device_address_local_indices_buffer: DeviceAddress = get_device_address(
                vulkan_context.device,
                &renderer_resources
                    .get_storage_buffer_ref(mesh_buffer.local_indices_buffer_id)
                    .buffer,
            );

            let mesh_object = MeshObject {
                device_address_vertex_buffer,
                device_address_vertex_indices_buffer,
                device_address_meshlets_buffer,
                device_address_local_indices_buffer,
            };

            mesh_object
        })
        .collect::<Vec<_>>();

    // FIXME: Currently we use the first buffer of mesh objects,
    // but later we need to use ring buffer pattern for safe read and write operations between the frames.
    let first_mesh_objects_buffer_id =
        *renderer_resources.mesh_objects_buffers_ids.first().unwrap();
    let mesh_objects_buffer: &mut AllocatedBuffer = unsafe {
        &mut *(renderer_resources.get_storage_buffer_ref_mut(first_mesh_objects_buffer_id)
            as *mut _)
    };
    let device_addresss_mesh_objects_buffer: DeviceAddress =
        get_device_address(vulkan_context.device, &mesh_objects_buffer.buffer);

    let mesh_object_size = std::mem::size_of::<MeshObject>();
    renderer_resources
        .get_mesh_buffers_iter_mut()
        .zip(mesh_objects_to_write.iter().enumerate())
        .for_each(|(mesh_buffer, (mesh_object_index, mesh_object))| {
            let ptr_mesh_object = mesh_object as *const _ as _;

            let offset_dst = mesh_object_index * mesh_object_size;

            mesh_buffer.mesh_object_device_address =
                device_addresss_mesh_objects_buffer + offset_dst as u64;

            unsafe {
                transfer_data_to_buffer_with_offset(
                    &vulkan_context.allocator,
                    mesh_objects_buffer,
                    ptr_mesh_object,
                    mesh_object_size,
                    Default::default(),
                    offset_dst,
                );
            }
        });

    commands.trigger(spawn_event);
}

fn write_mesh_buffers_to_mesh_objects_buffer(renderer_resources: &RendererResources) {}

fn get_mesh_indices(node: &Node, num_meshes: usize) -> Vec<usize> {
    let mut mesh_indices = Vec::with_capacity(num_meshes);
    if num_meshes > Default::default() {
        for mesh_index in node.mesh_indices() {
            mesh_indices.push(mesh_index);
        }
    }

    mesh_indices
}

fn create_buffer_and_update<T>(
    device: Device,
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
        transfer_data_to_buffer(&allocator, &mut allocated_buffer, data, allocation_size);
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

pub unsafe fn transfer_data_to_buffer(
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

pub unsafe fn transfer_data_to_buffer_with_offset(
    allocator: &Allocator,
    allocated_buffer: &mut AllocatedBuffer,
    mut src: *const c_void,
    size: usize,
    offset_src: usize,
    offset_dst: usize,
) {
    unsafe {
        src = src.add(offset_src);

        let mut p_mapped_memory = allocator
            .map_memory(&mut allocated_buffer.allocation)
            .unwrap();

        p_mapped_memory = p_mapped_memory.add(offset_dst);

        std::ptr::copy_nonoverlapping(src, p_mapped_memory as _, size);

        allocator.unmap_memory(&mut allocated_buffer.allocation);
    }
}
