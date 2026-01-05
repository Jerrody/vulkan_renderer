let image : RWTexture2D<float4>;

[shader("compute")]
[numthreads(16, 16, 1)]
func main(uint3 dispatch_thread_id: SV_DispatchThreadID, uint3 group_thread_id: SV_GroupThreadID)
{
    let texel_coord = (int2)dispatch_thread_id.xy;

    var width : uint;
    var height : uint;
    image.GetDimensions(width, height);

    if (texel_coord.x < (int)width && texel_coord.y < (int)height)
    {
        var color = float4(0.0, 0.0, 0.0, 1.0);

        if (group_thread_id.x != 0 && group_thread_id.y != 0)
        {
            color.x = (float)texel_coord.x / (float)width;
            color.y = (float)texel_coord.y / (float)height;
        }

        let coord = (int2)texel_coord;

        image[coord] = color;
    }
}
