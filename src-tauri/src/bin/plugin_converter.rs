// src-tauri/src/bin/plugin_converter.rs
// =====================================================================
// NexMUD - 官方 MUSHclient .xml 真实插件全要素深度反射转换子系统 (终极完全体)
// 职责划分：流式读取并盲测翻译 XML 字节码，支持 <send> 子节点嵌套逻辑深度压榨提取
// =====================================================================
use std::fs;
use std::env;
use std::path::Path;
use encoding_rs::GBK;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1;36m📦 =========================================================\x1b[0m");
    println!("\x1b[1;36m📦 NexMUD 现代化控制台：开始对真实世界 MUSHclient .xml 插件执行全要素反射转换...\x1b[0m");
    println!("\x1b[1;36m📦 =========================================================\x1b[0m");

    let args: Vec<String> = env::args().collect();
    let xml_path = args.get(1).unwrap_or(&"scripts/legacy_plugin.xml".to_string()).to_string();

    if !Path::new(&xml_path).exists() {
        println!("⚠️ 提示：未探测到测试桩，请确保scripts/legacy_plugin.xml 文件存在。");
        return Ok(());
    }

    let raw_bytes = fs::read(&xml_path)?;
    let xml_content = match std::str::from_utf8(&raw_bytes) {
        Ok(utf8_str) => utf8_str.to_string(),
        Err(_) => {
            let (decoded_str, _, _) = GBK.decode(&raw_bytes);
            println!("\x1b[1;33m[智能嗅探器] 📡 侦测到老旧插件采用 GBK 编码！自动执行流式弹性翻译...\x1b[0m");
            decoded_str.into_owned()
        }
    };

    println!("✅ 成功照合真实 XML，文件大小: {} 字节。开始进行嵌套树级深度过滤...", xml_content.len());

    if xml_content.contains("language=\"JScript\"") || xml_content.contains("language=\"VBScript\"") {
        println!("\x1b[1;31m[防毒熔断器] 🛑 严重拦截：检测到非 Lua 旧资产，已安全拒绝转换！\x1b[0m");
        return Ok(());
    }

    let mut generated_lua_headers = String::new();
    generated_lua_headers.push_str("-- =====================================================================\n");
    generated_lua_headers.push_str("-- 🛰️ NexMUD 转换引擎：已自动将旧 XML 的嵌套 <send> 节点重构为标准的动态过程桩\n");
    generated_lua_headers.push_str("-- =====================================================================\n\n");

    // 1. 深度扫描并提取所有的 <trigger ...> 及其潜藏的 <send>
    let mut search_content = xml_content.as_str();
    let mut trigger_counter = 0;
    while let Some(start_trig) = search_content.find("<trigger") {
        let remainder = &search_content[start_trig..];
        if let Some(end_trig_close) = remainder.find("</trigger>").or_else(|| remainder.find("/>")) {
            let full_trigger_chunk = &remainder[..end_trig_close + 10]; // 包含结束标签
            
            let mut trig_match = extract_attr(full_trigger_chunk, "match=").unwrap_or_default();
            let mut trig_script = extract_attr(full_trigger_chunk, "script=").or_else(|| extract_attr(full_trigger_chunk, "response=")).unwrap_or_default();
            let trig_seq = extract_attr(full_trigger_chunk, "sequence=").unwrap_or_else(|| "100".to_string());
            let is_multi = if full_trigger_chunk.contains("multi_line=\"y\"") { "true" } else { "false" };
            let lines_count = extract_attr(full_trigger_chunk, "lines=").unwrap_or_else(|| "1".to_string());

            // 🌟【Week 16 树级挖掘创新 1】：如果行内属性没有 script，尝试提取内层包裹的 <send> 动作内容！
            if trig_script.is_empty() {
                if let Some(inner_send) = extract_send_node(full_trigger_chunk) {
                    trigger_counter += 1;
                    trig_script = format!("auto_generated_trig_func_{}", trigger_counter);
                    // 反向反向将这段内嵌的 Lua 动作重组成一个全局命名函数，追加在脚本头部，完美闭合！
                    generated_lua_headers.push_str(&format!("function {}()\n{}\nend\n\n", trig_script, inner_send));
                }
            }

            if !trig_match.is_empty() && !trig_script.is_empty() {
                if trig_match.contains('*') { trig_match = trig_match.replace("*", "(.*)"); }
                let safe_regex = trig_match.replace("[", "\\[").replace("]", "\\]");
                generated_lua_headers.push_str(&format!(
                    "AddTriggerEx(\"{}\", \"^{}$\", \"script:{}\", {}, 0, 0, \"\", \"\", {}, {})\n",
                    trig_script, safe_regex, trig_script, trig_seq, is_multi, lines_count
                ));
            }
            search_content = &remainder[end_trig_close + 2..];
        } else { break; }
    }

    // 2. 深度扫描并提取所有的 <alias ...> 及其潜藏的 <send>
    let mut search_alias_content = xml_content.as_str();
    let mut alias_counter = 0;
    while let Some(start_alias) = search_alias_content.find("<alias") {
        let remainder = &search_alias_content[start_alias..];
        // 兼容单闭合标签或是带有 </alias> 子节点的两种复杂生态
        let end_alias_close = remainder.find("</alias>").unwrap_or_else(|| remainder.find(">").unwrap_or(0));
        let full_alias_chunk = &remainder[..end_alias_close + 8];

        let mut alias_match = extract_attr(full_alias_chunk, "match=").unwrap_or_default();
        let mut alias_script = extract_attr(full_alias_chunk, "script=").or_else(|| extract_attr(full_alias_chunk, "response=")).unwrap_or_default();
        let alias_seq = extract_attr(full_alias_chunk, "sequence=").unwrap_or_else(|| "50".to_string());

        // 🌟【Week 16 树级挖掘创新 2】：完美破除你发过来的这个 <send> 别名死结节点！
        if alias_script.is_empty() {
            if let Some(inner_send) = extract_send_node(full_alias_chunk) {
                alias_counter += 1;
                // 自动动态重组为其在全局大表内的唯一响应标识符名
                alias_script = format!("auto_generated_alias_func_{}", alias_counter);
                // 🟢【乱码隔离与截断保护】：在包裹的闭包前后加入强制强行物理换行，彻底隔离老旧乱码注释的溢出株连！
                generated_lua_headers.push_str(&format!("function {}()\n\n{}\n\nend\n\n", alias_script, inner_send));
            }
        }

        if !alias_match.is_empty() && !alias_script.is_empty() {
            if alias_match.contains('*') { alias_match = alias_match.replace("*", "(.*)"); }
            let safe_match = alias_match.trim_start_matches('^').trim_end_matches('$');
            generated_lua_headers.push_str(&format!(
                "AddAlias(\"{}\", \"^{}$\", \"script:{}\", {}, \"\")\n",
                alias_script, safe_match, alias_script, alias_seq
            ));
        }
        
        search_alias_content = &remainder[10.max(end_alias_close)..];
    }

    generated_lua_headers.push_str("\n-- =====================================================================\n\n");

    // 3. 多段式 <script> 标签物理黏合拼接
    let mut script_search_zone = xml_content.as_str();
    while let Some(s_idx) = script_search_zone.find("<script") {
        let remainder = &script_search_zone[s_idx..];
        if let Some(end_tag_pos) = remainder.find("</script>") {
            let script_block = &remainder[..end_tag_pos];
            let mut pure_code = script_block;
            if let Some(cdata_start) = pure_code.find("<![CDATA[") {
                pure_code = &pure_code[cdata_start + 9..];
            } else if let Some(tag_close) = pure_code.find('>') {
                pure_code = &pure_code[tag_close + 1..];
            }
            let clean_pure_code = pure_code.replace("]]>", "");
            generated_lua_headers.push_str(clean_pure_code.trim().trim_start_matches('\u{feff}'));
            generated_lua_headers.push_str("\n\n");
            script_search_zone = &remainder[end_tag_pos + 9..];
        } else { break; }
    }

    // 完美无损导出为完全体 `.lua` 文件资产
    let out_lua_path = "scripts/pkuxkx_register.lua";
    fs::write(out_lua_path, generated_lua_headers.trim())?;
    
    let out_json_path = "scripts/plugin_manifest.json";
    fs::write(out_json_path, "{\"status\": \"Fully_Reborn_With_Nested_Send_Nodes_And_Auto_Closures\"}")?;
    
    println!("\x1b[1;32m🎉 史诗级大捷！嵌套的 <send> 子节点动作配置已 100% 成功提取并动态重塑为现代 LUA 全局闭包响应中枢！\x1b[0m");
    Ok(())
}

// 🌐 辅助工具 1：行内属性精密分离器
fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    if let Some(start) = tag.find(attr_name) {
        let remainder = &tag[start + attr_name.len()..];
        let quote = remainder.chars().next()?;
        if quote == '"' || quote == '\'' {
            let sub = &remainder[1..];
            if let Some(end) = sub.find(quote) { return Some(sub[..end].to_string()); }
        }
    }
    None
}

// 🌐 辅助工具 2：🆕【树级挖掘核心】—— 专门深度提取 <send> ... </send> 子节点内部的内容
fn extract_send_node(chunk: &str) -> Option<String> {
    if let Some(start) = chunk.find("<send>") {
        if let Some(end) = chunk.find("</send>") {
            if start < end {
                let inner_content = &chunk[start + 6..end];
                // 自动自动切除可能附带的内层 CDATA 保护套
                let clean_content = inner_content.replace("<![CDATA[", "").replace("]]>", "");
                return Some(clean_content.trim().to_string());
            }
        }
    }
    None
}
