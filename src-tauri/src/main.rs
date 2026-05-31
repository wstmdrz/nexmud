// 确保引入你的 gRPC 模块，名字取决于你之前怎么定义的
pub mod world_proto {
    tonic::include_proto!("world"); // 确保这里的名字和你的 proto 包名一致
}

use tauri::{AppHandle, Emitter}; // Tauri v2 的事件发送机制
use world_proto::world_service_client::WorldServiceClient;
use world_proto::{CommandReq, StreamReq};

// =====================================================================
// 🌟 指令 1：初始化神经连接 (前端组件挂载时调用)
// =====================================================================
#[tauri::command]
async fn init_connection(app: AppHandle) -> Result<(), String> {
    println!("[Broker] 收到前端连接请求，准备对接 Daemon...");

    // 开一个后台异步任务专门负责监听 gRPC 流
    tauri::async_runtime::spawn(async move {
        // 1. 连接 Daemon
        let mut client = match WorldServiceClient::connect("http://127.0.0.1:50051").await {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("mud-stream-event", format!("\x1b[31m[系统] 无法连接到 Daemon: {}\x1b[0m", e));
                return;
            }
        };

        // 2. 申请流订阅
        let req = tonic::Request::new(StreamReq {
            session_id: "react-client".to_string(),
        });

        match client.stream_output(req).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                let _ = app.emit("mud-stream-event", "\x1b[32m[系统] 神经连接建立成功！接收端口已打开。\x1b[0m".to_string());
                
                // 3. 持续监听来自 Daemon 的真实 MUD 数据
                while let Ok(Some(chunk)) = stream.message().await {
                    // 收到数据，立刻通过 Tauri 事件广播给前端的 React
                    let _ = app.emit("mud-stream-event", chunk.text);
                }
                
                let _ = app.emit("mud-stream-event", "\x1b[33m[系统] 数据流已断开。\x1b[0m".to_string());
            }
            Err(e) => {
                let _ = app.emit("mud-stream-event", format!("\x1b[31m[系统] 订阅流失败: {}\x1b[0m", e));
            }
        }
    });

    Ok(())
}

// =====================================================================
// 🌟 指令 2：发送玩家指令 (前端输入框敲回车时调用)
// =====================================================================
#[tauri::command]
async fn send_command(input: String) -> Result<(), String> {
    // 每次发送指令建立一次连接（或者你也可以做成全局单例）
    let mut client = WorldServiceClient::connect("http://127.0.0.1:50051")
        .await
        .map_err(|e| format!("无法连接到 Daemon: {}", e))?;

    let req = tonic::Request::new(CommandReq {
        input,
    });

    // 把指令丢给 Daemon
    client.send_command(req).await.map_err(|e| format!("发送失败: {}", e))?;
    
    Ok(())
}

// =====================================================================
// 🚀 Tauri 启动点
// =====================================================================
fn main() {
    tauri::Builder::default()
        // 🚨 最关键的一步：在这里注册刚才写好的两个命令！如果不写在这，前端就会报 not found
        .invoke_handler(tauri::generate_handler![init_connection, send_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}