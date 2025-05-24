use anyhow::Result;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc as tokio_mpsc;

mod ipc;

use ipc::{IpcServer, Message, MessageType};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    init_prompt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // # 交互式任务规则

            // - 需要了解参数时使用 `ai-review-cli help` 命令
            // - 遇到问题或不确定时必须通过 `ai-review-cli [message] --timeout` 询问，等待回复
            // - 如果超时没有任何输出，默认为我对你做了正面的响应
            // - 聊天结束时必须运行 `ai-review-cli 你觉得我当前的任务是否已经完成`，不要自主/自以为完成了任务或结束对话，必须得到用户认可
            // - 然后在恢复对话之前检索输入值。

            init_prompt: "# 交互式任务规则\n\n- 需要了解参数时使用 `ai-review-cli help` 命令\n- 遇到问题或不确定时必须通过 `ai-review-cli [message] --timeout` 询问，等待回复\n- 如果超时没有任何输出，默认为我对你做了正面的响应\n- 聊天结束时必须运行 `ai-review-cli 你觉得我当前的任务是否已经完成`，不要自主/自以为完成了任务或结束对话，必须得到用户认可\n- 然后在恢复对话之前检索输入值。".to_string(),
        }
    }
}

#[derive(Debug)]
struct AppState {
    pending_requests: Mutex<HashMap<String, mpsc::Sender<String>>>,
    config: Mutex<AppConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pending_requests: Mutex::new(HashMap::new()),
            config: Mutex::new(AppConfig::default()),
        }
    }
}

#[tauri::command]
async fn respond_to_request(
    request_id: String,
    response: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    let mut pending = state.pending_requests.lock().unwrap();
    if let Some(sender) = pending.remove(&request_id) {
        sender.send(response).map_err(|_| "Failed to send response".to_string())?;

        // 如果是弹窗窗口，关闭它
        if window.label().starts_with("review-") {
            let _ = window.close();
        }
    }
    Ok(())
}

#[tauri::command]
async fn get_app_info() -> Result<String, String> {
    Ok("AI Review App v0.1.0".to_string())
}

