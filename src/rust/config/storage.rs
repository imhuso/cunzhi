use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, LogicalSize, Manager, State};

use super::settings::{AppConfig, AppState, default_shortcuts};

pub fn get_config_path(_app: &AppHandle) -> Result<PathBuf> {
    // 使用与独立配置相同的路径，确保一致性
    get_standalone_config_path()
}

pub async fn save_config(state: &State<'_, AppState>, app: &AppHandle) -> Result<()> {
    let config_path = get_config_path(app)?;

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config = state
        .config
        .lock()
        .map_err(|e| anyhow::anyhow!("获取配置失败: {}", e))?;
    let config_json = serde_json::to_string_pretty(&*config)?;

    // 写入文件
    fs::write(&config_path, config_json)?;

    // 强制刷新文件系统缓存
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&config_path) {
        let _ = file.sync_all();
    }

    log::debug!("配置已保存到: {:?}", config_path);

    Ok(())
}

/// Tauri应用专用的配置加载函数
pub async fn load_config(state: &State<'_, AppState>, app: &AppHandle) -> Result<()> {
    let config_path = get_config_path(app)?;

    if config_path.exists() {
        let config_json = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = serde_json::from_str(&config_json)?;

        // 合并默认快捷键配置，确保新的默认快捷键被添加
        merge_default_shortcuts(&mut config);

        // 迁移旧版本 Telegram 配置
        let needs_save = config.telegram_config.bot_token.is_some() ||
                        config.telegram_config.chat_id.is_some();
        config.telegram_config.migrate_from_legacy();

        {
            let mut config_guard = state
                .config
                .lock()
                .map_err(|e| anyhow::anyhow!("获取配置锁失败: {}", e))?;
            *config_guard = config;
        } // config_guard 在这里被释放

        // 如果进行了迁移，保存配置
        if needs_save {
            save_config(state, app).await?;
        }
    }

    Ok(())
}

pub async fn load_config_and_apply_window_settings(
    state: &State<'_, AppState>,
    app: &AppHandle,
) -> Result<()> {
    // 先加载配置
    load_config(state, app).await?;

    // 然后应用窗口设置
    let (always_on_top, window_config) = {
        let config = state
            .config
            .lock()
            .map_err(|e| anyhow::anyhow!("获取配置失败: {}", e))?;
        (
            config.ui_config.always_on_top,
            config.ui_config.window_config.clone(),
        )
    };

    // 应用到窗口
    if let Some(window) = app.get_webview_window("main") {
        // 应用置顶设置
        if let Err(e) = window.set_always_on_top(always_on_top) {
            log::warn!("设置窗口置顶失败: {}", e);
        } else {
            log::info!("窗口置顶状态已设置为: {} (配置加载时)", always_on_top);
        }

        // 应用窗口大小约束
        if let Err(e) = window.set_min_size(Some(LogicalSize::new(
            window_config.min_width,
            window_config.min_height,
        ))) {
            log::warn!("设置最小窗口大小失败: {}", e);
        }

        if let Err(e) = window.set_max_size(Some(LogicalSize::new(
            window_config.max_width,
            window_config.max_height,
        ))) {
            log::warn!("设置最大窗口大小失败: {}", e);
        }

        // 根据当前模式设置窗口大小
        let (target_width, target_height) = if window_config.fixed {
            // 固定模式：使用固定尺寸
            (window_config.fixed_width, window_config.fixed_height)
        } else {
            // 自由拉伸模式：使用自由拉伸尺寸
            (window_config.free_width, window_config.free_height)
        };

        // 应用窗口大小（移除调试信息）
        if let Err(_e) = window.set_size(LogicalSize::new(target_width, target_height)) {
            // 静默处理窗口大小设置失败
        }
    }

    Ok(())
}

/// 独立加载配置文件（用于MCP服务器等独立进程）
pub fn load_standalone_config() -> Result<AppConfig> {
    let config_path = get_standalone_config_path()?;

    if config_path.exists() {
        let config_json = fs::read_to_string(config_path)?;
        let mut config: AppConfig = serde_json::from_str(&config_json)?;

        // 合并默认快捷键配置
        merge_default_shortcuts(&mut config);

        // 迁移旧版本 Telegram 配置
        config.telegram_config.migrate_from_legacy();

        Ok(config)
    } else {
        // 如果配置文件不存在，返回默认配置
        Ok(AppConfig::default())
    }
}

/// 独立保存配置文件（用于MCP服务器等独立进程）
pub fn save_standalone_config(config: &AppConfig) -> Result<()> {
    let config_path = get_standalone_config_path()?;

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_json = serde_json::to_string_pretty(config)?;

    // 写入文件
    fs::write(&config_path, config_json)?;

    // 强制刷新文件系统缓存
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&config_path) {
        let _ = file.sync_all();
    }

    log::debug!("配置已保存到: {:?}", config_path);

    Ok(())
}

/// 独立加载Telegram配置（用于MCP模式下的配置检查）
pub fn load_standalone_telegram_config() -> Result<super::settings::TelegramConfig> {
    let config = load_standalone_config()?;
    Ok(config.telegram_config)
}

/// 获取独立配置文件路径（不依赖Tauri）
fn get_standalone_config_path() -> Result<PathBuf> {
    // 使用标准的配置目录
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
        .join("cunzhi");

    // 确保目录存在
    fs::create_dir_all(&config_dir)?;

    Ok(config_dir.join("config.json"))
}

/// 合并默认快捷键配置，确保新的默认快捷键被添加到现有配置中
fn merge_default_shortcuts(config: &mut AppConfig) {
    let default_shortcuts = default_shortcuts();

    // 遍历所有默认快捷键
    for (key, default_binding) in default_shortcuts {
        if !config.shortcut_config.shortcuts.contains_key(&key) {
            // 如果用户配置中不存在，则添加
            config.shortcut_config.shortcuts.insert(key, default_binding);
        } else if key == "enhance" {
            // 特殊处理：更新增强快捷键的默认值从 Shift+Enter 到 Ctrl+Shift+Enter
            let existing_binding = config.shortcut_config.shortcuts.get(&key).unwrap();

            // 检查是否是旧的默认值 (Shift+Enter)
            if existing_binding.key_combination.key == "Enter"
                && !existing_binding.key_combination.ctrl
                && existing_binding.key_combination.shift
                && !existing_binding.key_combination.alt
                && !existing_binding.key_combination.meta {
                // 更新为新的默认值 (Ctrl+Shift+Enter)
                config.shortcut_config.shortcuts.insert(key, default_binding);
            }
        }
    }
}
