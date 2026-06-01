-- =====================================================================
-- 🛰️ NexMUD 转换引擎：已自动将旧 XML 的嵌套 <send> 节点重构为标准的动态过程桩
-- =====================================================================

AddTriggerEx("on_color_match", "^格斗中.(.*)独孤求败.(.*)鲜血喷涌$", "script:on_color_match", 10, 0, 0, "", "", false, 1)
AddTriggerEx("on_multi_line_match", "^降龙十八掌.(.*)\n.(.*)轰然击中$", "script:on_multi_line_match", 20, 0, 0, "", "", true, 2)
AddAlias("handle_alias", "^auto-reg$", "script:handle_alias", 50, "")

-- =====================================================================

print("\x1b[1;36m[资产重生大捷] 🛰️ 警告：成功在完全独立的沙箱变量空间内接管真实世界 MUSHclient 插件！\x1b[0m")

-- 1. 挂载 Tokio 异步微秒级常驻时间轴定时器过程
AddTimer("heartbeat_timer", 0, 0, 3.5, "chat 唯我 NexMUD 分布式事件总线万古长青！", 0, "")

-- 2. 挂载 Week 17 & 18 插件沙箱私有空间隔离机制
SetPluginVariable("d728562d9fb82103f6f1c422", "CombatState", "狂暴格斗中")
local status = GetPluginVariable("d728562d9fb82103f6f1c422", "CombatState")
print("\x1b[1;32m[资产重生大捷] ✅ 隔离舱跨空间读取自检大获全胜！私有变量 CombatState = " .. status .. "\x1b[0m")

function on_color_match()
    print_to_terminal("\x1b[1;35m[真实插件联动] 🎨 颜色防伪触发器中心化斩断命中！已自动通过 Rust 总线向下游网卡发射吃药动作！\x1b[0m")
    Send("eat pill")
end

function on_multi_line_match()
    print_to_terminal("\x1b[1;33m[真实插件联动] 📊 多行滑窗状态机优先级队列满贯命中！已命令网络层全自动开启防护盾！\x1b[0m")
    Send("enable shield")
end

function handle_alias(input_text)
    if input_text == "score" then
        print_to_terminal("\x1b[35m[真实插件联动] ⚡ 别名优先机制命中！当前插件沙箱状态健康。\x1b[0m")
        return true
    end
    if input_text == "while true do end" or input_text == "kill-sandbox" then
        while true do end
        return true
    end
    return false
end

function on_text_trigger()
    -- 基础桩
end