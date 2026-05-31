use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    
    // 1. 保留你原有的 gRPC 构建与反射描述文件配置
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("world_descriptor.bin")) 
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../proto/broker.proto", "../proto/world.proto"],
            &["../proto"],
        )?;

    // 2. 隔离修复点：只有当正在编译主进程载体时，才允许注入 Tauri 的 Windows GUI 链接符号
    // 这样可以彻底避免子进程（world-daemon）的符号污染导致 0xc0000139 崩溃
    if env::var("CARGO_PKG_NAME").unwrap_or_default() == "nexmud-broker" {
        tauri_build::build();
    }

    Ok(())
}
