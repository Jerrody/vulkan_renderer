use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use bytemuck::{Pod, Zeroable};
use vma::{Alloc, Allocation, AllocationCreateInfo, Allocator, MemoryUsage};
use vulkanite::{
    Handle,
    vk::{rs::*, *},
};

use crate::engine::id::Id;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TextureMetadata {
    pub width: u32,
    pub height: u32,
    pub mip_levels_count: u32,
}

pub struct AllocatedImage {
    pub id: Id,
    pub index: usize,
    pub image: Image,
    pub image_view: ImageView,
    pub allocation: Allocation,
    pub extent: Extent3D,
    pub format: Format,
    pub subresource_range: ImageSubresourceRange,
}

#[derive(Clone)]
pub struct ImageReference {
    image_id: Id,
    weak_ptr: Weak<AllocatedImage>,
    texture_metadata: TextureMetadata,
}

impl ImageReference {
    pub fn new(
        image_id: Id,
        allocated_buffer: Weak<AllocatedImage>,
        texture_metadata: TextureMetadata,
    ) -> Self {
        Self {
            image_id,
            weak_ptr: allocated_buffer,
            texture_metadata,
        }
    }

    pub fn get_image<'a>(&'a self) -> Option<&'a AllocatedImage> {
        let mut allocated_image = None;

        if !self.weak_ptr.strong_count() != Default::default() {
            let allocated_image_ref = unsafe { &*(self.weak_ptr.as_ptr()) };

            if allocated_image_ref.id == self.image_id {
                allocated_image = Some(allocated_image_ref);
            }
        }

        allocated_image
    }

    #[inline(always)]
    pub fn get_image_id(&self) -> Id {
        let allocated_image = self.get_image();
        match allocated_image {
            Some(allocated_image) => allocated_image.id,
            None => Id::NULL,
        }
    }

    #[inline(always)]
    pub fn get_texture_metadata(&self) -> TextureMetadata {
        self.texture_metadata
    }
}

pub struct TexturesPool {
    device: Device,
    allocator: Allocator,
    images: Vec<Arc<AllocatedImage>>,
    images_map: HashMap<Id, usize>,
}

impl TexturesPool {
    pub fn new(device: Device, allocator: Allocator) -> Self {
        Self {
            device,
            allocator,
            images: Vec::with_capacity(8_192),
            images_map: HashMap::with_capacity(8_192),
        }
    }

    pub fn create_image_from_file(is_cached: bool) {}

    pub fn create_image(
        device: Device,
        allocator: &Allocator,
        format: Format,
        extent: Extent3D,
        usage_flags: ImageUsageFlags,
        mip_map_enabled: bool,
    ) -> AllocatedImage {
        let mut aspect_flags = ImageAspectFlags::Color;
        if format == Format::D32Sfloat {
            aspect_flags = ImageAspectFlags::Depth;
        }

        let allocation_info = AllocationCreateInfo {
            usage: MemoryUsage::Auto,
            required_flags: MemoryPropertyFlags::DeviceLocal,
            ..Default::default()
        };

        let image_create_info = Self::get_image_info(
            format,
            usage_flags,
            extent,
            ImageLayout::Undefined,
            level_count,
        );
        let (allocated_image, allocation) = unsafe {
            allocator
                .create_image(&image_create_info, &allocation_info)
                .unwrap()
        };

        let image = rs::Image::from_inner(allocated_image);
        let image_view_create_info =
            Self::get_image_view_info(format, &image, aspect_flags, level_count);
        let image_view = device.create_image_view(&image_view_create_info).unwrap();

        AllocatedImage {
            id: Id::new(image.as_raw()),
            index: usize::MIN,
            image,
            image_view,
            allocation,
            extent,
            format,
            subresource_range: image_view_create_info.subresource_range,
        }
    }

    fn insert_image(&mut self, allocated_image: AllocatedImage) -> Weak<AllocatedImage> {
        let allocated_image_id = allocated_image.id;
        let allocated_image = Arc::new(allocated_image);
        let weak_ptr_allocated_image = Arc::downgrade(&allocated_image);
        self.images.push(allocated_image);
        let image_index = self.images.len() - 1;

        if let Some(already_presented_image_index) =
            self.images_map.insert(allocated_image_id, image_index)
        {
            panic!("Textures Pool already has buffer by index: {already_presented_image_index}");
        }

        weak_ptr_allocated_image
    }

    pub fn get_image_info<'a>(
        format: Format,
        usage_flags: ImageUsageFlags,
        extent: Extent3D,
        initial_layout: ImageLayout,
        mip_levels: Option<u32>,
    ) -> ImageCreateInfo<'a> {
        ImageCreateInfo::default()
            .image_type(ImageType::Type2D)
            .format(format)
            .extent(extent)
            .mip_levels(mip_levels.unwrap_or(1))
            .array_layers(1)
            .samples(SampleCountFlags::Count1)
            .tiling(ImageTiling::Optimal)
            .usage(usage_flags)
            .sharing_mode(SharingMode::Exclusive)
            .initial_layout(initial_layout)
    }

    pub fn get_image_view_info<'a>(
        format: Format,
        image: &'a Image,
        image_aspect_flags: ImageAspectFlags,
        level_count: Option<u32>,
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
                    .level_count(level_count.unwrap_or(1))
                    .base_array_layer(Default::default())
                    .layer_count(1),
            );
        image_view_create_info = image_view_create_info.image(image);

        image_view_create_info
    }
}
