mod components;
mod descriptors;
mod events;
mod id;
mod resources;
mod setup;
mod systems;
mod utils;

use std::path::PathBuf;

use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    world::World,
};
use vulkanite::{
    Handle,
    vk::{self, rs::Device},
};
use winit::{event::ElementState, keyboard::KeyCode, window::Window};

use crate::engine::{
    components::{camera::Camera, time::Time},
    events::LoadModelEvent,
    resources::{
        AllocatedBuffer, AllocatedImage, FrameContext, RendererContext, RendererResources,
        VulkanContextResource,
    },
    systems::{
        begin_rendering, collect_instance_objects, end_rendering,
        general::{update_camera, update_time},
        on_load_model, on_spawn_mesh, prepare_frame, present, propogate_transforms, render_meshes,
        update_resources,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct ScheduleWorldUpdate;

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct ScheduleRendererUpdate;

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

        let renderer_resources = Self::create_renderer_resources(&mut world);
        world.insert_resource(renderer_resources);

        let frame_context = FrameContext::default();
        world.insert_resource(frame_context);

        world.insert_resource(Camera::new(0.05, 0.5));

        let mut world_schedule = Schedule::new(ScheduleWorldUpdate);
        world_schedule.add_systems((
            propogate_transforms,
            update_time::update_time,
            update_camera::update_camera.after(update_time::update_time),
        ));

        let mut renderer_schedule = Schedule::new(ScheduleRendererUpdate);
        renderer_schedule.add_systems((
            prepare_frame::prepare_frame,
            collect_instance_objects::collect_instance_objects.after(prepare_frame::prepare_frame),
            update_resources::update_resources
                .after(collect_instance_objects::collect_instance_objects),
            begin_rendering::begin_rendering.after(update_resources::update_resources),
            render_meshes::render_meshes.after(begin_rendering::begin_rendering),
            end_rendering::end_rendering.after(render_meshes::render_meshes),
            present::present.after(render_meshes::render_meshes),
        ));

        world.add_schedule(world_schedule);
        world.add_schedule(renderer_schedule);

        world.add_observer(on_load_model::on_load_model);
        world.add_observer(on_spawn_mesh::on_spawn_mesh);

        // TODO: TEMP
        world.trigger(LoadModelEvent {
            path: PathBuf::from(r"assets/helmet.glb"),
        });

        world.insert_resource(Time::new());

        Self { world }
    }

    pub fn update(&mut self) {
        self.world.flush();
        self.world.run_schedule(ScheduleWorldUpdate);
        self.world.run_schedule(ScheduleRendererUpdate);
    }

    pub fn process_input(&mut self, key_code: KeyCode, state: ElementState) {
        let mut camera = unsafe { self.world.get_resource_mut::<Camera>().unwrap_unchecked() };
        camera.process_keycode(key_code, state);
    }

    pub fn process_mouse(&mut self, mouse_delta: (f32, f32)) {
        let mut camera = unsafe { self.world.get_resource_mut::<Camera>().unwrap_unchecked() };
        camera.process_mouse(mouse_delta.0, mouse_delta.1);
    }

    unsafe fn destroy_buffer(
        &self,
        allocator: &vma::Allocator,
        allocated_buffer: &mut AllocatedBuffer,
    ) {
        let allocation = &mut allocated_buffer.allocation;
        unsafe {
            let buffer_raw = vk::raw::Buffer::from_raw(allocated_buffer.buffer.as_raw());
            allocator.destroy_buffer(buffer_raw, allocation);
        }
    }

    unsafe fn destroy_image(
        &self,
        device: Device,
        allocator: &vma::Allocator,
        allocated_image: &mut AllocatedImage,
    ) {
        let allocation = &mut allocated_image.allocation;
        unsafe {
            device.destroy_image_view(Some(allocated_image.image_view));

            let image_raw = vk::raw::Image::from_raw(allocated_image.image.as_raw());
            allocator.destroy_image(image_raw, allocation);
        }
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

        let device = vulkan_context_resource.device;

        device.wait_idle().unwrap();

        unsafe {
            let allocated_buffer = &mut renderer_resources.resources_descriptor_set_handle.buffer;
            self.destroy_buffer(&vulkan_context_resource.allocator, allocated_buffer);

            device.destroy_pipeline_layout(Some(
                renderer_resources
                    .resources_descriptor_set_handle
                    .pipeline_layout,
            ));

            device.destroy_descriptor_set_layout(Some(
                renderer_resources
                    .resources_descriptor_set_handle
                    .descriptor_set_layout_handle
                    .descriptor_set_layout,
            ));

            renderer_resources
                .get_samplers_iter()
                .for_each(|sampler_object| {
                    device.destroy_sampler(Some(sampler_object.sampler));
                });
            renderer_resources
                .get_textures_iter_mut()
                .for_each(|allocated_image| {
                    self.destroy_image(device, &vulkan_context_resource.allocator, allocated_image);
                });
            renderer_resources
                .resources_pool
                .memory_bucket
                .free_allocations();

            vulkan_context_resource.allocator.drop();

            device.destroy_shader_ext(Some(
                renderer_resources.gradient_compute_shader_object.shader,
            ));
            device.destroy_shader_ext(Some(renderer_resources.mesh_shader_object.shader));
            device.destroy_shader_ext(Some(renderer_resources.task_shader_object.shader));
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
