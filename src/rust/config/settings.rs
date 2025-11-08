use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::constants::{window, theme, audio, mcp, telegram, font};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_ui_config")]
    pub ui_config: UiConfig, // UI相关配置（主题、窗口、置顶等）
    #[serde(default = "default_audio_config")]
    pub audio_config: AudioConfig, // 音频相关配置
    #[serde(default = "default_reply_config")]
    pub reply_config: ReplyConfig, // 继续回复配置
    #[serde(default = "default_mcp_config")]
    pub mcp_config: McpConfig, // MCP工具配置
    #[serde(default = "default_telegram_config")]
    pub telegram_config: TelegramConfig, // Telegram Bot配置
    #[serde(default = "default_custom_prompt_config")]
    pub custom_prompt_config: CustomPromptConfig, // 自定义prompt配置
    #[serde(default = "default_shortcut_config")]
    pub shortcut_config: ShortcutConfig, // 自定义快捷键配置
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    // 主题设置
    #[serde(default = "default_theme")]
    pub theme: String, // "light", "dark"

    // 字体设置
    #[serde(default = "default_font_config")]
    pub font_config: FontConfig,

    // 窗口设置
    #[serde(default = "default_window_config")]
    pub window_config: WindowConfig,

    // 置顶设置
    #[serde(default = "default_always_on_top")]
    pub always_on_top: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FontConfig {
    // 字体系列
    #[serde(default = "default_font_family")]
    pub font_family: String, // "inter", "jetbrains-mono", "system", "custom"

    // 字体大小
    #[serde(default = "default_font_size")]
    pub font_size: String, // "small", "medium", "large"

    // 自定义字体系列（当 font_family 为 "custom" 时使用）
    #[serde(default = "default_custom_font_family")]
    pub custom_font_family: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    // 窗口约束设置
    #[serde(default = "default_auto_resize")]
    pub auto_resize: bool,
    #[serde(default = "default_max_width")]
    pub max_width: f64,
    #[serde(default = "default_max_height")]
    pub max_height: f64,
    #[serde(default = "default_min_width")]
    pub min_width: f64,
    #[serde(default = "default_min_height")]
    pub min_height: f64,

    // 当前模式
    #[serde(default = "default_window_fixed")]
    pub fixed: bool,

    // 固定模式的尺寸设置
    #[serde(default = "default_fixed_width")]
    pub fixed_width: f64,
    #[serde(default = "default_fixed_height")]
    pub fixed_height: f64,

    // 自由拉伸模式的尺寸设置
    #[serde(default = "default_free_width")]
    pub free_width: f64,
    #[serde(default = "default_free_height")]
    pub free_height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    #[serde(default = "default_audio_notification_enabled")]
    pub notification_enabled: bool,
    #[serde(default = "default_audio_url")]
    pub custom_url: String, // 自定义音效URL
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyConfig {
    #[serde(default = "default_enable_continue_reply")]
    pub enable_continue_reply: bool,
    #[serde(default = "default_auto_continue_threshold")]
    pub auto_continue_threshold: u32, // 字符数阈值
    #[serde(default = "default_continue_prompt")]
    pub continue_prompt: String, // 继续回复的提示词
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpConfig {
    #[serde(default = "default_mcp_tools")]
    pub tools: HashMap<String, bool>, // MCP工具启用状态
    pub acemcp_base_url: Option<String>, // acemcp API端点URL
    pub acemcp_token: Option<String>, // acemcp认证令牌
    pub acemcp_batch_size: Option<u32>, // acemcp批处理大小
    pub acemcp_max_lines_per_blob: Option<u32>, // acemcp最大行数/块
    pub acemcp_text_extensions: Option<Vec<String>>, // acemcp文件扩展名
    pub acemcp_exclude_patterns: Option<Vec<String>>, // acemcp排除模式
}

// 自定义prompt结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomPrompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default = "default_prompt_type")]
    pub r#type: String, // "normal" | "conditional"
    // 条件性prompt专用字段
    pub condition_text: Option<String>,    // 条件描述文本
    pub template_true: Option<String>,     // 开关为true时的模板
    pub template_false: Option<String>,    // 开关为false时的模板
    #[serde(default = "default_prompt_state")]
    pub current_state: bool,               // 当前开关状态（原default_state）
}

// 自定义prompt配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomPromptConfig {
    #[serde(default = "default_custom_prompts")]
    pub prompts: Vec<CustomPrompt>,
    #[serde(default = "default_custom_prompt_enabled")]
    pub enabled: bool,
    #[serde(default = "default_custom_prompt_max_prompts")]
    pub max_prompts: u32,
}

