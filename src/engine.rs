mod descriptors;
mod resources;
mod setup;
mod systems;
mod utils;

use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    world::World,
};
use vulkanite::{
    Handle,
    vk::raw::{Buffer, Image},
};
use winit::window::Window;

use crate::engine::{
    resources::{
        FrameContext, RendererContext, RendererResources, VulkanContextResource, render_resources,
        vulkan_context_resource,
    },
    systems::{prepare_frame, present, render},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct ScheduleLabelUpdate;

pub struct Engine {
    world: World,
}

impl Engine {
    pub fn new(window: &Box<dyn Window>) -> Self {
        let mut world: World = World::new();

        let vulkan_context_resource = Self::create_vulkan_context(&window);
        world.insert_resource(vulkan_context_resource);

        let device_properties_resource = Self::create_device_properties(&world);
        world.insert_resource(device_properties_resource);

        let render_context = Self::create_renderer_context(&window, &world);
        world.insert_resource(render_context);

        let renderer_resources = Self::create_renderer_resources(&world);
        world.insert_resource(renderer_resources);

        let frame_context = FrameContext::default();
        world.insert_resource(frame_context);

        let mut schedule = Schedule::new(ScheduleLabelUpdate);
        schedule.add_systems((
            prepare_frame::prepare_frame,
            render::render.after(prepare_frame::prepare_frame),
            present::present.after(render::render),
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
        let mut vulkan_context_resource = self
            .world
            .remove_resource::<VulkanContextResource>()
            .unwrap();
        let render_context_resource = self.world.remove_resource::<RendererContext>().unwrap();
        let mut renderer_resources = self.world.remove_resource::<RendererResources>().unwrap();

        let device = &mut vulkan_context_resource.device;

        device.wait_idle().unwrap();

        unsafe {
            device.destroy_image_view(Some(&renderer_resources.draw_image.image_view));

            let draw_image_desciptor_buffer = &mut renderer_resources.draw_image_descriptor_buffer;
            device.destroy_buffer(Some(&draw_image_desciptor_buffer.allocated_buffer.buffer));
            vulkan_context_resource
                .allocator
                .free_memory(&mut draw_image_desciptor_buffer.allocated_buffer.allocation);

            device.destroy_descriptor_set_layout(Some(
                &draw_image_desciptor_buffer.descriptor_set_layout,
            ));
            device.destroy_image(Some(&renderer_resources.draw_image.image));
            vulkan_context_resource
                .allocator
                .free_memory(&mut renderer_resources.draw_image.allocation);

            render_context_resource
                .frames_data
                .iter()
                .for_each(|frame_data| {
                    device.destroy_command_pool(Some(&frame_data.command_pool));
                    device.destroy_fence(Some(&frame_data.render_fence));
                    device.destroy_semaphore(Some(&frame_data.render_semaphore));
                    device.destroy_semaphore(Some(&frame_data.swapchain_semaphore));
                });

            render_context_resource
                .image_views
                .iter()
                .for_each(|image_view| {
                    vulkan_context_resource
                        .device
                        .destroy_image_view(Some(image_view));
                });
            vulkan_context_resource
                .device
                .destroy_swapchain_khr(Some(&vulkan_context_resource.swapchain));
            vulkan_context_resource.device.destroy();
        }
    }
}
