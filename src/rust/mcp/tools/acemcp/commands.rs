use tauri::{AppHandle, State};

use crate::config::{AppState, save_config};
use super::{AcemcpTool};
use super::types::AcemcpRequest;
use reqwest;

#[derive(Debug, serde::Deserialize)]
pub struct SaveAcemcpConfigArgs {
    #[serde(alias = "baseUrl", alias = "base_url")]
    pub base_url: String,
    #[serde(alias = "token", alias = "_token")]
    pub token: String,
    #[serde(alias = "batchSize", alias = "batch_size")]
    pub batch_size: u32,
    #[serde(alias = "maxLinesPerBlob", alias = "_max_lines_per_blob")]
    pub max_lines_per_blob: u32,
    #[serde(alias = "textExtensions", alias = "_text_extensions")]
    pub text_extensions: Vec<String>,
    #[serde(alias = "excludePatterns", alias = "_exclude_patterns")]
    pub exclude_patterns: Vec<String>,
}

#[tauri::command]
pub async fn save_acemcp_config(
    args: SaveAcemcpConfigArgs,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut base_url = args.base_url.trim().to_string();
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        base_url = format!("http://{}", base_url);
        log::warn!("BASE_URL 缺少协议，已自动补全为: {}", base_url);
    }
    {
        let mut config = state
            .config
            .lock()
            .map_err(|e| format!("获取配置失败: {}", e))?;

        config.mcp_config.acemcp_base_url = Some(base_url.clone());
        config.mcp_config.acemcp_token = Some(args.token.clone());
        config.mcp_config.acemcp_batch_size = Some(args.batch_size);
        config.mcp_config.acemcp_max_lines_per_blob = Some(args.max_lines_per_blob);
        config.mcp_config.acemcp_text_extensions = Some(args.text_extensions.clone());
        config.mcp_config.acemcp_exclude_patterns = Some(args.exclude_patterns.clone());
    }

    save_config(&state, &app)
        .await
        .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
pub struct TestAcemcpArgs {
    #[serde(alias = "baseUrl", alias = "base_url")]
    pub base_url: String,
    #[serde(alias = "token", alias = "_token")]
    pub token: String,
}

#[derive(Debug, serde::Serialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn test_acemcp_connection(
    args: TestAcemcpArgs,
    state: State<'_, AppState>,
) -> Result<TestConnectionResult, String> {
    // 获取配置并立即释放锁
    let (effective_base_url, effective_token) = {
        let config = state.config
            .lock()
            .map_err(|e| format!("获取配置失败: {}", e))?;
        
        let base_url = config.mcp_config.acemcp_base_url.as_ref().unwrap_or(&args.base_url).clone();
        let token = config.mcp_config.acemcp_token.as_ref().unwrap_or(&args.token).clone();
        (base_url, token)
    };
    
    // 验证 URL 格式
    if !effective_base_url.starts_with("http://") && !effective_base_url.starts_with("https://") {
        let msg = "无效的API端点URL格式，必须以 http:// 或 https:// 开头".to_string();
        return Ok(TestConnectionResult {
            success: false,
            message: msg,
        });
    }
    
    // 验证 token
    if effective_token.trim().is_empty() {
        let msg = "认证令牌不能为空".to_string();
        return Ok(TestConnectionResult {
            success: false,
            message: msg,
        });
    }
    
    // 规范化 base_url
    let normalized_url = if effective_base_url.ends_with('/') {
        effective_base_url[..effective_base_url.len() - 1].to_string()
    } else {
        effective_base_url.clone()
    };
    
    // 实际测试连接 - 发送一个简单的健康检查请求
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 尝试访问一个常见的端点（如果存在健康检查端点）
    let test_url = format!("{}/health", normalized_url);
    
    match client
        .get(&test_url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", effective_token))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            
            if status.is_success() {
                let msg = format!("连接测试成功！API 端点响应正常 (HTTP {})", status.as_u16());
                return Ok(TestConnectionResult {
                    success: true,
                    message: msg,
                });
            }
        }
        Err(_) => {
            // 健康检查端点可能不存在，继续测试实际 API 端点
        }
    }
    
    // 如果健康检查失败，尝试测试实际的代码库检索端点
    let search_url = format!("{}/agents/codebase-retrieval", normalized_url);
    
    // 发送一个最小的测试请求
    let test_payload = serde_json::json!({
        "information_request": "test",
        "blobs": {"checkpoint_id": null, "added_blobs": [], "deleted_blobs": []},
        "dialog": [],
        "max_output_length": 0,
        "disable_codebase_retrieval": false,
        "enable_commit_retrieval": false,
    });
    
    match client
        .post(&search_url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", effective_token))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&test_payload)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            
            if status.is_success() {
                let msg = format!("连接测试成功！API 端点响应正常 (HTTP {})", status.as_u16());
                Ok(TestConnectionResult {
                    success: true,
                    message: msg,
                })
            } else {
                let body = response.text().await.unwrap_or_default();
                let msg = format!("API 端点返回错误状态: {} {}", status.as_u16(), status.as_str());
                Ok(TestConnectionResult {
                    success: false,
                    message: format!("{} - 响应: {}", msg, if body.len() > 200 { format!("{}...", &body[..200]) } else { body }),
                })
            }
        }
        Err(e) => {
            let msg = format!("连接失败: {}", e);
            Ok(TestConnectionResult {
                success: false,
                message: msg,
            })
        }
    }
}

