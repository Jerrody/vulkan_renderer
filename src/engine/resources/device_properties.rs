use bevy_ecs::resource::Resource;
use vulkanalia::vk::{
    PhysicalDeviceDescriptorBufferPropertiesEXT, PhysicalDeviceProperties,
    PhysicalDeviceProperties2,
};

#[derive(Resource)]
pub struct DevicePropertiesResource {
    pub device_properties: PhysicalDeviceProperties,
    pub device_properties2: PhysicalDeviceProperties2,
    pub descriptor_buffer_properties: PhysicalDeviceDescriptorBufferPropertiesEXT,
}
