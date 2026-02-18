use std::path::Path;
use std::process::Command;
use std::{env, fs, io, path::PathBuf};

use cargo_metadata::MetadataCommand;

fn main() -> io::Result<()> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to get cargo metadata");

    let workspace_root = metadata.workspace_root;

    let dir = std::format!("{}/shaders/programs", workspace_root);
    println!("cargo:rerun-if-changed={}/shaders", workspace_root);

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

    fs::create_dir_all(std::format!("{}/intermediate/shaders", workspace_root))?;

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
        let output_filename = format!("{}/intermediate/shaders/{}.spv", workspace_root, filename);

        let slang_status = Command::new(&slangc_path)
            .arg("-I")
            .arg(std::format!("{}/shaders", workspace_root))
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

    let sysroot = get_sysroot();

    let bin_path = sysroot.join("bin");

    let file_prefix = "std-";

    let out_dir = PathBuf::from(std::format!("{}/target/debug", workspace_root));

    if let Err(e) = find_and_copy(&bin_path, out_dir.as_path(), file_prefix) {
        println!("cargo:warning=Could not copy std dll: {}", e);
    }

    Ok(())
}

fn get_sysroot() -> PathBuf {
    let output = Command::new(env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("Failed to execute rustc to get sysroot");

    let sysroot_str = String::from_utf8(output.stdout)
        .expect("Sysroot path is not valid UTF-8")
        .trim()
        .to_string();

    PathBuf::from(sysroot_str)
}

fn find_and_copy(src_dir: &Path, dst_dir: &Path, prefix: &str) -> std::io::Result<()> {
    if !src_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && let Some(filename) = path.file_name().and_then(|s| s.to_str())
            && filename.starts_with(prefix)
            && filename.ends_with(".dll")
        {
            let dest_path = dst_dir.join(filename);

            if !dest_path.exists() {
                fs::copy(&path, &dest_path)?;
            }

            break;
        }
    }
    Ok(())
}
