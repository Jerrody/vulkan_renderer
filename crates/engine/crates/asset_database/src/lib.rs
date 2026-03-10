use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct AssetDatabase {}

impl AssetDatabase {
    pub fn new() -> Self {
        AssetDatabase {}
    }
}
