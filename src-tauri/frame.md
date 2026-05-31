我们将整个生命周期划分为 6 个阶段（共 24 周/约 6 个月）。这里为你拆解出最精细的落地执行蓝图：

📅 NexMUD 落地实施总览看板
阶段	核心目标	关键攻坚点	周期
Phase 0	基础设施与骨架搭建	仓库多包配置、gRPC 原型、Tauri 与 React 联通	W1 - W2
Phase 1	进程模型与 IPC 机制	Broker 进程守护、Daemon 动态拉起、状态同步	W3 - W5
Phase 2	网络协议与高性能渲染	Tokio Telnet 状态机、Canvas/WebGL 文本流渲染	W6 - W9
Phase 3	Lua 引擎与自动代码生成	mlua 沙箱、MUSHclient API 自动生成器、死循环隔离	W10 - W14
Phase 4	MUD 核心引擎与插件	Trigger/Alias/Timer 匹配引擎、XML 转换工具	W15 - W18
Phase 5	高级协议与性能调优	MCCP/MXP 补全、万人 Spam 压测、内存指标对齐	W19 - W22
Phase 6	质量守卫与发布	兼容性测试卡点、CI/CD 自动化、v1.0.0 正式发布	W23 - W24
🔍 详细执行计划（精细到任务级）
Phase 0: 基础设施与骨架搭建 (Week 1 - Week 2)
目标：确立强制性的目录结构，让 AI Agent 和人类开发者有统一的代码战场。

Week 1: 现代多包存储库 (Monorepo) 配置

[ ] 使用 pnpm + Cargo 工作空间（Workspace）初始化根目录。

[ ] 配置全局 Lint 与格式化工具：clippy, eslint, prettier，并在 .githooks 中配置 pre-commit 强校验。

[ ] 编写 CLAUDE.md，作为 AI 助手的上下文记忆上下文。

Week 2: 定义单一事实来源 (gRPC & Proto)

[ ] 在 proto/ 下编写 broker.proto（包含 SpawnWorld, KillWorld, GetWorldStatus 接口）。

[ ] 编写 world.proto（包含 SendInput, StreamOutput, UpdateConfig 接口）。

[ ] 配置 tonic-build，确保 src-tauri 编译时自动生成 Rust 的 gRPC Client/Server 代码。

Phase 1: 进程模型与 IPC 突破 (Week 3 - Week 5)
目标：攻克“多进程隔离”这一核心宪法约束，实现主进程对子进程的绝对控制。

[Tauri Broker] ──(Command: Spawn)──> [OS Child Process: World Daemon]
[Tauri Broker] <──(gRPC: Connect)──── [World Daemon (gRPC Server)]
Week 3: Broker 守护进程实现

[ ] 在 broker/ 模块中实现子进程管理器（基于 std::process::Command）。

[ ] 实现动态端口分配机制：Daemon 启动时随机绑定本地高位端口，并通过 stdout 将端口号回报给 Broker。

Week 4: Daemon 骨架与 gRPC 通信

[ ] 实现 world-daemon 的独立编译入口，确保它可以作为独立二进制文件运行。

[ ] 落地 gRPC 双向流（Bidirectional Streaming）：Daemon 将收到的 MUD 文本实时 Stream 给 Broker。

Week 5: UI 与 Broker 联动

[ ] 前端采用 React 19 + Tailwind CSS 4.x 搭建多标签（Tabs）管理界面。

[ ] 技术选型落地：前端与 Broker 之间采用 Tauri 的 invoke 处理控制指令，高频文本流通过内嵌的 WebSocket 或 Tauri V2 的 Channel 机制传递，规避序列化性能瓶颈。

Phase 2: 网络协议层与高性能渲染 (Week 6 - Week 9)
目标：解决“高频文本流（Spam）卡顿”和“标准 Telnet 解析”问题。

Week 6: Tokio Telnet 基础状态机

[ ] 在 telnet/ 模块中基于 tokio::net::TcpStream 实现非阻塞网络连接。

[ ] 构建 RFC 854 标准的 Telnet 状态机，精确解析 IAC (255)、DO/DONT/WILL/WONT 字节流。

