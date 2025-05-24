use anyhow::Result;
use notify_rust::Notification;
use std::collections::HashMap;
use std::sync::{mpsc, Mutex};
use tauri::{Manager, State};
use tokio::sync::mpsc as tokio_mpsc;

mod ipc;

use ipc::{IpcServer, Message, MessageType};

#[derive(Debug)]
struct AppState {
    pending_requests: Mutex<HashMap<String, mpsc::Sender<String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pending_requests: Mutex::new(HashMap::new()),
        }
    }
}

#[tauri::command]
async fn respond_to_request(
    id: String,
    response: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    let mut pending = state.pending_requests.lock().unwrap();
    if let Some(sender) = pending.remove(&id) {
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

async fn start_ipc_server(app_handle: tauri::AppHandle) -> Result<()> {
    println!("Starting IPC server...");
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
                    
                    if let Some(window) = app_handle.get_window("main") {
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
                        
                        match tauri::WindowBuilder::new(
                            &app_handle,
                            &window_label,
                            tauri::WindowUrl::App("index.html".into())
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
        .invoke_handler(tauri::generate_handler![respond_to_request, get_app_info, close_popup_window])
        .setup(|app| {
            let app_handle = app.handle();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_ipc_server(app_handle).await {
                    eprintln!("❌ 启动IPC服务器失败: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("❌ 运行Tauri应用失败");

    Ok(())
}
