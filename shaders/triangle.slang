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
    private float4 _sv_position : SV_Position;
    private float3 _color;

    property float3 Color
    {
        [ForceInline]
        get
        {
            return _color;
        }
    }

    [ForceInline]
    __init(float4 sv_position, float3 color)
    {
        _sv_position = sv_position;
        _color = color;
    }
};

[shader("vertex")]
func main(uint vertex_id: SV_VertexID)->VertexOutput
{
    let current_position = positions[vertex_id];

    VertexOutput vertex_output = VertexOutput(float4(current_position.xy, 0.0, 1.0), colors[vertex_id]);

    return vertex_output;
}

[shader("fragment")]
func main(VertexOutput vertex_output)->float4
{
    let color = float4(vertex_output.Color.xyz, 1.0);

    return color;
}
