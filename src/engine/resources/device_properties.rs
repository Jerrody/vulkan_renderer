use bevy_ecs::resource::Resource;
use vulkanalia::vk::{PhysicalDeviceProperties, PhysicalDeviceProperties2};

#[derive(Resource)]
pub struct DevicePropertiesResource {
    pub device_properties: PhysicalDeviceProperties,
    pub device_properties2: PhysicalDeviceProperties2,
}