/// 读取日志文件内容
#[tauri::command]
pub async fn read_acemcp_logs(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let home = std::env::var("HOME").map_err(|_| "无法获取HOME目录".to_string())?;
    let log_path = format!("{}/.cunzhi/log/acemcp.log", home);
    
    // 确保日志目录存在
    let log_dir = std::path::Path::new(&log_path).parent().unwrap();
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)
            .map_err(|e| format!("创建日志目录失败: {}", e))?;
    }
    
    if !std::path::Path::new(&log_path).exists() {
        return Ok(vec![]);
    }
    
    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("读取日志文件失败: {}", e))?;
    
    // 返回最近1000行日志
    let all_lines: Vec<String> = content
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    // 只返回最后1000行
    let lines: Vec<String> = if all_lines.len() > 1000 {
        let skip_count = all_lines.len() - 1000;
        all_lines.into_iter().skip(skip_count).collect()
    } else {
        all_lines
    };
    
    Ok(lines)
}

#[tauri::command]
pub async fn clear_acemcp_cache(_state: State<'_, AppState>) -> Result<String, String> {
    let cache_dir = std::env::var("HOME").unwrap_or_default() + "/.acemcp/data";
    if std::path::Path::new(&cache_dir).exists() {
        std::fs::remove_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    log::info!("acemcp缓存已清除: {}", cache_dir);
    Ok(cache_dir)
}

#[derive(Debug, serde::Serialize)]
pub struct AcemcpConfigResponse {
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub batch_size: u32,
    pub max_lines_per_blob: u32,
    pub text_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[tauri::command]
pub async fn get_acemcp_config(state: State<'_, AppState>) -> Result<AcemcpConfigResponse, String> {
    let config = state.config
        .lock()
        .map_err(|e| format!("获取配置失败: {}", e))?;
    Ok(AcemcpConfigResponse {
        base_url: config.mcp_config.acemcp_base_url.clone(),
        token: config.mcp_config.acemcp_token.clone(),
        batch_size: config.mcp_config.acemcp_batch_size.unwrap_or(10),
        max_lines_per_blob: config.mcp_config.acemcp_max_lines_per_blob.unwrap_or(800),
        text_extensions: config.mcp_config.acemcp_text_extensions.clone().unwrap_or_else(|| {
            vec![".rs".to_string(), ".ts".to_string(), ".js".to_string(), ".md".to_string(), ".toml".to_string()]
        }),
        exclude_patterns: config.mcp_config.acemcp_exclude_patterns.clone().unwrap_or_else(|| {
            vec!["node_modules".to_string(), ".git".to_string(), "target".to_string(), "dist".to_string()]
        }),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct DebugSearchResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// 纯 Rust 的调试命令：直接执行 acemcp 搜索，返回结果
#[tauri::command]
pub async fn debug_acemcp_search(
    project_root_path: String,
    query: String,
    _app: AppHandle,
) -> Result<DebugSearchResult, String> {
    let req = AcemcpRequest { project_root_path, query };
    
    // 调用搜索函数（日志会通过 log crate 输出到 stderr）
    let search_result = AcemcpTool::search_context(req).await;
    
    match search_result {
        Ok(result) => {
            let mut result_text = String::new();
            if let Ok(val) = serde_json::to_value(&result) {
                if let Some(arr) = val.get("content").and_then(|v| v.as_array()) {
                    for item in arr {
                        if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                            if let Some(txt) = item.get("text").and_then(|t| t.as_str()) {
                                result_text.push_str(txt);
                            }
                        }
                    }
                }
            }
            
            Ok(DebugSearchResult {
                success: true,
                result: Some(result_text),
                error: None,
            })
        }
        Err(e) => {
            Ok(DebugSearchResult {
                success: false,
                result: None,
                error: Some(format!("执行失败: {}", e)),
            })
        }
    }
}

/// 执行acemcp工具
#[tauri::command]
pub async fn execute_acemcp_tool(
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    match tool_name.as_str() {
        "search_context" => {
            // 解析参数
            let project_root_path = arguments.get("project_root_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少project_root_path参数".to_string())?
                .to_string();
            
            let query = arguments.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少query参数".to_string())?
                .to_string();
            
            // 执行搜索
            let req = AcemcpRequest { project_root_path, query };
            match AcemcpTool::search_context(req).await {
                Ok(result) => {
                    // 转换结果为JSON
                    if let Ok(val) = serde_json::to_value(&result) {
                        Ok(serde_json::json!({
                            "status": "success",
                            "result": val
                        }))
                    } else {
                        Err("结果序列化失败".to_string())
                    }
                }
                Err(e) => Ok(serde_json::json!({
                    "status": "error",
                    "error": e.to_string()
                })),
            }
        }
        _ => Err(format!("未知的工具: {}", tool_name)),
    }
}


