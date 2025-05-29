use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::process::Command;
use std::fs;
use uuid::Uuid;

mod memory;
use memory::{MemoryManager, MemoryCategory};

#[derive(Debug, Serialize, Deserialize)]
struct PopupRequest {
    id: String,
    message: String,
    predefined_options: Option<Vec<String>>,
    is_markdown: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    #[serde(default = "default_jsonrpc_version")]
    jsonrpc: String,
    #[serde(default = "default_null_id")]
    id: Value,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

fn default_jsonrpc_version() -> String {
    "2.0".to_string()
}

fn default_null_id() -> Value {
    Value::Null
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
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
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if response == "CANCELLED" {
            return Err(anyhow::anyhow!("用户取消了操作"));
        }
        if response.is_empty() {
            return Err(anyhow::anyhow!("与ai-review-ui连接失败,请重试!"));
        }
        Ok(response)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        // 检查是否是连接相关的错误，提示重试
        if error.contains("连接失败") || error.contains("通道异常") || error.contains("需要重试") {
            Err(anyhow::anyhow!("弹窗连接异常，建议重试: {}", error))
        } else {
            Err(anyhow::anyhow!("弹窗创建失败: {}", error))
        }
    }
}

fn handle_initialize(id: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "ai-review-mcp",
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "tools": [
                {
                    "name": "ai_review_chat",
                    "description": "AI Review 智能代码审查交互工具，支持预定义选项和自由文本输入",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "要显示给用户的消息"
                            },
                            "predefined_options": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "预定义的选项列表（可选）"
                            },
                            "is_markdown": {
                                "type": "boolean",
                                "description": "消息是否为Markdown格式",
                                "default": false
                            }
                        },
                        "required": ["message"]
                    }
                },
                {
                    "name": "memory_manager",
                    "description": "全局记忆管理工具，用于存储和管理重要的开发规范、用户偏好和最佳实践",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": ["add", "get_project_info"],
                                "description": "操作类型：add(添加记忆), get_project_info(获取项目信息)"
                            },
                            "content": {
                                "type": "string",
                                "description": "记忆内容（add操作时必需）"
                            },
                            "category": {
                                "type": "string",
                                "enum": ["rule", "preference", "pattern", "context"],
                                "description": "记忆分类：rule(规范规则), preference(用户偏好), pattern(最佳实践), context(项目上下文)"
                            },

                            "project_path": {
                                "type": "string",
                                "description": "项目路径（必需）"
                            }
                        },
                        "required": ["action", "project_path"]
                    }
                }
            ]
        })),
        error: None,
    }
}

fn handle_ai_review_chat(id: Value, arguments: &Value) -> JsonRpcResponse {
    // 详细的参数验证和错误处理
    match arguments {
        Value::Object(args) => {
            // 验证必需的message参数
            let message = match args.get("message") {
                Some(Value::String(msg)) => {
                    if msg.trim().is_empty() {
                        eprintln!("错误: message参数不能为空字符串");
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: None,
                            error: Some(json!({
                                "code": -32602,
                                "message": "Invalid ai_review_chat params: message不能为空"
                            })),
                        };
                    }
                    msg.clone()
                }
                Some(_) => {
                    eprintln!("错误: message参数必须是字符串类型");
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32602,
                            "message": "Invalid ai_review_chat params: message必须是字符串类型"
                        })),
                    };
                }
                None => {
                    eprintln!("错误: 缺少必需的message参数");
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32602,
                            "message": "Invalid ai_review_chat params: 缺少必需的message参数"
                        })),
                    };
                }
            };

            // 验证可选的predefined_options参数
            let predefined_options = match args.get("predefined_options") {
                Some(Value::Array(arr)) => {
                    let options: Result<Vec<String>, String> = arr.iter()
                        .enumerate()
                        .map(|(i, v)| {
                            match v.as_str() {
                                Some(s) => Ok(s.to_string()),
                                None => Err(format!("predefined_options[{}]必须是字符串类型", i))
                            }
                        })
                        .collect();

                    match options {
                        Ok(opts) => Some(opts),
                        Err(err) => {
                            eprintln!("错误: {}", err);
                            return JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(json!({
                                    "code": -32602,
                                    "message": format!("Invalid ai_review_chat params: {}", err)
                                })),
                            };
                        }
                    }
                }
                Some(_) => {
                    eprintln!("错误: predefined_options参数必须是数组类型");
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32602,
                            "message": "Invalid ai_review_chat params: predefined_options必须是数组类型"
                        })),
                    };
                }
                None => None,
            };

            // 验证可选的is_markdown参数
            let is_markdown = match args.get("is_markdown") {
                Some(Value::Bool(b)) => *b,
                Some(_) => {
                    eprintln!("错误: is_markdown参数必须是布尔类型");
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32602,
                            "message": "Invalid ai_review_chat params: is_markdown必须是布尔类型"
                        })),
                    };
                }
                None => false,
            };

            let popup_request = PopupRequest {
                id: Uuid::new_v4().to_string(),
                message,
                predefined_options,
                is_markdown,
            };

            match create_tauri_popup(&popup_request) {
                Ok(response) => {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": response
                                }
                            ]
                        })),
                        error: None,
                    }
                }
                Err(e) => {
                    eprintln!("弹窗创建失败: {}", e);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32603,
                            "message": format!("弹窗创建失败: {}", e)
                        })),
                    }
                }
            }
        }
        _ => {
            eprintln!("错误: arguments参数必须是对象类型");
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32602,
                    "message": "Invalid ai_review_chat params: arguments必须是对象类型"
                })),
            }
        }
    }
}