// 快捷键配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutConfig {
    #[serde(default = "default_shortcuts")]
    pub shortcuts: HashMap<String, ShortcutBinding>,
}

// 快捷键绑定
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub action: String, // "submit", "exit", "custom"
    pub key_combination: ShortcutKey,
    pub enabled: bool,
    pub scope: String, // "global", "popup", "input"
}

// 快捷键组合
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutKey {
    pub key: String, // 主键，如 "Enter", "Q", "F4"
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // macOS的Cmd键
}

// 单个 Telegram Bot 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegramBotConfig {
    pub name: String, // Bot 名称/标识
    pub bot_token: String, // Bot Token
    pub chat_id: String, // Chat ID
    #[serde(default = "default_telegram_api_base_url")]
    pub api_base_url: String, // Telegram API基础URL
}

// Telegram 总配置（支持多个 Bot）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegramConfig {
    #[serde(default = "default_telegram_enabled")]
    pub enabled: bool, // 是否启用Telegram Bot
    #[serde(default = "default_telegram_hide_frontend_popup")]
    pub hide_frontend_popup: bool, // 是否隐藏前端弹窗，仅使用Telegram交互
    #[serde(default = "default_telegram_bots")]
    pub bots: Vec<TelegramBotConfig>, // Bot 列表
    #[serde(default = "default_telegram_default_bot")]
    pub default_bot: String, // 默认使用的 Bot 名称
    #[serde(default = "default_telegram_session_bot_mapping")]
    pub session_bot_mapping: HashMap<String, String>, // 会话 ID -> Bot 名称的映射
    #[serde(default = "default_telegram_pending_sessions")]
    pub pending_sessions: Vec<PendingSession>, // 待配置的会话列表

    // 旧版本兼容字段（用于自动迁移）
    #[serde(skip_serializing, default)]
    pub bot_token: Option<String>,
    #[serde(skip_serializing, default)]
    pub chat_id: Option<String>,
    #[serde(skip_serializing, default)]
    pub api_base_url: Option<String>,
}

// 待配置的会话信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingSession {
    pub session_id: String,
    pub first_seen: String, // ISO 8601 时间戳
    pub last_seen: String,  // ISO 8601 时间戳
    pub request_count: u32, // 请求次数
}

impl TelegramConfig {
    /// 获取指定名称的 Bot 配置
    pub fn get_bot(&self, name: &str) -> Option<&TelegramBotConfig> {
        self.bots.iter().find(|bot| bot.name == name)
    }

    /// 获取默认 Bot 配置
    pub fn get_default_bot(&self) -> Option<&TelegramBotConfig> {
        if self.default_bot.is_empty() {
            self.bots.first()
        } else {
            self.get_bot(&self.default_bot)
        }
    }

    /// 根据会话 ID 获取对应的 Bot 配置
    /// 如果会话有映射，使用映射的 bot；否则使用默认 bot
    pub fn get_bot_for_session(&self, session_id: Option<&str>) -> Option<&TelegramBotConfig> {
        if let Some(sid) = session_id {
            if let Some(bot_name) = self.session_bot_mapping.get(sid) {
                if let Some(bot) = self.get_bot(bot_name) {
                    return Some(bot);
                }
            }
        }
        self.get_default_bot()
    }

    /// 设置会话到 Bot 的映射
    pub fn set_session_bot_mapping(&mut self, session_id: String, bot_name: String) {
        self.session_bot_mapping.insert(session_id, bot_name);
    }

    /// 删除会话到 Bot 的映射
    pub fn remove_session_bot_mapping(&mut self, session_id: &str) -> bool {
        self.session_bot_mapping.remove(session_id).is_some()
    }

    /// 记录新的会话请求
    pub fn record_session_request(&mut self, session_id: &str) {
        // 如果已经有映射，不需要记录
        if self.session_bot_mapping.contains_key(session_id) {
            return;
        }

        // 查找是否已经在待配置列表中
        if let Some(pending) = self.pending_sessions.iter_mut().find(|p| p.session_id == session_id) {
            // 更新最后访问时间和请求次数
            pending.last_seen = chrono::Utc::now().to_rfc3339();
            pending.request_count += 1;
        } else {
            // 添加新的待配置会话
            let now = chrono::Utc::now().to_rfc3339();
            self.pending_sessions.push(PendingSession {
                session_id: session_id.to_string(),
                first_seen: now.clone(),
                last_seen: now,
                request_count: 1,
            });
        }
    }

