use asset_importer::{Importer, postprocess::PostProcessSteps};

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

    pub fn load_model<'a>(&self, path: &'a str) {
        let model = self
            .importer
            .read_file(path)
            .with_post_process(PostProcessSteps::REALTIME)
            .import()
            .unwrap();
        for mesh in model.meshes() {
            let vertices = mesh.vertices();
            println!("Mesh has {} vertices (copied)", vertices.len());
        }
    }
}
