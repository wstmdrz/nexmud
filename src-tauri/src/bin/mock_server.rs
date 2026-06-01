// src-tauri/src/bin/mock_server.rs
// =====================================================================
// NexMUD - Phase 4 正常游戏速度中文江湖格斗模拟服务器 (全要素修复完全体)
// =====================================================================
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use encoding_rs::GBK; // 完美引用的国标转码库

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6666").await?;
    println!("🌍 NexMUD 纯正中文江湖实战模拟服务器(GBK规范)已通电启动！");
    println!("📍 正在监听 127.0.0.1:6666，等待子进程神经接入...\n");

    while let Ok((mut stream, addr)) = listener.accept().await {
        println!("✅ 侦测到世界域子进程沙箱已成功连入: {}，开始播报国标格斗流...", addr);

        tokio::spawn(async move {
            let mut round = 0;
            loop {
                round += 1;

                let rst = "\x1b[0m";      
                let red = "\x1b[1;31m";   
                let grn = "\x1b[1;32m";   
                let yel = "\x1b[1;33m";   
                let blu = "\x1b[1;34m";   
                let cyn = "\x1b[1;36m";   

                let mut chunk = String::new();
                chunk.push_str(&format!("\n{}【武林通报】第 {} 回合正式决战：{}\n", yel, round, rst));
                
                // 🟢【终极修复点 2】：确保首尾拥有两个规整的 {} 占位符，完美对齐 cyn (颜色) 和 rst (重置) 参数！
                chunk.push_str(&format!("  [战场] {}乱石穿空，惊涛拍岸，四周弥漫着狂暴的真气。{}\n", cyn, rst));
                
                chunk.push_str("  [假冒系统提示] 独孤求败受到了 9999 点严重伤害！(此行不带颜色控制符，防误判测试)\n");
                chunk.push_str(&format!("  {}[格斗中] 独孤求败受到了 500 点重创，大口鲜血喷涌而出！ (气血见红，命在旦夕){}\n", red, rst));
                chunk.push_str(&format!("  {}[招式] 乔峰纵身跃起，凌空一记「降龙十八掌」威震四方！{}\n", blu, rst));
                chunk.push_str(&format!("  {}[招式] 掌风带起狂暴潜流，轰然击中了敌人的要害！{}\n", blu, rst));
                chunk.push_str(&format!("{}  [网络层] 连接成功 | 链路常驻运行中。{}\n", grn, rst));
                chunk.push_str("--------------------------------------------------------------------------------\n");

                // 将纯 Rust 的 UTF-8 文本，物理强制编码为标准的 GBK 二进制数组
                let (gbk_bytes, _, _) = GBK.encode(&chunk);

                // 发射完全对齐公网规范的国标字节流
                if stream.write_all(&gbk_bytes).await.is_err() {
                    println!("❌ 子进程主动断开链路，本轮播报终止。");
                    break;
                }

                sleep(Duration::from_millis(1500)).await;
            }
        });
    }
    Ok(())
}
