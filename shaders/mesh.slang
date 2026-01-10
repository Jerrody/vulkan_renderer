static const uint MAX_VERTICES = 64;
static const uint MAX_TRIANGLES = 124;
static const uint GROUP_SIZE = MAX_VERTICES;

struct Vertex
{
    float3 position;
    float3 normal;
    float2 uv;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float3 color : COLOR0;
    float2 uv : TEXCOORD0;
};

struct Meshlet
{
    uint32_t vertex_offset;
    uint32_t triangle_offset;
    uint32_t vertex_count;
    uint32_t triangle_count;
}

struct PushConstants
{
    float4x4 render_matrix;
    Meshlet *meshlets;
    Vertex *vertices;
    uint32_t *vertex_indices;
    uint8_t *local_indices;
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

        Vertex v = pushConsts.vertices[globalVertexId];
        outVerts[group_index] = process_vertex(v, pushConsts.render_matrix);
    }

    if (group_index < meshlet.triangle_count)
    {
        uint triangleLookupIndex = meshlet.triangle_offset + (group_index * 3);
        uint8_t i0 = pushConsts.local_indices[triangleLookupIndex + 0];
        uint8_t i1 = pushConsts.local_indices[triangleLookupIndex + 1];
        uint8_t i2 = pushConsts.local_indices[triangleLookupIndex + 2];

        outIndices[group_index] = uint3(i0, i1, i2);
    }
}

[ForceInline]
func process_vertex(v: Vertex, mat: float4x4)->VertexOutput
{
    VertexOutput vertex_output;
    vertex_output.position = mul(mat, float4(v.position, 1.0));
    vertex_output.color = float3(v.position);
    vertex_output.uv = v.uv;

    return vertex_output;
}

[shader("fragment")]
func main(VertexOutput vertex_output)->float4
{
    let color = float4(vertex_output.color, 1.0);

    return color;
}
