import modules.resources;

#ifdef MATERIAL_TYPE
typealias Material = MATERIAL_TYPE;
#else
typealias Material = DefaultMaterial;
#endif

struct PrimitiveData
{
    nointerpolation const let device_adddress_material : uint32_t;
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
}

struct InstanceObject
{
    const let model_matrix : float4x4;
    const let ptr_mesh_object : ImmutablePtr<MeshObject>;
    const let meshlet_count : uint32_t;
    const let device_address_material : uint32_t;
}

struct Payload
{
    const let instance_object_index : uint32_t;
}

struct SurfaceData
{
    var color : float4 = float4(1.0);
}

interface IMaterial
{
    func eval(surface_data: SurfaceData, const uv: float2)->SurfaceData;
}

struct UnlitMaterial : IMaterial
{
    const let base_color : float4;
    const let texture_index : uint32_t;
    const let sampler_index : uint32_t;

    func eval(SurfaceData surface_data, const uv: float2)->SurfaceData
    {
        const let sampler = samplers[sampler_index];
        const let texture = sampled_images[texture_index];

        var color = texture.Sample(sampler, uv);
        color *= base_color;

        surface_data.color = color;

        return surface_data;
    }
}

struct DefaultMaterial : IMaterial
{
    func eval(SurfaceData surface_data, const uv: float2)->SurfaceData
    {
        surface_data.color = float4(1.0);

        return surface_data;
    }
}

struct GlobalPushConstants
{
    const let view_projection_matrix : float4x4;
    const let ptr_instance_object : ImmutablePtr<InstanceObject>;
    const let ptr_material_data : ByteAddressBuffer;
    const let sampler_index : uint32_t;
    const let draw_image_index : uint32_t;
};

[[vk::push_constant]]
ConstantBuffer<GlobalPushConstants> push_constants;
