use chrono;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ZhiRequest {
    #[schemars(description = "要显示给用户的消息")]
    pub message: String,
    #[schemars(description = "预定义的选项列表（可选）")]
    #[serde(default)]
    pub predefined_options: Vec<String>,
    #[schemars(description = "消息是否为Markdown格式，默认为true")]
    #[serde(default = "default_is_markdown")]
    pub is_markdown: bool,
    #[schemars(description = "当前工作目录（强烈建议传递），用于会话识别和多Bot路由。格式：path:branch（例如：/Users/username/project:main）或 path（非Git仓库）。AI应该先获取当前Git分支（使用 git branch --show-current），然后构建 'path:branch' 格式的working_directory。如果不是Git仓库或无法获取分支，则只传递路径。这样可以确保同一目录的不同分支使用不同的Bot。")]
    #[serde(default)]
    pub working_directory: Option<String>,
}

fn default_is_markdown() -> bool {
    true
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JiyiRequest {
    #[schemars(description = "操作类型：记忆(添加记忆), 回忆(获取项目信息)")]
    pub action: String,
    #[schemars(description = "项目路径（必需）")]
    pub project_path: String,
    #[schemars(description = "记忆内容（记忆操作时必需）")]
    #[serde(default)]
    pub content: String,
    #[schemars(
        description = "记忆分类：rule(规范规则), preference(用户偏好), pattern(最佳实践), context(项目上下文)"
    )]
    #[serde(default = "default_category")]
    pub category: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AcemcpRequest {
    #[schemars(description = "项目根目录的绝对路径，使用正斜杠(/)作为分隔符")]
    pub project_root_path: String,
    #[schemars(description = "用于查找相关代码上下文的自然语言搜索查询")]
    pub query: String,
}

fn default_category() -> String {
    "context".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PopupRequest {
    pub id: String,
    pub message: String,
    pub predefined_options: Option<Vec<String>>,
    pub is_markdown: bool,
    #[serde(default)]
    pub bot_name: Option<String>, // 可选的 Telegram Bot 名称
    #[serde(default)]
    pub session_id: Option<String>, // 可选的会话 ID，用于自动选择 bot
}

/// 新的结构化响应数据格式
#[derive(Debug, Deserialize)]
pub struct McpResponse {
    pub user_input: Option<String>,
    pub selected_options: Vec<String>,
    pub images: Vec<ImageAttachment>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub data: String,
    pub media_type: String,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMetadata {
    pub timestamp: Option<String>,
    pub request_id: Option<String>,
    pub source: Option<String>,
}

/// 旧格式兼容性支持
#[derive(Debug, Deserialize)]
pub struct McpResponseContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub source: Option<ImageSource>,
}

#[derive(Debug, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// 统一的响应构建函数
///
/// 用于生成标准的JSON响应格式，确保无GUI和有GUI模式输出一致
pub fn build_mcp_response(
    user_input: Option<String>,
    selected_options: Vec<String>,
    images: Vec<ImageAttachment>,
    request_id: Option<String>,
    source: &str,
) -> serde_json::Value {
    serde_json::json!({
        "user_input": user_input,
        "selected_options": selected_options,
        "images": images,
        "metadata": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "request_id": request_id,
            "source": source
        }
    })
}

/// 构建发送操作的响应
pub fn build_send_response(
    user_input: Option<String>,
    selected_options: Vec<String>,
    images: Vec<ImageAttachment>,
    request_id: Option<String>,
    source: &str,
) -> String {
    let response = build_mcp_response(user_input, selected_options, images, request_id, source);
    response.to_string()
}

/// 构建继续操作的响应
pub fn build_continue_response(request_id: Option<String>, source: &str) -> String {
    // 动态获取继续提示词
    let continue_prompt = if let Ok(config) = crate::config::load_standalone_config() {
        config.reply_config.continue_prompt
    } else {
        "请按照最佳实践继续".to_string()
    };

    let response = build_mcp_response(Some(continue_prompt), vec![], vec![], request_id, source);
    response.to_string()
}
