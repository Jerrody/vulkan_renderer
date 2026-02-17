use std::path::PathBuf;

use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Query, Res},
};
use engine::{
    GamePlugin,
    engine::{LoadModelEvent, Time, Transform},
};
use glam::Vec3;

#[unsafe(no_mangle)]
pub extern "Rust" fn get_game() -> Box<dyn GamePlugin> {
    Box::new(Game)
}

struct Game;

impl GamePlugin for Game {
    fn add_systems_init(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems(spawn_entity);
    }

    fn add_systems_update(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems(move_slowly_entity);
    }
}

#[derive(Component)]
#[require(Transform)]
struct BulletTag;

fn spawn_entity(mut commands: Commands) {
    let mut bullet_entity = commands.spawn_empty();
    bullet_entity.insert(BulletTag);

    let bullet_entity_id = bullet_entity.id();

    // TODO: Deduplicate and simplify.
    let mut exe_path = std::env::current_exe().unwrap();

    exe_path.pop();
    exe_path.pop();
    exe_path.pop();

    commands.trigger(LoadModelEvent {
        path: PathBuf::from(std::format!(
            "{}/assets/structure.glb",
            exe_path.as_os_str().display()
        )),
        parent_entity: Some(bullet_entity_id),
    });
}

fn move_slowly_entity(mut bullets: Query<&mut Transform, With<BulletTag>>, time: Res<Time>) {
    let delta_time = time.get_delta_time();

    for mut transform in bullets.iter_mut() {
        transform.position += Vec3::new(1.0 * delta_time, 0.0, 0.0);
    }
}
