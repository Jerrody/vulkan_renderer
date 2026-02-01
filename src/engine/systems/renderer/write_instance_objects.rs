use bevy_ecs::system::ResMut;

use crate::engine::resources::{RendererResources, SwappableBuffer};

pub fn write_instance_objects(mut renderer_resources: ResMut<RendererResources>) {
    let instances_objects_buffer: &SwappableBuffer = unsafe {
        &*(renderer_resources
            .resources_pool
            .instances_buffer
            .as_ref()
            .unwrap() as *const SwappableBuffer)
    };

    let instances_objects_to_write = instances_objects_buffer.get_objects_to_write_as_slice();

    let instances_objects_buffer_reference = instances_objects_buffer.get_current_buffer();
    unsafe {
        renderer_resources
            .resources_pool
            .memory_bucket
            .transfer_data_to_buffer(
                instances_objects_buffer_reference,
                instances_objects_to_write,
                instances_objects_to_write.len(),
            );
    }
}
