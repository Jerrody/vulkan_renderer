use std::sync::Arc;

use vma::{Allocator, AllocatorOptions};
use vulkanalia::{
    Version,
    vk::{
        ColorSpaceKHR, Extent2D, Format, HasBuilder, ImageUsageFlags, OutputChainStruct,
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
            .request_validation_layers(false)
            .use_default_debug_messenger()
            .build()
            .unwrap();

        let mut physical_device_features12 = PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .descriptor_indexing(true)
            .timeline_semaphore(true);

        let mut unified_image_layout_feature =
            PhysicalDeviceUnifiedImageLayoutsFeaturesKHR::builder().unified_image_layouts(true);

        let mut descriptor_buffer_feature =
            PhysicalDeviceDescriptorBufferFeaturesEXT::builder().descriptor_buffer(true);

        let shader_objects_feature =
            PhysicalDeviceShaderObjectFeaturesEXT::builder().shader_object(true);

        let mut physical_device_features13 = PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .synchronization2(true);
        descriptor_buffer_feature.next = shader_objects_feature.next_mut();
        unified_image_layout_feature.next = descriptor_buffer_feature.next_mut();
        physical_device_features13.next = unified_image_layout_feature.next_mut();
        physical_device_features12.next = physical_device_features13.next_mut();

        let physical_device = PhysicalDeviceSelector::new(instance.clone())
            .preferred_device_type(PreferredDeviceType::Discrete)
            .add_required_extension_feature(*physical_device_features12)
            .select()
            .unwrap();

        let device = Arc::new(
            DeviceBuilder::new(physical_device, instance.clone())
                .build()
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

        let allocator_info = AllocatorOptions::new(
            &instance.instance,
            &device.device,
            device.physical_device.physical_device,
        );
        let allocator = unsafe { Allocator::new(&allocator_info).unwrap() };

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
