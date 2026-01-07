static const uint MAX_VERTS = 64;
static const uint MAX_TRIS = 21;
static const uint TRIANGLE_VERTEX_COUNT = 3;

struct Vertex
{
    float3 position;
    float3 normal;
    float4 color;
    float uv_x;
    float uv_y;
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
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConsts;

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(MAX_VERTS, 1, 1)]
void main(
    uint li: SV_GroupIndex,
    uint gi: SV_GroupID,
    OutputVertices<VertexOutput, MAX_VERTS> outVerts,
    OutputIndices<uint3, MAX_TRIS> outIndices)
{
    if (li == 0)
    {
        SetMeshOutputCounts(MAX_VERTS, MAX_TRIS);
    }

    uint vertexIndex = (gi * MAX_VERTS) + li;

    Vertex v = pushConsts.vertexBuffer[vertexIndex];

    VertexOutput vertex_ouput;
    vertex_ouput.position = float4(v.position, 1.0f);
    vertex_ouput.color = v.color.xyz;
    vertex_ouput.uv = float2(v.uv_x, v.uv_y);

    outVerts[li] = vertex_ouput;

    if (li < MAX_TRIS)
    {
        uint v0 = li * TRIANGLE_VERTEX_COUNT;
        uint v1 = li * TRIANGLE_VERTEX_COUNT + 1;
        uint v2 = li * TRIANGLE_VERTEX_COUNT + 2;

        outIndices[li] = uint3(v0, v1, v2);
    }
}
