use asset_database::AssetDatabase;
use bevy_ecs::{
    resource::Resource,
    system::{Res, ResMut},
};

#[derive(Resource)]
pub struct Loader {}

impl Loader {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn load_assets(loader: Res<Loader>, mut asset_database: ResMut<AssetDatabase>) {}