    /// 移除待配置会话（当配置完成后）
    pub fn remove_pending_session(&mut self, session_id: &str) {
        self.pending_sessions.retain(|p| p.session_id != session_id);
    }

    /// 添加 Bot 配置
    pub fn add_bot(&mut self, bot: TelegramBotConfig) {
        // 如果是第一个 bot，设置为默认
        if self.bots.is_empty() {
            self.default_bot = bot.name.clone();
        }
        self.bots.push(bot);
    }

    /// 删除 Bot 配置
    pub fn remove_bot(&mut self, name: &str) -> bool {
        if let Some(pos) = self.bots.iter().position(|bot| bot.name == name) {
            self.bots.remove(pos);
            // 如果删除的是默认 bot，重新设置默认
            if self.default_bot == name {
                self.default_bot = self.bots.first()
                    .map(|bot| bot.name.clone())
                    .unwrap_or_default();
            }
            true
        } else {
            false
        }
    }

    /// 从旧版本配置迁移
    pub fn migrate_from_legacy(&mut self) {
        // 如果存在旧版本字段且 bots 为空，则迁移
        if self.bots.is_empty() {
            if let (Some(token), Some(chat_id)) = (&self.bot_token, &self.chat_id) {
                if !token.is_empty() && !chat_id.is_empty() {
                    let legacy_bot = TelegramBotConfig {
                        name: "默认Bot".to_string(),
                        bot_token: token.clone(),
                        chat_id: chat_id.clone(),
                        api_base_url: self.api_base_url.clone()
                            .unwrap_or_else(|| telegram::API_BASE_URL.to_string()),
                    };
                    self.add_bot(legacy_bot);
                }
            }
        }
        // 清除旧字段
        self.bot_token = None;
        self.chat_id = None;
        self.api_base_url = None;
    }
}

#[derive(Debug)]
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub response_channel: Mutex<Option<tokio::sync::oneshot::Sender<String>>>,
    // 防误触退出机制
    pub exit_attempt_count: Mutex<u32>,
    pub last_exit_attempt: Mutex<Option<std::time::Instant>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui_config: default_ui_config(),
            audio_config: default_audio_config(),
            reply_config: default_reply_config(),
            mcp_config: default_mcp_config(),
            telegram_config: default_telegram_config(),
            custom_prompt_config: default_custom_prompt_config(),
            shortcut_config: default_shortcut_config(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(AppConfig::default()),
            response_channel: Mutex::new(None),
            exit_attempt_count: Mutex::new(0),
            last_exit_attempt: Mutex::new(None),
        }
    }
}

// 默认值函数
pub fn default_ui_config() -> UiConfig {
    UiConfig {
        theme: default_theme(),
        font_config: default_font_config(),
        window_config: default_window_config(),
        always_on_top: default_always_on_top(),
    }
}

pub fn default_audio_config() -> AudioConfig {
    AudioConfig {
        notification_enabled: default_audio_notification_enabled(),
        custom_url: default_audio_url(),
    }
}

pub fn default_mcp_config() -> McpConfig {
    McpConfig {
        tools: default_mcp_tools(),
        acemcp_base_url: None,
        acemcp_token: None,
        acemcp_batch_size: None,
        acemcp_max_lines_per_blob: None,
        acemcp_text_extensions: None,
        acemcp_exclude_patterns: None,
    }
}

pub fn default_telegram_config() -> TelegramConfig {
    TelegramConfig {
        enabled: default_telegram_enabled(),
        hide_frontend_popup: default_telegram_hide_frontend_popup(),
        bots: default_telegram_bots(),
        default_bot: default_telegram_default_bot(),
        session_bot_mapping: default_telegram_session_bot_mapping(),
        pending_sessions: default_telegram_pending_sessions(),
        // 旧版本兼容字段
        bot_token: None,
        chat_id: None,
        api_base_url: None,
    }
}

pub fn default_telegram_bots() -> Vec<TelegramBotConfig> {
    vec![]
}

