use bevy_ecs::world::World;

use crate::engine::{
    Engine,
    resources::{
        buffers_pool::BuffersPool, model_loader::ModelLoader, samplers_pool::SamplersPool,
        textures_pool::TexturesPool, *,
    },
};

impl Engine {
    pub fn prepare_renderer_resources(world: &mut World) {
        let vulkan_context = world.get_resource_ref::<VulkanContextResource>().unwrap();
        let render_context = world.get_resource_ref::<RendererContext>().unwrap();

        let resources_pool = ResourcesPool::new();

        let upload_command_group = render_context.upload_context.command_group;

        let device = vulkan_context.device;
        let allocator = vulkan_context.allocator;

        let renderer_resources = RendererResources {
            default_texture_reference: Default::default(),
            fallback_texture_reference: Default::default(),
            default_sampler_reference: Default::default(),
            mesh_objects_buffer_reference: Default::default(),
            resources_descriptor_set_handle: Default::default(),
            gradient_compute_shader_object: Default::default(),
            task_shader_object: Default::default(),
            mesh_shader_object: Default::default(),
            fragment_shader_object: Default::default(),
            model_loader: ModelLoader::new(),
            resources_pool,
            is_printed_scene_hierarchy: true,
        };

        let buffers_pool = BuffersPool::new(
            device,
            allocator,
            upload_command_group,
            vulkan_context.transfer_queue,
        );
        let textures_pool = TexturesPool::new(device, vulkan_context.allocator);
        let samplers_pool = SamplersPool::new(device);

        world.insert_resource(renderer_resources);
        world.insert_resource(buffers_pool);
        world.insert_resource(samplers_pool);
        world.insert_resource(textures_pool);
    }
}
