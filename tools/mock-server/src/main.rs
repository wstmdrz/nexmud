use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use encoding_rs::GBK;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 绑定本地端口
    let addr = "127.0.0.1:8888";
    let listener = TcpListener::bind(addr).await?;
    println!("🔥 NexMUD 高压水枪 (Mock Server) 已启动！监听地址: {}", addr);
    println!("等待被压测的客户端连接...");

    loop {
        // 接受客户端连接
        let (mut socket, peer_addr) = listener.accept().await?;
        println!("🔗 目标已锁定 (客户端已连接): {}", peer_addr);

        // 为每个连接生成一个独立的异步任务疯狂喷射数据
        tokio::spawn(async move {
            let mut counter = 0;
            
            // 模拟高频战斗刷屏 (Burst Mode)
            loop {
                // 构造一大块文本（包含各种 ANSI 颜色：红、绿、黄、加粗）
                let mut chunk = String::new();
                for _ in 0..50 { // 每次循环打包 50 行文本，模拟突发大流量
                    chunk.push_str(&format!(
                        "\x1b[1;31m[致命一击]\x1b[0m 你对 \x1b[33m测试假人\x1b[0m 造成了 \x1b[1;32m{} 点\x1b[0m 伤害！\n",
                        counter
                    ));
                    counter += 1;
                }

                // 将 UTF-8 转换为 GBK，完美模拟老牌 MUD 环境
                let (cow, _, _) = GBK.encode(&chunk);
                let gbk_bytes = cow.into_owned();

                // 发送给客户端
                if let Err(e) = socket.write_all(&gbk_bytes).await {
                    println!("❌ 目标崩溃或断开连接: {}", e);
                    break;
                }

                // 压测参数：每 10 毫秒发送 50 行（即 5000行/秒）
                // 你可以随时调小这个时间或增大 chunk，来测试前端的极限
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }
}