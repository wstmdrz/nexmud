# =====================================================================
# NexMUD - 一键自动化工程装配与多进程自愈通电脚手架 (dev.ps1)
# 职责划分：核心负责僵尸进程强杀、老旧XML插件官方转换重生、测试资产的一键物理拷贝同步
# =====================================================================

Clear-Host
Write-Host --------------------------------------------------------- -ForegroundColor Cyan
Write-Host NexMUD_工业级自动化流水线：开始执行老旧_XML_配置反射重构与清场... -ForegroundColor Cyan
Write-Host --------------------------------------------------------- -ForegroundColor Cyan

# 1. ?? 强力刮骨疗毒：绝不留情地强杀所有系统隐藏的文件进程锁
Write-Host 1._正在枪毙系统底层残留的旧_Broker_外壳与_Daemon_隔离沙箱... -ForegroundColor Yellow
Stop-Process -Name "nexmud-broker", "world-daemon" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# 2. ?? 后端完全体极速编译
Write-Host 2._正在调用_Cargo_工作空间，满载编译高度解耦的_C/Rust_子进程模块... -ForegroundColor Yellow
cd src-tauri
cargo build -p world-daemon
if ($LASTEXITCODE -ne 0) {
    Write-Host ?_编译终止：后端源码存在编译阻碍，请修正后重试！ -ForegroundColor Red
    cd ..
    Exit $LASTEXITCODE
}

# 3. ? 特权 Sidecar 看护区物理强刷覆盖
Write-Host 3._正在将全新编译的完全体子进程强刷复写至特权二进制看护区... -ForegroundColor Yellow
Copy-Item -Path "target/debug/world-daemon.exe" -Destination "binaries/world-daemon-x86_64-pc-windows-msvc.exe" -Force

# 4. ?【Week 17 & 18 新规】：一键编译并调用官方转换脚本，动态将老旧 XML 转换重生为现代解耦格式！
Write-Host 4._正在唤醒官方转换子系统，流式反射重生磁盘外部老旧_XML_插件资产... -ForegroundColor Yellow
cargo run --bin plugin_converter
if ($LASTEXITCODE -ne 0) {
    Write-Host ?_转换终止：XML插件格式存在严重断层！ -ForegroundColor Red
    cd ..
    Exit $LASTEXITCODE
}
cd ..

# 5. ?【物理拷贝对齐】：将转换重生的 3 大应用脚本，一键打包进运行区下的相对路径下
Write-Host 5._正在同步重组后的解耦脚本和_manifest.json_至开发运行看护区... -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "src-tauri/binaries/scripts" > $null

Copy-Item -Path src-tauri/scripts/pkuxkx_register.lua -Destination src-tauri/binaries/scripts/pkuxkx_register.lua -Force
Copy-Item -Path src-tauri/scripts/test_watchdog.lua -Destination src-tauri/binaries/scripts/test_watchdog.lua -Force
Copy-Item -Path src-tauri/scripts/test_trigger.lua -Destination src-tauri/binaries/scripts/test_trigger.lua -Force
Copy-Item -Path src-tauri/scripts/plugin_manifest.json -Destination src-tauri/binaries/scripts/plugin_manifest.json -Force

# 6. ? 自动化多进程通电起飞大结局
Write-Host --------------------------------------------------------- -ForegroundColor Green
Write-Host 资产重构及转换大获全胜！一键唤醒_React_19_+_Tauri_2.0_分布式母舰！ -ForegroundColor Green
Write-Host --------------------------------------------------------- -ForegroundColor Green

pnpm dev
