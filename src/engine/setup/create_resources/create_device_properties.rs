use std::marker::PhantomData;

use bevy_ecs::world::World;
use vulkanite::vk::*;

use crate::engine::{
    Engine,
    resources::{DevicePropertiesResource, VulkanContextResource},
};

impl Engine {
    pub(crate) fn create_device_properties(world: &World) -> DevicePropertiesResource {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();

        let (_, _, descriptor_buffer_properties): (
            _,
            PhysicalDeviceVulkan11Properties,
            PhysicalDeviceDescriptorBufferPropertiesEXT,
        ) = vulkan_context_resource.physical_device.get_properties2();

        let device_properties_resource = DevicePropertiesResource {
            descriptor_buffer_properties,
        };

        device_properties_resource
    }
}
