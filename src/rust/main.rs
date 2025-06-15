use anyhow::Result;
use cunzhi::config::{
    load_config_and_apply_window_settings, load_standalone_telegram_config, AppState,
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::Manager;

// 重新导出所有命令函数
pub use cunzhi::config::mcp_commands::*;
pub use cunzhi::config::telegram_commands::*;
pub use cunzhi::telegram::*;
pub use cunzhi::ui::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .manage(AudioController {
            should_stop: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_always_on_top,
            set_always_on_top,
            sync_window_state,
            get_audio_notification_enabled,
            set_audio_notification_enabled,
            get_audio_url,
            set_audio_url,
            play_notification_sound,
            test_audio_sound,
            stop_audio_sound,
            get_available_audio_assets,
            refresh_audio_assets,
            get_theme,
            set_theme,
            get_window_config,
            set_window_config,
            get_reply_config,
            set_reply_config,
            get_window_settings,
            set_window_settings,
            get_window_settings_for_mode,
            get_current_window_size,
            apply_window_constraints,
            update_window_size,
            get_mcp_tools_config,
            set_mcp_tool_enabled,
            get_mcp_tools_status,
            reset_mcp_tools_config,
            send_mcp_response,
            get_cli_args,
            read_mcp_request,
            select_image_files,
            get_telegram_config,
            set_telegram_config,
            test_telegram_connection_cmd,
            start_telegram_sync,
            exit_app,
            build_mcp_send_response,
            build_mcp_continue_response
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 在setup中直接调用，不使用spawn
            tauri::async_runtime::block_on(async {
                let state = app.state::<AppState>();
                if let Err(e) = load_config_and_apply_window_settings(&state, &app_handle).await {
                    eprintln!("加载配置失败: {}", e);
                }

                // 初始化音频资源管理器
                if let Err(e) = initialize_audio_asset_manager(&app_handle) {
                    eprintln!("初始化音频资源管理器失败: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() -> Result<()> {
    // 检查程序是如何被调用的
    let args: Vec<String> = std::env::args().collect();

    // 处理命令行参数
    if args.len() >= 3 && args[1] == "--mcp-request" {
        // MCP 请求模式：检查是否需要启动GUI
        let request_file = &args[2];

        // 检查Telegram配置，决定是否启用纯Telegram模式
        match load_standalone_telegram_config() {
            Ok(telegram_config) => {
                if telegram_config.enabled && telegram_config.hide_frontend_popup {
                    // 纯Telegram模式：不启动GUI，直接处理
                    println!("🔕 启用纯Telegram模式，不显示GUI界面");
                    if let Err(e) = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(handle_telegram_only_mcp_request(request_file))
                    {
                        eprintln!("处理Telegram请求失败: {}", e);
                        std::process::exit(1);
                    }
                    return Ok(());
                } else {
                    // 正常模式：启动GUI处理弹窗
                    run();
                }
            }
            Err(e) => {
                eprintln!("加载Telegram配置失败: {}，使用默认GUI模式", e);
                // 配置加载失败时，使用默认行为（启动GUI）
                run();
            }
        }
    } else if args.len() >= 2 && (args[1] == "--help" || args[1] == "-h") {
        // 显示帮助信息
        print_help();
    } else if args.len() >= 2 && (args[1] == "--version" || args[1] == "-v") {
        // 显示版本信息
        print_version();
    } else {
        // 正常启动 GUI 应用（设置界面）
        run();
    }

    Ok(())
}

/// 显示帮助信息
fn print_help() {
    println!("寸止 - 智能代码审查工具");
    println!();
    println!("用法:");
    println!("  等一下                    启动设置界面");
    println!("  等一下 --mcp-request <文件>  处理 MCP 请求");
    println!("  等一下 --help             显示此帮助信息");
    println!("  等一下 --version          显示版本信息");
}

/// 显示版本信息
fn print_version() {
    println!("寸止 v{}", env!("CARGO_PKG_VERSION"));
}

/// 处理纯Telegram模式的MCP请求（不启动GUI）
async fn handle_telegram_only_mcp_request(request_file: &str) -> Result<()> {
    use cunzhi::mcp::types::{build_continue_response, build_send_response, PopupRequest};
    use cunzhi::telegram::TelegramCore;
    use std::fs;
    use teloxide::prelude::*;

    // 读取MCP请求文件
    let request_json = fs::read_to_string(request_file)?;
    let request: PopupRequest = serde_json::from_str(&request_json)?;

    // 加载Telegram配置
    let telegram_config = load_standalone_telegram_config()?;

    if !telegram_config.enabled {
        eprintln!("Telegram未启用，无法处理请求");
        return Ok(());
    }

    if telegram_config.bot_token.trim().is_empty() || telegram_config.chat_id.trim().is_empty() {
        eprintln!("Telegram配置不完整");
        return Ok(());
    }

    // 创建Telegram核心实例
    let core = TelegramCore::new(
        telegram_config.bot_token.clone(),
        telegram_config.chat_id.clone(),
    )?;

    // 发送消息到Telegram
    let predefined_options = request.predefined_options.unwrap_or_default();

    // 发送选项消息
    core.send_options_message(&request.message, &predefined_options, request.is_markdown)
        .await?;

    // 短暂延迟确保消息顺序
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 发送操作消息（假设启用继续回复）
    core.send_operation_message(true).await?;

    println!("✅ 消息已发送到Telegram，等待用户响应...");

    // 启动简单的消息监听循环
    let mut offset = 0i32;
    let mut selected_options: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut user_input = String::new();
    let mut options_message_id: Option<i32> = None;

    // 获取当前最新的消息ID作为基准
    if let Ok(updates) = core.bot.get_updates().limit(10).await {
        if let Some(update) = updates.last() {
            offset = update.id.0 as i32 + 1;
        }
    }

    // 监听循环（简化版本，只等待发送或继续操作）
    loop {
        match core.bot.get_updates().offset(offset).timeout(10).await {
            Ok(updates) => {
                for update in updates {
                    offset = update.id.0 as i32 + 1;

                    match update.kind {
                        teloxide::types::UpdateKind::CallbackQuery(callback_query) => {
                            // 从callback_query中提取消息ID
                            if let Some(message) = &callback_query.message {
                                if options_message_id.is_none() {
                                    options_message_id = Some(message.id().0);
                                }
                            }

                            use cunzhi::telegram::handle_callback_query;
                            if let Ok(Some(option)) =
                                handle_callback_query(&core.bot, &callback_query, core.chat_id)
                                    .await
                            {
                                // 切换选项状态
                                if selected_options.contains(&option) {
                                    selected_options.remove(&option);
                                } else {
                                    selected_options.insert(option.clone());
                                }

                                // 更新按钮状态
                                if let Some(msg_id) = options_message_id {
                                    let selected_vec: Vec<String> =
                                        selected_options.iter().cloned().collect();
                                    let _ = core
                                        .update_inline_keyboard(
                                            msg_id,
                                            &predefined_options,
                                            &selected_vec,
                                        )
                                        .await;
                                }
                            }
                        }
                        teloxide::types::UpdateKind::Message(message) => {
                            // 检查是否是包含 inline keyboard 的选项消息
                            if let Some(inline_keyboard) = message.reply_markup() {
                                // 检查是否包含我们的选项按钮
                                let mut contains_our_options = false;
                                for row in &inline_keyboard.inline_keyboard {
                                    for button in row {
                                        if let teloxide::types::InlineKeyboardButtonKind::CallbackData(callback_data) = &button.kind {
                                            if callback_data.starts_with("toggle:") {
                                                contains_our_options = true;
                                                break;
                                            }
                                        }
                                    }
                                    if contains_our_options {
                                        break;
                                    }
                                }

                                if contains_our_options {
                                    options_message_id = Some(message.id.0);
                                }
                            }

                            use cunzhi::telegram::handle_text_message;
                            if let Ok(Some(event)) =
                                handle_text_message(&message, core.chat_id, None).await
                            {
                                match event {
                                    cunzhi::telegram::TelegramEvent::SendPressed => {
                                        // 使用统一的响应构建函数
                                        let selected_list: Vec<String> =
                                            selected_options.iter().cloned().collect();

                                        let user_input_option = if user_input.is_empty() {
                                            None
                                        } else {
                                            Some(user_input.clone())
                                        };

                                        let response = build_send_response(
                                            user_input_option,
                                            selected_list,
                                            vec![], // 无GUI模式下没有图片
                                            Some(request.id.clone()),
                                            "telegram",
                                        );

                                        // 输出JSON响应到stdout
                                        println!("{}", response);

                                        // 发送确认消息
                                        let _ = core.send_message("✅ 响应已发送").await;
                                        return Ok(());
                                    }
                                    cunzhi::telegram::TelegramEvent::ContinuePressed => {
                                        // 使用统一的继续响应构建函数
                                        let response = build_continue_response(
                                            Some(request.id.clone()),
                                            "telegram_continue",
                                        );

                                        // 输出JSON响应到stdout
                                        println!("{}", response);

                                        // 发送确认消息
                                        let _ = core.send_message("✅ 继续操作已确认").await;
                                        return Ok(());
                                    }
                                    cunzhi::telegram::TelegramEvent::TextUpdated { text } => {
                                        user_input = text;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        // 短暂延迟避免过于频繁的请求
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
