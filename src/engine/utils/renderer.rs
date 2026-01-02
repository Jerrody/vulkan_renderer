use std::sync::Arc;

use vulkanalia::vk::{
    AccessFlags2, CommandBuffer, CommandBufferBeginInfo, CommandBufferSubmitInfo,
    CommandBufferUsageFlags, DependencyInfo, Device, DeviceV1_3, HasBuilder, Image,
    ImageAspectFlags, ImageLayout, ImageMemoryBarrier2, ImageSubresourceRange, PipelineStageFlags2,
    REMAINING_ARRAY_LAYERS, REMAINING_MIP_LEVELS, Semaphore, SemaphoreSubmitInfo, SubmitInfo,
    SubmitInfo2,
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
