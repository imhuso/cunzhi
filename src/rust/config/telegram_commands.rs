use crate::config::{save_config, AppState, TelegramConfig};
use crate::telegram::process_telegram_markdown;
use tauri::{AppHandle, Emitter, State};
use teloxide::{
    prelude::*,
    types::{
        ChatId, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup,
        MessageId, ParseMode,
    },
    Bot,
};

/// 获取Telegram配置
#[tauri::command]
pub async fn get_telegram_config(state: State<'_, AppState>) -> Result<TelegramConfig, String> {
    let config = state
        .config
        .lock()
        .map_err(|e| format!("获取配置失败: {}", e))?;
    Ok(config.telegram_config.clone())
}

/// 设置Telegram配置
#[tauri::command]
pub async fn set_telegram_config(
    telegram_config: TelegramConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    {
        let mut config = state
            .config
            .lock()
            .map_err(|e| format!("获取配置失败: {}", e))?;
        config.telegram_config = telegram_config;
    }

    // 保存配置到文件
    save_config(&state, &app)
        .await
        .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok(())
}

/// 测试Telegram Bot连接
#[tauri::command]
pub async fn test_telegram_connection(
    bot_token: String,
    chat_id: String,
) -> Result<String, String> {
    if bot_token.trim().is_empty() {
        return Err("Bot Token不能为空".to_string());
    }

    if chat_id.trim().is_empty() {
        return Err("Chat ID不能为空".to_string());
    }

    // 创建Bot实例
    let bot = Bot::new(bot_token);

    // 验证Chat ID格式
    let chat_id_parsed: i64 = chat_id
        .parse()
        .map_err(|_| "Chat ID格式无效，请输入有效的数字ID".to_string())?;

    // 发送测试消息
    let test_message =
        "🤖 寸止应用测试消息\n\n这是一条来自寸止应用的测试消息，表示Telegram Bot配置成功！";

    match bot.send_message(ChatId(chat_id_parsed), test_message).await {
        Ok(_) => Ok("测试消息发送成功！Telegram Bot配置正确。".to_string()),
        Err(e) => {
            let error_msg = format!("发送测试消息失败: {}", e);
            Err(error_msg)
        }
    }
}

/// 发送Telegram消息（供其他模块调用）
pub async fn send_telegram_message(
    bot_token: &str,
    chat_id: &str,
    message: &str,
) -> Result<(), String> {
    send_telegram_message_with_markdown(bot_token, chat_id, message, false).await
}

/// 发送支持Markdown的Telegram消息
pub async fn send_telegram_message_with_markdown(
    bot_token: &str,
    chat_id: &str,
    message: &str,
    use_markdown: bool,
) -> Result<(), String> {
    if bot_token.trim().is_empty() || chat_id.trim().is_empty() {
        return Err("Bot Token或Chat ID未配置".to_string());
    }

    let bot = Bot::new(bot_token);
    let chat_id_parsed: i64 = chat_id.parse().map_err(|_| "Chat ID格式无效".to_string())?;

    let mut send_request = bot.send_message(ChatId(chat_id_parsed), message);

    // 如果启用Markdown，设置解析模式
    if use_markdown {
        send_request = send_request.parse_mode(ParseMode::MarkdownV2);
    }

    send_request
        .await
        .map_err(|e| format!("发送消息失败: {}", e))?;

    Ok(())
}

/// 启动Telegram同步（完整版本）
#[tauri::command]
pub async fn start_telegram_sync(
    message: String,
    predefined_options: Vec<String>,
    is_markdown: bool,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // 获取Telegram配置
    let (enabled, bot_token, chat_id, continue_reply_enabled) = {
        let config = state
            .config
            .lock()
            .map_err(|e| format!("获取配置失败: {}", e))?;
        (
            config.telegram_config.enabled,
            config.telegram_config.bot_token.clone(),
            config.telegram_config.chat_id.clone(),
            config.reply_config.enable_continue_reply,
        )
    };

    if !enabled {
        return Ok(());
    }

    if bot_token.trim().is_empty() || chat_id.trim().is_empty() {
        return Err("Telegram配置不完整".to_string());
    }

    // 发送消息一：选项消息（带inline keyboard）
    send_options_message(
        &bot_token,
        &chat_id,
        &message,
        &predefined_options,
        is_markdown,
    )
    .await?;

    // 短暂延迟确保消息顺序
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 发送消息二：操作消息（带reply keyboard）
    send_operation_message(&bot_token, &chat_id, continue_reply_enabled).await?;

    // 启动消息监听（如果有预定义选项）
    if !predefined_options.is_empty() {
        let bot_token_clone = bot_token.clone();
        let chat_id_clone = chat_id.clone();
        let app_handle_clone = app_handle.clone();

        tokio::spawn(async move {
            let _ = start_telegram_listener(bot_token_clone, chat_id_clone, app_handle_clone).await;
        });
    }

    Ok(())
}

