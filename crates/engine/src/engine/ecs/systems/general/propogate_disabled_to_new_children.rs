use bevy_ecs::{
    entity::Entity,
    entity_disabling::Disabled,
    query::{With, Without},
    system::{Commands, Query},
};

use crate::engine::Parent;

pub fn propagate_disabled_to_new_children_system(
    mut commands: Commands,
    active_children: Query<(Entity, &Parent), Without<Disabled>>,
    disabled_parents: Query<(), With<Disabled>>,
) {
    active_children.iter().for_each(|(child_entity, parent)| {
        if disabled_parents.contains(parent.0) {
            commands.entity(child_entity).insert(Disabled);
        }
    });
}
