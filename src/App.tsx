import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { Terminal } from 'xterm';
import { WebglAddon } from 'xterm-addon-webgl';
import { FitAddon } from 'xterm-addon-fit';

// 必须手动引入 xterm 的基础样式（Tauri 会自动打包）
import 'xterm/css/xterm.css';

const TABS = ["World-A", "World-B", "World-C"];

function App() {
  const [activeTab, setActiveTab] = useState<string>("World-A");
  const [input, setInput] = useState('');

  // 🌟【核心黑科技】：为 3 个独立的隔离世界分别创建专属的 Xterm 终端实例与自适应插件
  const terminalRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const terminals = useRef<Record<string, Terminal>>({});
  const fitAddons = useRef<Record<string, FitAddon>>({});

  // 1. 初始化 3 个独立的终端画布
  useEffect(() => {
    TABS.forEach((tabId) => {
      // 创建终端配置（支持全色彩、平滑滚动、等宽字体）
      const term = new Terminal({
        allowProposedApi: true,
        scrollback: 5000, // 限制滚动历史缓冲区，防止无限白嫖内存
        fontSize: 15,
        fontFamily: 'Courier New, monospace',
        theme: {
          background: '#000000',
          foreground: '#ffffff',
          cursor: '#00ff00',
        },
        convertEol: true, // 自动把 \n 转换为 \r\n，防止错行
      });

      const fitAddon = new FitAddon();
      term.loadAddon(fitAddon);

      terminals.current[tabId] = term;
      fitAddons.current[tabId] = fitAddon;
    });

    // 组件卸载时释放内存
    return () => {
      TABS.forEach((tabId) => {
        terminals.current[tabId]?.dispose();
      });
    };
  }, []);

  // 2. 当切换标签页或者容器渲染就绪时，把 Terminal 挂载到真实的 DOM 节点上
  useEffect(() => {
    TABS.forEach((tabId) => {
      const container = terminalRefs.current[tabId];
      const term = terminals.current[tabId];
      const fitAddon = fitAddons.current[tabId];

      if (container && term && !term.element) {
        term.open(container);
        fitAddon.fit();

        // 🚀【WebGL 硬件加速】：尝试开启 WebGL 渲染，若不支持则优雅降级为 Canvas 渲染
        try {
          term.loadAddon(new WebglAddon());
          term.write("\x1b[32m[母舰] ⚡ WebGL 硬件加速引擎已成功挂载。\x1b[0m\r\n");
        } catch (e) {
          term.write("\x1b[33m[母舰] ⚠️ 硬件加速挂载失败，已平滑降级为标准 Canvas 渲染。\x1b[0m\r\n");
        }

        term.write(`\x1b[36m[系统] 欢迎来到 NexMUD 虚拟终端 ${tabId}...\x1b[0m\r\n`);
      }
    });
  }, [activeTab]);

  // 3. 多沙盒初始化唤醒
  useEffect(() => {
    const bootSequence = async () => {
      for (const tabId of TABS) {
        try {
          terminals.current[tabId]?.write(`[母舰] 正在请求孵化隔离域 ${tabId} 的守护进程...\r\n`);
          await invoke('init_connection', { tabId: tabId });
        } catch (err) {
          terminals.current[tabId]?.write(`\x1b[31m[系统] 神经连接创建失败: ${err}\x1b[0m\r\n`);
        }
      }
    };
    bootSequence();
  }, []);

  // 4. 🌟【精准对齐】：接收后端的 gRPC 数据包，直接用二进制高性能流写入对应的 Terminal
  useEffect(() => {
    interface ServerPayload {
      tabId: string;
      text: string;
    }

    const unlisten = listen<ServerPayload>('mud-stream-event', (event) => {
      const { tabId, text } = event.payload;
      const term = terminals.current[tabId];
      if (term) {
        // xterm.js 内部天然支持标准的 ANSI 颜色代码（如 \x1b[31m），直接高速写入，无缝渲染颜色
        term.write(text);
      }
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // 5. 玩家指令处理
  const handleSend = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim() !== '') {
      const cmd = input.trim();
      setInput('');

      // 在对应的 Xterm 终端本地回显，给予标志性的黄色
      terminals.current[activeTab]?.write(`\x1b[33m> ${cmd}\x1b[0m\r\n`);

      try {
        await invoke('send_command', { tabId: activeTab, input: cmd });
      } catch (err) {
        terminals.current[activeTab]?.write(`\x1b[31m[系统] 指令路由发送失败: ${err}\x1b[0m\r\n`);
      }
    }
  };

  // 窗口大小改变时，自适应调整 Canvas 画布分辨率
  useEffect(() => {
    const handleResize = () => {
      TABS.forEach((tabId) => {
        fitAddons.current[tabId]?.fit();
      });
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', backgroundColor: '#000', color: '#fff', fontFamily: 'monospace' }}>

      {/* ===== 顶部标签切换区 ===== */}
      <div style={{ display: 'flex', backgroundColor: '#1a1a1a', borderBottom: '1px solid #333' }}>
        {TABS.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: '12px 24px',
              backgroundColor: activeTab === tab ? '#000' : 'transparent',
              color: activeTab === tab ? '#00FF00' : '#888',
              border: 'none',
              borderRight: '1px solid #333',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: activeTab === tab ? 'bold' : 'normal',
              outline: 'none'
            }}
          >
            🛰️ {tab}
          </button>
        ))}
        <div style={{ flex: 1, textAlign: 'right', padding: '12px', color: '#555', fontSize: '12px' }}>
          NexMUD WebGL Terminal Engine Pro
        </div>
      </div>

      {/* ===== 终端画布区（利用 display 隐藏非活跃画布，保持后台 Xterm 实例持续吞吐数据） ===== */}
      <div style={{ flex: 1, position: 'relative', backgroundColor: '#000', padding: '8px' }}>
        {TABS.map((tabId) => (
          <div
            key={tabId}
            ref={(el) => { terminalRefs.current[tabId] = el; }}
            style={{
              display: activeTab === tabId ? 'block' : 'none',
              position: 'absolute',
              top: '8px',
              left: '8px',
              right: '8px',
              bottom: '8px'
            }}
          />
        ))}
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
            fontSize: '16px',
            fontFamily: 'Courier New, monospace'
          }}
          autoFocus
          placeholder={`在 ${activeTab} 中输入指令，回车定向投递...`}
        />
      </div>
    </div>
  );
}

export default App;
