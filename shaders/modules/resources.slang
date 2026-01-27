[[vk::binding(0, 0)]]
const SamplerState samplers[16];

[[vk::binding(1, 0)]]
const RWTexture2D<float4> storage_images[128];

[[vk::binding(2, 0)]]
const Texture2D<float4> sampled_images[];
