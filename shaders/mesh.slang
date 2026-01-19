static const let MAX_VERTICES : uint32_t = 64;
static const let MAX_TRIANGLES : uint32_t = 124;
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
    const let meshlets : Ptr<Meshlet, Access.Read>;
    const let vertices : Ptr<Vertex, Access.Read>;
    const let vertex_indices : Ptr<uint32_t, Access.Read>;
    const let local_indices : Ptr<uint8_t, Access.Read>;
}

struct InstanceObject
{
    const let model_matrix : float4x4;
    const let mesh_index : uint32_t;
    const let texture_index : uint32_t;
}

struct GraphicsPushConstants
{
    const let view_projection_matrix : float4x4;
    const let instance_object_index : uint32_t;
    const let sampler_index : uint32_t;
    const let texture_image_index : uint32_t;
};

[[vk::push_constant]]
readonly ConstantBuffer<GraphicsPushConstants> graphics_push_constants;

[[vk::binding(2, 0)]]
readonly const MeshObject mesh_objects[4096];
[[vk::binding(3, 0)]]
readonly const StructuredBuffer<InstanceObject> instance_objects;

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(GROUP_SIZE, 1, 1)]
void main(
    const uint group_id: SV_GroupID,
    const uint group_index: SV_GroupIndex,
    OutputVertices<VertexOutput, MAX_VERTICES> outVerts,
    OutputIndices<uint3, MAX_TRIANGLES> outIndices)
{
    let instance_object = instance_objects[graphics_push_constants.instance_object_index];
    let mesh_object = mesh_objects[instance_object.mesh_index];

    let meshlet = mesh_object.meshlets[group_id];

    SetMeshOutputCounts(meshlet.vertex_count, meshlet.triangle_count);

    if (group_index < meshlet.vertex_count)
    {
        let vertexLookupIndex = meshlet.vertex_offset + group_index;
        let globalVertexId = mesh_object.vertex_indices[vertexLookupIndex];

        let v = mesh_object.vertices[globalVertexId];
        outVerts[group_index] = process_vertex(v, graphics_push_constants.view_projection_matrix);
    }

    if (group_index < meshlet.triangle_count)
    {
        let triangleLookupIndex = meshlet.triangle_offset + (group_index * 3);
        let i0 = mesh_object.local_indices[triangleLookupIndex + 0].x;
        let i1 = mesh_object.local_indices[triangleLookupIndex + 1].x;
        let i2 = mesh_object.local_indices[triangleLookupIndex + 2].x;

        outIndices[group_index] = uint32_t3(i0, i1, i2);
    }
}

[ForceInline]
func process_vertex(const v: Vertex, const mat: float4x4)->VertexOutput
{
    let position = mul(mat, float4(v.position, 1.0));
    let vertex_output : VertexOutput = VertexOutput(position, v.normal, v.uv);

    return vertex_output;
}

///////////////////////////////////////////////////// FRAGMENT //////////////////////////////////////////////////////////////

[[vk::binding(0, 0)]]
readonly const SamplerState samplers[16];
[[vk::binding(4, 0)]]
readonly const Texture2D<float4> images[4096];

[shader("fragment")]
func main(const vertex_output: VertexOutput)->float4
{
    let sampler = samplers[graphics_push_constants.sampler_index];
    let image = images[graphics_push_constants.texture_image_index];

    let color = image.Sample(sampler, vertex_output.uv);

    return color;
}
