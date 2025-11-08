use anyhow::Result;
use rmcp::{Error as McpError, model::*};

use crate::mcp::{ZhiRequest, PopupRequest};
use crate::mcp::handlers::{create_tauri_popup, parse_mcp_response};
use crate::mcp::utils::{generate_request_id, popup_error};

/// 智能代码审查交互工具
///
/// 支持预定义选项、自由文本输入和图片上传
#[derive(Clone)]
pub struct InteractionTool;

impl InteractionTool {
    pub async fn zhi(
        request: ZhiRequest,
    ) -> Result<CallToolResult, McpError> {
        use crate::log_important;

        // 调试：将请求参数写入临时文件
        let debug_file = std::env::temp_dir().join("cunzhi_zhi_request_debug.json");
        if let Ok(json) = serde_json::to_string_pretty(&request) {
            let _ = std::fs::write(&debug_file, json);
            log_important!(info, "🔍 请求参数已写入: {:?}", debug_file);
        }

        // 尝试获取会话 ID（工作目录）
        // 优先级：working_directory 参数 > CUNZHI_SESSION_ID > PWD > current_dir > 生成唯一ID
        let session_id = request.working_directory
            .clone()
            .or_else(|| std::env::var("CUNZHI_SESSION_ID").ok())
            .or_else(|| std::env::var("PWD").ok())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|path| path.to_str().map(|s| s.to_string()))
            })
            .or_else(|| {
                // 如果无法获取工作目录，生成一个基于时间戳的唯一会话ID
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                let random_suffix = std::process::id(); // 使用进程ID作为随机后缀
                Some(format!("session_{}_pid_{}", timestamp, random_suffix))
            })
            .map(|s| {
                // 标准化路径：移除末尾斜杠
                s.trim_end_matches('/').to_string()
            });

        // 调试信息
        log_important!(info, "🔍 working_directory 参数: {:?}", request.working_directory);
        log_important!(info, "🔍 CUNZHI_SESSION_ID 环境变量: {:?}", std::env::var("CUNZHI_SESSION_ID").ok());
        log_important!(info, "🔍 PWD 环境变量: {:?}", std::env::var("PWD").ok());
        log_important!(info, "🔍 current_dir(): {:?}", std::env::current_dir().ok());
        log_important!(info, "🔍 最终 session_id: {:?}", session_id);

        if let Some(ref sid) = session_id {
            if sid.starts_with("session_") {
                log_important!(info, "使用生成的会话ID: {}", sid);
            } else {
                log_important!(info, "使用工作目录作为会话ID: {}", sid);
            }
        } else {
            log_important!(warn, "⚠️ 没有会话ID");
        }

        let popup_request = PopupRequest {
            id: generate_request_id(),
            message: request.message,
            predefined_options: if request.predefined_options.is_empty() {
                None
            } else {
                Some(request.predefined_options)
            },
            bot_name: None, // 使用默认 bot 或根据 session_id 映射
            session_id: session_id.clone(),     // 传递会话 ID
            is_markdown: request.is_markdown,
        };

        log_important!(info, "📤 发送 PopupRequest，session_id: {:?}", popup_request.session_id);

        match create_tauri_popup(&popup_request) {
            Ok(response) => {
                // 解析响应内容，支持文本和图片
                let content = parse_mcp_response(&response)?;
                Ok(CallToolResult::success(content))
            }
            Err(e) => {
                Err(popup_error(e.to_string()).into())
            }
        }
    }
}
