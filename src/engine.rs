mod resources;
mod systems;
mod utils;

use std::sync::Arc;

use bevy_ecs::{
    schedule::{Schedule, ScheduleLabel},
    world::World,
};
use vma::{Alloc, AllocationOptions, Allocator, AllocatorOptions, MemoryUsage};
use vulkanalia::{Version, vk::*};
use vulkanalia_bootstrap::{
    DeviceBuilder, InstanceBuilder, PhysicalDeviceSelector, PreferredDeviceType, SwapchainBuilder,
};
use winit::window::Window;

use crate::engine::{
    resources::{
        AllocatedImage, FrameContext, FrameData, QueueData, RendererContext, RendererResources,
        VulkanContextResource,
    },
    systems::{prepare_frame, present, render},
    utils::{create_image_info, create_image_view_info},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct ScheduleLabelUpdate;

pub struct Engine {
    world: World,
}

impl Engine {
    pub fn new(window: Option<Arc<dyn Window>>) -> Self {
        let mut world = World::new();
        let window = &window.unwrap();

        let vulkan_context_resource = Self::create_vulkan_context(&window);
        world.insert_resource(vulkan_context_resource);

        let render_context = Self::create_renderer_context(&window, &world);
        world.insert_resource(render_context);

        let renderer_resources = Self::create_renderer_resources(&world);
        world.insert_resource(renderer_resources);

        let frame_context = FrameContext::default();
        world.insert_resource(frame_context);

        world.register_system(systems::render);

        let mut schedule = Schedule::new(ScheduleLabelUpdate);
        schedule.add_systems((
            prepare_frame::prepare_frame,
            render::render,
            present::present,
        ));

        world.add_schedule(schedule);

        Self { world }
    }

    pub fn update(&mut self) {
        self.world.run_schedule(ScheduleLabelUpdate);
    }

    fn create_vulkan_context(window: &Arc<dyn Window>) -> VulkanContextResource {
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

        let descriptor_buffer_feature =
            PhysicalDeviceDescriptorBufferFeaturesEXT::builder().descriptor_buffer(true);

        let mut physical_device_features13 = PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .synchronization2(true);
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

    fn create_renderer_context(window: &Arc<dyn Window>, world: &World) -> RendererContext {
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

                let render_fence = device
                    .create_fence(
                        &FenceCreateInfo::builder().flags(FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .unwrap();

                let semaphore_create_info = SemaphoreCreateInfo::default();
                let swapchain_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();
                let render_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();

                FrameData {
                    command_pool,
                    command_buffer,
                    render_fence,
                    swapchain_semaphore,
                    render_semaphore,
                }
            })
            .collect();

        let surface_size = window.surface_size();
        let draw_extent = Extent2D {
            width: surface_size.width,
            height: surface_size.height,
        };
        let render_context_resource = RendererContext {
            images,
            image_views,
            frame_overlap,
            draw_extent,
            frames_data,
            frame_number: Default::default(),
        };

        render_context_resource
    }

    pub fn create_renderer_resources(world: &World) -> RendererResources {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();

        let draw_image_extent = Extent3D {
            width: render_context.draw_extent.width,
            height: render_context.draw_extent.height,
            depth: 1,
        };
        let target_draw_image_format = Format::R16G16B16A16_SFLOAT;
        let image_usage_flags = ImageUsageFlags::TRANSFER_SRC
            | ImageUsageFlags::TRANSFER_DST
            | ImageUsageFlags::STORAGE
            | ImageUsageFlags::COLOR_ATTACHMENT;

        let image_create_info = create_image_info(
            target_draw_image_format,
            image_usage_flags,
            draw_image_extent,
        );
        let mut allocation_options = AllocationOptions::default();
        allocation_options.usage = MemoryUsage::Auto;
        allocation_options.required_flags = MemoryPropertyFlags::DEVICE_LOCAL;

        let (allocated_draw_image, allocation) = unsafe {
            vulkan_context
                .allocator
                .create_image(image_create_info, &allocation_options)
                .unwrap()
        };

        let image_view_create_info = create_image_view_info(
            target_draw_image_format,
            allocated_draw_image,
            ImageAspectFlags::COLOR,
        );
        let allocated_image_view = unsafe {
            vulkan_context
                .device
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        };

        let draw_image = AllocatedImage {
            image: allocated_draw_image,
            image_view: allocated_image_view,
            allocation: allocation,
            image_extent: draw_image_extent,
            format: Format::R16G16B16A16_SFLOAT,
        };

        let renderer_resources = RendererResources { draw_image };

        renderer_resources
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let vulkan_context_resource = self.world.get_resource::<VulkanContextResource>().unwrap();
        let render_context_resource = self.world.get_resource::<RendererContext>().unwrap();
        let renderer_resources = self.world.get_resource::<RendererResources>().unwrap();

        let device = &vulkan_context_resource.device;

        unsafe {
            device.device_wait_idle().unwrap();
        }

        unsafe {
            device.destroy_image_view(renderer_resources.draw_image.image_view, None);
            vulkan_context_resource.allocator.destroy_image(
                renderer_resources.draw_image.image,
                renderer_resources.draw_image.allocation,
            );
        }

        render_context_resource
            .frames_data
            .iter()
            .for_each(|frame_data| unsafe {
                device.destroy_command_pool(frame_data.command_pool, None);
                device.destroy_fence(frame_data.render_fence, None);
                device.destroy_semaphore(frame_data.render_semaphore, None);
                device.destroy_semaphore(frame_data.swapchain_semaphore, None);
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
