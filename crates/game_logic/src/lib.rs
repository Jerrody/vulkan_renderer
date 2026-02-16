use engine::GamePlugin;

#[unsafe(no_mangle)]
pub extern "Rust" fn get_game() -> Box<dyn GamePlugin> {
    Box::new(Game)
}

struct Game;

impl GamePlugin for Game {
    fn add_systems_init(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems(hello_init);
    }

    fn add_systems_update(&self, schedule: &mut bevy_ecs::schedule::Schedule) {
        schedule.add_systems(hello_update);
    }
}

fn hello_init() {
    println!("HELLO!");
}

fn hello_update() {
    println!("BRUH UPDATE");
}
