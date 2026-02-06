use asset_importer::{Importer, Scene, postprocess::PostProcessSteps};

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

    pub fn load_model<'a>(&self, path: &'a str) -> Scene {
        let scene = self
            .importer
            .read_file(path)
            .with_post_process(PostProcessSteps::REALTIME | PostProcessSteps::FLIP_UVS)
            .import()
            .unwrap();

        scene
    }
}
