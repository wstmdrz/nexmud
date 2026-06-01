-- src-tauri/scripts/test_watchdog.lua
print("\x1b[1;31m[测试子系统] 💥 已按需加载 W14 [看门狗超时强杀] 测试模块！\x1b[0m")
print("\x1b[1;31m[测试子系统] 🛑 请在控制台打入 'kill-sandbox' 激活不可退出死循环压测。\x1b[0m")

function handle_alias(input_text)
    if input_text == "kill-sandbox" or input_text == "while true do end" then
        print_to_terminal("\x1b[31m[测试子系统] ⚠️ 正在注入死循环代码，虚拟机即将卡死，等待 2 秒熔断制裁...\x1b[0m")
        while true do end
        return true
    end
    return false
end
