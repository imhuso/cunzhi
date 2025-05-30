use anyhow::Result;
use rmcp::{
    Error as McpError, ServerHandler, ServiceExt,
    model::*,
    tool, transport::stdio,
};

use std::process::Command;
use std::fs;
use uuid::Uuid;

mod memory;
use memory::{MemoryManager, MemoryCategory};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AIReviewChatRequest {
    #[schemars(description = "要显示给用户的消息")]
    pub message: String,
    #[schemars(description = "预定义的选项列表（可选）")]
    #[serde(default)]
    pub predefined_options: Vec<String>,
    #[schemars(description = "消息是否为Markdown格式")]
    #[serde(default)]
    pub is_markdown: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MemoryManagerRequest {
    #[schemars(description = "操作类型：add(添加记忆), get_project_info(获取项目信息)")]
    pub action: String,
    #[schemars(description = "项目路径（必需）")]
    pub project_path: String,
    #[schemars(description = "记忆内容（add操作时必需）")]
    #[serde(default)]
    pub content: String,
    #[schemars(description = "记忆分类：rule(规范规则), preference(用户偏好), pattern(最佳实践), context(项目上下文)")]
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_category() -> String {
    "context".to_string()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PopupRequest {
    id: String,
    message: String,
    predefined_options: Option<Vec<String>>,
    is_markdown: bool,
}

#[derive(Clone)]
pub struct AIReviewServer {
    // 可以添加状态字段
}

#[tool(tool_box)]
impl AIReviewServer {
    pub fn new() -> Self {
        Self {}
    }

    #[tool(description = "AI Review 智能代码审查交互工具，支持预定义选项和自由文本输入")]
    async fn ai_review_chat(
        &self,
        #[tool(aggr)] request: AIReviewChatRequest,
    ) -> Result<CallToolResult, McpError> {
        let popup_request = PopupRequest {
            id: Uuid::new_v4().to_string(),
            message: request.message,
            predefined_options: if request.predefined_options.is_empty() { 
                None 
            } else { 
                Some(request.predefined_options) 
            },
            is_markdown: request.is_markdown,
        };

        match create_tauri_popup(&popup_request) {
            Ok(response) => {
                Ok(CallToolResult::success(vec![Content::text(response)]))
            }
            Err(e) => {
                Err(McpError::internal_error(format!("弹窗创建失败: {}", e), None))
            }
        }
    }

    #[tool(description = "全局记忆管理工具，用于存储和管理重要的开发规范、用户偏好和最佳实践")]
    async fn memory_manager(
        &self,
        #[tool(aggr)] request: MemoryManagerRequest,
    ) -> Result<CallToolResult, McpError> {
        // 检查项目路径是否存在
        if !std::path::Path::new(&request.project_path).exists() {
            return Err(McpError::invalid_params(
                format!("项目路径不存在: {}", request.project_path),
                None
            ));
        }

        let manager = MemoryManager::new(&request.project_path)
            .map_err(|e| McpError::internal_error(format!("创建记忆管理器失败: {}", e), None))?;

        let result = match request.action.as_str() {
            "add" => {
                if request.content.trim().is_empty() {
                    return Err(McpError::invalid_params("缺少记忆内容".to_string(), None));
                }
                
                let category = match request.category.as_str() {
                    "rule" => MemoryCategory::Rule,
                    "preference" => MemoryCategory::Preference,
                    "pattern" => MemoryCategory::Pattern,
                    "context" => MemoryCategory::Context,
                    _ => MemoryCategory::Context,
                };

                let id = manager.add_memory(&request.content, category)
                    .map_err(|e| McpError::internal_error(format!("添加记忆失败: {}", e), None))?;
                
                format!("✅ 记忆已添加，ID: {}\n📝 内容: {}\n📂 分类: {:?}", id, request.content, category)
            }
            "get_project_info" => {
                manager.get_project_info()
                    .map_err(|e| McpError::internal_error(format!("获取项目信息失败: {}", e), None))?
            }
            _ => {
                return Err(McpError::invalid_params(
                    format!("未知的操作类型: {}", request.action),
                    None
                ));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[tool(tool_box)]
impl ServerHandler for AIReviewServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "ai-review-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("AI Review 智能代码审查工具，支持交互式对话和记忆管理".to_string()),
        }
    }
}

fn create_tauri_popup(request: &PopupRequest) -> Result<String> {
    // 创建临时请求文件
    let temp_file = format!("/tmp/mcp_request_{}.json", request.id);
    let request_json = serde_json::to_string_pretty(request)?;
    fs::write(&temp_file, request_json)?;

    // 调用全局安装的ai-review-ui命令
    let output = Command::new("ai-review-ui")
        .arg("--mcp-request")
        .arg(&temp_file)
        .output()?;

    // 清理临时文件
    let _ = fs::remove_file(&temp_file);

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);
        let response = response.trim();
        if response.is_empty() {
            Ok("用户取消了操作".to_string())
        } else {
            Ok(response.to_string())
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("UI进程失败: {}", error);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建并运行服务器
    let service = AIReviewServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            eprintln!("启动服务器失败: {}", e);
        })?;

    // 等待服务器关闭
    service.waiting().await?;
    Ok(())
}