Week 7 & 8: 高性能终端渲染器（技术卡点攻坚）

[ ] 严禁使用原生 DOM。引入基于 Canvas/WebGL 的渲染引擎（如深度定制的 xterm.js 或自研 Canvas 渲染层）。

[ ] 实现 ANSI Escape Code（颜色、粗体、闪烁、背景色）的解析器，将其转化为 Canvas 渲染指令。

[ ] 测试极限吞吐量：模拟每秒 10,000 行文本刷屏，要求 CPU 占用率 <15%，无掉帧卡顿。

Week 9: 基础功能闭环

[ ] 前端输入框发送命令 -> Broker -> gRPC -> Daemon -> MUD 服务器。

[ ] MUD 服务器响应 -> Daemon -> gRPC Stream -> Broker -> UI Canvas 渲染。

Phase 3: Lua 引擎与自动代码生成 (Week 10 - Week 14)
目标：实现 100% MUSHclient Lua API 兼容性，并确保死循环无法卡死主程序。

Week 10: mlua 沙箱环境构建

[ ] 在 Daemon 内初始化 mlua::Lua 实例，加载 Lua 5.1/5.4 运行时。

[ ] 严格裁剪 Lua 标准库，禁用 os.execute, io 等危险操作，确保沙箱安全性。

Week 11: MUSHclient API 自动化生成器 (Tools)

[ ] 编写 tools/api-generator（基于 Python 或 Rust），解析 MUSHclient 官方 XML/HTML 文档的 API 导出表。

[ ] 自动生成 src-tauri/src/world-daemon/lua/generated_bindings.rs，包含 500+ 个 MUSHclient 标准 API 的 Rust 签名骨架。

Week 12 & 13: 核心高级 API 精细实现

[ ] 手动啃硬骨头：实现 GetVariable, SetVariable, DoAfterSpecial, Send 等涉及跨模块调度的核心 API。

[ ] 建立 Lua 与 Rust 核心事件总线（Event Bus）的映射。

Week 14: 脚本死循环隔离测试

[ ] 利用 lua.set_hook 设置指令计数器（Instruction Counter），当单个脚本连续执行超过 2 秒或指定步数时，强制抛出错误。

[ ] 编写测试：故意在 Lua 中写入 while true do end，验证该子进程崩溃/被捕获时，其余标签页的 MUD 连接完好无损。

Phase 4: MUD 核心引擎与插件系统 (Week 15 - Week 18)
目标：实现 MUD 客户端的灵魂——触发器和宏，以及旧插件重生。

Week 15: 文本匹配引擎 (Triggers / Aliases)

[ ] 基于高性能正则库 regex（Rust），在 core/ 层实现触发器和别名匹配引擎。

[ ] 支持多行触发（Multi-line triggers）与带颜色触发（Color-matching triggers）。

Week 16: 定时器与高级调度

[ ] 基于 tokio::time 实现微秒级定时器（Timers）。

[ ] 实现优先级队列，确保当 Trigger、Alias、Timer 同时触发时，按 MUSHclient 标准优先级顺序执行。

Week 17 & 18: 旧插件转换器与现代化配置

[ ] 编写官方转换脚本：将 MUSHclient 的 .xml 插件解析并导出为 NexMUD 的新格式（JSON/YAML 配置 + 分离的 .lua 脚本）。

[ ] 落地插件沙箱，隔离不同插件的全局变量空间。

Phase 5: 高级协议与全面性能调优 (Week 19 - Week 22)
目标：补全高级协议，榨干 Rust 后端和前端渲染的最后一点性能。

Week 19: 高级 MUD 协议支持

[ ] 实现 MCCP v1/v2：集成 flate2 库，在 Telnet 层对数据流进行 zlib 解压。

[ ] 实现 MXP (MUD eXtension Protocol)：解析超链接、字体改变及图片标记。

[ ] 实现 MSDP / GMCP：实现带外数据（Out-of-band data）解析，为前端 UI 仪表盘（血条、蓝条）提供结构化数据。

