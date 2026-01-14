mod components;
mod descriptors;
mod events;
mod id;
mod resources;
mod setup;
mod systems;
mod utils;

use std::str::FromStr;

use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    world::World,
};
use vulkanite::{
    Handle,
    vk::{self},
};
use winit::window::Window;

use crate::engine::{
    events::LoadModelEvent,
    resources::{FrameContext, RendererContext, RendererResources, VulkanContextResource},
    systems::{
        begin_rendering, end_rendering, on_load_model, on_spawn_mesh, prepare_frame, present,
        render_meshes,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct ScheduleLabelUpdate;

pub struct Engine {
    world: World,
}

impl Engine {
    pub fn new(window: &dyn Window) -> Self {
        let mut world: World = World::new();

        let vulkan_context_resource = Self::create_vulkan_context(window);
        world.insert_resource(vulkan_context_resource);

        let device_properties_resource = Self::create_device_properties(&world);
        world.insert_resource(device_properties_resource);

        let render_context = Self::create_renderer_context(window, &world);
        world.insert_resource(render_context);

        let descriptor_set_builder_resource = Self::create_descriptor_set_builder_resource();
        world.insert_resource(descriptor_set_builder_resource);

        let renderer_resources = Self::create_renderer_resources(&world);
        world.insert_resource(renderer_resources);

        let frame_context = FrameContext::default();
        world.insert_resource(frame_context);

        let mut schedule = Schedule::new(ScheduleLabelUpdate);
        schedule.add_systems((
            prepare_frame::prepare_frame,
            begin_rendering::begin_rendering.after(prepare_frame::prepare_frame),
            render_meshes::render_meshes.after(begin_rendering::begin_rendering),
            end_rendering::end_rendering.after(render_meshes::render_meshes),
            present::present.after(render_meshes::render_meshes),
        ));

        world.add_schedule(schedule);

        world.add_observer(on_load_model::on_load_model);
        world.add_observer(on_spawn_mesh::on_spawn_mesh);

        // TODO: TEMP
        world.trigger(LoadModelEvent {
            path: String::from_str(r"assets/basicmesh.glb").unwrap(),
        });

        Self { world }
    }

    pub fn update(&mut self) {
        self.world.flush();
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
            let draw_image_desciptor_buffer = &renderer_resources.draw_image_descriptor_set_handle;
            let white_image_desciptor_buffer =
                &renderer_resources.white_image_descriptor_set_handle;
            device.destroy_image_view(Some(renderer_resources.draw_image.image_view));
            device.destroy_image_view(Some(renderer_resources.depth_image.image_view));
            device.destroy_image_view(Some(renderer_resources.white_image.image_view));

            device.destroy_sampler(Some(renderer_resources.nearest_sampler));

            let pipeline_layout = renderer_resources
                .draw_image_descriptor_set_handle
                .pipeline_layout;
            device.destroy_pipeline_layout(Some(pipeline_layout));

            let pipeline_layout = renderer_resources
                .white_image_descriptor_set_handle
                .pipeline_layout;
            device.destroy_pipeline_layout(Some(pipeline_layout));

            let descriptor_set_layout = draw_image_desciptor_buffer
                .descriptor_set_layout_handle
                .descriptor_set_layout;
            device.destroy_descriptor_set_layout(Some(descriptor_set_layout));
            let descriptor_set_layout = white_image_desciptor_buffer
                .descriptor_set_layout_handle
                .descriptor_set_layout;
            device.destroy_descriptor_set_layout(Some(descriptor_set_layout));

            let draw_image_descriptor_buffer_raw =
                vk::raw::Buffer::from_raw(draw_image_desciptor_buffer.buffer.buffer.as_raw());
            let mut allocation = draw_image_desciptor_buffer.buffer.allocation;
            vulkan_context_resource
                .allocator
                .destroy_buffer(draw_image_descriptor_buffer_raw, &mut allocation);

            let white_image_descriptor_buffer_raw =
                vk::raw::Buffer::from_raw(white_image_desciptor_buffer.buffer.buffer.as_raw());
            let mut allocation = white_image_desciptor_buffer.buffer.allocation;
            vulkan_context_resource
                .allocator
                .destroy_buffer(white_image_descriptor_buffer_raw, &mut allocation);

            renderer_resources
                .mesh_buffers
                .iter_mut()
                .for_each(|mesh_buffer| {
                    vulkan_context_resource.allocator.destroy_buffer(
                        *mesh_buffer.vertex_buffer.buffer,
                        &mut mesh_buffer.vertex_buffer.allocation,
                    );
                    vulkan_context_resource.allocator.destroy_buffer(
                        *mesh_buffer.vertex_indices_buffer.buffer,
                        &mut mesh_buffer.vertex_indices_buffer.allocation,
                    );
                    vulkan_context_resource.allocator.destroy_buffer(
                        *mesh_buffer.meshlets_buffer.buffer,
                        &mut mesh_buffer.meshlets_buffer.allocation,
                    );
                    vulkan_context_resource.allocator.destroy_buffer(
                        *mesh_buffer.local_indices_buffer.buffer,
                        &mut mesh_buffer.local_indices_buffer.allocation,
                    );
                });
            device.destroy_pipeline_layout(Some(renderer_resources.mesh_pipeline_layout));

            let draw_image_raw =
                vk::raw::Image::from_raw(renderer_resources.draw_image.image.as_raw());
            vulkan_context_resource.allocator.destroy_image(
                draw_image_raw,
                &mut renderer_resources.draw_image.allocation,
            );
            let depth_image_raw =
                vk::raw::Image::from_raw(renderer_resources.depth_image.image.as_raw());
            vulkan_context_resource.allocator.destroy_image(
                depth_image_raw,
                &mut renderer_resources.depth_image.allocation,
            );
            let white_image_raw =
                vk::raw::Image::from_raw(renderer_resources.white_image.image.as_raw());
            vulkan_context_resource.allocator.destroy_image(
                white_image_raw,
                &mut renderer_resources.white_image.allocation,
            );
            drop(vulkan_context_resource.allocator);

            device.destroy_shader_ext(Some(
                renderer_resources.gradient_compute_shader_object.shader,
            ));
            device.destroy_shader_ext(Some(renderer_resources.mesh_shader_object.shader));
            device.destroy_shader_ext(Some(renderer_resources.fragment_shader_object.shader));

            device.destroy_command_pool(Some(
                render_context_resource
                    .upload_context
                    .command_group
                    .command_pool,
            ));
            device.destroy_fence(Some(
                render_context_resource.upload_context.command_group.fence,
            ));

            render_context_resource
                .frames_data
                .iter()
                .for_each(|frame_data| {
                    device.destroy_command_pool(Some(frame_data.command_group.command_pool));
                    device.destroy_fence(Some(frame_data.command_group.fence));
                    device.destroy_semaphore(Some(frame_data.render_semaphore));
                    device.destroy_semaphore(Some(frame_data.swapchain_semaphore));
                });

            render_context_resource
                .image_views
                .iter()
                .for_each(|image_view| {
                    vulkan_context_resource
                        .device
                        .destroy_image_view(Some(*image_view));
                });

            if let Some(debug_utils_messenger) = vulkan_context_resource.debug_utils_messenger {
                vulkan_context_resource
                    .instance
                    .destroy_debug_utils_messenger_ext(Some(debug_utils_messenger));
            }

            vulkan_context_resource
                .device
                .destroy_swapchain_khr(Some(vulkan_context_resource.swapchain));
            vulkan_context_resource.device.destroy();
            vulkan_context_resource
                .instance
                .destroy_surface_khr(Some(vulkan_context_resource.surface));
            vulkan_context_resource.instance.destroy();
        }
    }
}
