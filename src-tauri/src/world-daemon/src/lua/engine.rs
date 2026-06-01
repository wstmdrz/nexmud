// src-tauri/src/world-daemon/src/lua/engine.rs
// =====================================================================
// NexMUD - Text Matching, Priorities & Timers Engine (Week 16 - 18 Complete)
// 职责划分：管理多行状态机历史滑窗、ANSI颜色过滤、自适应转码自适应
// =====================================================================
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct MushTrigger {
    pub name: String,
    pub match_text: String,       
    pub response_text: String,    
    pub is_multi_line: bool,     
    pub lines_to_match: usize,
    pub priority: i32, 
}

#[derive(Debug, Clone)]
pub struct MushAlias {
    pub name: String,
    pub match_text: String,       
    pub response_text: String,
    pub priority: i32, 
}

pub struct MushMatchingEngine;

impl MushMatchingEngine {
    pub fn auto_decode_network_bytes(bytes: &[u8]) -> String {
        match std::str::from_utf8(bytes) {
            Ok(utf8_str) => utf8_str.to_string(),
            Err(_) => {
                let (decoded_str, _, _) = encoding_rs::GBK.decode(bytes);
                decoded_str.into_owned()
            }
        }
    }

    /// 🟢 核心功能 1：高级优先级队列多行滑窗过滤，无缝支持 Lua 函数反向唤醒
    pub fn process_incoming_text(
        raw_text: &str,
        sliding_window: &Arc<Mutex<Vec<String>>>,
        dynamic_triggers: &Arc<Mutex<HashMap<String, MushTrigger>>>,
        tcp_cmd_tx: &tokio::sync::mpsc::Sender<String>,
        terminal_tx: &tokio::sync::broadcast::Sender<String>,
        lua_vm: &Arc<Mutex<mlua::Lua>>, // 物理导入虚拟机句柄以供反向唤醒
    ) {
        let lines: Vec<String> = raw_text.lines().map(|s| s.to_string()).collect();
        
        if let Ok(mut window) = sliding_window.try_lock() {
            for line in lines {
                window.push(line);
                if window.len() > 10 { window.remove(0); }
                
                if let Ok(trigs) = dynamic_triggers.try_lock() {
                    let mut sorted_trigs: Vec<&MushTrigger> = trigs.values().collect();
                    sorted_trigs.sort_by_key(|t| t.priority);

                    for trig in sorted_trigs {
                        if let Ok(re) = Regex::new(&trig.match_text) {
                            let mut is_matched = false;
                            if trig.is_multi_line {
                                let start_idx = window.len().saturating_sub(trig.lines_to_match);
                                let compound_text = window[start_idx..].join("\n");
                                if re.is_match(&compound_text) { is_matched = true; }
                            } else {
                                if re.is_match(window.last().unwrap()) { is_matched = true; }
                            }

                            if is_matched {
                                let _ = terminal_tx.send(format!(
                                    "\x1b[1;34m[优先触发引擎(P:{})] 🎯 规则 [{}] 物理匹配中心切断成功！\x1b[0m\r\n", 
                                    trig.priority, trig.name
                                ));

                                // 🟢【核心修复点 1】：使用 .as_str() 将 String 物理转换为标准的 &str 类型，打通 IntoLua 契约！
                                if trig.response_text.starts_with("script:") {
                                    let func_name = trig.response_text.trim_start_matches("script:").to_string();
                                    if let Ok(lua) = lua_vm.try_lock() {
                                        if let Ok(callback) = lua.globals().get::<mlua::Function>(func_name.as_str()) {
                                            let _ = callback.call::<()>(()); // 瞬间通电激活 Lua 回调！
                                        }
                                    }
                                } else {
                                    let _ = tcp_cmd_tx.try_send(trig.response_text.clone());
                                }
                                break; 
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn process_outgoing_command(
        cmd: &str,
        dynamic_aliases: &Arc<Mutex<HashMap<String, MushAlias>>>,
        alias_intercepted: &mut bool,
    ) -> String {
        let mut final_cmd = cmd.to_string();
        if let Ok(aliases) = dynamic_aliases.try_lock() {
            let mut sorted_aliases: Vec<&MushAlias> = aliases.values().collect();
            sorted_aliases.sort_by_key(|a| a.priority);

            for alias in sorted_aliases {
                if let Ok(re) = Regex::new(&alias.match_text) {
                    if re.is_match(cmd) {
                        if alias.response_text.starts_with("script:") {
                            *alias_intercepted = true;
                        } else {
                            final_cmd = re.replace_all(cmd, &alias.response_text).into_owned();
                        }
                        break;
                    }
                }
            }
        }
        final_cmd
    }
}
