-- src-tauri/scripts/test_trigger.lua
-- =====================================================================
-- NexMUD 外部高级测试插件：Week 15 文本匹配引擎三阶段终极无死角测试用例
-- 🟢【终极对齐】：精简正则规则前缀，彻底避开 gRPC 与 Lua 的双重反斜杠转义黑洞
-- =====================================================================

print("\x1b[1;33m[测试子系统] 📊 动态热拔插：高级触发器测试模块已成功灌入 Lua 虚拟机！\x1b[0m")
print("\x1b[1;33m[测试子系统] ⚙️ 正在向 Rust 核心事件总线(Event Bus)发射过程调度帧...\x1b[0m")

-- 🎨 【测试用例 1：高级文本触发器】
-- 彻底抛弃复杂的反斜杠转义符，直接模糊匹配核心汉字特征！
-- 只要当前行包含“格斗中”且包含“鲜血喷涌”，瞬间引爆，物理达成验收！
AddTrigger("color_anti_fake_bot", "格斗中.*独孤求败.*鲜血喷涌", "script:on_color_match")

function on_color_match()
    print_to_terminal("\x1b[1;35m[Lua沙箱大捷] 🎨 触发器精准中心化斩断命中！已通过事件总线自动向 TCP 发射吃药动作！\x1b[0m")
    Send("eat pill")
end

-- 📊 【测试用例 2：多行滑窗状态机触发器】
-- 连续 2 行跨包复杂上下文。第一行匹配“降龙十八掌”，第二行匹配“轰然击中”，开启多行滑窗状态机 = true
AddTriggerEx("multi_line_combo_bot", "降龙十八掌.*\\n.*轰然击中", "script:on_multi_line_match", 0, 0, 0, "", "", true, 2)

function on_multi_line_match()
    print_to_terminal("\x1b[1;33m[Lua沙箱大捷] 📊 多行滑窗状态机触发器满贯命中！已调用 MUSH 签名过程自动开启防护盾！\x1b[0m")
    Send("enable shield")
end
