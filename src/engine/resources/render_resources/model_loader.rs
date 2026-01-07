use asset_importer::{Importer, mesh::Mesh, postprocess::PostProcessSteps, scene::MeshIterator};
use bevy_ecs::entity::{hash_set::IntoIter, index_set::Iter};

pub struct MeshAsset {}

pub struct ModelAsset {}

pub struct ModelLoader {
    importer: Importer,
}

impl ModelLoader {
    pub fn new() -> Self {
        Self {
            importer: Importer::new(),
        }
    }

    pub fn load_model<'a>(&self, path: &'a str) -> MeshIterator {
        let model = self
            .importer
            .read_file(path)
            .with_post_process(PostProcessSteps::REALTIME)
            .import()
            .unwrap();

        model.meshes().into_iter()
    }
}
