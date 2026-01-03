use std::process::Command;
use std::{fs, io};

fn main() -> io::Result<()> {
    let dir = "shaders";

    println!("cargo:rerun-if-changed=shaders");
    let mut file_paths = Vec::new();

    fs::create_dir_all("shaders/output")?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let shader_types = ["slang"];
        if path.parent().unwrap().to_str().unwrap() != "ignore_for_compilation"
            && path.is_file()
            && shader_types.contains(&path.extension().and_then(|s| s.to_str()).unwrap())
        {
            let shader_extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap()
                .to_string()
                .clone();
            file_paths.push((path, shader_extension));
        }
    }

    for file_path in &file_paths {
        let filename = file_path.0.file_name().unwrap().to_str().unwrap();
        let status = Command::new("bin/slangc")
            // generate SPIR-V
            .arg("-target")
            .arg("spirv")
            .arg("-O3")
            // optionally force direct SPIR-V emission (avoids GLSL path)
            // .arg("-emit-spirv-directly")
            // specify output .spv path
            .arg("-o")
            .arg(std::format!("shaders/output/{}.spv", filename))
            // input Slang source path (the same path you used before)
            .arg(std::format!("shaders/{}", filename))
            // optional: if your Slang module has a non-default entry point name, pass -entry
            // .arg("-entry").arg("computeMain")
            .status()?;

        println!("Compiled shader: {:?} | Status: {:?}", file_path, status);
    }

    Ok(())
}
