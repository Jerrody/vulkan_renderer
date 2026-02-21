use std::path::PathBuf;

use bevy_ecs::{
    component::Component,
    system::{Commands, Query, Res},
};
use engine::{
    GamePlugin,
    engine::{Camera, ClippingPlanes, Input, LoadModelEvent, Time, Transform},
};
use glam::FloatExt;
use winit::keyboard::KeyCode;

#[unsafe(no_mangle)]
pub extern "Rust" fn get_game() -> Box<dyn GamePlugin> {
    Box::new(Game)
}

struct Game;

impl GamePlugin for Game {
    fn add_systems_init(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems((spawn_planet, spawn_asteroid, spawn_player));
    }

    fn add_systems_update(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems((move_player, rotate_player, jump_player));
    }
}

#[derive(Component)]
#[require(Camera)]
pub struct PlayerStats {
    pub move_speed: f32,
    pub run_speed: f32,
    pub rotation_speed: f32,
}

#[derive(Default, Component)]
pub struct PlayerJump {
    pub jump_duration: f32,
    pub jump_height: f32,
    pub current_duration: f32,
    pub initial_y_height: f32,
    pub is_jumping: bool,
    pub is_falling: bool,
}

#[derive(Component)]
#[require(Transform)]
struct BulletTag;

#[derive(Component)]
#[require(Transform)]
pub struct PlanetTag;

fn spawn_planet(mut commands: Commands) {
    // TODO: Deduplicate and simplify.
    let mut exe_path = std::env::current_exe().unwrap();

    exe_path.pop();
    exe_path.pop();
    exe_path.pop();

    let planet_scale = 1.0;
    let mut planet_transform = Transform::IDENTITY;
    planet_transform.local_scale *= planet_scale;

    let planet_entity = commands.spawn((PlanetTag, planet_transform));
    let planet_entity_id = planet_entity.id();

    commands.trigger(LoadModelEvent {
        path: PathBuf::from(std::format!(
            "{}/assets/planet.glb",
            exe_path.as_os_str().display()
        )),
        parent_entity: Some(planet_entity_id),
    });

    let asteroid = 1.0;
    let mut asteroid_transform = Transform::IDENTITY;
    asteroid_transform.local_scale *= asteroid;

    let asteroid_entity = commands.spawn((PlanetTag, asteroid_transform));
    let asteroid_entity_id = asteroid_entity.id();

    commands.trigger(LoadModelEvent {
        path: PathBuf::from(std::format!(
            "{}/assets/asteroid.glb",
            exe_path.as_os_str().display()
        )),
        parent_entity: Some(asteroid_entity_id),
    });
}

fn spawn_asteroid(mut commands: Commands) {
    return;
    // TODO: Deduplicate and simplify.
    let mut exe_path = std::env::current_exe().unwrap();

    exe_path.pop();
    exe_path.pop();
    exe_path.pop();

    let asteroid = 1.0;
    let mut asteroid_transform = Transform::IDENTITY;
    asteroid_transform.local_scale *= asteroid;

    let asteroid_entity = commands.spawn((PlanetTag, asteroid_transform));
    let asteroid_entity_id = asteroid_entity.id();

    commands.trigger(LoadModelEvent {
        path: PathBuf::from(std::format!(
            "{}/assets/asteroid.glb",
            exe_path.as_os_str().display()
        )),
        parent_entity: Some(asteroid_entity_id),
    });
}

fn spawn_player(mut commands: Commands) {
    let camera_component = Camera {
        fov: 75.0,
        clipping_planes: ClippingPlanes {
            near: 0.1,
            far: 1000.0,
        },
    };
    let player_stats_component = PlayerStats {
        move_speed: 5.0,
        run_speed: 15.0,
        rotation_speed: 5.0,
    };

    let player_jump = PlayerJump {
        jump_duration: 0.9,
        jump_height: 4.0,
        ..Default::default()
    };

    let mut player_entity = commands.spawn_empty();
    let mut player_transform = Transform::IDENTITY;
    /*     player_transform.local_position.z = 150.0;
    player_transform.local_position.y = -5.0; */

    player_entity.insert((
        camera_component,
        player_stats_component,
        player_jump,
        player_transform,
    ));
}

fn move_player(
    mut player_query: Query<(&mut Transform, &PlayerStats, &PlayerJump)>,
    time: Res<Time>,
    input: Res<Input>,
) {
    let delta_time = time.get_delta_time();

    let (mut transform, player_stats, player_jump) = player_query.single_mut().unwrap();

    let target_speed = if input.pressed(KeyCode::ShiftLeft) && !player_jump.is_jumping {
        player_stats.run_speed
    } else {
        player_stats.move_speed
    };

    let forward = transform.forward();
    let right = transform.right();
    if input.pressed(KeyCode::KeyW) {
        transform.local_position += forward * target_speed * delta_time;
    }

    if input.pressed(KeyCode::KeyS) {
        transform.local_position -= forward * target_speed * delta_time;
    }

    if input.pressed(KeyCode::KeyA) {
        transform.local_position -= right * target_speed * delta_time;
    }

    if input.pressed(KeyCode::KeyD) {
        transform.local_position += right * target_speed * delta_time;
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
    angles.x = angles.x.clamp(-90.0, 90.0);

    transform.set_local_euler_angles(angles);
}

fn jump_player(
    mut player_query: Query<(&mut Transform, &mut PlayerJump)>,
    time: Res<Time>,
    input: Res<Input>,
) {
    let delta_time = time.get_delta_time();

    let (mut transform, mut player_jump) = player_query.single_mut().unwrap();

    if player_jump.is_jumping {
        player_jump.current_duration += delta_time;

        let in_start = if player_jump.is_falling {
            player_jump.jump_duration / 2.0
        } else {
            Default::default()
        };

        let in_end = if player_jump.is_falling {
            player_jump.jump_duration
        } else {
            player_jump.jump_duration / 2.0
        };

        let jump_height = player_jump.initial_y_height + player_jump.jump_height;
        let out_start = if player_jump.is_falling {
            jump_height
        } else {
            player_jump.initial_y_height
        };

        let out_end = if player_jump.is_falling {
            player_jump.initial_y_height
        } else {
            jump_height
        };

        let new_height = player_jump
            .current_duration
            .remap(in_start, in_end, out_start, out_end);
        transform.local_position.y = new_height;
        player_jump.is_falling = player_jump.current_duration > player_jump.jump_duration / 2.0;

        if player_jump.is_falling {
            if f32::abs(transform.local_position.y - player_jump.initial_y_height) < 0.1 {
                player_jump.is_jumping = false;
                player_jump.is_falling = false;
            }
        }
    } else {
        if input.just_pressed(KeyCode::Space) {
            player_jump.is_jumping = true;
            player_jump.is_falling = false;
            player_jump.current_duration = Default::default();
            player_jump.initial_y_height = transform.local_position.y;
        }
    }
}
