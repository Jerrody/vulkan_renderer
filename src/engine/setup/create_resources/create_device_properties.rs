use std::ffi::c_void;

use bevy_ecs::world::World;
use vulkanalia::vk::{
    InstanceV1_1, OutputChainStruct, PhysicalDeviceDescriptorBufferPropertiesEXT,
    PhysicalDeviceProperties2,
};

use crate::engine::{
    Engine,
    resources::{DevicePropertiesResource, VulkanContextResource},
};

impl Engine {
    pub(crate) fn create_device_properties(world: &World) -> DevicePropertiesResource {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let instance = &vulkan_context_resource.instance.instance;

        let (device_properties2, descriptor_buffer_properties) = unsafe {
            let mut descriptor_buffer_propeties =
                PhysicalDeviceDescriptorBufferPropertiesEXT::default();
            let mut device_properties2 = PhysicalDeviceProperties2::default();
            device_properties2.next = (&mut descriptor_buffer_propeties) as *mut _ as *mut c_void;

            instance.get_physical_device_properties2(
                vulkan_context_resource
                    .device
                    .physical_device
                    .physical_device,
                &mut device_properties2,
            );

            (device_properties2, descriptor_buffer_propeties)
        };

        let device_properties_resource = DevicePropertiesResource {
            device_properties: vulkan_context_resource.device.physical_device.properties,
            device_properties2,
            descriptor_buffer_properties,
        };

        device_properties_resource
    }
}
