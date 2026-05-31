use encoding_rs::GBK;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tonic::{transport::Server, Request, Response, Status};

pub mod world_proto {
    tonic::include_proto!("world"); 
}

use world_proto::world_service_server::{WorldService, WorldServiceServer};
use world_proto::{CommandAck, CommandReq, OutputChunk, StreamReq};

#[derive(Debug)]
pub struct MyWorldService {
    pub tx: broadcast::Sender<String>,      
    pub cmd_tx: mpsc::Sender<String>,       
}

#[tonic::async_trait]
impl WorldService for MyWorldService {
    async fn send_command(
        &self,
        request: Request<CommandReq>,
    ) -> Result<Response<CommandAck>, Status> {
        let cmd = request.into_inner().input;
        if let Err(e) = self.cmd_tx.send(cmd).await {
            return Err(Status::internal(format!("指令投递失败: {}", e)));
        }
        Ok(Response::new(CommandAck { success: true }))
    }

    type StreamOutputStream = tokio_stream::wrappers::ReceiverStream<Result<OutputChunk, Status>>;

    async fn stream_output(
        &self,
        _request: Request<StreamReq>,
    ) -> Result<Response<Self::StreamOutputStream>, Status> {
        let mut rx = self.tx.subscribe();
        let (tx, grpc_rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            while let Ok(text) = rx.recv().await {
                let chunk = OutputChunk {
                    text,
                    has_ansi: true,
                };
                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            grpc_rx,
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let grpc_port = args.get(1).unwrap_or(&"50051".to_string()).to_string();
    
    let addr_str = format!("127.0.0.1:{}", grpc_port);
    let addr = addr_str.parse()?;

    println!("🌍 World Daemon 正在启动，分配 gRPC 端口: {}", grpc_port);

    let (tx, _) = broadcast::channel::<String>(100);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(32);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            while tx_clone.receiver_count() == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

            // 🌐【终极对齐】：连接到北大侠客行标准端口
            let mud_addr = "mud.pkuxkx.net:8080"; 
            let _ = tx_clone.send(format!(
                "\x1b[36m\n[系统] 正在尝试建立与公网节点 {} 的安全连接...\x1b[0m\n",
                mud_addr
            ));

            match TcpStream::connect(mud_addr).await {
                Ok(tcp_stream) => {
                    let _ = tx_clone.send("\x1b[32m[系统] ✅ 江湖连接成功！正在进行 Telnet 协议动态协商...\x1b[0m\n".to_string());
                    let (mut tcp_reader, mut tcp_writer) = tcp_stream.into_split();
                    let mut buffer = [0u8; 8192];

                    loop {
                        tokio::select! {
                            // 监听 MUD 服务器发过来的真实数据
                            result = tcp_reader.read(&mut buffer) => {
                                match result {
                                    Ok(0) => {
                                        let _ = tx_clone.send("\x1b[33m[系统] ⚠️ 江湖节点主动断开了连接。\x1b[0m\n".to_string());
                                        break; 
                                    }
                                    Ok(n) => {
                                        let mut clean_buf = Vec::with_capacity(n);
                                        let mut i = 0;
                                        
                                        // 🛡️【核心黑科技】：Telnet 协议自动协商状态机
                                        // 拦截服务器发来的 \xff (IAC) 指令并自动回复，假装自己是个专业的 MUD 客户端
                                        while i < n {
                                            if buffer[i] == 255 && i + 2 < n {
                                                let command = buffer[i+1];
                                                let option = buffer[i+2];
                                                
                                                // 如果服务器发送 DO (253) 或 WILL (251)，我们一律回复 WONT (252) 或 DONT (254)
                                                // 拒绝所有复杂的终端属性协商，直接强行逼迫服务器吐出原始文本数据
                                                let reply_cmd = if command == 253 { 252 } else if command == 251 { 254 } else { 0 };
                                                if reply_cmd != 0 {
                                                    let reply = vec![255, reply_cmd, option];
                                                    let _ = tcp_writer.write_all(&reply).await;
                                                }
                                                i += 3;
                                            } else {
                                                clean_buf.push(buffer[i]);
                                                i += 1;
                                            }
                                        }

                                        if !clean_buf.is_empty() {
                                            let (decoded_str, _, _) = GBK.decode(&clean_buf);
                                            let _ = tx_clone.send(decoded_str.into_owned());
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx_clone.send(format!("\x1b[31m[系统] ❌ 神经链路读取异常: {}\x1b[0m\n", e));
                                        break;
                                    }
                                }
                            }
                            
                            // 监听前端下发的玩家操作指令
                            Some(cmd) = cmd_rx.recv() => {
                                let full_cmd = format!("{}\n", cmd.trim());
                                let (gbk_bytes, _, _) = GBK.encode(&full_cmd);
                                if tcp_writer.write_all(&gbk_bytes).await.is_err() {
                                    let _ = tx_clone.send("\x1b[31m[系统] ❌ 指令发送失败，链路已破裂。\x1b[0m\n".to_string());
                                    break; 
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx_clone.send(format!("\x1b[31m[系统] ❌ 无法连入公网世界: {}\x1b[0m\n", e));
                }
            }

            let _ = tx_clone.send("\x1b[90m[系统] 将在 3 秒后尝试重新连接世界...\x1b[0m\n".to_string());
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    });

    let world_service = MyWorldService {
        tx: tx.clone(),
        cmd_tx,
    };

    println!("🚀 Daemon gRPC Server 准备就绪，监听地址: {}", addr);

    Server::builder()
        .add_service(WorldServiceServer::new(world_service))
        .serve(addr)
        .await?;

    Ok(())
}
