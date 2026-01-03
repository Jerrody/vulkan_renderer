[[vk::binding(0, 0)]]
[[vk::image_format("rgba16f")]]
RWTexture2D<float4> image;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 dispatch_thread_id: SV_DispatchThreadID, uint3 group_thread_id: SV_GroupThreadID)
{
    vector<int, 2> texel_coord = dispatch_thread_id.xy;

    uint width, height;
    image.GetDimensions(width, height);

    if (texel_coord.x < width && texel_coord.y < height)
    {
        float4 color = float4(0.0, 0.0, 0.0, 1.0);
        if (group_thread_id.x != 0 && group_thread_id.y != 0)
        {
            color.x = (float)texel_coord.x / width;
            color.x = (float)texel_coord.y / height;
        }

        int2 coord = int2(texel_coord);
        image[coord] = color;
    }
}
