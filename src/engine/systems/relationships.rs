use bevy_ecs::{
    component::ComponentId,
    entity::Entity,
    lifecycle::HookContext,
    query::Without,
    system::{ParamSet, Query},
    world::{self, DeferredWorld},
};
use glam::Mat4;

use crate::engine::components::transform::{Children, GlobalTransform, Parent, Transform};

pub fn propogate_transforms(
    root_query: Query<(Entity, &Transform), Without<Parent>>,
    children_query: Query<&Children>,
    mut transforms: ParamSet<(Query<&mut GlobalTransform>, Query<&Transform>)>,
) {
    let mut stack = Vec::new();

    for (entity, transform) in root_query.iter() {
        let matrix = transform.get_matrix();

        if let Ok(mut global_transform) = transforms.p0().get_mut(entity) {
            global_transform.0 = matrix;
        }

        for &children in children_query.get(entity).iter() {
            for child in children.0.iter() {
                stack.push((*child, matrix));
            }
        }
    }

    while let Some((child_entity, parent_matrix)) = stack.pop() {
        let local_matrix = if let Ok(transform) = transforms.p1().get(child_entity) {
            transform.get_matrix()
        } else {
            Mat4::IDENTITY
        };

        let child_global_matrix = parent_matrix * local_matrix;
        if let Ok(mut child_global_transform) = transforms.p0().get_mut(child_entity) {
            child_global_transform.0 = child_global_matrix;
        }

        if let Ok(children) = children_query.get(child_entity) {
            for &child in children.0.iter() {
                stack.push((child, child_global_matrix));
            }
        }
    }
}

pub fn on_add_parent(mut world: DeferredWorld, hook_context: HookContext) {
    let entity = hook_context.entity;
    let parent_entity = world.get::<Parent>(entity).unwrap().0;

    if world.get::<Children>(parent_entity).is_none() {
        world
            .commands()
            .entity(parent_entity)
            .insert(Children::default());
    }

    world
        .commands()
        .entity(parent_entity)
        .entry::<Children>()
        .and_modify(move |mut children| {
            if !children.0.contains(&entity) {
                children.0.push(entity);
            }
        });
}

pub fn on_remove_parent(mut world: DeferredWorld, hook_context: HookContext) {
    let entity = hook_context.entity;
    let parent_entity = world.get::<Parent>(entity).unwrap().0;

    world
        .commands()
        .entity(parent_entity)
        .entry::<Children>()
        .and_modify(move |mut children| {
            children.0.retain(|&child_entity| child_entity != entity);
        });
}
