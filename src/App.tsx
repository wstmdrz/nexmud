import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core'; // 注意：Tauri v2 是 @tauri-apps/api/core
import Ansi from 'ansi-to-react';

function App() {
  const [logs, setLogs] = useState<string[]>(["[系统] 欢迎来到 NexMUD 终端..."]);
  const [input, setInput] = useState('');

  // 🌟 1. 这是一个“锚点”，用来让终端自动滚动到底部
  const bottomRef = useRef<HTMLDivElement>(null);

  // 🌟 2. 只要 logs 发生变化，就平滑滚动到底部
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  // 🌟 新增：组件一加载，就自动调用 Tauri 后端的连接指令
  useEffect(() => {
    const bootSequence = async () => {
      try {
        setLogs(prev => [...prev, "[系统] 正在建立与世界守护进程的神经连接..."]);

        // ⚠️ 极其关键：把下面的 'init_connection' 替换成你之前“初始化神经连接”按钮实际调用的命令名！
        // 可能是 'start_stream' / 'connect_grpc' / 'init_mud' 等，看你 main.rs 里是怎么写的
        await invoke('init_connection');

      } catch (err) {
        setLogs(prev => [...prev, `\x1b[31m[系统] 连接失败: ${err}\x1b[0m`]);
      }
    };

    bootSequence();
  }, []);


  // 🌟 3. 监听后端发来的 MUD 数据
  useEffect(() => {
    const unlisten = listen<string>('mud-stream-event', (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // 🌟 4. 处理回车发送指令
  const handleSend = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim() !== '') {
      const cmd = input.trim();
      setInput(''); // 清空输入框

      // (可选) 在本地回显你输入的命令，加上黄色的 ANSI 颜色码，方便区分你输入的和服务器返回的
      setLogs((prev) => [...prev, `\x1b[33m> ${cmd}\x1b[0m`]);

      try {
        // 调用你 Tauri (Broker) 里的 command
        // ⚠️ 注意：这里的 'send_command' 必须和你 src-tauri/src/main.rs 里 #[tauri::command] 的名字对应！
        // 传递的参数名（比如 cmd 还是 input）也要和 Rust 里的参数名严格一致。
        await invoke('send_command', { input: cmd });
      } catch (err) {
        console.error("发送指令失败:", err);
        setLogs((prev) => [...prev, `\x1b[31m[系统] 指令发送失败: ${err}\x1b[0m`]);
      }
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', backgroundColor: '#000', color: '#fff', fontFamily: 'monospace', fontSize: '15px' }}>

      {/* ===== 终端显示区 ===== */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px' }}>
        {logs.map((log, index) => (
          <div key={index} style={{ whiteSpace: 'pre-wrap', marginBottom: '2px', wordBreak: 'break-word', lineHeight: '1.4' }}>
            <Ansi useClasses={false}>{log}</Ansi>
          </div>
        ))}
        {/* 这里放置我们的滚动锚点 */}
        <div ref={bottomRef} />
      </div>

      {/* ===== 底部输入区 ===== */}
      <div style={{ display: 'flex', alignItems: 'center', padding: '12px 16px', backgroundColor: '#111', borderTop: '1px solid #333' }}>
        <span style={{ color: '#0f0', marginRight: '10px', fontWeight: 'bold' }}>$&gt;</span>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleSend}
          style={{
            flex: 1,
            backgroundColor: 'transparent',
            border: 'none',
            color: '#fff',
            outline: 'none',
            fontFamily: 'monospace',
            fontSize: '16px'
          }}
          autoFocus
          placeholder="在此输入指令，按回车发送..."
        />
      </div>
    </div>
  );
}

export default App;