pub fn default_telegram_default_bot() -> String {
    String::new()
}

pub fn default_telegram_session_bot_mapping() -> HashMap<String, String> {
    HashMap::new()
}

pub fn default_telegram_pending_sessions() -> Vec<PendingSession> {
    vec![]
}

pub fn default_custom_prompt_config() -> CustomPromptConfig {
    CustomPromptConfig {
        prompts: default_custom_prompts(),
        enabled: default_custom_prompt_enabled(),
        max_prompts: default_custom_prompt_max_prompts(),
    }
}

pub fn default_always_on_top() -> bool {
    window::DEFAULT_ALWAYS_ON_TOP
}

pub fn default_audio_notification_enabled() -> bool {
    audio::DEFAULT_NOTIFICATION_ENABLED
}

pub fn default_theme() -> String {
    theme::DEFAULT.to_string()
}

pub fn default_audio_url() -> String {
    audio::DEFAULT_URL.to_string()
}

pub fn default_window_config() -> WindowConfig {
    WindowConfig {
        auto_resize: window::DEFAULT_AUTO_RESIZE,
        max_width: window::MAX_WIDTH,
        max_height: window::MAX_HEIGHT,
        min_width: window::MIN_WIDTH,
        min_height: window::MIN_HEIGHT,
        fixed: window::DEFAULT_FIXED_MODE,
        fixed_width: window::DEFAULT_WIDTH,
        fixed_height: window::DEFAULT_HEIGHT,
        free_width: window::DEFAULT_WIDTH,
        free_height: window::DEFAULT_HEIGHT,
    }
}

pub fn default_reply_config() -> ReplyConfig {
    ReplyConfig {
        enable_continue_reply: mcp::DEFAULT_CONTINUE_REPLY_ENABLED,
        auto_continue_threshold: mcp::DEFAULT_AUTO_CONTINUE_THRESHOLD,
        continue_prompt: mcp::DEFAULT_CONTINUE_PROMPT.to_string(),
    }
}

pub fn default_auto_resize() -> bool {
    true
}

pub fn default_max_width() -> f64 {
    window::MAX_WIDTH
}

pub fn default_max_height() -> f64 {
    window::MAX_HEIGHT
}

pub fn default_min_width() -> f64 {
    window::MIN_WIDTH
}

pub fn default_min_height() -> f64 {
    window::MIN_HEIGHT
}

pub fn default_enable_continue_reply() -> bool {
    mcp::DEFAULT_CONTINUE_REPLY_ENABLED
}

pub fn default_auto_continue_threshold() -> u32 {
    mcp::DEFAULT_AUTO_CONTINUE_THRESHOLD
}

pub fn default_continue_prompt() -> String {
    mcp::DEFAULT_CONTINUE_PROMPT.to_string()
}

pub fn default_mcp_tools() -> HashMap<String, bool> {
    let mut tools = HashMap::new();
    tools.insert(mcp::TOOL_ZHI.to_string(), true); // 寸止工具默认启用
    tools.insert(mcp::TOOL_JI.to_string(), false); // 记忆管理工具默认关闭
    tools.insert(mcp::TOOL_SOU.to_string(), false); // 代码搜索工具默认关闭
    tools
}

pub fn default_window_width() -> f64 {
    window::DEFAULT_WIDTH
}

pub fn default_window_height() -> f64 {
    window::DEFAULT_HEIGHT
}

pub fn default_window_fixed() -> bool {
    window::DEFAULT_FIXED_MODE
}

pub fn default_fixed_width() -> f64 {
    window::DEFAULT_WIDTH
}

pub fn default_fixed_height() -> f64 {
    window::DEFAULT_HEIGHT
}

pub fn default_free_width() -> f64 {
    window::DEFAULT_WIDTH
}

pub fn default_free_height() -> f64 {
    window::DEFAULT_HEIGHT
}

pub fn default_telegram_enabled() -> bool {
    telegram::DEFAULT_ENABLED
}

pub fn default_telegram_bot_token() -> String {
    telegram::DEFAULT_BOT_TOKEN.to_string()
}

pub fn default_telegram_chat_id() -> String {
    telegram::DEFAULT_CHAT_ID.to_string()
}

pub fn default_telegram_hide_frontend_popup() -> bool {
    telegram::DEFAULT_HIDE_FRONTEND_POPUP
}