Week 20 & 21: 极端压力测试与内存泄露排查

[ ] 使用 tracy 或 cargo-valgrind 监控多进程的内存。

[ ] 模拟重度玩家挂机 72 小时，确保单进程内存占用维持在 50MB 以下，无因 Lua 频繁 GC 导致的垃圾堆积。

Week 22: UI/UX 抛光

[ ] 利用 Tailwind CSS 4.x 的新特性，打磨精致的暗黑系 MUD 界面。

[ ] 落地快捷键绑定系统、命令历史回溯（Command History）及小地图/状态栏组件。

Phase 6: 质量守卫与交付发布 (Week 23 - Week 24)
目标：零缺陷通过兼容性测试，合入主干并发布。

Week 23: 兼容性卡点测试（Zero Failure）

[ ] 运行 tests/compatibility/ 下的自动化测试，确保常用的大型 MUD 机器人脚本（如《北大侠客行》、《西游记》的复杂 Lua 机器人）在 NexMUD 中无需修改直接跑通。

[ ] 完善协议合规性测试。

Week 24: CI/CD 与多平台分发

[ ] 配置 GitHub Actions：触发 cargo tauri build。

[ ] 自动化构建出 Windows (.msi, .exe)、macOS (.dmg, Apple Silicon/Intel 双架构)、Linux (.deb, .AppImage) 的多平台安装包。

[ ] 正式发布 v1.0.0-RELEASE。

⚠️ 战术级避坑指南（关键风险红线）
关于 gRPC 序列化开销：Daemon 将字符流传给 Broker 时，千万不要逐字或逐行发送。必须在 Daemon 端建立一个 20ms 延时的微型缓冲区（Buffer），合并成块（Chunk）后打包发送，否则 gRPC 的 Context 切换会把 CPU 吃满。

关于 Lua 与 Rust 的类型转换：mlua 在处理大批量 Trigger 传参时，频繁的 String 拷贝非常昂贵。能用 &str 的地方绝不用 String，尽量利用 Rust 的生命周期和借用机制。

关于 Windows 平台的子进程隐患：在 Windows 上，如果主进程被强制结束（如任务管理器杀死），子进程可能会变成孤儿进程继续留在后台。必须在 Rust 中利用 Win32 Job Object 将子进程与主进程绑定，确保主死随死。

这是一个标准的、面向中大型重构项目的工业级推进计划。你打算一个人+AI 独立全栈开发，还是有前端/后端研发伙伴协作？我们可以针对你的团队构成，进一步细化分工和首周的初始化代码（比如先写出 Proto 文件）。

以下是为您制定的 NexMUD 24周逐阶段验收指标与 Demo 确认方案：
Phase 0: 基础设施与骨架搭建 (W1 - W2)验收指标：Monorepo 目录结构严丝合缝，没有跨模块的相对路径越界引用。cargo check 与 pnpm lint 在 CI 中平滑通过，警告归零。Protobuf 成功编译，Rust 自动生成对应的 tonic 桩代码。Demo 确认方案：运行命令：在根目录执行一个脚本 pnpm dev:all。现象呈现：控制台无报错启动，打印出自动生成的 broker.rs 和 world.rs 的绝对路径。通过本地 grpcurl 工具，能够成功扫描到暴露的 gRPC 服务列表（反射机制）。
Phase 1: 进程模型与 IPC 机制 (W3 - W5)验收指标：主进程与子进程生命周期完全绑定。通过 gRPC Stream 实现无阻塞的跨进程数据传输。前端 React 19 能流畅接收到跨进程分发过来的模拟数据。Demo 确认方案【关键硬核 Demo】：操作步骤：打开 Tauri UI 界面，点击“新建世界”按钮 3 次（标签页：A、B、C）。打开操作系统的任务管理器/活动监视器。预期现象：任务管理器中显式出现 1 个 Tauri 主进程和 3 个独立的 world-daemon 子进程（每个进程有独立的 PID）。在标签页 A 中点击“发送测试数据”，只有 A 对应的子进程 PID 收到请求，并在 UI 上回显。容灾测试：在任务管理器中**强制结束（Kill）**标签页 B 的进程。UI 上 B 标签立即显示“连接断开，正在自动拉起...”，并在 1 秒内自动生成一个新的 world-daemon 进程（新 PID），而 A 和 C 标签页完全不受影响。

