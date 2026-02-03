import modules.resources;

#ifdef MATERIAL_TYPE
typealias Material = MATERIAL_TYPE;
#else
typealias Material = DefaultMaterial;
#endif

struct PrimitiveData
{
    nointerpolation const let device_address_material : ImmutablePtr<Material>;
}

enum MaterialType : uint8_t
{
    Opaque,
    Transparent,
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
    const float2 uv : TEXCOORD0;
    const let normal : float3;
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
    const let device_address_material : ImmutablePtr<Material>;
    const let meshlet_count : uint32_t;
    const let material_type : MaterialType;
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

    [ForceInline]
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

struct SceneData
{
    const let camera_view_matrix : float4x4;
    const let light_color : float3;
    const let ambient_strength : float32_t;
    const let light_position : float3;
    const let specular_strength : float32_t;
}

struct GlobalPushConstants
{
    const let ptr_scene_data : ImmutablePtr<SceneData>;
    const let ptr_instance_object : ImmutablePtr<InstanceObject>;
    const let draw_image_index : uint32_t;
    const let current_material_type : MaterialType;
};

[[vk::push_constant]]
ConstantBuffer<GlobalPushConstants> push_constants;
