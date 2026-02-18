use std::path::PathBuf;

use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Query, Res},
};
use engine::{
    GamePlugin,
    engine::{Camera, ClippingPlanes, Input, LoadModelEvent, Time, Transform},
};
use glam::Vec3;
use winit::keyboard::KeyCode;

#[unsafe(no_mangle)]
pub extern "Rust" fn get_game() -> Box<dyn GamePlugin> {
    Box::new(Game)
}

struct Game;

impl GamePlugin for Game {
    fn add_systems_init(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems((spawn_scene, spawn_entity));
    }

    fn add_systems_update(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems((move_player, rotate_player));
    }
}

#[derive(Component)]
#[require(Camera)]
pub struct PlayerStats {
    pub move_speed: f32,
    pub rotation_speed: f32,
}

#[derive(Component)]
#[require(Transform)]
struct BulletTag;

fn spawn_scene(mut commands: Commands) {
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
        parent_entity: None,
    });
}

fn spawn_entity(mut commands: Commands) {
    let camera_component = Camera {
        fov: 75.0,
        clipping_planes: ClippingPlanes {
            near: 0.1,
            far: 1000.0,
        },
    };
    let player_stats_component = PlayerStats {
        move_speed: 5.0,
        rotation_speed: 5.0,
    };

    let mut player_entity = commands.spawn_empty();
    player_entity.insert((camera_component, player_stats_component));
}

fn move_player(
    mut player_query: Query<(&mut Transform, &PlayerStats)>,
    time: Res<Time>,
    input: Res<Input>,
) {
    let delta_time = time.get_delta_time();

    let (mut transform, player_stats) = player_query.single_mut().unwrap();

    // TODO: Something wrong with transformations, need to check later (W should be negative, but S should be positive, but it should be vice versa).
    if input.pressed(KeyCode::KeyW) {
        transform.local_position.z -= player_stats.move_speed * delta_time;
    }

    if input.pressed(KeyCode::KeyS) {
        transform.local_position.z += player_stats.move_speed * delta_time;
    }

    if input.pressed(KeyCode::KeyA) {
        transform.local_position.x -= player_stats.move_speed * delta_time;
    }

    if input.pressed(KeyCode::KeyD) {
        transform.local_position.x += player_stats.move_speed * delta_time;
    }
}

fn rotate_player(
    mut player_query: Query<(&mut Transform, &PlayerStats)>,
    time: Res<Time>,
    input: Res<Input>,
) {
    let delta_time = time.get_delta_time();
    let mouse_axis = input.get_mouse_axis();

    let (mut transform, player_stats) = player_query.single_mut().unwrap();

    let mut angles = transform.get_local_euler_angles();

    angles.y -= player_stats.rotation_speed * mouse_axis.x * delta_time;

    angles.x += player_stats.rotation_speed * mouse_axis.y * delta_time;

    //angles.x = angles.x.clamp(-90.0, 90.0);

    transform.set_local_euler_angles(angles);
}
