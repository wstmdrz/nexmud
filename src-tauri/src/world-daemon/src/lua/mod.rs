// src-tauri/src/world-daemon/src/lua/mod.rs
pub mod sandbox;
pub mod bindings;
pub mod engine;
pub mod protocol; // 🟢【模块解耦引入点】：将全新的高级协议状态机大模块引出

use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs;

pub struct LuaLifecycleManager;

impl LuaLifecycleManager {
    pub fn boot_sandbox(sandbox: sandbox::WorldSandbox) -> Result<Arc<Mutex<mlua::Lua>>, Box<dyn std::error::Error>> {
        let lua = mlua::Lua::new();

        bindings::MushBindingFactory::register_all(&lua, sandbox)?;

        let script_path = "scripts/pkuxkx_register.lua";
        match fs::read_to_string(script_path) {
            Ok(script_content) => {
                let clean_content = script_content.trim_start_matches('\u{feff}');
                lua.load(clean_content).exec()?;
                println!("[Lua Sandbox] ⚡ 外部控制自动化应用插件成功通电。");
            }
            Err(_) => {
                lua.load(r#"
                    function handle_alias() return false end
                    function on_text_trigger() end
                "#).exec()?;
            }
        }

        let start_time = Arc::new(Mutex::new(std::time::Instant::now()));
        let watchdog_time = start_time.clone();
        lua.set_hook(mlua::HookTriggers {
            every_line: false, every_nth_instruction: Some(10000), on_calls: false, on_returns: false,
        }, move |_, _| {
            if let Ok(t) = watchdog_time.try_lock() {
                if t.elapsed() > std::time::Duration::from_secs(2) {
                    eprintln!("💥 [Watchdog 熔断器] 脚本执行超时自毁！");
                    std::process::exit(9); 
                }
            }
            Ok(mlua::VmState::Continue)
        });

        Ok(Arc::new(Mutex::new(lua)))
    }
}
