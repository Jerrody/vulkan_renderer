use bevy_ecs::{
    query::{With, Without},
    relationship::RelationshipTarget,
    system::Query,
};

use crate::engine::components::transform::{Children, GlobalTransform, Parent, Transform};

pub fn propogate_transforms_system(
    mut root_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), Without<Parent>>,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    let mut stack = Vec::with_capacity(child_query.iter().len());

    for (transform, mut global_transform, children) in root_query.iter_mut() {
        let matrix = transform.local_to_world_matrix();

        global_transform.0 = matrix;

        if let Some(children) = children {
            for child in children.iter() {
                stack.push((child, matrix));
            }
        }
    }

    while let Some((child_entity, parent_matrix)) = stack.pop() {
        if let Ok((transform, mut global_transform, children)) = child_query.get_mut(child_entity) {
            let local_matrix = transform.local_to_world_matrix();
            let child_global_matrix = parent_matrix * local_matrix;

            global_transform.0 = child_global_matrix;

            if let Some(children) = children {
                for next_child in children.iter() {
                    stack.push((next_child, child_global_matrix));
                }
            }
        }
    }
}
