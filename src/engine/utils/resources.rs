use vulkanite::vk::{rs::*, *};

pub fn create_image_info<'a>(
    format: Format,
    image_usage_flags: ImageUsageFlags,
    extent: Extent3D,
    initial_layout: ImageLayout,
) -> ImageCreateInfo<'a> {
    ImageCreateInfo::default()
        .image_type(ImageType::Type2D)
        .format(format)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        .samples(SampleCountFlags::Count1)
        .tiling(ImageTiling::Optimal)
        .usage(image_usage_flags)
        .sharing_mode(SharingMode::Exclusive)
        .initial_layout(initial_layout)
}

pub fn create_image_view_info<'a>(
    format: Format,
    image: &'a Image,
    image_aspect_flags: ImageAspectFlags,
) -> ImageViewCreateInfo<'a> {
    let mut image_view_create_info = ImageViewCreateInfo::default()
        .view_type(ImageViewType::Type2D)
        .format(format)
        .components(ComponentMapping {
            r: ComponentSwizzle::R,
            g: ComponentSwizzle::G,
            b: ComponentSwizzle::B,
            a: ComponentSwizzle::A,
        })
        .subresource_range(
            ImageSubresourceRange::default()
                .aspect_mask(image_aspect_flags)
                .base_mip_level(Default::default())
                .level_count(1)
                .base_array_layer(Default::default())
                .layer_count(1),
        );
    image_view_create_info = image_view_create_info.image(image);

    image_view_create_info
}
