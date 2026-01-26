struct PrimitiveData
{
    nointerpolation const let texture_index : uint32_t;
    nointerpolation const let base_color : float4;
}

struct Vertex
{
    const let position : float3;
    const let normal : float3;
    const let uv : float2;
};

struct VertexOutput
{
    const float4 position : SV_Position;
    const float3 color : COLOR0;
    const float2 uv : TEXCOORD0;
};

struct Meshlet
{
    const let vertex_offset : uint32_t;
    const let triangle_offset : uint32_t;
    const let vertex_count : uint32_t;
    const let triangle_count : uint32_t;
}

struct MeshObject
{
    const let vertices : ImmutablePtr<Vertex>;
    const let vertex_indices : ImmutablePtr<uint32_t>;
    const let meshlets : ImmutablePtr<Meshlet>;
    const let local_indices : ImmutablePtr<uint8_t>;
    const let base_color : float4; // TODO: Should be part of material data.
}

struct InstanceObject
{
    const let model_matrix : float4x4;
    const let ptr_mesh_object : ImmutablePtr<MeshObject>;
    const let meshlet_count : uint32_t;
    const let texture_index : uint32_t;
}

struct Payload
{
    const let instance_object_index : uint32_t;
}

struct GlobalPushConstants
{
    const let view_projection_matrix : float4x4;
    const let ptr_instance_object : ImmutablePtr<InstanceObject>;
    const let sampler_index : uint32_t;
    const let draw_image_index : uint32_t;
};
