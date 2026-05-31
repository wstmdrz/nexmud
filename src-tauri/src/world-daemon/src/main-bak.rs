mod codec;
mod telnet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ NexMUD World Daemon 测试启动. PID: {}", std::process::id());
    
    // 连接北大侠客行
    //let mut conn = telnet::MudConnection::new("mud.pkuxkx.net:8080");
    let mut conn = telnet::MudConnection::new("127.0.0.1:8888"); // 指向 Mock Server
    
    if let Err(e) = conn.connect().await {
        eprintln!("连接发生致命错误: {}", e);
    }
    
    Ok(())
}