/// 发送选项消息（消息一）
async fn send_options_message(
    bot_token: &str,
    chat_id: &str,
    message: &str,
    predefined_options: &[String],
    is_markdown: bool,
) -> Result<(), String> {
    let bot = Bot::new(bot_token);
    let chat_id_parsed: i64 = chat_id.parse().map_err(|_| "Chat ID格式无效".to_string())?;

    // 处理消息内容
    let processed_message = if is_markdown {
        process_telegram_markdown(message)
    } else {
        message.to_string()
    };

    // 创建inline keyboard
    let mut keyboard_rows = Vec::new();

    // 添加选项按钮（每行最多2个）
    for chunk in predefined_options.chunks(2) {
        let mut row = Vec::new();
        for option in chunk {
            let callback_data = format!("toggle:{}", option);
            row.push(InlineKeyboardButton::callback(
                format!("☐ {}", option),
                callback_data,
            ));
        }
        keyboard_rows.push(row);
    }

    let inline_keyboard = InlineKeyboardMarkup::new(keyboard_rows);

    // 发送消息
    let mut send_request = bot
        .send_message(ChatId(chat_id_parsed), processed_message)
        .reply_markup(inline_keyboard);

    // 如果是Markdown，设置解析模式
    if is_markdown {
        send_request = send_request.parse_mode(ParseMode::MarkdownV2);
    }

    match send_request.await {
        Ok(_) => {}
        Err(e) => {
            let error_str = e.to_string();
            // 检查是否是JSON解析错误但消息实际发送成功
            let has_parsing_json = error_str.contains("parsing JSON");
            let has_ok_true = error_str.contains("\\\"ok\\\":true");

            if has_parsing_json && has_ok_true {
                // 消息实际发送成功，继续执行
            } else {
                return Err(format!("发送选项消息失败: {}", e));
            }
        }
    }

    Ok(())
}

/// 发送操作消息（消息二）
async fn send_operation_message(
    bot_token: &str,
    chat_id: &str,
    continue_reply_enabled: bool,
) -> Result<(), String> {
    let bot = Bot::new(bot_token);
    let chat_id_parsed: i64 = chat_id.parse().map_err(|_| "Chat ID格式无效".to_string())?;

    // 创建reply keyboard
    let mut keyboard_buttons = vec![KeyboardButton::new("↗️发送")];

    if continue_reply_enabled {
        keyboard_buttons.insert(0, KeyboardButton::new("⏩继续"));
    }

    let reply_keyboard = KeyboardMarkup::new(vec![keyboard_buttons])
        .resize_keyboard(true)
        .one_time_keyboard(false);

    // 发送操作消息
    let operation_message = "键盘上选择操作完成对话";

    match bot
        .send_message(ChatId(chat_id_parsed), operation_message)
        .reply_markup(reply_keyboard)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            let error_str = e.to_string();
            // 检查是否是JSON解析错误但消息实际发送成功
            if error_str.contains("parsing JSON") && error_str.contains("\\\"ok\\\":true") {
                // 消息实际发送成功，继续执行
            } else {
                return Err(format!("发送操作消息失败: {}", e));
            }
        }
    }

    Ok(())
}

