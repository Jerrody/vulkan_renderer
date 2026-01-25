static const let MAX_VERTICES : uint32_t = 64;
static const let MAX_TRIANGLES : uint32_t = 64;
static const let GROUP_SIZE : uint32_t = MAX_VERTICES;

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
    nointerpolation const let texture_index : uint32_t;
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
    const let texture_index : uint32_t;
}

struct GraphicsPushConstants
{
    const let view_projection_matrix : float4x4;
    const let ptr_instance_object : ImmutablePtr<InstanceObject>;
    const let sampler_index : uint32_t;
    const let texture_image_index : uint32_t;
};

[[vk::push_constant]]
ConstantBuffer<GraphicsPushConstants> graphics_push_constants;

struct Payload
{
    const let instance_object_index : uint32_t;
}

groupshared Payload payload;

///////////////////////////////////////////////////// TASK //////////////////////////////////////////////////////////////

[shader("amplification")]
[numthreads(1, 1, 1)]
void main(const uint32_t group_id: SV_GroupID)
{
    const let instance_object = graphics_push_constants.ptr_instance_object[group_id];

    payload = Payload(group_id);

    DispatchMesh(instance_object.meshlet_count, 1, 1, payload);
}

///////////////////////////////////////////////////// MESH //////////////////////////////////////////////////////////////

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(GROUP_SIZE, 1, 1)]
void main(
    const uint group_id: SV_GroupID,
    const uint group_index: SV_GroupIndex,
    OutputVertices<VertexOutput, MAX_VERTICES> outVerts,
    OutputIndices<uint3, MAX_TRIANGLES> outIndices)
{
    const let instance_object = graphics_push_constants.ptr_instance_object[payload.instance_object_index];
    let ptr_mesh_object = instance_object.ptr_mesh_object;
    let meshlet = ptr_mesh_object.meshlets[group_id];

    SetMeshOutputCounts(meshlet.vertex_count, meshlet.triangle_count);

    if (group_index < meshlet.vertex_count)
    {
        let vertexLookupIndex = meshlet.vertex_offset + group_index;
        let globalVertexId = ptr_mesh_object.vertex_indices[vertexLookupIndex];

        let v = ptr_mesh_object.vertices[globalVertexId];
        outVerts[group_index] = process_vertex(v,
                                               instance_object.model_matrix,
                                               graphics_push_constants.view_projection_matrix,
                                               instance_object.texture_index);
    }

    if (group_index < meshlet.triangle_count)
    {
        let triangleLookupIndex = meshlet.triangle_offset + (group_index * 3);
        let i0 = ptr_mesh_object.local_indices[triangleLookupIndex + 0];
        let i1 = ptr_mesh_object.local_indices[triangleLookupIndex + 1];
        let i2 = ptr_mesh_object.local_indices[triangleLookupIndex + 2];

        outIndices[group_index] = uint32_t3(i0, i1, i2);
    }
}

[ForceInline]
func process_vertex(const v: Vertex, const model_matrix: float4x4, const view_projection_matrix: float4x4, texture_index: uint32_t)->VertexOutput
{
    let position = float4(v.position, 1.0);
    let world_position = mul(model_matrix, position);
    let clip_position = mul(view_projection_matrix, world_position);

    let vertex_output : VertexOutput = VertexOutput(clip_position, v.normal, v.uv, texture_index);

    return vertex_output;
}

///////////////////////////////////////////////////// FRAGMENT //////////////////////////////////////////////////////////////

[[vk::binding(0, 0)]]
const SamplerState samplers[16];
[[vk::binding(2, 0)]]
const Texture2D<float4> images[];

[shader("fragment")]
func main(const vertex_output: VertexOutput)->float4
{
    let sampler = samplers[graphics_push_constants.sampler_index];
    let image = images[vertex_output.texture_index];

    let color = image.Sample(sampler, vertex_output.uv);
    // let color = float4(vertex_output.color, 1.0);

    return color;
}
