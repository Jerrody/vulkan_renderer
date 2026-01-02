use std::sync::Arc;

use vulkanalia::vk::{
    AccessFlags2, BlitImageInfo2, CommandBuffer, CommandBufferBeginInfo, CommandBufferSubmitInfo,
    CommandBufferUsageFlags, DependencyInfo, DeviceV1_3, Extent2D, Filter, HasBuilder, Image,
    ImageAspectFlags, ImageBlit2, ImageLayout, ImageMemoryBarrier2, ImageSubresourceLayers,
    ImageSubresourceRange, Offset3D, PipelineStageFlags2, REMAINING_ARRAY_LAYERS,
    REMAINING_MIP_LEVELS, Semaphore, SemaphoreSubmitInfo, SubmitInfo2,
};

pub fn create_command_buffer_begin_info(flags: CommandBufferUsageFlags) -> CommandBufferBeginInfo {
    let command_buffer_begin_info = CommandBufferBeginInfo::builder().flags(flags).build();

    command_buffer_begin_info
}

pub fn transition_image(
    device: &Arc<vulkanalia_bootstrap::Device>,
    command_buffer: CommandBuffer,
    image: Image,
    old_image_layout: ImageLayout,
    new_image_layout: ImageLayout,
) {
    let image_memory_barrier = ImageMemoryBarrier2 {
        src_stage_mask: PipelineStageFlags2::ALL_COMMANDS,
        src_access_mask: AccessFlags2::MEMORY_WRITE,
        dst_stage_mask: PipelineStageFlags2::ALL_COMMANDS,
        dst_access_mask: AccessFlags2::MEMORY_READ | AccessFlags2::MEMORY_WRITE,
        old_layout: old_image_layout,
        new_layout: new_image_layout,
        image,
        subresource_range: image_subresource_range(ImageAspectFlags::COLOR),
        ..Default::default()
    };

    let image_memory_barriers = [image_memory_barrier];
    let dependency_info = DependencyInfo::builder()
        .image_memory_barriers(&image_memory_barriers)
        .build();

    unsafe {
        device.cmd_pipeline_barrier2(command_buffer, &dependency_info);
    }
}

pub fn image_subresource_range(aspect_mask: ImageAspectFlags) -> ImageSubresourceRange {
    let image_subresource_range = ImageSubresourceRange {
        aspect_mask,
        base_mip_level: Default::default(),
        level_count: REMAINING_MIP_LEVELS,
        base_array_layer: Default::default(),
        layer_count: REMAINING_ARRAY_LAYERS,
    };

    image_subresource_range
}

pub fn semaphore_submit_info(
    stage_mask: PipelineStageFlags2,
    semaphore: Semaphore,
) -> SemaphoreSubmitInfo {
    let semaphore_submit_info = SemaphoreSubmitInfo::builder()
        .semaphore(semaphore)
        .stage_mask(stage_mask)
        .build();

    semaphore_submit_info
}

pub fn command_buffer_submit_info(command_buffer: CommandBuffer) -> CommandBufferSubmitInfo {
    let command_buffer_submit_info = CommandBufferSubmitInfo::builder()
        .command_buffer(command_buffer)
        .build();

    command_buffer_submit_info
}

pub fn submit_info(
    command_buffer_submit_infos: &[CommandBufferSubmitInfo],
    wait_semaphores: &[SemaphoreSubmitInfo],
    signal_semaphores: &[SemaphoreSubmitInfo],
) -> SubmitInfo2 {
    let submit_info = SubmitInfo2::builder()
        .wait_semaphore_infos(&wait_semaphores)
        .signal_semaphore_infos(&signal_semaphores)
        .command_buffer_infos(command_buffer_submit_infos)
        .build();

    submit_info
}

pub fn copy_image_to_image(
    device: &Arc<vulkanalia_bootstrap::Device>,
    command_buffer: CommandBuffer,
    source_image: Image,
    destination_image: Image,
    src_extent: Extent2D,
    dst_extent: Extent2D,
) {
    let src_offsets = [
        Offset3D::default(),
        Offset3D {
            x: src_extent.width as _,
            y: src_extent.height as _,
            z: 1,
        },
    ];
    let dst_offsets = [
        Offset3D::default(),
        Offset3D {
            x: dst_extent.width as _,
            y: dst_extent.height as _,
            z: 1,
        },
    ];

    let src_subresource = ImageSubresourceLayers {
        aspect_mask: ImageAspectFlags::COLOR,
        mip_level: Default::default(),
        base_array_layer: Default::default(),
        layer_count: 1,
    };
    let dst_subresource = ImageSubresourceLayers {
        aspect_mask: ImageAspectFlags::COLOR,
        mip_level: Default::default(),
        base_array_layer: Default::default(),
        layer_count: 1,
    };
    let blit_region = ImageBlit2 {
        src_subresource,
        src_offsets,
        dst_subresource,
        dst_offsets,
        ..Default::default()
    };

    let regions = [blit_region];
    let image_blit_info = BlitImageInfo2 {
        src_image: source_image,
        src_image_layout: ImageLayout::GENERAL,
        dst_image: destination_image,
        dst_image_layout: ImageLayout::GENERAL,
        region_count: regions.len() as _,
        regions: regions.as_ptr(),
        filter: Filter::LINEAR,
        ..Default::default()
    };

    unsafe {
        device.cmd_blit_image2(command_buffer, &image_blit_info);
    }
}
