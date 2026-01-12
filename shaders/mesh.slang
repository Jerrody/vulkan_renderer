static const let MAX_VERTICES : uint32_t = 64;
static const let MAX_TRIANGLES : uint32_t = 124;
static const let GROUP_SIZE : uint32_t = MAX_VERTICES;

struct Vertex
{
    let position : float3;
    let normal : float3;
    let uv : float2;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float3 color : COLOR0;
    float2 uv : TEXCOORD0;
};

struct Meshlet
{
    let vertex_offset : uint32_t;
    let triangle_offset : uint32_t;
    let vertex_count : uint32_t;
    let triangle_count : uint32_t;
}

struct PushConstants
{
    let render_matrix : float4x4;
    let meshlets : Meshlet *;
    let vertices : Vertex *;
    let vertex_indices : uint32_t *;
    let local_indices : uint8_t *;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConsts;

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(GROUP_SIZE, 1, 1)]
void main(
    uint group_id: SV_GroupID,
    uint group_index: SV_GroupIndex,
    OutputVertices<VertexOutput, MAX_VERTICES> outVerts,
    OutputIndices<uint3, MAX_TRIANGLES> outIndices)
{
    let meshlet = pushConsts.meshlets[group_id];

    SetMeshOutputCounts(meshlet.vertex_count, meshlet.triangle_count);

    if (group_index < meshlet.vertex_count)
    {
        uint vertexLookupIndex = meshlet.vertex_offset + group_index;
        uint globalVertexId = pushConsts.vertex_indices[vertexLookupIndex];

        let v = pushConsts.vertices[globalVertexId];
        outVerts[group_index] = process_vertex(v, pushConsts.render_matrix);
    }

    if (group_index < meshlet.triangle_count)
    {
        uint triangleLookupIndex = meshlet.triangle_offset + (group_index * 3);
        let i0 = pushConsts.local_indices[triangleLookupIndex + 0];
        let i1 = pushConsts.local_indices[triangleLookupIndex + 1];
        let i2 = pushConsts.local_indices[triangleLookupIndex + 2];

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

SamplerState nearest_sampler;
Texture2D<float4> image;

[shader("fragment")]
func main(VertexOutput vertex_output)->float4
{
    let color = image.Sample(nearest_sampler, vertex_output.uv);

    return color;
}
