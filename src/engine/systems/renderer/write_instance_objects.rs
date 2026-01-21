use std::ptr::copy_nonoverlapping;

use bevy_ecs::system::{Res, ResMut};

use crate::engine::resources::{RendererResources, VulkanContextResource};

pub fn write_instance_objects(
    vk_ctx: Res<VulkanContextResource>,
    mut renderer_resources: ResMut<RendererResources>,
) {
    let instances_objects_to_write = renderer_resources.get_instances_objects_to_write_as_slice();
    let instances_objects_to_write_count = instances_objects_to_write.len();
    let ptr_instances_objects_to_write = instances_objects_to_write.as_ptr();

    let instance_set_buffer_id = renderer_resources.get_current_instance_set_buffer_id();
    let instance_set_buffer = renderer_resources.get_storage_buffer_ref_mut(instance_set_buffer_id);

    let allocator = &vk_ctx.allocator;

    unsafe {
        let ptr_instance_set_buffer = allocator
            .map_memory(&mut instance_set_buffer.allocation)
            .unwrap();

        copy_nonoverlapping(
            ptr_instances_objects_to_write,
            ptr_instance_set_buffer as *mut _,
            instances_objects_to_write_count,
        );

        allocator.unmap_memory(&mut instance_set_buffer.allocation);
    }
}
