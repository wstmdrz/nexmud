# NexMUD - Next Generation Distributed Client & Engine

基于 **Tauri 2.0 + React 19 + pnpm Monorepo + Rust gRPC (Tonic)** 架构的次世代 MUD 客户端复刻版。

## 🏗️ 架构宪法 (Architecture Overview)

本项目颠覆了传统 MUD 客户端（如 MUSHclient、Mudlet）的单进程阻塞单例模型，全面倒向微服务分布式多进程沙箱架构。从根源上斩断了客户端在遭遇高频文本、巨量触发器断言或复杂 Lua 脚本假死时带来的主界面掉帧、卡死与闪退隐患：

- **`nexmud-broker` (主进程 / Tauri 2.0)**: 负责外壳 GUI 渲染、全局路由分发、权限隔离，以及通过原子计数器动态分发通信端口，实现对子进程生命周期的异步生成、监控与守护。
- **`world-daemon` (子进程 / Rust)**: 每一个独立的 MUD 游戏世界（Tab页）在后台完全由主进程独立 fork 出来一个沙箱载体。内部集成标准的 **Telnet 状态机动态自适应协议协商** 突破防火墙，搭载高级异步 TCP 管道，并配合 `encoding_rs::GBK` 满载执行极速非阻塞实时解码。
- **`gRPC 通信隧道 (Tonic)`**: 跨进程交互彻底抛弃原始的匿名管道劫持，改用基于 TCP 环回网络的高性能本地 gRPC 契约（`50051`, `50052`, `50053` ...），通过 `stream_output` 申请无阻塞的跨进程高性能数据穿透流订阅。
- **`JSON 统一路由命名空间`**: 后端序列化严格对齐 `#[serde(rename_all = "camelCase")]` 规则，实现面向 React 19 多标签状态机缓存的 100% 精准、解耦、定向日志投递。

## 🚀 研发状态 (Milestones)

### 🟢 Phase 0 & Phase 1: 基础设施与微服务进程模型 (W1 - W5) — 【已闭环】
- [x] pnpm Monorepo 依赖共享工作区建立，彻底隔绝前后端环境。
- [x] Cargo Workspace 后端多包解耦配置，共享 `tokio`、`tonic` 与 `prost` 祖先依赖。
- [x] 确立以 `proto/world.proto` 为单一事实来源（Single Source of Truth）的契约定义与自动反射机制。
- [x] 攻克 Windows 平台特有的 `0xc0000139` 动态链接符号污染与 `rc.exe` 图标打包硬编码拦截。
- [x] **【硬核 Demo 交付 1】**：前端 React 19 完美支持点击/自动循环加载多标签页视窗。
- [x] **【硬核 Demo 交付 2】**：任务管理器中 1 个母舰外壳完美挂载 3 个独立 PID 的 `world-daemon.exe` 矩阵，各自独占 50051/50052/50053 通信端口。
- [x] **【硬核 Demo 交付 3】**：公网真实游戏世界压测。成功接入 `mud.pkuxkx.net:1234`（北大侠客行），实现每秒数万字全色彩 ANSI 高频文本流跨进程零延迟、零卡顿、丝滑分流刷屏。
- [x] **【硬核 Demo 交付 4（断线自愈）】**：在任务管理器中强制 Kill 结束掉任意一个子进程，受损标签页可在 1 秒之内抛出灾备红字并自动孵化全新 PID 满血重现，其余并行的游戏世界完全不受波及。
