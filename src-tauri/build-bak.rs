use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    
    tonic_build::configure()
        // 关键：告诉编译器把 proto 结构打包成二进制描述文件，供反射使用
        .file_descriptor_set_path(out_dir.join("world_descriptor.bin")) 
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../proto/broker.proto", "../proto/world.proto"],
            &["../proto"],
        )?;
    Ok(())
}