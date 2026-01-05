static const float2 positions[3] = float2[](
    float2(0.0, -0.5),
    float2(0.5, 0.5),
    float2(-0.5, 0.5));

static const float3 colors[3] = float3[](
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0));

struct VertexOutput
{
    float4 sv_position : SV_Position;
    float3 color;

    __init(float4 new_sv_position, float3 new_color)
    {
        sv_position = new_sv_position;
        color = new_color;
    }
};

[shader("vertex")]
func vs_main(uint vertex_id: SV_VertexID)->VertexOutput
{
    let current_position = positions[vertex_id];

    VertexOutput vertex_output = VertexOutput(float4(current_position.xy, 0.0, 1.0), colors[vertex_id]);

    return vertex_output;
}

[shader("fragment")]
func fs_main(VertexOutput vertex_output)->float4
{
    let color = float4(vertex_output.color.xyz, 1.0);

    return color;
}
