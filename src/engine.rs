mod descriptors;
mod resources;
mod setup;
mod systems;
mod utils;

use std::sync::Arc;

use bevy_ecs::{
    schedule::{Schedule, ScheduleLabel},
    world::World,
};
use vulkanalia::vk::DeviceV1_0;
use winit::window::Window;

use crate::engine::{
    resources::{FrameContext, RendererContext, RendererResources, VulkanContextResource},
    systems::{prepare_frame, present, render},
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

        /*         let device_properties_resource = Self::create_device_properties(&world);
        world.insert_resource(device_properties_resource); */

        let render_context = Self::create_renderer_context(&window, &world);
        world.insert_resource(render_context);

        let renderer_resources = Self::create_renderer_resources(&world);
        world.insert_resource(renderer_resources);

        let frame_context = FrameContext::default();
        world.insert_resource(frame_context);

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
