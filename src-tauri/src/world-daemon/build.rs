// src-tauri/src/world-daemon/build.rs
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    
    // 注意：因为此文件在 src-tauri/src/world-daemon/ 下，
    // 寻找根目录的 proto 文件夹需要向上返回 3 层 (../../../proto)
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("world_descriptor.bin")) 
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../../proto/broker.proto", "../../../proto/world.proto"],
            &["../../../proto"],
        )?;
    Ok(())
}