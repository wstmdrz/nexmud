// src-tauri/src/world-daemon/src/service.rs
// =====================================================================
// NexMUD - Subprocess gRPC Service Implementation (Week 16)
// 职责划分：实质性承载世界域网络接口，隔离 gRPC 业务层与启动层
// =====================================================================
use tonic::{Request, Response, Status};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};

// 引入子进程在编译期自动膨胀生成的协议存根
use super::world_proto::world_service_server::WorldService;
use super::world_proto::{CommandAck, CommandReq, OutputChunk, StreamReq};
use super::lua::sandbox::WorldSandbox;
use super::lua::engine::MushMatchingEngine;

#[derive(Debug, Clone)]
pub struct MyWorldService {
    pub tx: broadcast::Sender<String>,      // 向 Tauri 母舰广播原始文本的通道
    pub cmd_tx: mpsc::Sender<String>,       // 接收来自前端指令并向后投递给 TCP 的通道
    pub lua_vm: Arc<Mutex<mlua::Lua>>,       // 独占常驻的嵌入式 Lua 5.4 虚拟机
    pub sandbox: WorldSandbox,              // 独立多要素沙箱（内含动态优先级匹配池）
}

#[tonic::async_trait]
impl WorldService for MyWorldService {
    // 🟢 IPC 接口 1：处理前端下发的玩家动作（最高优先权特权指令捕获 + 动态别名池过滤）
    async fn send_command(&self, request: Request<CommandReq>) -> Result<Response<CommandAck>, Status> {
        let raw_cmd = request.into_inner().input;
        let cmd = raw_cmd.trim().to_string();
        
        // 🛡️ 拦截特权 1：工业级 MUSHclient 标准特权指令 /include 绝对捕获器
        if cmd.starts_with("include(") || cmd.starts_with("/include(") {
            if let Some(start) = cmd.find('"') {
                if let Some(end) = cmd.rfind('"') {
                    if start < end {
                        let file_name = cmd[start+1..end].to_string();
                        let lua = self.lua_vm.lock().await;
                        if let Ok(inc_fn) = lua.globals().get::<mlua::Function>("include") {
                            let _ = inc_fn.call::<()>(file_name);
                            return Ok(Response::new(CommandAck { success: true }));
                        }
                    }
                }
            }
        }
        
        // 🛡️ 拦截特权 2：向前兼容 Week 16 高内聚带优先级序列化的别名宏替换
        let mut alias_intercepted = false;
        let final_cmd = MushMatchingEngine::process_outgoing_command(&cmd, &self.sandbox.dynamic_aliases, &mut alias_intercepted);

        // 🛡️ 拦截特权 3：向前兼容原生内置的 handle_alias 回调
        let alias_matched = alias_intercepted || {
            let lua = self.lua_vm.lock().await;
            let mut matched = false;
            if let Ok(handle_fn) = lua.globals().get::<mlua::Function>("handle_alias") {
                if let Ok(接管成功) = handle_fn.call::<bool>(final_cmd.clone()) {
                    if 接管成功 { matched = true; }
                }
            }
            matched 
        };

        if alias_matched {
            return Ok(Response::new(CommandAck { success: true }));
        }

        // 平滑降级：将最终清洗过滤完成后的命令投递给下游 TCP 引擎发包
        if let Err(e) = self.cmd_tx.send(final_cmd).await {
            return Err(Status::internal(format!("指令投递失败: {}", e)));
        }
        Ok(Response::new(CommandAck { success: true }))
    }

    // 🟢 IPC 接口 2：gRPC 高性能流式文本回吐通道 (连接 React 前端画布)
    type StreamOutputStream = tokio_stream::wrappers::ReceiverStream<Result<OutputChunk, Status>>;
    
    async fn stream_output(&self, _request: Request<StreamReq>) -> Result<Response<Self::StreamOutputStream>, Status> {
        let mut rx = self.tx.subscribe();
        let (tx, grpc_rx) = tokio::sync::mpsc::channel(128);
        tokio::spawn(async move {
            while let Ok(text) = rx.recv().await {
                let chunk = OutputChunk { text, has_ansi: true };
                if tx.send(Ok(chunk)).await.is_err() { break; }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(grpc_rx)))
    }
}
