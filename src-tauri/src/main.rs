#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod world_proto {
    tonic::include_proto!("world");
}

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, State, WindowEvent}; // 🟢 引入 WindowEvent 视窗钩子
use world_proto::world_service_client::WorldServiceClient;
use world_proto::{CommandReq, StreamReq};
use tonic::transport::Channel;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LogPayload {
    tab_id: String,
    text: String,
}

// 存储全局活跃子进程 PID 的线程安全注册表，用来做临终枪毙大扫除
#[derive(Default)]
struct GlobalProcessRegistry {
    pids: Arc<Mutex<Vec<u32>>>,
}

struct AppState {
    clients: Arc<Mutex<HashMap<String, WorldServiceClient<Channel>>>>,
    tab_ports: Arc<Mutex<HashMap<String, u16>>>,
    next_port: AtomicU16,
    registry: GlobalProcessRegistry, // 🟢 挂载注册表资产
}

async fn spawn_world_daemon(app: AppHandle, tab_id: String, grpc_port: u16, registry: Arc<Mutex<Vec<u32>>>) {
    tokio::spawn(async move {
        loop {
            let mut ext = "";
            if cfg!(target_os = "windows") { ext = ".exe"; }
            let binary_path = format!("binaries/world-daemon-x86_64-pc-windows-msvc{}", ext);

            let _ = app.emit("mud-stream-event", LogPayload { 
                tab_id: tab_id.clone(), 
                text: format!("\x1b[36m[母舰] 正在为隔离域 {} 动态分配资源...\x1b[0m\r\n", tab_id) 
            });

            let current_dir = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();

            let mut child = match tokio::process::Command::new(&binary_path)
                .arg(grpc_port.to_string()) 
                .arg("mud.fy-vi.cn") 
                .arg("6666")      
                .arg(current_dir) 
                .spawn() 
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = app.emit("mud-stream-event", LogPayload { 
                        tab_id: tab_id.clone(), 
                        text: format!("\x1b[31m[母舰] ❌ 孵化失败！请确保 {} 已存在: {}\x1b[0m\r\n", binary_path, e) 
                    });
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let pid = child.id().unwrap_or(0);
            
            // 🟢【防泄露核心 1】：子进程孵化成功后，瞬间将其 PID 送入全局大表看护
            if pid > 0 {
                if let Ok(mut list) = registry.try_lock() {
                    list.push(pid);
                }
            }

            let _ = app.emit("mud-stream-event", LogPayload { 
                tab_id: tab_id.clone(), 
                text: format!("\x1b[32m[母舰] ✅ Daemon 启动成功 | 世界域: {} | 核心PID: {} | 独占端口: {}\x1b[0m\r\n", tab_id, pid, grpc_port) 
            });

            let status = child.wait().await.unwrap();

            // 子进程如果正常退出，将其从看护榜上移除
            if let Ok(mut list) = registry.try_lock() {
                list.retain(|&x| x != pid);
            }

            let _ = app.emit("mud-stream-event", LogPayload { 
                tab_id: tab_id.clone(), 
                text: format!("\x1b[31m\n[警报] ⚠️ 检测到网络节点 {} (PID: {}) 意外破裂: {}\x1b[0m\n", tab_id, pid, status) 
            });
            let _ = app.emit("mud-stream-event", LogPayload { 
                tab_id: tab_id.clone(), 
                text: "\x1b[33m[灾备] 启动自愈宪法，1秒后重新孵化沙箱进程...\x1b[0m\r\n".to_string() 
            });

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}

#[tauri::command(rename_all = "camelCase")]
async fn init_connection(app: AppHandle, tab_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let port = {
        let mut ports_lock = state.tab_ports.lock().await;
        if ports_lock.contains_key(&tab_id) {
            return Ok(());
        }
        let assigned_port = state.next_port.fetch_add(1, Ordering::SeqCst);
        ports_lock.insert(tab_id.clone(), assigned_port);
        assigned_port 
    };

    let app_clone = app.clone();
    let tab_id_clone = tab_id.clone();
    let pids_registry_clone = state.registry.pids.clone();
    
    tokio::spawn(async move {
        spawn_world_daemon(app_clone, tab_id_clone, port, pids_registry_clone).await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    let grpc_url = format!("http://127.0.0.1:{}", port);
    let clients_map = state.clients.clone(); 
    let app_grpc = app.clone();
    let tab_id_grpc = tab_id.clone();
    
    tokio::spawn(async move {
        loop {
            let mut client = match WorldServiceClient::connect(grpc_url.clone()).await {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                    continue;
                }
            };
            
            clients_map.lock().await.insert(tab_id_grpc.clone(), client.clone());

            let req = tonic::Request::new(StreamReq { session_id: tab_id_grpc.clone() });

            if let Ok(response) = client.stream_output(req).await {
                let mut stream = response.into_inner();
                let _ = app_grpc.emit("mud-stream-event", LogPayload { 
                    tab_id: tab_id_grpc.clone(), 
                    text: format!("\x1b[32m[系统] {} 的专属 gRPC 通信流已自动接管，恢复数据流穿透！\x1b[0m\r\n", tab_id_grpc) 
                });
                
                while let Ok(Some(chunk)) = stream.message().await {
                    let _ = app_grpc.emit("mud-stream-event", LogPayload {
                        tab_id: tab_id_grpc.clone(),
                        text: chunk.text,
                    });
                }
                
                let _ = app_grpc.emit("mud-stream-event", LogPayload { 
                    tab_id: tab_id_grpc.clone(), 
                    text: format!("\x1b[33m[系统] {} 检测到后台通道猝死，正在拦截异常并启动无感重连...\x1b[0m\r\n", tab_id_grpc) 
                });
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
async fn send_command(tab_id: String, input: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut lock = state.clients.lock().await;
    if let Some(client) = lock.get_mut(&tab_id) {
        let req = tonic::Request::new(CommandReq { input });
        client.send_command(req).await.map_err(|e| format!("投递失败: {}", e))?;
        Ok(())
    } else {
        Err(format!("错误：未找到世界域 {} 的活动链路！", tab_id))
    }
}

fn main() {
    // 实例化共享全局 PID 表
    let global_pids = Arc::new(Mutex::new(Vec::<u32>::new()));
    let pids_for_hook = global_pids.clone();

    tauri::Builder::default()
        .manage(AppState {
            clients: Arc::new(Mutex::new(HashMap::new())),
            tab_ports: Arc::new(Mutex::new(HashMap::new())),
            next_port: AtomicU16::new(50051), 
            registry: GlobalProcessRegistry { pids: global_pids },
        })
        .invoke_handler(tauri::generate_handler![init_connection, send_command])
        // 🟢【防泄露核心 2】：注入 Tauri 临终视窗断开钩子事件！
        // 只要用户点红叉关闭大窗体，主进程临死前无条件向所有存活子进程 PID 补刀枪毙！
        .on_window_event(move |window, event| {
            if let WindowEvent::Destroyed = event {
                println!("🚨 [母舰自毁哨兵] 检测到视窗物理关闭，开始执行多进程硬核清场大扫除...");
                if let Ok(list) = pids_for_hook.try_lock() {
                    for pid in list.iter() {
                        if *pid > 0 {
                            println!("  ➔ 💥 [系统大扫除] 正在强杀残留子进程 PID: {}", pid);
                            // 物理调起 Windows 内核硬杀指令
                            let _ = std::process::Command::new("taskkill")
                                .arg("/f")
                                .arg("/pid")
                                .arg(pid.to_string())
                                .spawn();
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