/// 启动Telegram消息监听
async fn start_telegram_listener(
    bot_token: String,
    chat_id: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let bot = Bot::new(bot_token);
    let chat_id_parsed: i64 = chat_id.parse().map_err(|_| "Chat ID格式无效".to_string())?;
    let target_chat_id = ChatId(chat_id_parsed);

    let mut offset = 0;
    let mut _operation_message_id: Option<i32> = None;

    // 获取当前最新的消息ID作为基准
    if let Ok(updates) = bot.get_updates().limit(10).await {
        if let Some(update) = updates.last() {
            offset = update.id + 1;
        }
    }

    // 监听循环
    loop {
        match bot.get_updates().offset(offset).timeout(10).await {
            Ok(updates) => {
                for update in updates {
                    offset = update.id + 1;

                    match &update.kind {
                        teloxide::types::UpdateKind::Message(_) => {
                            // 将在后面单独处理
                        }
                        teloxide::types::UpdateKind::CallbackQuery(_) => {
                            // 将在后面单独处理
                        }
                        teloxide::types::UpdateKind::InlineQuery(_) => {}
                        teloxide::types::UpdateKind::ChosenInlineResult(_) => {}
                        teloxide::types::UpdateKind::EditedMessage(_) => {}
                        teloxide::types::UpdateKind::ChannelPost(_) => {}
                        teloxide::types::UpdateKind::EditedChannelPost(_) => {}
                        teloxide::types::UpdateKind::ShippingQuery(_) => {}
                        teloxide::types::UpdateKind::PreCheckoutQuery(_) => {}
                        teloxide::types::UpdateKind::Poll(_) => {}
                        teloxide::types::UpdateKind::PollAnswer(_) => {}
                        teloxide::types::UpdateKind::MyChatMember(_) => {}
                        teloxide::types::UpdateKind::ChatMember(_) => {}
                        teloxide::types::UpdateKind::ChatJoinRequest(_) => {}
                        teloxide::types::UpdateKind::Error(err) => {
                            // 检查错误是否包含callback_query数据
                            let err_str = err.to_string();
                            if err_str.contains("callback_query") {
                                // 尝试从错误字符串中提取callback_query数据
                                if let Some(start) = err_str.find("\"data\":\"") {
                                    if let Some(end) = err_str[start + 8..].find("\"") {
                                        let callback_data = &err_str[start + 8..start + 8 + end];

                                        if callback_data.starts_with("toggle:") {
                                            let option = callback_data
                                                .strip_prefix("toggle:")
                                                .unwrap()
                                                .to_string();

                                            // 发送事件到前端
                                            let event = serde_json::json!({
                                                "type": "option_toggled",
                                                "option": option
                                            });

                                            let _ = app_handle.emit("telegram-event", &event);

                                            // 尝试提取callback query ID和消息ID
                                            if let Some(id_start) = err_str.find("\"id\":\"") {
                                                if let Some(id_end) =
                                                    err_str[id_start + 6..].find("\"")
                                                {
                                                    let callback_id = &err_str
                                                        [id_start + 6..id_start + 6 + id_end];

                                                    // 提取消息ID用于编辑
                                                    let message_id = if let Some(msg_start) =
                                                        err_str.find("\"message_id\":")
                                                    {
                                                        if let Some(msg_end) =
                                                            err_str[msg_start + 13..].find(",")
                                                        {
                                                            err_str[msg_start + 13
                                                                ..msg_start + 13 + msg_end]
                                                                .parse::<i32>()
                                                                .ok()
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        None
                                                    };

                                                    // 回答callback query并尝试更新按钮状态
                                                    let bot_clone = bot.clone();
                                                    let callback_id_clone = callback_id.to_string();
                                                    let _option_clone = option.clone();
                                                    let _chat_id_clone = target_chat_id;

                                                    tokio::spawn(async move {
                                                        // 回答callback query
                                                        let _ = bot_clone
                                                            .answer_callback_query(
                                                                callback_id_clone,
                                                            )
                                                            .await;

                                                        // 尝试更新按钮状态
                                                        if let Some(msg_id) = message_id {
                                                            // 从错误字符串中提取真实的选项列表
                                                            let mut real_options = Vec::new();

                                                            // 解析inline_keyboard中的所有选项
                                                            if let Some(keyboard_start) =
                                                                err_str.find("\"inline_keyboard\":")
                                                            {
                                                                let keyboard_section =
                                                                    &err_str[keyboard_start..];

                                                                // 查找所有callback_data中的toggle:选项
                                                                let mut pos = 0;
                                                                while let Some(toggle_pos) =
                                                                    keyboard_section[pos..]
                                                                        .find("toggle:")
                                                                {
                                                                    let start =
                                                                        pos + toggle_pos + 7; // "toggle:".len()
                                                                    if let Some(end) =
                                                                        keyboard_section[start..]
                                                                            .find("\"")
                                                                    {
                                                                        let option =
                                                                            keyboard_section[start
                                                                                ..start + end]
                                                                                .to_string();
                                                                        if !real_options
                                                                            .contains(&option)
                                                                        {
                                                                            real_options
                                                                                .push(option);
                                                                        }
                                                                    }
                                                                    pos = start;
                                                                }
                                                            }

                                                            // 如果没有找到选项，使用默认选项
                                                            if real_options.is_empty() {
                                                                real_options = vec![
                                                                    "只修改性能问题".to_string(),
                                                                    "需要更详细的说明".to_string(),
                                                                    "代码风格问题".to_string(),
                                                                    "安全性问题".to_string(),
                                                                    "功能性问题".to_string(),
                                                                ];
                                                            }

                                                            // 从错误字符串中解析当前选中的选项
                                                            let mut selected_options =
                                                                std::collections::HashSet::new();

                                                            // 查找所有已选中的按钮（包含☑️的）
                                                            for option in &real_options {
                                                                let selected_pattern =
                                                                    format!("☑️ {}", option);
                                                                if err_str
                                                                    .contains(&selected_pattern)
                                                                {
                                                                    selected_options
                                                                        .insert(option.clone());
                                                                }
                                                            }

                                                            // 切换当前点击的选项状态
                                                            if selected_options
                                                                .contains(&_option_clone)
                                                            {
                                                                selected_options
                                                                    .remove(&_option_clone);
                                                            } else {
                                                                selected_options
                                                                    .insert(_option_clone.clone());
                                                            }

                                                            // 创建更新后的keyboard
                                                            let mut keyboard_rows = Vec::new();
                                                            for chunk in real_options.chunks(2) {
                                                                let mut row = Vec::new();
                                                                for option in chunk {
                                                                    let callback_data = format!(
                                                                        "toggle:{}",
                                                                        option
                                                                    );
                                                                    // 根据选中状态显示按钮
                                                                    let button_text =
                                                                        if selected_options
                                                                            .contains(option)
                                                                        {
                                                                            format!("☑️ {}", option)
                                                                        } else {
                                                                            format!("☐ {}", option)
                                                                        };
                                                                    row.push(InlineKeyboardButton::callback(
                                                                        button_text,
                                                                        callback_data,
                                                                    ));
                                                                }
                                                                keyboard_rows.push(row);
                                                            }

                                                            let new_keyboard =
                                                                InlineKeyboardMarkup::new(
                                                                    keyboard_rows,
                                                                );

                                                            // 更新消息的reply_markup
                                                            let _ = bot_clone
                                                                .edit_message_reply_markup(
                                                                    _chat_id_clone,
                                                                    MessageId(msg_id),
                                                                )
                                                                .reply_markup(new_keyboard)
                                                                .await;
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 处理callback query（inline keyboard点击）
                    if let teloxide::types::UpdateKind::CallbackQuery(callback_query) = &update.kind
                    {
                        if let Some(message) = &callback_query.message {
                            if message.chat.id == target_chat_id {
                                handle_callback_query(&bot, callback_query.clone(), &app_handle)
                                    .await;
                            }
                        }
                    }

                    // 处理文本消息
                    if let teloxide::types::UpdateKind::Message(message) = &update.kind {
                        if message.chat.id == target_chat_id {
                            // 检查是否是操作消息（包含"键盘上选择操作完成对话"）
                            if let Some(text) = message.text() {
                                if text == "键盘上选择操作完成对话" {
                                    _operation_message_id = Some(message.id.0);
                                    continue;
                                }
                            }

                            // 处理用户消息
                            handle_text_message(&message, &app_handle).await;
                        }
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

/// 处理callback query（inline keyboard点击）
async fn handle_callback_query(bot: &Bot, callback_query: CallbackQuery, app_handle: &AppHandle) {
    if let Some(data) = callback_query.data {
        if data.starts_with("toggle:") {
            let option = data.strip_prefix("toggle:").unwrap().to_string();

            // 发送事件到前端
            let event = serde_json::json!({
                "type": "option_toggled",
                "option": option
            });

            let _ = app_handle.emit("telegram-event", &event);
        }
    }

    // 回答callback query
    let _ = bot.answer_callback_query(callback_query.id).await;
}

/// 处理文本消息
async fn handle_text_message(message: &teloxide::types::Message, app_handle: &AppHandle) {
    if let Some(text) = message.text() {
        match text {
            "⏩继续" => {
                let event = serde_json::json!({
                    "type": "continue_pressed"
                });
                let _ = app_handle.emit("telegram-event", &event);
            }
            "↗️发送" => {
                let event = serde_json::json!({
                    "type": "send_pressed"
                });
                let _ = app_handle.emit("telegram-event", &event);
            }
            _ => {
                // 普通文本输入
                let event = serde_json::json!({
                    "type": "text_updated",
                    "text": text
                });
                let _ = app_handle.emit("telegram-event", &event);
            }
        }
    }
}