fn handle_memory_add(manager: &MemoryManager, args: &serde_json::Map<String, Value>) -> Result<String> {
    let content = args.get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("缺少记忆内容"))?;

    let category_str = args.get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("context");

    let category = match category_str {
        "rule" => MemoryCategory::Rule,
        "preference" => MemoryCategory::Preference,
        "pattern" => MemoryCategory::Pattern,
        "context" => MemoryCategory::Context,
        _ => MemoryCategory::Context,
    };

    let id = manager.add_memory(content, category)?;
    Ok(format!("✅ 记忆已添加，ID: {}\n📝 内容: {}\n📂 分类: {:?}",
               id, content, category))
}









fn handle_memory_get_project_info(manager: &MemoryManager) -> Result<String> {
    manager.get_project_info()
}







fn handle_memory_manager(id: Value, arguments: &Value) -> JsonRpcResponse {
    if let Value::Object(args) = arguments {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 要求调用方明确提供项目路径，不进行自动fallback
        let project_path = match args.get("project_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(json!({
                        "code": -32602,
                        "message": "缺少必需的 project_path 参数。请在调用 memory_manager 工具时明确指定项目路径，例如：{\"action\": \"add\", \"project_path\": \"/path/to/your/project\", \"content\": \"...\", \"category\": \"preference\"}"
                    })),
                };
            }
        };

        // 检查项目路径是否存在
        if !std::path::Path::new(&project_path).exists() {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32602,
                    "message": format!("项目路径不存在: {}", project_path)
                })),
            };
        }

        match MemoryManager::new(&project_path) {
            Ok(manager) => {
                let result = match action {
                    "add" => handle_memory_add(&manager, args),
                    "get_project_info" => handle_memory_get_project_info(&manager),
                    _ => Err(anyhow::anyhow!("未知的操作类型: {}", action)),
                };

                match result {
                    Ok(content) => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: Some(json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": content
                                    }
                                ]
                            })),
                            error: None,
                        };
                    }
                    Err(e) => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: None,
                            error: Some(json!({
                                "code": -32603,
                                "message": format!("记忆管理操作失败: {}", e)
                            })),
                        };
                    }
                }
            }
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(json!({
                        "code": -32603,
                        "message": format!("MCP error -32603: 创建记忆管理器失败，项目路径: {}, 错误: {}", project_path, e)
                    })),
                };
            }
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(json!({
            "code": -32602,
            "message": "Invalid memory_manager params"
        })),
    }
}

fn handle_tools_call(id: Value, params: Option<Value>) -> JsonRpcResponse {
    if let Some(Value::Object(map)) = params {
        if let (Some(Value::String(name)), Some(arguments)) =
            (map.get("name"), map.get("arguments")) {

            if name == "ai_review_chat" {
                return handle_ai_review_chat(id, arguments);
            } else if name == "memory_manager" {
                return handle_memory_manager(id, arguments);
            }
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(json!({
            "code": -32602,
            "message": "Invalid params"
        })),
    }
}

fn parse_flexible_request(line: &str) -> Result<JsonRpcRequest, serde_json::Error> {
    // 首先尝试标准解析
    if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(line) {
        return Ok(request);
    }

    // 如果失败，尝试解析为通用JSON并手动构建请求
    let value: Value = serde_json::from_str(line)?;
    if let Value::Object(map) = value {
        let method = map.get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing method field"
            )))?
            .to_string();

        let id = map.get("id").cloned().unwrap_or(Value::Null);
        let jsonrpc = map.get("jsonrpc")
            .and_then(|v| v.as_str())
            .unwrap_or("2.0")
            .to_string();
        let params = map.get("params").cloned();

        Ok(JsonRpcRequest {
            jsonrpc,
            id,
            method,
            params,
        })
    } else {
        Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Request must be a JSON object"
        )))
    }
}

fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "tools/list" => handle_tools_list(request.id),
        "tools/call" => handle_tools_call(request.id, request.params),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": "Method not found"
            })),
        }
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match parse_flexible_request(&line) {
            Ok(request) => {
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(_e) => {

                // 尝试从原始JSON中提取id
                let request_id = match serde_json::from_str::<Value>(&line) {
                    Ok(Value::Object(map)) => {
                        map.get("id").cloned().unwrap_or(Value::Null)
                    }
                    _ => Value::Null,
                };

                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request_id,
                    result: None,
                    error: Some(json!({
                        "code": -32700,
                        "message": "Parse error"
                    })),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