Phase 2: 网络协议层与高性能渲染 (W6 - W9)验收指标：正确解析标准的 Telnet 协商指令，不把 IAC 字符误渲染为乱码。Canvas/WebGL 终端组件对高频文本无掉帧、无内存泄漏。Demo 确认方案：运行环境：启动一个本地的 Mock TCP Server，以每秒 5,000 行的速度向客户端狂喷带有 ANSI 颜色代码（如 \x1b[31m）的文本流。现象呈现：NexMUD UI 的 Canvas 区域开始极速滚屏，文本颜色、粗体格式正确渲染。打开 Chrome DevTools（Tauri 调试窗口），查看 Performance 面板：FPS 稳定在 55-60 帧，DOM 节点数量恒定（证明使用的是虚拟列表或 Canvas 缓存），内存占用曲线呈平稳锯齿状（GC 正常）。

Phase 3: Lua 引擎与自动代码生成 (W10 - W14)验收指标：通过工具自动生成的 500+ 个 API 骨架在 Rust 中编译通过。Lua 沙箱能够阻断恶意代码与死循环。Demo 确认方案：操作步骤：在前端的“脚本控制台”中，手动输入一段恶意的 Lua 代码并执行：Lua-- 死循环测试
while true do
    -- Nothing
end
预期现象：主界面 React UI 依然可以自由点击、切换标签，完全没有卡死。大约 2 秒后，控制台弹窗或打印红字错误：[Lua Runtime Error]: Script execution aborted. Instruction limit exceeded (Timeout). 证明 mlua 钩子成功熔断了死循环。

Phase 4: MUD 核心引擎与插件系统 (W15 - W18)验收指标：触发器（Trigger）在 Rust 核心层的匹配延迟小于 1 毫秒。MUSHclient 旧 XML 插件能被成功解析并转换为新系统的沙箱插件。Demo 确认方案：操作步骤：在 UI 界面配置一个 Trigger：匹配正则为 ^你的气血：(\d+)/(\d+)，触发动作（Lua）为 if %1 < 100 then Send("eat medicine") end。运行官方提供的 plugin-converter 工具，导入一个旧的 MUSHclient 经典插件（如大侠客行的自动练功 .xml）。预期现象：向 Mock Server 发送 你的气血：50/200，Canvas 终端中下一行瞬间自动吐出向服务器发送的命令 eat medicine。转换后的插件成功加载进子进程，并在插件列表中显示绿灯“Active”，其内部定义的别名（Alias）全部生效。

Phase 5: 高级协议与全面性能调优 (W19 - W22)验收指标：MCCP 压缩开启后，网络吞吐量大幅下降，数据解析无误。GMCP/MSDP 带外数据能直接驱动前端 React 组件的状态更新。Demo 确认方案：操作步骤：连接到一个支持 GMCP 协议的真实/模拟 MUD 服务器（会发送类似 Char.Vitals { hp: 150, maxhp: 200 } 的 JSON 数据）。预期现象：网络监控日志显示数据流经过了 zlib 解压（MCCP 生效）。Canvas 渲染区没有打印出这段 GMCP 的结构化 JSON，但是 NexMUD 界面底部的 React 血条组件（UI Widget）随之缩短或伸长，证明带外数据成功走通了 Daemon -> gRPC -> Broker -> React State 链路。

Phase 6: 质量守卫与交付发布 (W23 - W24)验收指标：集成测试套件覆盖率 $> 80\%$。跨平台编译产物完备。Demo 确认方案：现象呈现：GitHub Actions 触发，流水线全绿通过。在 Release 页面自动生成 NexMUD_windows_x64.msi、NexMUD_macos_aarch64.dmg 以及 NexMUD_linux_amd64.deb。下载任意一个安装包，在一台干净的、未配置开发环境的机器上双击安装，能正常打开、连接服务器并畅玩 1 小时无崩溃。