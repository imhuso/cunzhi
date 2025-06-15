use crate::config::{save_config, AppState, TelegramConfig};
use crate::telegram::{
    handle_callback_query, handle_text_message, test_telegram_connection, TelegramCore,
};
use tauri::{AppHandle, Emitter, State};
use teloxide::prelude::*;

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
pub async fn test_telegram_connection_cmd(
    bot_token: String,
    chat_id: String,
) -> Result<String, String> {
    test_telegram_connection(&bot_token, &chat_id)
        .await
        .map_err(|e| e.to_string())
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
    let core =
        TelegramCore::new(bot_token.to_string(), chat_id.to_string()).map_err(|e| e.to_string())?;

    core.send_message_with_markdown(message, use_markdown)
        .await
        .map_err(|e| e.to_string())
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

    // 创建Telegram核心实例
    let core = TelegramCore::new(bot_token.clone(), chat_id.clone())
        .map_err(|e| format!("创建Telegram核心失败: {}", e))?;

    // 发送选项消息
    core.send_options_message(&message, &predefined_options, is_markdown)
        .await
        .map_err(|e| format!("发送选项消息失败: {}", e))?;

    // 短暂延迟确保消息顺序
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 发送操作消息
    core.send_operation_message(continue_reply_enabled)
        .await
        .map_err(|e| format!("发送操作消息失败: {}", e))?;

    // 启动消息监听（如果有预定义选项）
    if !predefined_options.is_empty() {
        println!(
            "🤖 [Telegram] 启动消息监听，选项数量: {}",
            predefined_options.len()
        );
        let bot_token_clone = bot_token.clone();
        let chat_id_clone = chat_id.clone();
        let app_handle_clone = app_handle.clone();

        tokio::spawn(async move {
            println!("🤖 [Telegram] 消息监听任务已启动");
            match start_telegram_listener(
                bot_token_clone,
                chat_id_clone,
                app_handle_clone,
                predefined_options,
            )
            .await
            {
                Ok(_) => println!("🤖 [Telegram] 消息监听正常结束"),
                Err(e) => println!("🤖 [Telegram] 消息监听出错: {}", e),
            }
        });
    } else {
        println!("🤖 [Telegram] 没有预定义选项，跳过消息监听启动");
    }

    Ok(())
}

