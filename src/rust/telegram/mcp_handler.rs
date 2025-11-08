use anyhow::Result;
use std::collections::HashSet;
use teloxide::prelude::*;

use crate::config::{load_standalone_config, save_standalone_config};
use crate::mcp::types::{build_continue_response, build_send_response, PopupRequest};
use crate::telegram::{handle_callback_query, handle_text_message, TelegramCore, TelegramEvent};
use crate::log_important;

/// 处理纯Telegram模式的MCP请求（不启动GUI）
pub async fn handle_telegram_only_mcp_request(request_file: &str) -> Result<()> {
    // 读取MCP请求文件
    let request_json = std::fs::read_to_string(request_file)?;
    let request: PopupRequest = serde_json::from_str(&request_json)?;

    // 加载完整配置
    let mut app_config = load_standalone_config()?;

    // 记录会话请求（如果有 session_id）
    if let Some(session_id) = &request.session_id {
        app_config.telegram_config.record_session_request(session_id);
        // 保存配置以记录待配置会话
        if let Err(e) = save_standalone_config(&app_config) {
            log_important!(warn, "保存会话记录失败: {}", e);
        }
    }

    let telegram_config = &app_config.telegram_config;

    if !telegram_config.enabled {
        log_important!(warn, "Telegram未启用，无法处理请求");
        return Ok(());
    }

    // 获取要使用的 bot 配置
    // 优先级：bot_name > session_id 映射 > 默认 bot
    log_important!(info, "🔍 Bot 选择逻辑:");
    log_important!(info, "  - bot_name: {:?}", request.bot_name);
    log_important!(info, "  - session_id: {:?}", request.session_id);
    log_important!(info, "  - session_bot_mapping: {:?}", telegram_config.session_bot_mapping);
    log_important!(info, "  - default_bot: {}", telegram_config.default_bot);

    let bot_config = if let Some(bot_name) = &request.bot_name {
        // 1. 如果明确指定了 bot_name，使用指定的 bot
        log_important!(info, "  ✅ 使用指定的 Bot: {}", bot_name);
        telegram_config.get_bot(bot_name)
            .ok_or_else(|| anyhow::anyhow!("Bot '{}' 不存在", bot_name))?
    } else if let Some(session_id) = &request.session_id {
        // 2. 如果提供了 session_id，尝试从映射中获取对应的 bot
        let bot = telegram_config.get_bot_for_session(Some(session_id))
            .ok_or_else(|| anyhow::anyhow!("没有可用的 Bot 配置"))?;
        log_important!(info, "  ✅ 根据 session_id 选择 Bot: {}", bot.name);
        bot
    } else {
        // 3. 否则使用默认 bot
        let bot = telegram_config.get_default_bot()
            .ok_or_else(|| anyhow::anyhow!("没有可用的 Bot 配置"))?;
        log_important!(info, "  ✅ 使用默认 Bot: {}", bot.name);
        bot
    };

    // 写入调试信息到临时文件
    let debug_info = format!(
        "Bot Selection Debug:\n\
         - bot_name: {:?}\n\
         - session_id: {:?}\n\
         - session_bot_mapping: {:?}\n\
         - selected_bot: {}\n",
        request.bot_name,
        request.session_id,
        telegram_config.session_bot_mapping,
        bot_config.name
    );
    let _ = std::fs::write("/tmp/cunzhi_bot_selection_debug.txt", debug_info);

    // 创建Telegram核心实例，使用配置中的API URL
    let api_url = if bot_config.api_base_url == crate::constants::telegram::API_BASE_URL {
        None
    } else {
        Some(bot_config.api_base_url.clone())
    };

    let core = TelegramCore::new_with_api_url(
        bot_config.bot_token.clone(),
        bot_config.chat_id.clone(),
        api_url,
    )?;

    // 发送消息到Telegram
    let predefined_options = request.predefined_options.clone().unwrap_or_default();

    // 发送选项消息
    core.send_options_message(&request.message, &predefined_options, request.is_markdown)
        .await?;

    // 短暂延迟确保消息顺序
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 发送操作消息（假设启用继续回复）
    core.send_operation_message(true).await?;

    // 启动消息监听循环
    start_telegram_mcp_listener(core, request, predefined_options).await
}