#[tauri::command]
async fn close_popup_window(window: tauri::Window) -> Result<(), String> {
    if window.label().starts_with("review-") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn get_init_prompt(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().unwrap();
    Ok(config.init_prompt.clone())
}

#[tauri::command]
async fn set_init_prompt(prompt: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    {
        let mut config = state.config.lock().unwrap();
        config.init_prompt = prompt;
    }

    // 保存配置到文件
    save_config(&state, &app).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn reset_init_prompt(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<String, String> {
    let default_prompt = AppConfig::default().init_prompt;
    {
        let mut config = state.config.lock().unwrap();
        config.init_prompt = default_prompt.clone();
    }

    // 保存配置到文件
    save_config(&state, &app).await.map_err(|e| e.to_string())?;
    Ok(default_prompt)
}

#[tauri::command]
async fn check_ipc_status() -> Result<bool, String> {
    use std::os::unix::net::UnixStream;
    let socket_path = ipc::get_socket_path();

    match UnixStream::connect(&socket_path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn install_cli_command(app: tauri::AppHandle) -> Result<String, String> {
    install_cli_symlink(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_cli_installed() -> Result<bool, String> {
    let cli_path = "/usr/local/bin/ai-review-cli";
    Ok(std::path::Path::new(cli_path).exists())
}

async fn install_cli_symlink(app: &tauri::AppHandle) -> Result<String> {
    use std::process::Command;

    // 获取当前应用的路径
    let app_path = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("无法获取应用路径: {}", e))?;

    let app_dir = app_path.parent()
        .ok_or_else(|| anyhow::anyhow!("无法获取应用目录"))?;

    // CLI二进制文件在应用bundle中的路径
    let cli_source = app_dir.join("ai-review-cli");
    let cli_target = "/usr/local/bin/ai-review-cli";

    // 检查源文件是否存在
    if !cli_source.exists() {
        return Err(anyhow::anyhow!("CLI二进制文件不存在: {:?}", cli_source));
    }

    // 创建 /usr/local/bin 目录（如果不存在）
    let output = Command::new("mkdir")
        .args(["-p", "/usr/local/bin"])
        .output()
        .map_err(|e| anyhow::anyhow!("创建目录失败: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("创建 /usr/local/bin 目录失败"));
    }

    // 移除旧的符号链接（如果存在）
    if std::path::Path::new(cli_target).exists() {
        let _ = std::fs::remove_file(cli_target);
    }

    // 创建符号链接
    let output = Command::new("ln")
        .args(["-s", &cli_source.to_string_lossy(), cli_target])
        .output()
        .map_err(|e| anyhow::anyhow!("创建符号链接失败: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("创建符号链接失败: {}", error_msg));
    }

    // 设置执行权限
    let output = Command::new("chmod")
        .args(["+x", cli_target])
        .output()
        .map_err(|e| anyhow::anyhow!("设置执行权限失败: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("设置执行权限失败: {}", error_msg));
    }

    Ok(format!("CLI命令已成功安装到: {}", cli_target))
}

async fn auto_install_cli_on_startup(app: &tauri::AppHandle) -> Result<()> {
    // 检查CLI是否已安装
    let cli_path = "/usr/local/bin/ai-review-cli";
    if !std::path::Path::new(cli_path).exists() {
        println!("🔧 CLI命令未安装，正在自动安装...");
        match install_cli_symlink(app).await {
            Ok(msg) => {
                println!("✅ {}", msg);

                // 发送通知
                let notification_result = Notification::new()
                    .summary("AI Review")
                    .body("CLI命令已自动安装，您现在可以在终端中使用 'ai-review-cli' 命令")
                    .icon("dialog-information")
                    .timeout(5000)
                    .show();

                if let Err(e) = notification_result {
                    eprintln!("❌ 发送安装通知失败: {}", e);
                }
            }
            Err(e) => {
                eprintln!("❌ 自动安装CLI命令失败: {}", e);
            }
        }
    } else {
        println!("✅ CLI命令已安装");
    }

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
    let config = {
        let config_guard = state.config.lock().unwrap();
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

        let mut config_guard = state.config.lock().unwrap();
        *config_guard = config;
    }

    Ok(())
}

async fn start_ipc_server(app_handle: AppHandle) -> Result<()> {
    println!("Starting IPC server...");
    let socket_path = ipc::get_socket_path();
    println!("📁 Socket路径: {:?}", socket_path);

    let (tx, mut rx) = tokio_mpsc::unbounded_channel::<(Message, mpsc::Sender<String>)>();

    // 启动IPC服务器
    let server = IpcServer::new(tx)?;
    println!("IPC server created successfully");
    tokio::spawn(async move {
        println!("IPC server task started");
        if let Err(e) = server.run().await {
            eprintln!("IPC server error: {}", e);
        }
    });

    // 处理接收到的消息
    tokio::spawn(async move {
        println!("📨 消息处理任务已启动");
        while let Some((message, response_sender)) = rx.recv().await {
            println!("📥 从IPC接收到消息: {:?}", message);
            match message.message_type {
                MessageType::Request => {
                    println!("🎯 处理请求类型消息");

                    // 检查是否是 init 指令
                    if message.content.trim().eq_ignore_ascii_case("init") {
                        println!("🔧 检测到 init 指令，返回提示词");
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let config = state.config.lock().unwrap();
                            let init_prompt = config.init_prompt.clone();
                            drop(config); // 释放锁

                            // 直接发送提示词作为响应
                            if let Err(e) = response_sender.send(init_prompt) {
                                eprintln!("❌ 发送 init 提示词失败: {}", e);
                            } else {
                                println!("✅ init 提示词已发送");
                            }
                        } else {
                            eprintln!("❌ 无法获取应用状态");
                            let _ = response_sender.send("错误：无法获取配置".to_string());
                        }
                        continue; // 跳过正常的UI处理流程
                    }

                    // 存储待处理的请求
                    println!("💾 正在存储待处理的请求...");
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let mut pending = state.pending_requests.lock().unwrap();
                        pending.insert(message.id.clone(), response_sender);
                        println!("✅ 已存储请求，ID: {}", message.id);
                    } else {
                        println!("❌ 无法获取应用状态");
                    }

                    // 直接使用主窗口来处理请求（简化版本）
                    println!("📨 接收到消息内容: {}", message.content);
                    println!("🔄 正在使用主窗口显示消息...");

                    if let Some(window) = app_handle.get_webview_window("main") {
                        println!("✅ 找到主窗口，正在发送消息...");

                        // 显示并聚焦主窗口
                        if let Err(e) = window.show() {
                            eprintln!("❌ 显示主窗口失败: {}", e);
                        } else {
                            println!("✅ 主窗口显示成功");
                        }

                        if let Err(e) = window.set_focus() {
                            eprintln!("❌ 设置主窗口焦点失败: {}", e);
                        } else {
                            println!("✅ 主窗口焦点设置成功");
                        }

                        if let Err(e) = window.set_always_on_top(true) {
                            eprintln!("❌ 设置主窗口置顶失败: {}", e);
                        } else {
                            println!("✅ 主窗口置顶设置成功");
                        }

                        // 发送消息到主窗口
                        let message_clone = message.clone();
                        let window_clone = window.clone();
                        tokio::spawn(async move {
                            println!("⏳ 等待500毫秒后发送消息到主窗口...");
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                            match window_clone.emit("new-request", &message_clone) {
                                Ok(_) => println!("✅ 成功发送消息到主窗口"),
                                Err(e) => eprintln!("❌ 发送消息到主窗口失败: {}", e),
                            }
                        });

                        // 发送系统通知
                        println!("🔔 正在发送系统通知...");
                        let notification_result = Notification::new()
                            .summary("AI Review - 新消息")
                            .body(&format!("收到新消息: {}", &message.content))
                            .icon("dialog-information")
                            .timeout(5000) // 5秒后自动消失
                            .show();

                        match notification_result {
                            Ok(_) => println!("✅ 系统通知发送成功"),
                            Err(e) => eprintln!("❌ 系统通知发送失败: {}", e),
                        }
                    } else {
                        eprintln!("❌ 主窗口未找到!");

                        // 尝试创建新窗口作为最后的备选方案
                        println!("🔄 尝试创建新窗口作为备选方案...");
                        let window_label = format!("review-{}", message.id);

                        match tauri::WebviewWindowBuilder::new(
                            &app_handle,
                            &window_label,
                            tauri::WebviewUrl::App("index.html".into())
                        )
                        .title("AI Review - 快速回复")
                        .inner_size(500.0, 400.0)
                        .center()
                        .resizable(true)
                        .build() {
                            Ok(window) => {
                                println!("✅ 成功创建备选窗口: {}", window_label);
                                let _ = window.show();
                                let _ = window.set_focus();

                                let message_clone = message.clone();
                                let window_clone = window.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                    let _ = window_clone.emit("new-request", &message_clone);
                                });
                            }
                            Err(e) => {
                                eprintln!("❌ 创建备选窗口失败: {}", e);
                            }
                        }
                    }
                }
                _ => {
                    println!("接收到非请求类型消息: {:?}", message);
                }
            }
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_state = AppState::default();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            respond_to_request,
            get_app_info,
            close_popup_window,
            get_init_prompt,
            set_init_prompt,
            reset_init_prompt,
            check_ipc_status,
            install_cli_command,
            check_cli_installed
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_handle_clone = app_handle.clone();
            let app_handle_cli = app_handle.clone();

            // 加载配置
            tauri::async_runtime::spawn(async move {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Err(e) = load_config(&state, &app_handle).await {
                        eprintln!("❌ 加载配置失败: {}", e);
                    }
                }
            });

            // 自动安装CLI命令
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auto_install_cli_on_startup(&app_handle_cli).await {
                    eprintln!("❌ 自动安装CLI命令失败: {}", e);
                }
            });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_ipc_server(app_handle_clone).await {
                    eprintln!("❌ 启动IPC服务器失败: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("❌ 运行Tauri应用失败");

    Ok(())
}
