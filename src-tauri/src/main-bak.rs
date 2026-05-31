#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

fn main() {
    println!("🚀 NexMUD Broker (主进程) 启动...");

    tauri::Builder::default()
        // 1. 初始化 Shell 插件
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 2. 通过简写名称获取 sidecar 句柄
            let sidecar_command = app.shell().sidecar("world-daemon").unwrap();
            
            // 3. 异步拉起子进程
            let (mut rx, _child) = sidecar_command
                .spawn()
                .expect("致命错误：无法启动 world-daemon 子进程");

            println!("✅ 成功拉起 World Daemon 子进程！正在监听其输出...");

            // 4. 开启一个后台协程，实时接管并打印子进程的控制台输出
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    if let CommandEvent::Stdout(line) = event {
                        // 子进程的标准输出会被捕获并转发到这里
                        let text = String::from_utf8_lossy(&line);
                        println!("[Daemon] {}", text);
                    } else if let CommandEvent::Stderr(line) = event {
                        let text = String::from_utf8_lossy(&line);
                        eprintln!("[Daemon Error] {}", text);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用程序时发生错误");
}