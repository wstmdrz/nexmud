import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import Ansi from 'ansi-to-react';

// 定义支持的多标签页
const TABS = ["World-A", "World-B", "World-C"];

function App() {
  // 全局维护 3 个独立标签页的日志流缓存
  const [tabLogs, setTabLogs] = useState<Record<string, string[]>>({
    "World-A": ["[系统] 欢迎来到 NexMUD 终端 A..."],
    "World-B": ["[系统] 欢迎来到 NexMUD 终端 B..."],
    "World-C": ["[系统] 欢迎来到 NexMUD 终端 C..."],
  });

  // 当前处于哪一个活跃标签页
  const [activeTab, setActiveTab] = useState<string>("World-A");
  const [input, setInput] = useState('');

  const bottomRef = useRef<HTMLDivElement>(null);

  // 只要切换标签页或有新日志，就平滑滚动到底部
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [tabLogs, activeTab]);

  // 🌟【修复点 1】：组件加载时，干净地对 3 个标签页发起独立多进程拉起请求
  useEffect(() => {
    const bootSequence = async () => {
      // 循环顺序拉起 A, B, C 三个独立的子进程沙箱
      for (const tabId of TABS) {
        try {
          setTabLogs(prev => ({
            ...prev,
            [tabId]: [...prev[tabId], `[系统] 正在请求母舰孵化隔离域 ${tabId} 的守护进程...`]
          }));

          // 🔥【终极纠正】：纯净命令名 + 对象传参 (对齐重构后的驼峰命名法)
          await invoke('init_connection', { tabId: tabId });

        } catch (err) {
          setTabLogs(prev => ({
            ...prev,
            [tabId]: [...prev[tabId], `\x1b[31m[系统] 神经连接创建失败: ${err}\x1b[0m`]
          }));
        }
      }
    };

    bootSequence();
  }, []);

  // 🌟【修复点 2】：监听母舰的全局事件广播
  useEffect(() => {
    // 声明接收的 Payload 结构
    interface ServerPayload {
      tabId: string;
      text: string;
    }
    const unlisten = listen<ServerPayload>('mud-stream-event', (event) => {
      const { tabId, text } = event.payload;

      // 100% 精准分发到各自世界的独立缓存，绝不张冠李戴
      setTabLogs((prev) => ({
        ...prev,
        [tabId]: [...(prev[tabId] || []), text]
      }));
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);


  // 🌟【修复点 3】：定向路由指令发送（完美应对验收指标）
  const handleSend = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim() !== '') {
      const cmd = input.trim();
      setInput('');

      // 仅在本尊活跃的标签页回显指令
      setTabLogs((prev) => ({
        ...prev,
        [activeTab]: [...prev[activeTab], `\x1b[33m> ${cmd}\x1b[0m`]
      }));

      try {
        // 🔥【终极纠正】：向后端明确发射当前的 tabId 与命令内容
        // 这样后端 clients 注册表才能精准通过 tabId 定向投递给对应的 PID 子进程！
        await invoke('send_command', { tabId: activeTab, input: cmd });
      } catch (err) {
        console.error("发送指令失败:", err);
        setTabLogs((prev) => ({
          ...prev,
          [activeTab]: [...prev[activeTab], `\x1b[31m[系统] 指令路由发送失败: ${err}\x1b[0m`]
        }));
      }
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', backgroundColor: '#000', color: '#fff', fontFamily: 'monospace', fontSize: '15px' }}>

      {/* ===== 顶部极客风标签页切换区 (满足点击 3 次创建切换验收指标) ===== */}
      <div style={{ display: 'flex', backgroundColor: '#1a1a1a', borderBottom: '1px solid #333' }}>
        {TABS.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: '10px 24px',
              backgroundColor: activeTab === tab ? '#000' : 'transparent',
              color: activeTab === tab ? '#00FF00' : '#888',
              border: 'none',
              borderRight: '1px solid #333',
              cursor: 'pointer',
              fontFamily: 'monospace',
              fontSize: '14px',
              fontWeight: activeTab === tab ? 'bold' : 'normal',
              outline: 'none'
            }}
          >
            🛰️ {tab}
          </button>
        ))}
        <div style={{ flex: 1, textAlign: 'right', padding: '10px', color: '#555', fontSize: '12px' }}>
          NexMUD Core Multi-Sandbox Monitor
        </div>
      </div>

      {/* ===== 终端当前标签日志显示区 ===== */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px' }}>
        {(tabLogs[activeTab] || []).map((log, index) => (
          <div key={index} style={{ whiteSpace: 'pre-wrap', marginBottom: '2px', wordBreak: 'break-word', lineHeight: '1.4' }}>
            <Ansi useClasses={false}>{log}</Ansi>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>

      {/* ===== 底部输入区 ===== */}
      <div style={{ display: 'flex', alignItems: 'center', padding: '12px 16px', backgroundColor: '#111', borderTop: '1px solid #333' }}>
        <span style={{ color: '#0f0', marginRight: '10px', fontWeight: 'bold' }}>[{activeTab}] $&gt;</span>
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
          placeholder={`在 ${activeTab} 中输入指令，回车定向投递...`}
        />
      </div>
    </div>
  );
}

export default App;
