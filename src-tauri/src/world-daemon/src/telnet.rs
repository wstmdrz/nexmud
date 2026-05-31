use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::codec::{to_gbk_bytes, from_gbk_bytes};

pub struct MudConnection {
    addr: String,
}

impl MudConnection {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在连接到 MUD 服务器: {}...", self.addr);
        let stream = TcpStream::connect(&self.addr).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);

        // 异步读取流
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => {
                        println!("\n[服务器主动断开连接]");
                        break;
                    }
                    Ok(n) => {
                        let raw_text = from_gbk_bytes(&buf[..n]);
                        print!("{}", raw_text); 
                    }
                    Err(e) => {
                        eprintln!("\n[读取数据失败: {}]", e);
                        break;
                    }
                }
            }
        });

        // 模拟等待 1 秒后发送指令
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        let cmd = "look\n"; 
        let gbk_payload = to_gbk_bytes(cmd);
        writer.write_all(&gbk_payload).await?;
        writer.flush().await?;

        // 挂起主线程，保持进程存活观测输出
        tokio::signal::ctrl_c().await?;
        Ok(())
    }
}