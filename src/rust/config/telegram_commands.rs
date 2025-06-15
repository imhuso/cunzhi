use crate::config::{save_config, AppState, TelegramConfig};
use tauri::{AppHandle, State};
use teloxide::{prelude::*, Bot};

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
    if bot_token.trim().is_empty() || chat_id.trim().is_empty() {
        return Err("Bot Token或Chat ID未配置".to_string());
    }

    let bot = Bot::new(bot_token);
    let chat_id_parsed: i64 = chat_id.parse().map_err(|_| "Chat ID格式无效".to_string())?;

    bot.send_message(ChatId(chat_id_parsed), message)
        .await
        .map_err(|e| format!("发送消息失败: {}", e))?;

    Ok(())
}
