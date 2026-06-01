// src-tauri/src/world-daemon/src/lua/sandbox.rs
// =====================================================================
// NexMUD - Sandbox Space & Week 19 Advanced Protocols Total Definition
// 职责划分：托管 GMCP 带外数据运行池、MXP 富文本切片元数据总线
// =====================================================================
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use super::engine::{MushTrigger, MushAlias};

#[derive(Debug, Clone)]
pub struct MushTimer {
    pub name: String,
    pub seconds: f64,
    pub response_text: String,
    pub is_enabled: bool,
    pub is_one_shot: bool,
}

/// 🌟【Week 19 新规】：优先级队列与高级协议流转统一事件帧
#[derive(Debug, Clone)]
pub enum MushEventFrame {
    AddTrigger { name: String, match_text: String, response_text: String, is_multi_line: bool, lines_to_match: usize, priority: i32 },
    AddAlias { name: String, match_text: String, response_text: String, priority: i32 },
    AddTimer { name: String, seconds: f64, response_text: String, is_one_shot: bool },
    DoAfterSpecial { seconds: f64, send_text: String },
    // 🌟【GMCP 特权帧】：负责将子进程底层网络解出来的结构化数据（如 hp/mp），高能跨线程派发！
    GmcpUpdate { module: String, json_data: String },
}

/// 每一个独立 MUD 世界在内存中的完全自愈隔离沙箱
#[derive(Debug, Clone)]
pub struct WorldSandbox {
    pub root_dir: String,
    pub plugin_variables: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    pub variables: Arc<Mutex<HashMap<String, String>>>,
    
    // 🌟【Week 19 新规】：GMCP 带外结构化数据运行池！
    // 存储格式例如：G表内存储 "char.vitals" -> "{ \"hp\": 500, \"mp\": 300 }"
    pub gmcp_registry: Arc<Mutex<HashMap<String, String>>>,

    pub dynamic_timers: Arc<Mutex<HashMap<String, MushTimer>>>,
    pub dynamic_triggers: Arc<Mutex<HashMap<String, MushTrigger>>>,
    pub dynamic_aliases: Arc<Mutex<HashMap<String, MushAlias>>>,
    pub sliding_window: Arc<Mutex<Vec<String>>>,
    
    pub terminal_tx: broadcast::Sender<String>,
    pub tcp_cmd_tx: mpsc::Sender<String>,
    pub event_bus_tx: mpsc::Sender<MushEventFrame>,
}

impl WorldSandbox {
    pub fn new(root_dir: String, terminal_tx: broadcast::Sender<String>, tcp_cmd_tx: mpsc::Sender<String>, event_bus_tx: mpsc::Sender<MushEventFrame>) -> Self {
        Self {
            root_dir,
            plugin_variables: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            gmcp_registry: Arc::new(Mutex::new(HashMap::new())), // 初始化大表
            dynamic_timers: Arc::new(Mutex::new(HashMap::new())),
            dynamic_triggers: Arc::new(Mutex::new(HashMap::new())),
            dynamic_aliases: Arc::new(Mutex::new(HashMap::new())),
            sliding_window: Arc::new(Mutex::new(Vec::with_capacity(10))),
            terminal_tx,
            tcp_cmd_tx,
            event_bus_tx,
        }
    }
}
