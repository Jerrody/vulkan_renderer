struct PushConstants
{
    let draw_image_index : uint64_t;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConsts;

[[vk::binding(1, 0)]]
RWTexture2D<float4> images[128];

[shader("compute")]
[numthreads(16, 16, 1)]
func main(uint3 dispatch_thread_id: SV_DispatchThreadID, uint3 group_thread_id: SV_GroupThreadID)
{
    let texel_coord = dispatch_thread_id.xy;

    let image = images[pushConsts.draw_image_index];

    var width : uint;
    var height : uint;
    image.GetDimensions(width, height);

    if (texel_coord.x < width && texel_coord.y < height)
    {
        var color = float4(0.0, 0.0, 0.0, 1.0);

        if (group_thread_id.x != 0 && group_thread_id.y != 0)
        {
            color.x = (float)texel_coord.x / (float)width;
            color.y = (float)texel_coord.y / (float)height;
        }

        image[texel_coord] = color;
    }
}