/// 启动Telegram消息监听（简化版本）
async fn start_telegram_listener(
    bot_token: String,
    chat_id: String,
    app_handle: AppHandle,
    predefined_options_list: Vec<String>,
) -> Result<(), String> {
    println!("🤖 [Telegram] 创建监听器，Chat ID: {}", chat_id);

    let core = TelegramCore::new(bot_token, chat_id)
        .map_err(|e| format!("创建Telegram核心失败: {}", e))?;

    let mut offset = 0;

    // 用于跟踪选项状态和消息ID
    let mut selected_options: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut options_message_id: Option<i32> = None;
    let predefined_options = predefined_options_list;

    // 获取当前最新的消息ID作为基准
    if let Ok(updates) = core.bot.get_updates().limit(10).await {
        if let Some(update) = updates.last() {
            offset = update.id + 1;
            println!("🤖 [Telegram] 设置起始偏移量: {}", offset);
        }
    }

    // 监听循环
    println!("🤖 [Telegram] 开始监听循环");
    loop {
        match core.bot.get_updates().offset(offset).timeout(10).await {
            Ok(updates) => {
                if !updates.is_empty() {
                    println!("🤖 [Telegram] 简化监听器收到 {} 个更新", updates.len());
                }

                for update in updates {
                    offset = update.id + 1;
                    println!("🤖 [Telegram] 简化监听器处理更新 ID: {}", update.id);

                    match update.kind {
                        teloxide::types::UpdateKind::CallbackQuery(callback_query) => {
                            println!(
                                "🤖 [Telegram] 简化监听器收到 CallbackQuery: {:?}",
                                callback_query.data
                            );

                            if let Ok(Some(option)) =
                                handle_callback_query(&core.bot, &callback_query, core.chat_id)
                                    .await
                            {
                                // 切换选项状态
                                let selected = if selected_options.contains(&option) {
                                    selected_options.remove(&option);
                                    false
                                } else {
                                    selected_options.insert(option.clone());
                                    true
                                };

                                println!(
                                    "🤖 [Telegram] 选项 '{}' 状态切换为: {}",
                                    option, selected
                                );

                                // 发送事件到前端
                                use crate::telegram::TelegramEvent;
                                let event = TelegramEvent::OptionToggled {
                                    option: option.clone(),
                                    selected,
                                };

                                println!("🤖 [Telegram] 简化监听器发送事件: {:?}", event);
                                match app_handle.emit("telegram-event", &event) {
                                    Ok(_) => println!("🤖 [Telegram] ✅ 简化监听器事件发送成功"),
                                    Err(e) => {
                                        println!("🤖 [Telegram] ❌ 简化监听器事件发送失败: {}", e)
                                    }
                                }

                                // 更新按钮状态
                                if let Some(msg_id) = options_message_id {
                                    let selected_vec: Vec<String> =
                                        selected_options.iter().cloned().collect();
                                    match core
                                        .update_inline_keyboard(
                                            msg_id,
                                            &predefined_options,
                                            &selected_vec,
                                        )
                                        .await
                                    {
                                        Ok(_) => println!("🤖 [Telegram] ✅ 按钮状态更新成功"),
                                        Err(e) => {
                                            println!("🤖 [Telegram] ⚠️ 按钮状态更新失败: {}", e)
                                        }
                                    }
                                } else {
                                    println!("🤖 [Telegram] ⚠️ 未找到选项消息ID，无法更新按钮状态");
                                }
                            } else {
                                println!("🤖 [Telegram] CallbackQuery 处理返回 None 或失败");
                            }
                        }
                        teloxide::types::UpdateKind::Message(message) => {
                            println!(
                                "🤖 [Telegram] 简化监听器收到消息: {:?} 来自聊天: {}",
                                message.text(),
                                message.chat.id
                            );

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
                                    println!("🤖 [Telegram] 检测到选项消息，ID: {}", message.id.0);
                                }
                            }

                            if let Ok(Some(event)) = handle_text_message(
                                &message,
                                core.chat_id,
                                None, // 简化版本不过滤消息ID
                            )
                            .await
                            {
                                println!("🤖 [Telegram] 简化监听器文本处理成功: {:?}", event);
                                match app_handle.emit("telegram-event", &event) {
                                    Ok(_) => {
                                        println!("🤖 [Telegram] ✅ 简化监听器文本事件发送成功")
                                    }
                                    Err(e) => println!(
                                        "🤖 [Telegram] ❌ 简化监听器文本事件发送失败: {}",
                                        e
                                    ),
                                }
                            }
                        }
                        teloxide::types::UpdateKind::InlineQuery(inline_query) => {
                            println!("🤖 [Telegram] 收到 InlineQuery: {:?}", inline_query.query);
                        }
                        teloxide::types::UpdateKind::ChosenInlineResult(chosen_result) => {
                            println!(
                                "🤖 [Telegram] 收到 ChosenInlineResult: {:?}",
                                chosen_result.result_id
                            );
                        }
                        teloxide::types::UpdateKind::EditedMessage(edited_message) => {
                            println!(
                                "🤖 [Telegram] 收到 EditedMessage: {:?}",
                                edited_message.text()
                            );
                        }
                        teloxide::types::UpdateKind::ChannelPost(channel_post) => {
                            println!("🤖 [Telegram] 收到 ChannelPost: {:?}", channel_post.text());
                        }
                        teloxide::types::UpdateKind::EditedChannelPost(edited_channel_post) => {
                            println!(
                                "🤖 [Telegram] 收到 EditedChannelPost: {:?}",
                                edited_channel_post.text()
                            );
                        }
                        teloxide::types::UpdateKind::ShippingQuery(shipping_query) => {
                            println!("🤖 [Telegram] 收到 ShippingQuery: {:?}", shipping_query.id);
                        }
                        teloxide::types::UpdateKind::PreCheckoutQuery(pre_checkout_query) => {
                            println!(
                                "🤖 [Telegram] 收到 PreCheckoutQuery: {:?}",
                                pre_checkout_query.id
                            );
                        }
                        teloxide::types::UpdateKind::Poll(poll) => {
                            println!("🤖 [Telegram] 收到 Poll: {:?}", poll.id);
                        }
                        teloxide::types::UpdateKind::PollAnswer(poll_answer) => {
                            println!("🤖 [Telegram] 收到 PollAnswer: {:?}", poll_answer.poll_id);
                        }
                        teloxide::types::UpdateKind::MyChatMember(my_chat_member) => {
                            println!(
                                "🤖 [Telegram] 收到 MyChatMember: {:?}",
                                my_chat_member.chat.id
                            );
                        }
                        teloxide::types::UpdateKind::ChatMember(chat_member) => {
                            println!("🤖 [Telegram] 收到 ChatMember: {:?}", chat_member.chat.id);
                        }
                        teloxide::types::UpdateKind::ChatJoinRequest(chat_join_request) => {
                            println!(
                                "🤖 [Telegram] 收到 ChatJoinRequest: {:?}",
                                chat_join_request.chat.id
                            );
                        }
                        teloxide::types::UpdateKind::Error(error) => {
                            println!("🤖 [Telegram] 收到 Error: {:?}", error);

                            // 尝试从错误中提取 callback_query 数据
                            let error_str = error.to_string();
                            if error_str.contains("callback_query") {
                                println!(
                                    "🤖 [Telegram] 在 Error 中发现 callback_query 数据，尝试处理"
                                );

                                // 提取 callback_query 的 data 字段
                                if let Some(start) = error_str.find("\"data\":\"") {
                                    if let Some(end) = error_str[start + 8..].find("\"") {
                                        let callback_data = &error_str[start + 8..start + 8 + end];
                                        println!(
                                            "🤖 [Telegram] 提取到 callback_data: {}",
                                            callback_data
                                        );

                                        if callback_data.starts_with("toggle:") {
                                            let option = callback_data
                                                .strip_prefix("toggle:")
                                                .unwrap()
                                                .to_string();

                                            println!(
                                                "🤖 [Telegram] 从 Error 中提取到选项: {}",
                                                option
                                            );

                                            // 切换选项状态
                                            let selected = if selected_options.contains(&option) {
                                                selected_options.remove(&option);
                                                false
                                            } else {
                                                selected_options.insert(option.clone());
                                                true
                                            };

                                            println!(
                                                "🤖 [Telegram] 从 Error 中选项 '{}' 状态切换为: {}",
                                                option, selected
                                            );

                                            // 发送选项切换事件
                                            use crate::telegram::TelegramEvent;
                                            let event = TelegramEvent::OptionToggled {
                                                option: option.clone(),
                                                selected,
                                            };

                                            println!(
                                                "🤖 [Telegram] 从 Error 中发送事件: {:?}",
                                                event
                                            );
                                            match app_handle.emit("telegram-event", &event) {
                                                Ok(_) => println!(
                                                    "🤖 [Telegram] ✅ Error 中的事件发送成功"
                                                ),
                                                Err(e) => println!(
                                                    "🤖 [Telegram] ❌ Error 中的事件发送失败: {}",
                                                    e
                                                ),
                                            }

                                            // 更新按钮状态
                                            if let Some(msg_id) = options_message_id {
                                                let selected_vec: Vec<String> =
                                                    selected_options.iter().cloned().collect();
                                                match core.update_inline_keyboard(msg_id, &predefined_options, &selected_vec).await {
                                                    Ok(_) => println!("🤖 [Telegram] ✅ 从 Error 中按钮状态更新成功"),
                                                    Err(e) => println!("🤖 [Telegram] ⚠️ 从 Error 中按钮状态更新失败: {}", e),
                                                }
                                            } else {
                                                println!("🤖 [Telegram] ⚠️ 从 Error 中未找到选项消息ID，无法更新按钮状态");
                                            }
                                        }
                                    }
                                }

                                // 尝试从 callback_query 中提取消息ID
                                if options_message_id.is_none() {
                                    if let Some(msg_start) = error_str.find("\"message_id\":") {
                                        // 找到数字部分
                                        let after_colon = &error_str[msg_start + 13..]; // "message_id":".len() = 13
                                        if let Some(number_start) =
                                            after_colon.find(char::is_numeric)
                                        {
                                            if let Some(number_end) = after_colon[number_start..]
                                                .find(|c: char| !c.is_numeric())
                                            {
                                                let number_str = &after_colon
                                                    [number_start..number_start + number_end];
                                                if let Ok(msg_id) = number_str.parse::<i32>() {
                                                    options_message_id = Some(msg_id);
                                                    println!(
                                                        "🤖 [Telegram] 从 Error 中提取到消息ID: {}",
                                                        msg_id
                                                    );

                                                    // 立即更新按钮状态
                                                    let selected_vec: Vec<String> =
                                                        selected_options.iter().cloned().collect();
                                                    if let Err(e) = core
                                                        .update_inline_keyboard(
                                                            msg_id,
                                                            &predefined_options,
                                                            &selected_vec,
                                                        )
                                                        .await
                                                    {
                                                        println!("🤖 [Telegram] ❌ 从 Error 中更新按钮状态失败: {}", e);
                                                    } else {
                                                        println!("🤖 [Telegram] ✅ 从 Error 中更新按钮状态成功");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // 尝试提取 callback query ID 并回答
                                if let Some(id_start) = error_str.find("\"id\":\"") {
                                    if let Some(id_end) = error_str[id_start + 6..].find("\"") {
                                        let callback_id =
                                            &error_str[id_start + 6..id_start + 6 + id_end];
                                        println!(
                                            "🤖 [Telegram] 提取到 callback_id: {}",
                                            callback_id
                                        );

                                        // 异步回答 callback query
                                        let bot_clone = core.bot.clone();
                                        let callback_id_clone = callback_id.to_string();
                                        tokio::spawn(async move {
                                            match bot_clone.answer_callback_query(callback_id_clone).await {
                                                Ok(_) => println!("🤖 [Telegram] ✅ 从 Error 中回答 callback query 成功"),
                                                Err(e) => println!("🤖 [Telegram] ❌ 从 Error 中回答 callback query 失败: {}", e),
                                            }
                                        });
                                    }
                                }
                            }
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
