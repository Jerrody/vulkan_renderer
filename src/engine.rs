mod resources;

use std::sync::Arc;

use bevy_ecs::world::World;
use vulkanalia::{Version, vk::*};
use vulkanalia_bootstrap::{
    DeviceBuilder, InstanceBuilder, PhysicalDeviceSelector, PreferredDeviceType, Swapchain,
    SwapchainBuilder,
};
use winit::window::Window;

use crate::engine::resources::{
    FrameData, QueueData, RenderContextResource, VulkanContextResource, render_context_resource,
    vulkan_context_resource,
};

pub struct Engine {
    world: World,
}

impl Engine {
    pub fn new(window: Option<Arc<dyn Window>>) -> Self {
        let mut world = World::new();

        let vulkan_context_resource = Self::create_vulkan_context(window);
        world.insert_resource(vulkan_context_resource);

        let render_context_resource = Self::create_render_context(&world);
        world.insert_resource(render_context_resource);

        Self { world }
    }

    fn create_vulkan_context(window: Option<Arc<dyn Window>>) -> VulkanContextResource {
        let instance = InstanceBuilder::new(window.clone())
            .app_name("Render")
            .engine_name("Engine Name")
            .app_version(Version::V1_4_0)
            .request_validation_layers(true)
            .use_default_debug_messenger()
            .build()
            .unwrap();

        let mut physical_device_features12 = PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .descriptor_indexing(true)
            .timeline_semaphore(true);

        let unified_image_layout_feature =
            PhysicalDeviceUnifiedImageLayoutsFeaturesKHR::builder().unified_image_layouts(true);

        let mut physical_device_features13 = PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .synchronization2(true);
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

        let window_size = window.unwrap().surface_size();
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
            .desired_present_mode(PresentModeKHR::FIFO_RELAXED)
            .desired_size(Extent2D {
                width: window_size.width,
                height: window_size.height,
            })
            .build()
            .unwrap();

        let vulkan_context_resource = VulkanContextResource {
            instance,
            device,
            graphics_queue_data,
            swapchain,
        };

        vulkan_context_resource
    }

    fn create_render_context(world: &World) -> RenderContextResource {
        let vulkan_context_resource = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let swapchain = &vulkan_context_resource.swapchain;

        let images = swapchain.get_images().unwrap();
        let image_views = swapchain.get_image_views().unwrap();
        let frame_overlap = image_views.len();

        let command_pool_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();

        let device = &vulkan_context_resource.device;
        let frames_data = (0..frame_overlap)
            .map(|_| unsafe {
                let command_pool = device
                    .create_command_pool(&command_pool_info, None)
                    .unwrap();

                let command_buffer_allocate_info = CommandBufferAllocateInfo::builder()
                    .command_pool(command_pool)
                    .command_buffer_count(1)
                    .build();
                let command_buffer = device
                    .allocate_command_buffers(&command_buffer_allocate_info)
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone();

                let mut semaphore_type_create_info = SemaphoreTypeCreateInfo::builder()
                    .semaphore_type(SemaphoreType::TIMELINE)
                    .build();
                let test_semaphore = device
                    .create_semaphore(
                        &SemaphoreCreateInfo::builder()
                            .push_next(&mut semaphore_type_create_info)
                            .build(),
                        None,
                    )
                    .unwrap();

                let render_semaphore = device
                    .create_semaphore(&SemaphoreCreateInfo::builder().build(), None)
                    .unwrap();
                let present_semaphore = device
                    .create_semaphore(&SemaphoreCreateInfo::builder().build(), None)
                    .unwrap();

                let fence = device
                    .create_fence(
                        &FenceCreateInfo::builder().flags(FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .unwrap();

                FrameData {
                    command_pool,
                    command_buffer,
                    fence,
                    test_semaphore,
                    render_semaphore,
                    present_semaphore,
                }
            })
            .collect();

        let render_context_resource = RenderContextResource {
            images,
            image_views,
            frame_overlap,
            frames_data,
        };

        render_context_resource
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let vulkan_context_resource = self.world.get_resource::<VulkanContextResource>().unwrap();
        let render_context_resource = self.world.get_resource::<RenderContextResource>().unwrap();

        let device = &vulkan_context_resource.device;

        render_context_resource
            .frames_data
            .iter()
            .for_each(|frame_data| unsafe {
                device.destroy_command_pool(frame_data.command_pool, None);
                device.destroy_semaphore(frame_data.test_semaphore, None);
                device.destroy_semaphore(frame_data.render_semaphore, None);
                device.destroy_semaphore(frame_data.present_semaphore, None);
                device.destroy_fence(frame_data.fence, None);
            });

        vulkan_context_resource
            .swapchain
            .destroy_image_views()
            .unwrap();
        vulkan_context_resource.swapchain.destroy();
        vulkan_context_resource.device.destroy();
        vulkan_context_resource.instance.destroy();
    }
}
