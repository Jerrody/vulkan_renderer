use bevy_ecs::world::World;
use vulkanalia::vk::{InstanceV1_1, PhysicalDeviceProperties2};

use crate::engine::{
    Engine,
    resources::{DevicePropertiesResource, VulkanContextResource},
};

impl Engine {
    pub(crate) fn create_device_properties(world: &World) -> DevicePropertiesResource {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();

        let device_properties2 = unsafe {
            let mut device_properties2 = PhysicalDeviceProperties2::default();

            vulkan_context_resource
                .instance
                .instance
                .get_physical_device_properties2(
                    vulkan_context_resource
                        .device
                        .physical_device
                        .physical_device,
                    &mut device_properties2,
                );

            device_properties2
        };

        let device_properties_resource = DevicePropertiesResource {
            device_properties: vulkan_context_resource.device.physical_device.properties,
            device_properties2,
        };

        device_properties_resource
    }
}
