use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, Emitter};
use std::time::Duration;
use std::os::unix::fs as unix_fs;

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
    pub timeout: Option<u64>, // 超时时间（秒）
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
async fn send_mcp_response(response: serde_json::Value, state: State<'_, AppState>) -> Result<(), String> {
    // 将响应序列化为JSON字符串
    let response_str = serde_json::to_string(&response)
        .map_err(|e| format!("序列化响应失败: {}", e))?;

    if response_str.trim().is_empty() {
        return Err("响应内容不能为空".to_string());
    }

    // 通过channel发送响应（如果有的话）
    let sender = {
        let mut channel = state.response_channel.lock().map_err(|e| format!("获取响应通道失败: {}", e))?;
        channel.take()
    };

    if let Some(sender) = sender {
        let _ = sender.send(response_str);
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
async fn exit_app(app: AppHandle) -> Result<(), String> {
    // 关闭所有窗口
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }

    // 强制退出应用
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    app.exit(0);
    Ok(())
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
    let config_path = get_config_path(app)?;
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let config = state.config.lock().map_err(|e| anyhow::anyhow!("获取配置失败: {}", e))?;
    let config_json = serde_json::to_string_pretty(&*config)?;
    
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

/// 创建命令行工具的软链接
async fn create_cli_symlinks() -> Result<()> {
    // 只在 macOS 上执行
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // 获取当前应用的路径
        let current_exe = std::env::current_exe()?;
        
        // 检查是否在 App Bundle 中运行
        if let Some(app_bundle_path) = get_app_bundle_path(&current_exe) {
            let bin_dir = "/usr/local/bin";
            let ui_binary = current_exe;
            let mcp_binary = app_bundle_path.join("Contents/Resources/ai-review-mcp");
            
            // 检查 MCP 二进制文件是否存在
            if !mcp_binary.exists() {
                println!("⚠️  MCP 二进制文件不存在，跳过软链接创建: {:?}", mcp_binary);
                return Ok(());
            }
            
            // 检查 /usr/local/bin 目录是否存在且可写
            if !std::path::Path::new(bin_dir).exists() {
                println!("📁 创建 {} 目录...", bin_dir);
                let output = Command::new("sudo")
                    .args(&["mkdir", "-p", bin_dir])
                    .output();
                    
                if let Err(e) = output {
                    println!("⚠️  无法创建 bin 目录: {}", e);
                    return Ok(());
                }
            }
            
            // 创建软链接
            let ui_link = format!("{}/ai-review-ui", bin_dir);
            let mcp_link = format!("{}/ai-review-mcp", bin_dir);
            
            // 移除旧的软链接
            let _ = std::fs::remove_file(&ui_link);
            let _ = std::fs::remove_file(&mcp_link);
            
            // 尝试创建软链接
            match unix_fs::symlink(&ui_binary, &ui_link) {
                Ok(_) => println!("✅ 创建 UI 软链接: {} -> {:?}", ui_link, ui_binary),
                Err(e) => {
                    // 如果普通用户无权限，尝试使用 sudo
                    println!("🔐 需要管理员权限创建软链接...");
                    let output = Command::new("sudo")
                        .args(&["ln", "-sf", &ui_binary.to_string_lossy(), &ui_link])
                        .output();
                        
                    match output {
                        Ok(result) if result.status.success() => {
                            println!("✅ 创建 UI 软链接: {}", ui_link);
                        }
                        _ => println!("⚠️  无法创建 UI 软链接: {}", e),
                    }
                }
            }
            
            match unix_fs::symlink(&mcp_binary, &mcp_link) {
                Ok(_) => println!("✅ 创建 MCP 软链接: {} -> {:?}", mcp_link, mcp_binary),
                Err(e) => {
                    // 如果普通用户无权限，尝试使用 sudo
                    let output = Command::new("sudo")
                        .args(&["ln", "-sf", &mcp_binary.to_string_lossy(), &mcp_link])
                        .output();
                        
                    match output {
                        Ok(result) if result.status.success() => {
                            println!("✅ 创建 MCP 软链接: {}", mcp_link);
                        }
                        _ => println!("⚠️  无法创建 MCP 软链接: {}", e),
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 获取 App Bundle 的路径
#[cfg(target_os = "macos")]
fn get_app_bundle_path(current_exe: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut path = current_exe;
    
    // 向上查找直到找到 .app 目录
    while let Some(parent) = path.parent() {
        if let Some(name) = parent.file_name() {
            if name.to_string_lossy().ends_with(".app") {
                return Some(parent.to_path_buf());
            }
        }
        path = parent;
    }
    
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    // 检查程序是如何被调用的
    let program_name = std::env::args().next()
        .map(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("ai-review-ui")
                .to_string()
        })
        .unwrap_or_else(|| "ai-review-ui".to_string());

    // 如果是以 ai-review-mcp 名称调用，提示用户使用独立的MCP服务器
    if program_name == "ai-review-mcp" {
        println!("🚀 启动 AI Review MCP 服务器...");
        println!("请使用独立的 ai-review-mcp 二进制文件");
        std::process::exit(1);
    }

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

            // 创建命令行工具的软链接
            tauri::async_runtime::spawn(async move {
                if let Err(e) = create_cli_symlinks().await {
                    eprintln!("创建软链接失败: {}", e);
                }
            });

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

    // 尝试建立弹窗连接，支持重连机制
    let max_retries = 3;
    for attempt in 1..=max_retries {
        match try_create_popup_connection(&app_handle, &request, attempt).await {
            Ok(response) => {
                println!("{}", response.trim());

                // 关闭所有窗口
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.close();
                }

                // 强制退出应用
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                app_handle.exit(0);
                return Ok(());
            }
            Err(e) if attempt < max_retries => {
                eprintln!("弹窗连接失败 (尝试 {}/{}): {}", attempt, max_retries, e);
                // 等待一段时间后重试
                tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;
                continue;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("弹窗连接最终失败: {}", e));
            }
        }
    }

    Ok(())
}

async fn try_create_popup_connection(app_handle: &AppHandle, request: &McpPopupRequest, _attempt: u64) -> Result<String> {
    // 设置响应通道
    let (sender, receiver) = tokio::sync::oneshot::channel();
    if let Some(state) = app_handle.try_state::<AppState>() {
        let mut channel = state.response_channel.lock()
            .map_err(|e| anyhow::anyhow!("获取响应通道失败: {}", e))?;
        *channel = Some(sender);
    }

    // 获取主窗口并发送MCP请求事件
    if let Some(window) = app_handle.get_webview_window("main") {
        // 立即显示窗口和设置属性
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        
        // 先发送事件，后设置焦点
        window.emit("mcp-request", &request)
            .map_err(|e| anyhow::anyhow!("发送MCP请求事件失败: {}", e))?;
            
        // 延迟设置焦点，让Vue组件有时间初始化
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = window.set_focus();

        // 等待用户响应，根据配置决定是否超时
        if let Some(timeout_secs) = request.timeout {
            // 有超时配置，使用配置的超时时间
            match tokio::time::timeout(Duration::from_secs(timeout_secs), receiver).await {
                Ok(Ok(response)) => {
                    if response.trim() == "CANCELLED" {
                        return Err(anyhow::anyhow!("用户取消了操作"));
                    }
                    Ok(response)
                }
                Ok(Err(_)) => {
                    Err(anyhow::anyhow!("响应通道异常关闭，可能需要重试"))
                }
                Err(_) => {
                    // 超时处理 - 返回默认完成消息
                    Ok("完成".to_string())
                }
            }
        } else {
            // 无超时配置，使用健康检查的无限等待
            wait_for_response_with_health_check(receiver, app_handle).await
        }
    } else {
        Err(anyhow::anyhow!("无法获取主窗口"))
    }
}

async fn wait_for_response_with_health_check(
    mut receiver: tokio::sync::oneshot::Receiver<String>,
    app_handle: &AppHandle
) -> Result<String> {
    // 使用健康检查的无限等待
    let health_check_interval = Duration::from_secs(30); // 每30秒检查一次

    loop {
        tokio::select! {
            // 等待用户响应
            result = &mut receiver => {
                match result {
                    Ok(response) => {
                        if response.trim() == "CANCELLED" {
                            return Err(anyhow::anyhow!("用户取消了操作"));
                        }
                        return Ok(response);
                    }
                    Err(_) => {
                        return Err(anyhow::anyhow!("响应通道异常关闭，建议重试"));
                    }
                }
            }
            // 健康检查
            _ = tokio::time::sleep(health_check_interval) => {
                // 检查窗口是否仍然存在和可见
                if let Some(window) = app_handle.get_webview_window("main") {
                    if let Ok(is_visible) = window.is_visible() {
                        if !is_visible {
                            // 窗口不可见，尝试重新显示
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("主窗口丢失，需要重试"));
                }
            }
        }
    }
}
