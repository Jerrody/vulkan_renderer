use bevy_ecs::{
    entity::Entity,
    entity_disabling::Disabled,
    hierarchy::ChildOf,
    query::{With, Without},
    system::{Commands, Query},
};

pub fn propagate_disabled_to_new_children_system(
    mut commands: Commands,
    active_children: Query<(Entity, &ChildOf), Without<Disabled>>,
    disabled_parents: Query<(), With<Disabled>>,
) {
    active_children.iter().for_each(|(child_entity, parent)| {
        if disabled_parents.contains(parent.0) {
            commands.entity(child_entity).insert(Disabled);
        }
    });
}
