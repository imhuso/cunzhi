use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::process::Command;
use std::fs;
use uuid::Uuid;

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
        if response == "CANCELLED" || response.is_empty() {
            return Err(anyhow::anyhow!("用户取消了操作"));
        }
        Ok(response)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Tauri弹窗失败: {}", error))
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
                }
            ]
        })),
        error: None,
    }
}

fn handle_tools_call(id: Value, params: Option<Value>) -> JsonRpcResponse {
    if let Some(Value::Object(map)) = params {
        if let (Some(Value::String(name)), Some(arguments)) =
            (map.get("name"), map.get("arguments")) {

            if name == "ai_review_chat" {
                if let Value::Object(args) = arguments {
                    let message = args.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No message provided");

                    let predefined_options = args.get("predefined_options")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect::<Vec<String>>()
                        });

                    let is_markdown = args.get("is_markdown")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let popup_request = PopupRequest {
                        id: Uuid::new_v4().to_string(),
                        message: message.to_string(),
                        predefined_options,
                        is_markdown,
                    };

                    match create_tauri_popup(&popup_request) {
                        Ok(response) => {
                            return JsonRpcResponse {
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
                            };
                        }
                        Err(e) => {
                            eprintln!("弹窗创建失败: {}", e);
                        }
                    }
                }
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
    eprintln!("🚀 AI Review MCP 服务器已启动，等待连接...");
    eprintln!("📋 支持的工具: ai_review_chat");
    eprintln!("🎯 弹窗方式: 直接调用 Tauri 应用");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match parse_flexible_request(&line) {
            Ok(request) => {
                eprintln!("📨 收到请求: {}", request.method);
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("❌ 解析请求失败: {}", e);
                eprintln!("📄 原始请求: {}", line);

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
                        "message": "Parse error",
                        "data": format!("Failed to parse JSON-RPC request: {}", e)
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