pub fn default_telegram_api_base_url() -> String {
    telegram::API_BASE_URL.to_string()
}

impl WindowConfig {
    // 获取当前模式的宽度
    pub fn current_width(&self) -> f64 {
        if self.fixed {
            self.fixed_width
        } else {
            self.free_width
        }
    }

    // 获取当前模式的高度
    pub fn current_height(&self) -> f64 {
        if self.fixed {
            self.fixed_height
        } else {
            self.free_height
        }
    }

    // 更新当前模式的尺寸
    pub fn update_current_size(&mut self, width: f64, height: f64) {
        if self.fixed {
            self.fixed_width = width;
            self.fixed_height = height;
        } else {
            self.free_width = width;
            self.free_height = height;
        }
    }
}

// 字体配置默认值函数
pub fn default_font_config() -> FontConfig {
    FontConfig {
        font_family: default_font_family(),
        font_size: default_font_size(),
        custom_font_family: default_custom_font_family(),
    }
}

pub fn default_font_family() -> String {
    font::DEFAULT_FONT_FAMILY.to_string()
}

pub fn default_font_size() -> String {
    font::DEFAULT_FONT_SIZE.to_string()
}

pub fn default_custom_font_family() -> String {
    font::DEFAULT_CUSTOM_FONT_FAMILY.to_string()
}

pub fn default_prompt_type() -> String {
    "normal".to_string()
}

pub fn default_prompt_state() -> bool {
    false
}



// 自定义prompt默认值函数
pub fn default_custom_prompts() -> Vec<CustomPrompt> {
    vec![
        CustomPrompt {
            id: "default_1".to_string(),
            name: "✅Done".to_string(),
            content: "结束当前对话".to_string(),
            description: Some("请求AI结束工作".to_string()),
            sort_order: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "normal".to_string(),
            condition_text: None,
            template_true: None,
            template_false: None,
            current_state: false,
        },
        CustomPrompt {
            id: "default_2".to_string(),
            name: "🧹Clear".to_string(),
            content: "".to_string(),
            description: Some("清空输入框内容".to_string()),
            sort_order: 2,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "normal".to_string(),
            condition_text: None,
            template_true: None,
            template_false: None,
            current_state: false,
        },
        CustomPrompt {
            id: "default_3".to_string(),
            name: "✨New Issue".to_string(),
            content: "ok，完美，新的需求or问题，".to_string(),
            description: Some("准备新的需求or问题".to_string()),
            sort_order: 3,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "normal".to_string(),
            condition_text: None,
            template_true: None,
            template_false: None,
            current_state: false,
        },
        CustomPrompt {
            id: "default_4".to_string(),
            name: "🧠Remember".to_string(),
            content: "请记住，".to_string(),
            description: Some("寸止的另一个工具，请记住".to_string()),
            sort_order: 4,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "normal".to_string(),
            condition_text: None,
            template_true: None,
            template_false: None,
            current_state: false,
        },
        CustomPrompt {
            id: "default_5".to_string(),
            name: "📝Summary And Restart".to_string(),
            content: "本次对话的上下文已经太长了，我打算关掉并重新开一个新的会话。你有什么想对你的继任者说的，以便它能更好的理解你当前的工作并顺利继续？".to_string(),
            description: Some("总结-开新会话".to_string()),
            sort_order: 5,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "normal".to_string(),
            condition_text: None,
            template_true: None,
            template_false: None,
            current_state: false,
        },
        CustomPrompt {
            id: "default_6".to_string(),
            name: "🔍Review And Plan".to_string(),
            content: "请执行以下项目进度检查和规划任务：\n\n1. **项目进度分析**：\n   - 查看当前代码库状态，分析已完成的功能模块\n   - 识别已完成、进行中和待开始的功能点\n\n2. **里程碑确定**：\n   - 基于当前进度和剩余工作量，定义清晰的里程碑节点\n   - 为每个里程碑设定具体的完成标准和时间预期\n   - 优先考虑核心任务管理功能的里程碑\n\n3. **文档更新**（注意：仅更新现有文档，不创建新文档）：\n   - 更新项目规划文档中的进度状态\n   - 修正任何与实际实现不符的技术方案描述\n   - 确保文档反映当前的技术栈和架构决策\n\n4. **下一步工作规划**：\n   - 基于用户偏好（系统化开发方法、前端优先、分步骤反馈）制定具体的下一阶段工作计划\n   - 识别关键路径上的阻塞点和依赖关系\n   - 提供3-5个具体的下一步行动项，按优先级排序\n\n5. **反馈收集**：\n   - 在完成分析后，使用寸止工具收集用户对进度评估和下一步计划的反馈\n   - 提供多个可选的发展方向供用户选择".to_string(),
            description: Some("项目进度检查和规划任务".to_string()),
            sort_order: 6,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "normal".to_string(),
            condition_text: None,
            template_true: None,
            template_false: None,
            current_state: false,
        },
        CustomPrompt {
            id: "default_7".to_string(),
            name: "是否生成总结性Markdown文档".to_string(),
            content: "".to_string(),
            description: Some("是否生成总结性Markdown文档".to_string()),
            sort_order: 7,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "conditional".to_string(),
            condition_text: Some("是否生成总结性Markdown文档".to_string()),
            template_true: Some("✔️请记住，帮我生成总结性Markdown文档".to_string()),
            template_false: Some("❌请记住，不要生成总结性Markdown文档".to_string()),
            current_state: false,
        },
        CustomPrompt {
            id: "default_8".to_string(),
            name: "是否生成测试脚本".to_string(),
            content: "".to_string(),
            description: Some("是否生成测试脚本".to_string()),
            sort_order: 8,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "conditional".to_string(),
            condition_text: Some("是否生成测试脚本".to_string()),
            template_true: Some("✔️请记住，帮我生成测试脚本".to_string()),
            template_false: Some("❌请记住，不要生成测试脚本".to_string()),
            current_state: false,
        },
        CustomPrompt {
            id: "default_9".to_string(),
            name: "是否主动编译".to_string(),
            content: "".to_string(),
            description: Some("是否主动编译".to_string()),
            sort_order: 9,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "conditional".to_string(),
            condition_text: Some("是否主动编译".to_string()),
            template_true: Some("✔️请记住，帮我编译".to_string()),
            template_false: Some("❌请记住，不要编译，用户自己编译".to_string()),
            current_state: false,
        },
        CustomPrompt {
            id: "default_10".to_string(),
            name: "是否主动运行".to_string(),
            content: "".to_string(),
            description: Some("是否主动运行".to_string()),
            sort_order: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "conditional".to_string(),
            condition_text: Some("是否主动运行".to_string()),
            template_true: Some("✔️请记住，帮我运行".to_string()),
            template_false: Some("❌请记住，不要运行，用户自己运行".to_string()),
            current_state: false,
        },
    ]
}

