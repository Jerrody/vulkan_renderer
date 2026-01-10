use bevy_ecs::event::Event;

#[derive(Event)]
pub struct LoadModelEvent {
    pub path: String,
}
