// src-tauri/src/shared/codec.rs
use encoding_rs::GBK;

/// 将前端传来的 UTF-8 字符串安全转换为 MUD 要求的 GBK 字节流
pub fn to_gbk_bytes(text: &str) -> Vec<u8> {
    let (cow, _, _) = GBK.encode(text);
    // 强制转换为拥有所有权的 Vec<u8>，延长生命周期，彻底杜绝借用检查错误
    cow.into_owned() 
}

/// 将 MUD 服务器吐出的 GBK 字节流安全转换为 Rust 的 UTF-8 String
pub fn from_gbk_bytes(bytes: &[u8]) -> String {
    let (cow, _, _) = GBK.decode(bytes);
    cow.into_owned()
}