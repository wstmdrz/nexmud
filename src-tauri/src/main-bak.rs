// 确保引入你的 gRPC 模块，名字取决于你之前怎么定义的
pub mod world_proto {
    tonic::include_proto!("world"); // 确保这里的名字和你的 proto 包名一致
}

use tauri::{AppHandle, Emitter}; // Tauri v2 的事件发送机制
use world_proto::world_service_client::WorldServiceClient;
use world_proto::{CommandReq, StreamReq};
use tokio::process::Command;

// =====================================================================
// 🚀 核心逻辑：带自动容灾容错的子进程孵化器
// =====================================================================
async fn spawn_world_daemon(app: AppHandle, tab_id: String, grpc_port: u16) {
    // 启动一个后台异步任务专门盯梢这个子进程
    tokio::spawn(async move {
        loop {
            // 💡 提示：这里的路径假设你的 Tauri 和 target 目录结构是标准的
            // 根据你的操作系统，如果你是 Windows，加上 .exe 后缀
            let binary_name = if cfg!(target_os = "windows") {
                "../../target/debug/world-daemon.exe" // 请根据实际相对路径调整
            } else {
                "../../target/debug/world-daemon"
            };

            let _ = app.emit("mud-stream-event", format!("\x1b[36m[Tauri 母舰] 正在为标签页 {} 分配资源...\x1b[0m\n", tab_id));

            // 1. 执行孵化指令
            let mut child = match Command::new(binary_name)
                .arg(grpc_port.to_string()) // 把端口传给 Daemon
                .spawn() 
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = app.emit("mud-stream-event", format!("\x1b[31m[Tauri 母舰] 孵化失败，找不到可执行文件: {}\x1b[0m\n", e));
                    // 找不到文件就别死循环了，休息久一点
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            // 2. 提取并广播 PID（完美契合你的 Demo 验收标准）
            let pid = child.id().unwrap_or(0);
            let _ = app.emit("mud-stream-event", format!("\x1b[32m[Tauri 母舰] ✅ Daemon 启动成功 | 目标: {} | PID: {} | 端口: {}\x1b[0m\n", tab_id, pid, grpc_port));

            // 3. 阻塞等待：死死盯住这个子进程的状态
            let status = child.wait().await.unwrap();

            // 4. 走到这里，说明进程被 Kill 掉了！触发容灾恢复！
            let _ = app.emit("mud-stream-event", format!("\x1b[31m\n[⚠️ 警报] 检测到标签页 {} (PID: {}) 的守护进程意外崩溃: {}\x1b[0m\n", tab_id, pid, status));
            let _ = app.emit("mud-stream-event", "\x1b[33m[系统] 正在启动灾备预案，1秒后自动拉起...\x1b[0m\n".to_string());

            // 休息1秒防止端口占用未完全释放，然后外层 loop 会自动重新执行 spawn！
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}

use std::sync::atomic::{AtomicU16, Ordering};

// 用一个全局原子变量来分配端口，从 50051 开始递增
static NEXT_PORT: AtomicU16 = AtomicU16::new(50051);

#[tauri::command]
async fn init_connection(app: AppHandle, tab_id: String) -> Result<(), String> {
    // 1. 动态分配一个独占的 gRPC 端口
    let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
    
    // 2. 呼叫母舰，孵化并守护这个独立的 Daemon 进程
    spawn_world_daemon(app.clone(), tab_id.clone(), port).await;

    // 为了确保子进程起来了并且绑定了端口，稍微等一小会儿再连 gRPC
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 3. 将 Tauri 的 gRPC 客户端连向这个新分配的专属端口！
    let grpc_url = format!("http://127.0.0.1:{}", port);
    
    // 👇 下面的逻辑和你之前的一样，只是连接地址变成了动态的 grpc_url
    tauri::async_runtime::spawn(async move {
        let mut client = match world_proto::world_service_client::WorldServiceClient::connect(grpc_url).await {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("mud-stream-event", format!("\x1b[31m[系统] gRPC 握手失败: {}\x1b[0m", e));
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