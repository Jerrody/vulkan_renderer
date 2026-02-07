use std::process::Command;
use std::{env, fs, io, path::PathBuf};

fn main() -> io::Result<()> {
    let dir = "shaders/programs";
    println!("cargo:rerun-if-changed=shaders");

    let sdk_path = env::var("VULKAN_SDK")
        .expect("VULKAN_SDK environment variable not found. Is the Vulkan SDK installed?");

    let sdk_bin = PathBuf::from(sdk_path).join("Bin");

    let exe_suffix = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        Default::default()
    };

    let slangc_path = sdk_bin.join(format!("slangc{}", exe_suffix));
    let spirv_opt_path = sdk_bin.join(format!("spirv-opt{}", exe_suffix));

    fs::create_dir_all("intermediate/shaders")?;

    let mut file_paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("slang")
            && !path.to_str().unwrap().contains("ignore_for_compilation")
        {
            file_paths.push(path);
        }
    }

    for path in file_paths {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let output_filename = format!("intermediate/shaders/{}.spv", filename);

        let slang_status = Command::new(&slangc_path)
            .arg("-I")
            .arg("shaders")
            .arg(format!("-DMATERIAL_TYPE={}", "UnlitMaterial"))
            .arg("-target")
            .arg("spirv")
            .arg("-profile")
            .arg("spirv_1_6")
            .arg("-fvk-use-scalar-layout")
            .arg("-emit-spirv-directly")
            .arg("-matrix-layout-column-major")
            .arg("-O3")
            .arg("-o")
            .arg(&output_filename)
            .arg(&path)
            .status()?;

        if !slang_status.success() {
            panic!("Failed to compile shader with slangc: {}", filename);
        }

        let opt_status = Command::new(&spirv_opt_path)
            .arg(&output_filename)
            .arg("-O")
            .arg("-o")
            .arg(&output_filename)
            .status()?;

        if opt_status.success() {
            println!("cargo:warning=Compiled and optimized shader: {}", filename);
        }
    }

    Ok(())
}
