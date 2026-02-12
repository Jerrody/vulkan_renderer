mod ecs;
mod events;
mod general;
mod id;
mod setup;
mod utils;

use ecs::*;

use std::path::PathBuf;

use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    world::World,
};
use vulkanite::{Handle, vk};
use winit::{event::ElementState, keyboard::KeyCode, window::Window};

use crate::engine::{
    components::{camera::Camera, time::Time},
    ecs::{
        buffers_pool::{AllocatedBuffer, BuffersPool},
        general::{update_camera, update_time},
        samplers_pool::SamplersPool,
        textures_pool::TexturesPool,
    },
    events::LoadModelEvent,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct SchedulerWorldUpdate;
#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct SchedulerRendererSetup;

#[derive(Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel, Debug)]
struct SchedulerRendererUpdate;

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

        let mut world_schedule = Schedule::new(SchedulerWorldUpdate);
        world_schedule.add_systems((
            propogate_transforms_system,
            update_time::update_time_system,
            update_camera::update_camera_system.after(update_time::update_time_system),
        ));

        let mut scheduler_renderer_update = Schedule::new(SchedulerRendererUpdate);
        scheduler_renderer_update.add_systems((
            prepare_frame::prepare_frame_system,
            collect_instance_objects::collect_instance_objects_system
                .after(prepare_frame::prepare_frame_system),
            update_resources::update_resources_system
                .after(collect_instance_objects::collect_instance_objects_system),
            begin_rendering::begin_rendering_system
                .after(update_resources::update_resources_system),
            render_meshes::render_meshes_system.after(begin_rendering::begin_rendering_system),
            end_rendering::end_rendering_system.after(render_meshes::render_meshes_system),
            present::present_system.after(render_meshes::render_meshes_system),
        ));

        let scheduler_renderer_setup = Schedule::new(SchedulerRendererSetup);

        world.add_schedule(world_schedule);
        world.add_schedule(scheduler_renderer_update);
        world.add_schedule(scheduler_renderer_setup);

        world.add_observer(on_load_model::on_load_model_system);
        world.add_observer(on_spawn_mesh::on_spawn_mesh_system);

        // TODO: TEMP
        world.trigger(LoadModelEvent {
            path: PathBuf::from(r"assets/sponza_variation_02.glb"),
        });

        world.insert_resource(Time::new());

        world.run_schedule(SchedulerRendererSetup);

        Self { world }
    }

    pub fn update(&mut self) {
        self.world.flush();
        self.world.run_schedule(SchedulerWorldUpdate);
        self.world.run_schedule(SchedulerRendererUpdate);
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
}

impl Drop for Engine {
    fn drop(&mut self) {
        let mut vulkan_context_resource = self
            .world
            .remove_resource::<VulkanContextResource>()
            .unwrap();
        let render_context_resource = self.world.remove_resource::<RendererContext>().unwrap();
        let mut buffers_pool = self.world.remove_resource::<BuffersPool>().unwrap();
        let mut textures_pool = self.world.remove_resource::<TexturesPool>().unwrap();
        let mut samplers_pool = self.world.remove_resource::<SamplersPool>().unwrap();
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

            buffers_pool.free_allocations();
            textures_pool.free_allocations();
            samplers_pool.destroy_samplers();

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
