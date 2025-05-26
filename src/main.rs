use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, Emitter};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    init_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPopupRequest {
    pub id: String,
    pub message: String,
    pub predefined_options: Option<Vec<String>>,
    pub is_markdown: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            init_prompt: "# 交互式任务规则\n\n- 需要了解参数时使用 `ai-review-cli help` 命令\n- 遇到问题或不确定时必须通过 `ai-review-cli [message] --timeout` 询问，等待回复\n- 如果超时没有任何输出，默认为我对你做了正面的响应\n- 聊天结束时必须运行 `ai-review-cli 你觉得我当前的任务是否已经完成`，不要自主/自以为完成了任务或结束对话，必须得到用户认可\n- 然后在恢复对话之前检索输入值。".to_string(),
        }
    }
}

#[derive(Debug)]
struct AppState {
    config: Mutex<AppConfig>,
    response_channel: Mutex<Option<tokio::sync::oneshot::Sender<String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(AppConfig::default()),
            response_channel: Mutex::new(None),
        }
    }
}

#[tauri::command]
async fn get_app_info() -> Result<String, String> {
    Ok("AI Review App v0.1.0".to_string())
}

#[tauri::command]
async fn get_init_prompt(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
    Ok(config.init_prompt.clone())
}

#[tauri::command]
async fn set_init_prompt(prompt: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    if prompt.trim().is_empty() {
        return Err("提示词不能为空".to_string());
    }

    {
        let mut config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
        config.init_prompt = prompt.trim().to_string();
    }

    // 保存配置到文件
    save_config(&state, &app).await.map_err(|e| format!("保存配置失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn reset_init_prompt(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<String, String> {
    let default_prompt = AppConfig::default().init_prompt;
    {
        let mut config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
        config.init_prompt = default_prompt.clone();
    }

    // 保存配置到文件
    save_config(&state, &app).await.map_err(|e| format!("保存配置失败: {}", e))?;
    Ok(default_prompt)
}

#[tauri::command]
async fn send_mcp_response(response: String, state: State<'_, AppState>) -> Result<(), String> {
    if response.trim().is_empty() {
        return Err("响应内容不能为空".to_string());
    }

    // 通过channel发送响应（如果有的话）
    let sender = {
        let mut channel = state.response_channel.lock().map_err(|e| format!("获取响应通道失败: {}", e))?;
        channel.take()
    };

    if let Some(sender) = sender {
        let _ = sender.send(response);
    }

    Ok(())
}

#[tauri::command]
fn get_cli_args() -> Result<serde_json::Value, String> {
    let args: Vec<String> = std::env::args().collect();
    let mut result = serde_json::Map::new();

    // 检查是否有 --mcp-request 参数
    if args.len() >= 3 && args[1] == "--mcp-request" {
        result.insert("mcp_request".to_string(), serde_json::Value::String(args[2].clone()));
    }

    Ok(serde_json::Value::Object(result))
}

#[tauri::command]
fn read_mcp_request(file_path: String) -> Result<serde_json::Value, String> {
    if !std::path::Path::new(&file_path).exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return Err("文件内容为空".to_string());
            }
            match serde_json::from_str(&content) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("解析JSON失败: {}", e))
            }
        }
        Err(e) => Err(format!("读取文件失败: {}", e))
    }
}

#[tauri::command]
fn exit_app() -> Result<(), String> {
    std::process::exit(0);
}

fn get_config_path(app: &AppHandle) -> Result<PathBuf> {
    let app_dir = app.path()
        .app_config_dir()
        .map_err(|e| anyhow::anyhow!("无法获取应用配置目录: {}", e))?;

    // 确保目录存在
    fs::create_dir_all(&app_dir)?;

    Ok(app_dir.join("config.json"))
}

async fn save_config(state: &State<'_, AppState>, app: &AppHandle) -> Result<()> {
    let config = {
        let config_guard = state.config.lock()
            .map_err(|e| anyhow::anyhow!("获取配置锁失败: {}", e))?;
        config_guard.clone()
    };

    let config_path = get_config_path(app)?;
    let config_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, config_json)?;

    Ok(())
}

async fn load_config(state: &State<'_, AppState>, app: &AppHandle) -> Result<()> {
    let config_path = get_config_path(app)?;

    if config_path.exists() {
        let config_json = fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&config_json)?;

        let mut config_guard = state.config.lock()
            .map_err(|e| anyhow::anyhow!("获取配置锁失败: {}", e))?;
        *config_guard = config;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_state = AppState::default();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_init_prompt,
            set_init_prompt,
            reset_init_prompt,
            send_mcp_response,
            get_cli_args,
            read_mcp_request,
            exit_app
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 检查命令行参数
            let args: Vec<String> = std::env::args().collect();
            if args.len() >= 3 && args[1] == "--mcp-request" {
                // MCP弹窗模式
                let request_file = args[2].clone();
                let app_handle_mcp = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = handle_mcp_popup_mode(app_handle_mcp, &request_file).await {
                        eprintln!("MCP弹窗模式处理失败: {}", e);
                        std::process::exit(1);
                    }
                });
            } else {
                // 正常模式 - 只加载配置，不启动文件监听
                let app_handle_normal = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app_handle_normal.try_state::<AppState>() {
                        if let Err(e) = load_config(&state, &app_handle_normal).await {
                            eprintln!("加载配置失败: {}", e);
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("应用启动失败");

    Ok(())
}

async fn handle_mcp_popup_mode(app_handle: AppHandle, request_file: &str) -> Result<()> {
    // 检查请求文件是否存在
    if !std::path::Path::new(request_file).exists() {
        return Err(anyhow::anyhow!("MCP请求文件不存在: {}", request_file));
    }

    // 读取MCP请求数据
    let request_json = fs::read_to_string(request_file)?;
    if request_json.trim().is_empty() {
        return Err(anyhow::anyhow!("MCP请求文件内容为空"));
    }

    let request: McpPopupRequest = serde_json::from_str(&request_json)?;

    // 设置响应通道
    let (sender, receiver) = tokio::sync::oneshot::channel();
    if let Some(state) = app_handle.try_state::<AppState>() {
        let mut channel = state.response_channel.lock()
            .map_err(|e| anyhow::anyhow!("获取响应通道失败: {}", e))?;
        *channel = Some(sender);
    }

    // 等待窗口创建完成
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 获取主窗口并发送MCP请求事件
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("mcp-request", &request);
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.set_always_on_top(true);

        // 等待用户响应
        match tokio::time::timeout(Duration::from_secs(60), receiver).await {
            Ok(Ok(response)) => {
                println!("{}", response.trim());
                app_handle.exit(0);
            }
            Ok(Err(_)) => {
                println!("取消");
                app_handle.exit(0);
            }
            Err(_) => {
                // 超时处理
                println!("取消");
                app_handle.exit(0);
            }
        }
    } else {
        return Err(anyhow::anyhow!("无法获取主窗口"));
    }

    Ok(())
}
