use vulkanalia::vk::{
    ComponentMapping, ComponentSwizzle, Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo,
    ImageLayout, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags,
    ImageViewCreateInfo, ImageViewType, SampleCountFlags, SharingMode,
};

pub fn create_image_info(
    format: Format,
    image_usage_flags: ImageUsageFlags,
    extent: Extent3D,
) -> ImageCreateInfo {
    let image_create_info = ImageCreateInfo {
        image_type: ImageType::_2D,
        format,
        extent,
        mip_levels: 1,
        array_layers: 1,
        samples: SampleCountFlags::_1,
        tiling: ImageTiling::OPTIMAL,
        usage: image_usage_flags,
        sharing_mode: SharingMode::EXCLUSIVE,
        initial_layout: ImageLayout::UNDEFINED,
        ..Default::default()
    };

    return image_create_info;
}

pub fn create_image_view_info(
    format: Format,
    image: Image,
    image_aspect_flags: ImageAspectFlags,
) -> ImageViewCreateInfo {
    let image_view_create_info = ImageViewCreateInfo {
        image,
        view_type: ImageViewType::_2D,
        format,
        components: ComponentMapping {
            r: ComponentSwizzle::R,
            g: ComponentSwizzle::G,
            b: ComponentSwizzle::B,
            a: ComponentSwizzle::A,
        },
        subresource_range: ImageSubresourceRange {
            aspect_mask: image_aspect_flags,
            base_mip_level: Default::default(),
            level_count: 1,
            base_array_layer: Default::default(),
            layer_count: 1,
        },
        ..Default::default()
    };

    return image_view_create_info;
}
