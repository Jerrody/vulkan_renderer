use std::ffi::c_void;
use vulkanite::vk::BufferUsageFlags;

use bevy_ecs::{
    observer::On,
    system::{Res, ResMut},
};
use glam::{Vec2, Vec3};
use meshopt::{
    VertexDataAdapter, build_meshlets, optimize_vertex_cache_in_place, optimize_vertex_fetch,
    optimize_vertex_fetch_remap, remap_index_buffer, remap_vertex_buffer, typed_to_bytes,
};
use vma::Allocator;
use vulkanite::vk::rs::Device;

use crate::engine::{
    events::LoadModelEvent,
    id::Id,
    resources::{
        AllocatedBuffer, MeshBuffer, Meshlet, RendererResources, Vertex, VulkanContextResource,
        allocation::create_buffer,
    },
};

pub fn on_load_model(
    load_model_event: On<LoadModelEvent>,
    vulkan_context: Res<VulkanContextResource>,
    mut renderer_resources: ResMut<RendererResources>,
) {
    let device = vulkan_context.device;
    let allocator = &vulkan_context.allocator;
    let model_loader = &renderer_resources.model_loader;

    let meshes = model_loader.load_model(&load_model_event.path);

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

        let mesh_buffer = MeshBuffer {
            id: Id::new(mesh.name()),
            vertex_buffer: vertex_buffer,
            vertex_indices_buffer,
            meshlets_buffer,
            local_indices_buffer,
            meshlets_count: meshlets.len(),
        };

        mesh_buffers.push(mesh_buffer);
    }

    renderer_resources
        .mesh_buffers
        .extend(mesh_buffers.into_iter());
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
        transfer_data(&allocator, &mut allocated_buffer, data, allocation_size);
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