pub fn default_custom_prompt_enabled() -> bool {
    true
}

pub fn default_custom_prompt_max_prompts() -> u32 {
    50
}

// 快捷键默认值函数
pub fn default_shortcut_config() -> ShortcutConfig {
    ShortcutConfig {
        shortcuts: default_shortcuts(),
    }
}

pub fn default_shortcuts() -> HashMap<String, ShortcutBinding> {
    let mut shortcuts = HashMap::new();

    // 快速发送快捷键
    shortcuts.insert("quick_submit".to_string(), ShortcutBinding {
        id: "quick_submit".to_string(),
        name: "快速发送".to_string(),
        description: "快速提交当前输入内容".to_string(),
        action: "submit".to_string(),
        key_combination: ShortcutKey {
            key: "Enter".to_string(),
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        },
        enabled: true,
        scope: "popup".to_string(),
    });

    // 增强快捷键
    shortcuts.insert("enhance".to_string(), ShortcutBinding {
        id: "enhance".to_string(),
        name: "增强".to_string(),
        description: "增强当前输入内容".to_string(),
        action: "enhance".to_string(),
        key_combination: ShortcutKey {
            key: "Enter".to_string(),
            ctrl: true,
            alt: false,
            shift: true,
            meta: false,
        },
        enabled: true,
        scope: "popup".to_string(),
    });

    // 继续快捷键
    shortcuts.insert("continue".to_string(), ShortcutBinding {
        id: "continue".to_string(),
        name: "继续".to_string(),
        description: "继续对话".to_string(),
        action: "continue".to_string(),
        key_combination: ShortcutKey {
            key: "Enter".to_string(),
            ctrl: false,
            alt: true,
            shift: false,
            meta: false,
        },
        enabled: true,
        scope: "popup".to_string(),
    });

    shortcuts
}