/// 启动Telegram MCP消息监听循环
async fn start_telegram_mcp_listener(
    core: TelegramCore,
    request: PopupRequest,
    predefined_options: Vec<String>,
) -> Result<()> {
    let mut offset = 0i32;
    let mut selected_options: HashSet<String> = HashSet::new();
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
                            if let Err(e) = handle_callback_query_update(
                                &core,
                                &callback_query,
                                &predefined_options,
                                &mut selected_options,
                                &mut options_message_id,
                            ).await {
                                log_important!(warn, "处理callback query失败: {}", e);
                            }
                        }
                        teloxide::types::UpdateKind::Message(message) => {
                            // 处理选项消息ID识别
                            if let Err(e) = handle_message_update(
                                &core,
                                &message,
                                &predefined_options,
                                &mut options_message_id,
                                &mut user_input,
                                &selected_options,
                                &request,
                            ).await {
                                if let Some(_result) = e.downcast_ref::<ProcessingComplete>() {
                                    return Ok(());
                                }
                                log_important!(warn, "处理消息失败: {}", e);
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

/// 处理callback query更新
async fn handle_callback_query_update(
    core: &TelegramCore,
    callback_query: &teloxide::types::CallbackQuery,
    predefined_options: &[String],
    selected_options: &mut HashSet<String>,
    options_message_id: &mut Option<i32>,
) -> Result<()> {
    // 只有当有预定义选项时才处理 callback queries
    if predefined_options.is_empty() {
        return Ok(());
    }

    // 从callback_query中提取消息ID
    if let Some(message) = &callback_query.message {
        if options_message_id.is_none() {
            *options_message_id = Some(message.id().0);
        }
    }

    if let Ok(Some(option)) = handle_callback_query(&core.bot, callback_query, core.chat_id).await {
        // 切换选项状态
        if selected_options.contains(&option) {
            selected_options.remove(&option);
        } else {
            selected_options.insert(option.clone());
        }

        // 更新按钮状态
        if let Some(msg_id) = *options_message_id {
            let selected_vec: Vec<String> = selected_options.iter().cloned().collect();
            let _ = core
                .update_inline_keyboard(msg_id, predefined_options, &selected_vec)
                .await;
        }
    }

    Ok(())
}

/// 处理消息更新
async fn handle_message_update(
    core: &TelegramCore,
    message: &teloxide::types::Message,
    predefined_options: &[String],
    options_message_id: &mut Option<i32>,
    user_input: &mut String,
    selected_options: &HashSet<String>,
    request: &PopupRequest,
) -> Result<()> {
    // 识别选项消息ID
    identify_options_message_id(message, predefined_options, options_message_id);

    // 处理文本消息事件
    if let Ok(Some(event)) = handle_text_message(message, core.chat_id, None).await {
        match event {
            TelegramEvent::SendPressed => {
                handle_send_pressed(core, selected_options, user_input, request).await?;
                return Err(ProcessingComplete.into());
            }
            TelegramEvent::ContinuePressed => {
                handle_continue_pressed(core, request).await?;
                return Err(ProcessingComplete.into());
            }
            TelegramEvent::TextUpdated { text } => {
                *user_input = text;
            }
            _ => {}
        }
    }

    Ok(())
}

/// 识别选项消息ID
fn identify_options_message_id(
    message: &teloxide::types::Message,
    predefined_options: &[String],
    options_message_id: &mut Option<i32>,
) {
    // 只有当有预定义选项时才检查 inline keyboard
    if predefined_options.is_empty() {
        return;
    }

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
            *options_message_id = Some(message.id.0);
        }
    }
}

/// 处理发送按钮按下
async fn handle_send_pressed(
    core: &TelegramCore,
    selected_options: &HashSet<String>,
    user_input: &str,
    request: &PopupRequest,
) -> Result<()> {
    // 使用统一的响应构建函数
    let selected_list: Vec<String> = selected_options.iter().cloned().collect();

    let user_input_option = if user_input.is_empty() {
        None
    } else {
        Some(user_input.to_string())
    };

    let response = build_send_response(
        user_input_option,
        selected_list.clone(),
        vec![], // 无GUI模式下没有图片
        Some(request.id.clone()),
        "telegram",
    );

    // 输出JSON响应到stdout（MCP协议要求）
    println!("{}", response);

    // 发送确认消息（使用统一的反馈消息生成函数）
    let feedback_message = crate::telegram::core::build_feedback_message(
        &selected_list,
        user_input,
        false, // 不是继续操作
    );
    let _ = core.send_message(&feedback_message).await;

    Ok(())
}

/// 处理继续按钮按下
async fn handle_continue_pressed(
    core: &TelegramCore,
    request: &PopupRequest,
) -> Result<()> {
    // 使用统一的继续响应构建函数
    let response = build_continue_response(
        Some(request.id.clone()),
        "telegram_continue",
    );

    // 输出JSON响应到stdout（MCP协议要求）
    println!("{}", response);

    // 发送确认消息（使用统一的反馈消息生成函数）
    let feedback_message = crate::telegram::core::build_feedback_message(
        &[],  // 继续操作没有选项
        "",   // 继续操作没有用户输入
        true, // 是继续操作
    );
    let _ = core.send_message(&feedback_message).await;

    Ok(())
}

/// 处理完成标记（用于从监听循环中退出）
#[derive(Debug)]
struct ProcessingComplete;

impl std::fmt::Display for ProcessingComplete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Processing complete")
    }
}

impl std::error::Error for ProcessingComplete {}
