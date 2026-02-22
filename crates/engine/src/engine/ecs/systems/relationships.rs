use bevy_ecs::{
    entity::Entity,
    query::Without,
    relationship::RelationshipTarget,
    system::{ParamSet, Query},
};
use glam::Mat4;

use crate::engine::components::transform::{Children, GlobalTransform, Parent, Transform};

pub fn propogate_transforms_system(
    root_query: Query<(Entity, &Transform), Without<Parent>>,
    children_query: Query<&Children>,
    mut transforms: ParamSet<(Query<&mut GlobalTransform>, Query<&Transform>)>,
) {
    let mut stack = Vec::with_capacity(children_query.iter().len());

    for (entity, transform) in root_query.iter() {
        let matrix = transform.local_to_world_matrix();

        if let Ok(mut global_transform) = transforms.p0().get_mut(entity) {
            global_transform.0 = matrix;
        }

        for &children in children_query.get(entity).iter() {
            for entity in children.iter() {
                stack.push((entity, matrix));
            }
        }
    }

    while let Some((child_entity, parent_matrix)) = stack.pop() {
        let local_matrix = if let Ok(transform) = transforms.p1().get(child_entity) {
            transform.local_to_world_matrix()
        } else {
            Mat4::IDENTITY
        };

        let child_global_matrix = parent_matrix * local_matrix;
        if let Ok(mut child_global_transform) = transforms.p0().get_mut(child_entity) {
            child_global_transform.0 = child_global_matrix;
        }

        if let Ok(children) = children_query.get(child_entity) {
            for entity in children.iter() {
                stack.push((entity, child_global_matrix));
            }
        }
    }
}
