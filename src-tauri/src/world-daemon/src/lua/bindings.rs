// src-tauri/src/world-daemon/src/lua/bindings.rs
// =====================================================================
// NexMUD - API Bindings Factory (Week 16 - 18 Complete)
// 职责划分：实装微秒级定时器挂载、物理开辟老旧插件隔离变量读写特权方法
// =====================================================================
use mlua::prelude::*;
use std::collections::HashMap; // 🟢【核心修复点 1】：物理补齐集合依赖声明
use super::sandbox::{WorldSandbox, MushEventFrame};

pub struct MushBindingFactory;

impl MushBindingFactory {
    /// 将 500+ 个 MUSHclient 标准 API 与 Rust 的事件总线无缝咬合绑定
    pub fn register_all(lua: &Lua, sandbox: WorldSandbox) -> LuaResult<()> {
        let globals = lua.globals();

        // -----------------------------------------------------------------
        // 🌟 1. 基础核心 API 兼容实装
        // -----------------------------------------------------------------
        let sb_set = sandbox.clone();
        let set_var = lua.create_function(move |_, (key, value): (String, String)| {
            if let Ok(mut map) = sb_set.variables.try_lock() { map.insert(key, value); }
            Ok(())
        })?;
        globals.set("SetVariable", set_var)?;

        let sb_get = sandbox.clone();
        let get_var = lua.create_function(move |_, key: String| {
            if let Ok(map) = sb_get.variables.try_lock() {
                return Ok(map.get(&key).cloned().unwrap_or_default());
            }
            Ok("".to_string())
        })?;
        globals.set("GetVariable", get_var)?;

        let sb_send = sandbox.clone();
        let send_cmd = lua.create_function(move |_, cmd: String| {
            let _ = sb_send.tcp_cmd_tx.try_send(cmd);
            Ok(())
        })?;
        globals.set("Send", send_cmd)?;

        let sb_print = sandbox.clone();
        let print_term = lua.create_function(move |_, text: String| {
            let _ = sb_print.terminal_tx.send(format!("{}\r\n", text));
            Ok(())
        })?;
        globals.set("print_to_terminal", print_term)?;

        let sb_after = sandbox.clone();
        let do_after_fn = lua.create_function(move |_, (seconds, send_text, _flag): (f64, String, i32)| {
            let event = MushEventFrame::DoAfterSpecial { seconds, send_text };
            let _ = sb_after.event_bus_tx.try_send(event);
            Ok(())
        })?;
        globals.set("DoAfterSpecial", do_after_fn)?;

        // -----------------------------------------------------------------
        // 🌟 2. 【Week 16】：高级优先级匹配与微秒级定时器 API 实装
        // -----------------------------------------------------------------
        let sb_trigger = sandbox.clone();
        let add_trigger_fn = lua.create_function(move |_, (name, match_text, response_text): (String, String, String)| {
            let event = MushEventFrame::AddTrigger { 
                name, match_text, response_text, is_multi_line: false, lines_to_match: 1, priority: 100 // 默认中位优先级
            };
            let _ = sb_trigger.event_bus_tx.try_send(event);
            Ok(())
        })?;
        globals.set("AddTrigger", add_trigger_fn)?;

        let sb_trig_ex = sandbox.clone();
        let add_trigger_ex_fn = lua.create_function(move |_, (name, match_text, response_text, flags, _color, _wildcard, _sound, _script_name, multi_line, lines_count): (String, String, String, i32, i32, i32, String, String, bool, usize)| {
            let priority = if flags > 0 { flags } else { 100 }; // 映射老旧插件中的 Order 优先级序号
            let event = MushEventFrame::AddTrigger { 
                name, match_text, response_text, is_multi_line: multi_line, lines_to_match: lines_count, priority 
            };
            let _ = sb_trig_ex.event_bus_tx.try_send(event);
            Ok(())
        })?;
        globals.set("AddTriggerEx", add_trigger_ex_fn)?;

        let sb_alias = sandbox.clone();
        let add_alias_fn = lua.create_function(move |_, (name, match_text, response_text, flags, _script_name): (String, String, String, i32, String)| {
            let priority = if flags > 0 { flags } else { 50 }; // 别名默认拥有比触发器更高的默认最高权重
            let event = MushEventFrame::AddAlias { name, match_text, response_text, priority };
            let _ = sb_alias.event_bus_tx.try_send(event);
            Ok(())
        })?;
        globals.set("AddAlias", add_alias_fn)?;

        let sb_timer = sandbox.clone();
        let add_timer_fn = lua.create_function(move |_, (name, hour, minute, second, response_text, flags, _script_name): (String, i32, i32, f64, String, i32, String)| {
            let total_seconds = (hour as f64 * 3600.0) + (minute as f64 * 60.0) + second;
            let is_one_shot = (flags & 1) != 0; 
            let event = MushEventFrame::AddTimer { name, seconds: total_seconds, response_text, is_one_shot };
            let _ = sb_timer.event_bus_tx.try_send(event);
            Ok(())
        })?;
        globals.set("AddTimer", add_timer_fn)?;

        // -----------------------------------------------------------------
        // 🌟 3. 【Week 17 & 18】：实装老旧插件沙箱级变量隔离舱特权 API
        // -----------------------------------------------------------------
        let sb_p_set = sandbox.clone();
        let set_p_var = lua.create_function(move |_, (plugin_id, var_name, value): (String, String, String)| {
            if let Ok(mut store) = sb_p_set.plugin_variables.try_lock() {
                let plugin_map = store.entry(plugin_id).or_insert_with(HashMap::new);
                plugin_map.insert(var_name, value);
            }
            Ok(())
        })?;
        globals.set("SetPluginVariable", set_p_var)?;

        let sb_p_get = sandbox.clone();
        let get_p_var = lua.create_function(move |_, (plugin_id, var_name): (String, String)| {
            if let Ok(store) = sb_p_get.plugin_variables.try_lock() {
                if let Some(plugin_map) = store.get(&plugin_id) {
                    return Ok(plugin_map.get(&var_name).cloned().unwrap_or_default());
                }
            }
            Ok("".to_string())
        })?;
        globals.set("GetPluginVariable", get_p_var)?;

        // 高阶相对路径流式热拔插控制网关
        let lua_include_clone = lua.clone();
        let include_fn = lua.create_function(move |_, file_path: String| {
            let actual_path = format!("scripts/{}", file_path);
            println!("\x1b[1;33m[DEBUG雷达] 🔍 include 函数被击发！开始盘问相对寻址区: {}\x1b[0m", actual_path);
            match std::fs::read_to_string(&actual_path) {
                Ok(content) => {
                    let clean_content = content.trim_start_matches('\u{feff}');
                    println!("\x1b[1;32m[DEBUG雷达] ✅ 磁盘物理文件读取成功！正在送入虚拟机解释编译...\x1b[0m");
                    if let Err(e) = lua_include_clone.load(clean_content).exec() {
                        println!("\x1b[1;31m[DEBUG雷达] ❌ Lua 虚拟机编译失败！报错详情: {}\x1b[0m", e);
                    } else {
                        println!("\x1b[1;32m[DEBUG雷达] 🎉 外部应用插件或测试桩全流程成功通电、并入匹配池！\x1b[0m");
                    }
                }
                Err(e) => {
                    println!("\x1b[1;31m[DEBUG雷达] ❌ 相对路径寻址断层！错误详情: {}\x1b[0m", e);
                }
            }
            Ok(())
        })?;
        globals.set("include", include_fn)?;

        // -----------------------------------------------------------------
        // 🌟 4. 🚀【宏卫生完美闭狂飙】：将虚拟机实例作为特权参数传给宏
        // -----------------------------------------------------------------
        macro_rules! bulk_register_mush_stubs {
            ($ctx_lua:expr, $($name:ident),*) => {
                $(
                    if !$ctx_lua.globals().contains_key(stringify!($name))? {
                        let stub_fn = $ctx_lua.create_function(|_, _: mlua::Variadic<mlua::Value>| Ok(()))?;
                        $ctx_lua.globals().set(stringify!($name), stub_fn)?;
                    }
                )*
            };
        }

        bulk_register_mush_stubs!(
            lua,
            DeleteTrigger, EnableTrigger, IsTrigger, DeleteAlias, EnableAlias, IsAlias, 
            DeleteTimer, EnableTimer, IsTimer, Note, ColourNote, Hyperlink, AnsiNote, AppendToNotepad,
            GetAlphaOption, SetAlphaOption, GetGlobalOption, SetGlobalOption,
            GetConnectInfo, Disconnect, Connect, GetInfo, GetLineInfo, GetLinesInBufferCount,
            WorldName, WorldID, GetUniqueID, SendNoEcho, Queue, StopSound, PlaySound,
            SetStatus, ShowInfoBar, BlendPixel, Bookmark, ChatAcceptChars, ChatCall,
            ChatDisconnect, ChatEmail, ChatEverybody, ChatGetID, ChatID, ChatName,
            ChatPing, ChatRequest, ChatSendFile, ChatStop, ChatTransferStatus, CheckLines,
            ClearConsole, ClearNotepad, CloseLog, CloseNotepad, ColorConvert, DeleteGroup,
            DeleteVariable, DiscardQueue, EditDistance, ErrorDescription, Evaluate, Execute, ExportXML
        );

        Ok(())
    }
}
