// src-tauri/src/world-daemon/src/main.rs
// =====================================================================
// NexMUD - Fully Decoupled Subprocess Launcher & Engine (Week 16 Complete)
// 文件主入口：src-tauri/src/world-daemon/src/main.rs (第一部分：引导与高频全自愈看护层)
// =====================================================================
use encoding_rs::GBK;
use std::env;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, Mutex};
use tonic::transport::Server;

pub mod service; 
pub mod lua;
pub mod world_proto {
    tonic::include_proto!("world"); 
}

use lua::engine::{MushTrigger, MushAlias};
use lua::sandbox::{MushEventFrame, WorldSandbox};
use world_proto::world_service_server::WorldServiceServer;
use lua::engine::MushMatchingEngine;
use service::MyWorldService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let grpc_port = args.get(1).unwrap_or(&"50051".to_string()).to_string();
    let target_host = args.get(2).unwrap_or(&"127.0.0.1".to_string()).to_string();
    let target_port = args.get(3).unwrap_or(&"6666".to_string()).to_string();
    let addr_str = format!("127.0.0.1:{}", grpc_port);
    let addr = addr_str.parse()?;

    println!("🌍 World Daemon 启动 | 独占gRPC端口: {} | 目标世界: {}:{}", grpc_port, target_host, target_port);

    let (tx, _) = broadcast::channel::<String>(100);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(32);
    let (event_bus_tx, mut event_bus_rx) = mpsc::channel::<MushEventFrame>(128);

    // =====================================================================
    // 🛡️【子进程双保险自裁熔断轴】：每 300 毫秒高频检查前端 Canvas 广播接收者总计数
    // 在 Tauri 2.0 中，不管是在 pnpm dev 还是打包环境，只要母舰大窗体物理关闭，
    // 前端订阅流（Receiver）会被底层强行全量注销、跌零归零！子进程捕获后瞬间 0 延迟秒杀自己！
    // =====================================================================
    let tx_for_suicide = tx.clone();
    tokio::spawn(async move {
        // 先让子进程跑个 2 秒钟，等待前端母舰正常建立连接订阅
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            if tx_for_suicide.receiver_count() == 0 {
                println!("🛑 [自裁守护] 侦测到前端 Canvas 链路已清空归零，母舰已合眼闭灯！");
                println!("🛑 [自裁守护] 正在执行 0 残留多进程硬核自毁自退...");
                std::process::exit(0); // 瞬间原地猝死，物理断绝僵尸泄露！
            }
        }
    });

    let sandbox = WorldSandbox::new(format!("tab-{}", grpc_port), tx.clone(), cmd_tx.clone(), event_bus_tx);
    let sandbox_for_bus = sandbox.clone();
    let lua_vm = lua::LuaLifecycleManager::boot_sandbox(sandbox.clone())?;

    // 覆盖重载原生 print
    {
        let lua = lua_vm.lock().await;
        if let Ok(p_fn) = lua.globals().get::<mlua::Function>("print_to_terminal") {
            let _ = lua.globals().set("print", p_fn);
        }
    }

    // 🚀 1. 事件总线常驻监听
    tokio::spawn(async move {
        while let Some(event_frame) = event_bus_rx.recv().await {
            match event_frame {
                MushEventFrame::AddTrigger { name, match_text, response_text, is_multi_line, lines_to_match, priority } => {
                    if let Ok(mut trigs) = sandbox_for_bus.dynamic_triggers.try_lock() {
                        trigs.insert(name.clone(), MushTrigger { name: name.clone(), match_text, response_text, is_multi_line, lines_to_match, priority });
                        let _ = sandbox_for_bus.terminal_tx.send(format!("\x1b[34m[EventBus] ⚙️ Trigger [{}] (P:{}) 成功挂载！\x1b[0m\r\n", name, priority));
                    }
                }
                MushEventFrame::AddAlias { name, match_text, response_text, priority } => {
                    if let Ok(mut aliases) = sandbox_for_bus.dynamic_aliases.try_lock() {
                        aliases.insert(name.clone(), MushAlias { name: name.clone(), match_text, response_text, priority });
                        let _ = sandbox_for_bus.terminal_tx.send(format!("\x1b[34m[EventBus] ⚙️ Alias [{}] (P:{}) 成功挂载！\x1b[0m\r\n", name, priority));
                    }
                }
                MushEventFrame::AddTimer { name, seconds, response_text, is_one_shot } => {
                    let tx_timer = sandbox_for_bus.terminal_tx.clone(); let cmd_timer = sandbox_for_bus.tcp_cmd_tx.clone();
                    let _ = tx_timer.send(format!("\x1b[36m[EventBus] ⏰ 定时器 [{}] 成功挂载！时间跨度: {} 秒\x1b[0m\r\n", name, seconds));
                    tokio::spawn(async move {
                        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs_f64(seconds));
                        interval.tick().await; 
                        loop {
                            interval.tick().await;
                            let _ = cmd_timer.send(response_text.clone()).await;
                            let _ = tx_timer.send(format!("\x1b[36m[Tokio时间轴] ⏰ 定时器 [{}] 周期到达！\x1b[0m\r\n", name));
                            if is_one_shot { break; }
                        }
                    });
                }
                MushEventFrame::DoAfterSpecial { seconds, send_text } => {
                    let tx_delay = sandbox_for_bus.terminal_tx.clone(); let cmd_delay = sandbox_for_bus.tcp_cmd_tx.clone();
                    tokio::spawn(async move {
                        let _ = tx_delay.send(format!("\x1b[34m[EventBus] ⏳ 定时触发...\x1b[0m\r\n"));
                        tokio::time::sleep(tokio::time::Duration::from_secs_f64(seconds)).await; let _ = cmd_delay.send(send_text).await;
                    });
                }
            }
        }
    });
        // 🚀 2. 高并发事件调度常驻线程
    let (trigger_tx, mut trigger_rx) = mpsc::channel::<String>(128);
    let lua_vm_trigger_worker = lua_vm.clone();
    tokio::spawn(async move {
        while let Some(raw_text) = trigger_rx.recv().await {
            if let Ok(lua) = lua_vm_trigger_worker.try_lock() {
                if let Ok(trigger_fn) = lua.globals().get::<mlua::Function>("on_text_trigger") { let _ = trigger_fn.call::<()>(raw_text); }
            }
        }
    });

    // 💡 3. TCP 核心网络发包与状态机匹配引擎
    let tx_clone = tx.clone();
    let sandbox_for_tcp = sandbox.clone();
    let trigger_tx_clone = trigger_tx.clone();
    let lua_vm_for_tcp = lua_vm.clone(); 
    let start_time = Arc::new(Mutex::new(std::time::Instant::now()));

    tokio::spawn(async move {
        loop {
            while tx_clone.receiver_count() == 0 { tokio::time::sleep(tokio::time::Duration::from_millis(200)).await; }
            let mud_addr = format!("{}:{}", target_host, target_port);
            let _ = tx_clone.send(format!("\x1b[36m\n[系统] 正在尝试建立与世界节点 {} 的神经连接...\x1b[0m\n", mud_addr));

            match TcpStream::connect(&mud_addr).await {
                Ok(tcp_stream) => {
                    let _ = tx_clone.send("\x1b[32m[系统] ✅ 神经连接建立成功！数据流已接入。\x1b[0m\n".to_string());
                    let (mut tcp_reader, mut tcp_writer) = tcp_stream.into_split();
                    let mut buffer = [0u8; 8192];
                    loop {
                        tokio::select! {
                            result = tcp_reader.read(&mut buffer) => {
                                match result {
                                    Ok(0) => { let _ = tx_clone.send("\x1b[33m[系统] ⚠️ 目标世界主动断开了连接。\x1b[0m\n".to_string()); break; }
                                    Ok(n) => {
                                        let mut clean_buf = Vec::with_capacity(n); let mut i = 0;
                                        while i < n {
                                            if buffer[i] == 255 && i + 2 < n {
                                                let command = buffer[i+1]; let option = buffer[i+2];
                                                let reply_cmd = if command == 253 { 252 } else if command == 251 { 254 } else { 0 };
                                                if reply_cmd != 0 { let reply = vec![255, reply_cmd, option]; let _ = tcp_writer.write_all(&reply).await; }
                                                i += 3;
                                            } else { clean_buf.push(buffer[i]); i += 1; }
                                        }
                                        if !clean_buf.is_empty() {
                                            let raw_text = MushMatchingEngine::auto_decode_network_bytes(&clean_buf);
                                            MushMatchingEngine::process_incoming_text(
                                                &raw_text, 
                                                &sandbox_for_tcp.sliding_window, 
                                                &sandbox_for_tcp.dynamic_triggers, 
                                                &sandbox_for_tcp.tcp_cmd_tx, 
                                                &sandbox_for_tcp.terminal_tx,
                                                &lua_vm_for_tcp
                                            );
                                            let _ = trigger_tx_clone.try_send(raw_text.clone());
                                            let _ = tx_clone.send(raw_text);
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                            Some(cmd) = cmd_rx.recv() => {
                                if let Ok(mut t) = start_time.try_lock() { *t = std::time::Instant::now(); }
                                let full_cmd = format!("{}\n", cmd.trim()); let (gbk_bytes, _, _) = GBK.encode(&full_cmd);
                                if tcp_writer.write_all(&gbk_bytes).await.is_err() { break; }
                            }
                        }
                    }
                }
                Err(_) => {}
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    });

    // 💡 4. 实体绑定启动 gRPC 专属服务器体
    let world_service = MyWorldService { tx: tx.clone(), cmd_tx, lua_vm, sandbox: sandbox.clone() };
    println!("🚀 Daemon gRPC Server 准备就绪，正在监听: {}", addr);
    
    Server::builder()
        .add_service(WorldServiceServer::new(world_service))
        .serve(addr)
        .await?;

    Ok(())
}

