use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use encoding_rs::GBK; // 确保在 world-daemon 的 Cargo.toml 中添加了 encoding_rs = "0.8"

pub mod world_proto {
    tonic::include_proto!("world");
}
use world_proto::world_service_server::{WorldService, WorldServiceServer};
use world_proto::{CommandAck, CommandReq, OutputChunk, StreamReq};

pub struct MyWorldService {
    // 用于把游戏文本推给前端的 gRPC 流
    tx: broadcast::Sender<String>,
    // 用于把前端指令发送给 TCP 写入任务的通道
    cmd_tx: mpsc::Sender<String>,
}

#[tonic::async_trait]
impl WorldService for MyWorldService {
    async fn send_command(
        &self,
        request: Request<CommandReq>,
    ) -> Result<Response<CommandAck>, Status> {
        let cmd = request.into_inner().input;
        println!("📥 收到前端指令，准备转发给真实 MUD: {}", cmd.trim());

        // 将指令放入通道，让负责 TCP 写入的后台任务去发送
        if self.cmd_tx.send(cmd).await.is_err() {
            return Err(Status::internal("无法连接到 MUD 服务器的写入通道"));
        }

        Ok(Response::new(CommandAck { success: true }))
    }

    type StreamOutputStream = ReceiverStream<Result<OutputChunk, Status>>;

    async fn stream_output(
        &self,
        request: Request<StreamReq>,
    ) -> Result<Response<Self::StreamOutputStream>, Status> {
        println!("🔗 Broker 申请订阅 MUD 输出流, Session: {}", request.into_inner().session_id);
        let mut rx = self.tx.subscribe();
        let (stream_tx, stream_rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let chunk = OutputChunk {
                    text: msg,
                    has_ansi: true, // 真实 MUD 带有颜色代码，设为 true
                };
                if stream_tx.send(Ok(chunk)).await.is_err() {
                    println!("❌ 客户端流已断开");
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Daemon] 🛡️ NexMUD World Gateway 启动...");

    // 1. 初始化内部通信通道
    let (tx, _) = broadcast::channel(1024);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(100);
    
    // 2. 连接到真实的 MUD 服务器 
    // 示例：此处可填入你想测试的真实 MUD 地址和端口（例如某个公共测试 MUD，或者本地 MUD 服务器）
    let mud_addr = "mud.pkuxkx.net:8080"; 
    println!("[Daemon] 🌐 正在尝试连接真实 MUD 服务器: {} ...", mud_addr);
    
    let mut tcp_stream = TcpStream::connect(mud_addr).await
        .expect("❌ 无法连接到目标 MUD 服务器，请检查地址或服务是否开启");
    println!("[Daemon] ✅ 已成功建立与真实 MUD 的 TCP 神经连接！");

    // 3. 拆分 TCP 读写半截
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.into_split();

    // 4. 任务 A：持续读取 MUD 服务器的原始字节，解码为 UTF-8 并广播给前端
   

    // 4 & 5. 终极合并任务：具有断线自动重连功能的守护循环
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // 🌀 最外层循环：负责不断尝试连接和重连
        loop {
            // 🚥 红绿灯：等待前端 React 连接上 gRPC 频道
            while tx_clone.receiver_count() == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

            let mud_addr = "mud.pkuxkx.net:8080"; // 记得换成你测试的真实 MUD 地址
            let _ = tx_clone.send(format!("\x1b[36m\n[系统] 正在尝试建立与世界节点 {} 的神经连接...\x1b[0m\n", mud_addr));

            // 🌐 尝试建立 TCP 连接
            match TcpStream::connect(mud_addr).await {
                Ok(tcp_stream) => {
                    let _ = tx_clone.send("\x1b[32m[系统] ✅ 神经连接建立成功！数据流已接入。\x1b[0m\n".to_string());
                    let (mut tcp_reader, mut tcp_writer) = tcp_stream.into_split();
                    let mut buffer = [0u8; 4096];

                    // ⚙️ 内层循环：使用 select! 同时监听“读网络”和“写网络”
                    loop {
                        tokio::select! {
                            // 分支 1：如果 MUD 服务器发来了数据
                            result = tcp_reader.read(&mut buffer) => {
                                match result {
                                    Ok(0) => {
                                        let _ = tx_clone.send("\x1b[33m[系统] ⚠️ 目标世界主动断开了连接。\x1b[0m\n".to_string());
                                        break; // 退出内层循环，触发外层的重连
                                    }
                                    Ok(n) => {
                                        let (decoded_str, _, _) = GBK.decode(&buffer[..n]);
                                        let _ = tx_clone.send(decoded_str.into_owned());
                                    }
                                    Err(e) => {
                                        let _ = tx_clone.send(format!("\x1b[31m[系统] ❌ 神经链路读取异常: {}\x1b[0m\n", e));
                                        break;
                                    }
                                }
                            }
                            
                            // 分支 2：如果前端玩家敲击了回车，发来了指令
                            Some(cmd) = cmd_rx.recv() => {
                                let full_cmd = format!("{}\n", cmd.trim());
                                let (gbk_bytes, _, _) = GBK.encode(&full_cmd);
                                if tcp_writer.write_all(&gbk_bytes).await.is_err() {
                                    let _ = tx_clone.send("\x1b[31m[系统] ❌ 指令发送失败，链路已破裂。\x1b[0m\n".to_string());
                                    break; // 退出内层循环，触发外层的重连
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx_clone.send(format!("\x1b[31m[系统] ❌ 无法连接到世界节点: {}\x1b[0m\n", e));
                }
            }

            // ⏳ 断线缓冲期：断开后不要立刻死循环疯狂重试，休息 3 秒钟
            let _ = tx_clone.send("\x1b[90m[系统] 将在 3 秒后尝试重新连接...\x1b[0m\n".to_string());
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    });

    // 6. 启动 gRPC 服务给 Tauri 进程调用
    let my_service = MyWorldService { tx, cmd_tx };
    let grpc_addr = "127.0.0.1:50051".parse()?;
    println!("[Daemon] 📡 gRPC 服务已挂起，正在监听: {}", grpc_addr);

    Server::builder()
        .add_service(WorldServiceServer::new(my_service))
        .serve(grpc_addr)
        .await?;

    Ok(())
}