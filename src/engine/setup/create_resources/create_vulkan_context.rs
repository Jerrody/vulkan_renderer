use std::{ffi::c_void, sync::Arc};

use vma::{Allocator, AllocatorCreateFlags, AllocatorOptions};
use vulkanalia::{
    Version,
    vk::{
        ColorSpaceKHR, EXT_DESCRIPTOR_BUFFER_EXTENSION, EXT_SHADER_OBJECT_EXTENSION, Extent2D,
        Format, HasBuilder, ImageUsageFlags, KHR_UNIFIED_IMAGE_LAYOUTS_EXTENSION,
        PhysicalDeviceDescriptorBufferFeaturesEXT, PhysicalDeviceShaderObjectFeaturesEXT,
        PhysicalDeviceUnifiedImageLayoutsFeaturesKHR, PhysicalDeviceVulkan12Features,
        PhysicalDeviceVulkan13Features, PresentModeKHR, SurfaceFormat2KHR, SurfaceFormatKHR,
    },
};
use vulkanalia_bootstrap::{
    DeviceBuilder, InstanceBuilder, PhysicalDeviceSelector, PreferredDeviceType, SwapchainBuilder,
};
use winit::window::Window;

use crate::engine::{
    Engine,
    resources::{QueueData, VulkanContextResource},
};

impl Engine {
    pub(crate) fn create_vulkan_context(window: &Arc<dyn Window>) -> VulkanContextResource {
        let instance = InstanceBuilder::new(Some(window.clone()))
            .app_name("Render")
            .engine_name("Engine Name")
            .app_version(Version::V1_4_0)
            .require_api_version(Version::V1_4_0)
            .request_validation_layers(true)
            .use_default_debug_messenger()
            .build()
            .unwrap();

        let physical_device_features12 =
            PhysicalDeviceVulkan12Features::builder().buffer_device_address(true);

        let mut unified_image_layout_feature =
            PhysicalDeviceUnifiedImageLayoutsFeaturesKHR::builder().unified_image_layouts(true);

        let mut descriptor_buffer_feature =
            PhysicalDeviceDescriptorBufferFeaturesEXT::builder().descriptor_buffer(true);

        let mut shader_objects_feature =
            PhysicalDeviceShaderObjectFeaturesEXT::builder().shader_object(true);

        let mut physical_device_features13 = PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .synchronization2(true);

        descriptor_buffer_feature.next = (&mut shader_objects_feature) as *mut _ as *mut c_void;
        unified_image_layout_feature.next =
            (&mut descriptor_buffer_feature) as *mut _ as *mut c_void;
        physical_device_features13.next =
            (&mut unified_image_layout_feature) as *mut _ as *mut c_void;

        let physical_device = PhysicalDeviceSelector::new(instance.clone())
            .add_required_extension_feature(*physical_device_features12)
            .add_required_extension_feature(*physical_device_features13)
            .select()
            .unwrap();

        let extension_names = [
            KHR_UNIFIED_IMAGE_LAYOUTS_EXTENSION.name,
            EXT_SHADER_OBJECT_EXTENSION.name,
            EXT_DESCRIPTOR_BUFFER_EXTENSION.name,
        ];
        let device = Arc::new(
            DeviceBuilder::new(physical_device, instance.clone())
                .build(&extension_names)
                .unwrap(),
        );

        let (graphics_queue_index, graphics_queue) = device
            .get_queue(vulkanalia_bootstrap::QueueType::Graphics)
            .unwrap();

        let graphics_queue_data = QueueData::new(graphics_queue_index, graphics_queue);

        let window_size = window.surface_size();
        let swapchain = SwapchainBuilder::new(instance.clone(), device.clone())
            .desired_format(
                SurfaceFormat2KHR::builder()
                    .surface_format(
                        SurfaceFormatKHR::builder()
                            .format(Format::B8G8R8A8_UNORM)
                            .color_space(ColorSpaceKHR::SRGB_NONLINEAR)
                            .build(),
                    )
                    .build(),
            )
            .add_image_usage_flags(
                ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::TRANSFER_DST,
            )
            .desired_present_mode(PresentModeKHR::FIFO)
            .desired_size(Extent2D {
                width: window_size.width,
                height: window_size.height,
            })
            .build()
            .unwrap();

        let mut allocation_options = AllocatorOptions::new(
            &instance.instance,
            &device.device,
            device.physical_device.physical_device,
        );
        allocation_options.flags = AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;

        let allocator = unsafe { Allocator::new(&allocation_options).unwrap() };

        let vulkan_context_resource = VulkanContextResource {
            instance,
            device,
            allocator,
            graphics_queue_data,
            swapchain,
        };

        vulkan_context_resource
    }
}
