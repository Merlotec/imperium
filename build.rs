
extern crate glsl_to_spirv;

use std::error::Error;
use std::io::Read;
use glsl_to_spirv::ShaderType;

fn main() -> Result<(), Box<Error>> {
    // Tell the build script to only run again if we change our source shaders
    println!("cargo:rerun-if-changed=shaders/src");

    // Create destination path if necessary
    std::fs::create_dir_all("shaders/bin")?;

    for entry in std::fs::read_dir("shaders/src")? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let in_path = entry.path();

            let mut shader_type = ShaderType::Vertex;
            // Support only vertex and fragment shaders currently
            if let Some(name) = in_path.file_name() {
                if let Some(str_name) = name.to_str() {
                    if str_name.contains("_v") {
                        shader_type = ShaderType::Vertex;
                    } else if str_name.contains("_f") {
                        shader_type = ShaderType::Fragment;
                    }
                }
            }

            let source = std::fs::read_to_string(&in_path)?;
            let mut compiled_file = glsl_to_spirv::compile(&source, shader_type)?;
            // Read the binary data from the compiled file
            let mut compiled_bytes = Vec::new();
            compiled_file.read_to_end(&mut compiled_bytes)?;
            let path_str = in_path.file_name().unwrap().to_str().unwrap();
            let use_str = path_str.replace(&(String::from(".") + in_path.extension().unwrap().to_str().unwrap()), "");

            // Determine the output path based on the input name
            let out_path = format!(
                "shaders/bin/{}.spv",
                use_str
            );

            std::fs::write(&out_path, &compiled_bytes)?;

        }
    }

    Ok(())
}