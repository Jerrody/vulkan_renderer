static float2 positions[3] = float2[](
    float2(0.0, -0.5),
    float2(0.5, 0.5),
    float2(-0.5, 0.5));

static float3 colors[3] = float3[](
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0));

struct VertexOutput
{
    float4 sv_position : SV_Position;
    float3 color;
};

[shader("vertex")]
VertexOutput vs_main(uint vertex_id: SV_VertexID)
{
    float2 current_position = positions[vertex_id];

    VertexOutput vertex_output;
    vertex_output.sv_position = float4(current_position.xy, 0.0, 1.0);
    vertex_output.color = colors[vertex_id];

    return vertex_output;
}

[shader("fragment")]
float4 fs_main(VertexOutput vertex_output)
{
    float4 color = float4(vertex_output.color.xyz, 1.0);

    return color;
}
