use bevy_ecs::component::Component;

use crate::engine::id::Id;

#[derive(Component)]
pub struct Mesh {
    pub buffer_id: Id,
    pub texture_id: Id,
}
