use bevy_ecs::system::Query;

use crate::engine::components::transform::Transform;

pub fn update_instance_objects(transforms_query: Query<&Transform>) {
    for transform in transforms_query.iter() {}
}
