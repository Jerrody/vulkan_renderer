static const uint MAX_TRIS = 64;
static const uint GROUP_SIZE = MAX_TRIS;
static const uint MAX_VERTS_OUT = GROUP_SIZE * 3;

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

struct PushConstants
{
    float4x4 render_matrix;
    Vertex *vertexBuffer;
    uint32_t *indexBuffer;
    uint32_t triangle_count;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConsts;

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(GROUP_SIZE, 1, 1)]
void main(
    uint group_thread_id: SV_GroupThreadID,
    uint group_id: SV_GroupID,
    OutputVertices<VertexOutput, MAX_VERTS_OUT> outVerts,
    OutputIndices<uint3, MAX_TRIS> outIndices)
{
    uint triangle_index = (group_id * GROUP_SIZE) + group_thread_id;

    if (triangle_index >= pushConsts.triangle_count)
    {
        return;
    }

    if (group_thread_id == 0)
    {
        uint remaning_triangles = pushConsts.triangle_count - (group_id * GROUP_SIZE);
        uint count = min(GROUP_SIZE, remaning_triangles);

        SetMeshOutputCounts(count * 3, count);
    }

    uint index = triangle_index * 3;
    uint i0 = pushConsts.indexBuffer[index + 0];
    uint i1 = pushConsts.indexBuffer[index + 1];
    uint i2 = pushConsts.indexBuffer[index + 2];

    Vertex v0 = pushConsts.vertexBuffer[i0];
    Vertex v1 = pushConsts.vertexBuffer[i1];
    Vertex v2 = pushConsts.vertexBuffer[i2];

    uint out_base = group_thread_id * 3;

    outVerts[out_base + 0] = process_vertex(v0, pushConsts.render_matrix);
    outVerts[out_base + 1] = process_vertex(v1, pushConsts.render_matrix);
    outVerts[out_base + 2] = process_vertex(v2, pushConsts.render_matrix);

    outIndices[group_thread_id] = uint3(out_base + 0, out_base + 1, out_base + 2);
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
