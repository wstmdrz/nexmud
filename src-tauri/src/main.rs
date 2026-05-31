#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod world_proto {
    tonic::include_proto!("world");
}

use serde::Serialize;

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]

struct LogPayload {
    tab_id: String,
    text: String,
}


use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use world_proto::world_service_client::WorldServiceClient;
use world_proto::{CommandReq, StreamReq};
use tonic::transport::Channel;

// 1. 定义多标签页全局状态注册表
struct AppState {
    // 映射：tab_id -> 该世界独占的 gRPC 客户端
    clients: Arc<Mutex<HashMap<String, WorldServiceClient<Channel>>>>,
    // 分配端口的计数器，从 50051 开始自增
    next_port: AtomicU16,
}

// =====================================================================
// 🚀 核心逻辑：多沙盒独立守护进程拉起引擎
// =====================================================================
// 1. 修改 spawn_world_daemon 内部的监听部分：
async fn spawn_world_daemon(app: AppHandle, tab_id: String, grpc_port: u16) {
    tokio::spawn(async move {
        loop {
            let mut ext = "";
            if cfg!(target_os = "windows") { ext = ".exe"; }
            let binary_path = format!("binaries/world-daemon-x86_64-pc-windows-msvc{}", ext);

            let _ = app.emit("mud-stream-event", LogPayload { tab_id: tab_id.clone(), text: format!("\x1b[36m[母舰] 正在为隔离域 {} 动态分配资源...\x1b[0m\n", tab_id) });

            let mut child = match tokio::process::Command::new(&binary_path)
                .arg(grpc_port.to_string()) 
                .spawn() 
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = app.emit("mud-stream-event", LogPayload { tab_id: tab_id.clone(), text: format!("\x1b[31m[母舰] ❌ 孵化失败！请确保 {} 已存在: {}\x1b[0m\n", binary_path, e) });
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let pid = child.id().unwrap_or(0);
            let _ = app.emit("mud-stream-event", LogPayload { tab_id: tab_id.clone(), text: format!("\x1b[32m[母舰] ✅ Daemon 启动成功 | 世界域: {} | 核心PID: {} | 通信端口: {}\x1b[0m\n", tab_id, pid, grpc_port) });

            let status = child.wait().await.unwrap();

            let _ = app.emit("mud-stream-event", LogPayload { tab_id: tab_id.clone(), text: format!("\x1b[31m\n[警报] ⚠️ 检测到网络节点 {} (PID: {}) 意外破裂: {}\x1b[0m\n", tab_id, pid, status) });
            let _ = app.emit("mud-stream-event", LogPayload { tab_id: tab_id.clone(), text: "\x1b[33m[灾备] 启动自愈宪法，1秒后重新孵化沙箱进程...\x1b[0m\n".to_string() });

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}


// =====================================================================
// 🟢 IPC Command 1: 初始化动态神经连接
// =====================================================================
#[tauri::command(rename_all = "camelCase")]
async fn init_connection(app: AppHandle, tab_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let port = state.next_port.fetch_add(1, Ordering::SeqCst);
    
    // 异步拉起，绝不阻塞 Tauri 主线程
    let app_clone = app.clone();
    let tab_id_clone = tab_id.clone();
    tokio::spawn(async move {
        spawn_world_daemon(app_clone, tab_id_clone, port).await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    let grpc_url = format!("http://127.0.0.1:{}", port);

    let clients_map = state.clients.clone();
    
    tauri::async_runtime::spawn(async move {
        let mut client = match WorldServiceClient::connect(grpc_url).await {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("mud-stream-event", format!("\x1b[31m[系统] gRPC 握手失败: {}\x1b[0m", e));
                return;
            }
        };
        
        // 🔥【核心重构】：建立成功后，将此客户端登记到全局注册表，供该标签页后续精准调用
        clients_map.lock().await.insert(tab_id.clone(), client.clone());

        let req = tonic::Request::new(StreamReq { session_id: tab_id.clone() });

        if let Ok(response) = client.stream_output(req).await {
            let mut stream = response.into_inner();
            let _ = app.emit("mud-stream-event", format!("\x1b[32m[系统] {} 的高性能数据穿透流订阅成功。\x1b[0m", tab_id));
            
            while let Ok(Some(chunk)) = stream.message().await {
                // 广播给 React 19 渲染
                let _ = app.emit("mud-stream-event", LogPayload {
        tab_id: tab_id.clone(),
        text: chunk.text,
    });
            }
        }
    });

    Ok(())
}

// =====================================================================
// 🟢 IPC Command 2: 定向路由指令发送
// =====================================================================
#[tauri::command(rename_all = "camelCase")]
async fn send_command(tab_id: String, input: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut lock = state.clients.lock().await;
    
    // 🔥【验收指标】：通过 tab_id 寻找专属客户端，确保只有 A 标签页发送时，只有 A 的进程收到
    if let Some(client) = lock.get_mut(&tab_id) {
        let req = tonic::Request::new(CommandReq { input });
        client.send_command(req).await.map_err(|e| format!("投递失败: {}", e))?;
        Ok(())
    } else {
        Err(format!("错误：未找到世界域 {} 的活动链路！", tab_id))
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_port: AtomicU16::new(50051),
        })
        .invoke_handler(tauri::generate_handler![init_connection, send_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
