use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Once;
use log::LevelFilter;
use env_logger::{Builder, Target};

static INIT: Once = Once::new();

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: LevelFilter,
    /// 日志文件路径（None 表示不输出到文件）
    pub file_path: Option<String>,
    /// 是否为 MCP 模式（MCP 模式下不输出到 stderr）
    pub is_mcp_mode: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LevelFilter::Warn,
            file_path: None,
            is_mcp_mode: false,
        }
    }
}

/// 初始化日志系统
pub fn init_logger(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        let mut builder = Builder::new();
        
        // 设置日志级别
        builder.filter_level(config.level);
        
        // 设置日志格式
        builder.format(|buf, record| {
            let log_line = format!(
                "{} [{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.module_path().unwrap_or("unknown"),
                record.args()
            );
            
            // 写入到原始目标（stderr 或文件）
            writeln!(buf, "{}", log_line)?;
            
            Ok(())
        });
        
        // 根据模式设置输出目标
        if config.is_mcp_mode {
            // MCP 模式：只输出到文件，不输出到 stderr
            if let Some(file_path) = &config.file_path {
                if let Ok(log_file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path) 
                {
                    builder.target(Target::Pipe(Box::new(log_file)));
                } else {
                    // 如果文件打开失败，禁用日志输出
                    builder.filter_level(LevelFilter::Off);
                }
            } else {
                // MCP 模式下没有指定文件路径，禁用日志输出
                builder.filter_level(LevelFilter::Off);
            }
        } else {
            // 非 MCP 模式：如果指定了文件路径，同时输出到文件和 stderr
            if let Some(file_path) = &config.file_path {
                // 尝试打开文件，如果成功则同时输出到文件和 stderr
                if let Ok(log_file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path) 
                {
                    // 使用自定义目标，同时写入文件和 stderr
                    use std::io::Write;
                    struct DualWriter {
                        file: std::fs::File,
                    }
                    impl Write for DualWriter {
                        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                            let written = self.file.write(buf)?;
                            let _ = std::io::stderr().write_all(buf);
                            Ok(written)
                        }
                        fn flush(&mut self) -> std::io::Result<()> {
                            self.file.flush()?;
                            std::io::stderr().flush()
                        }
                    }
                    builder.target(Target::Pipe(Box::new(DualWriter { file: log_file })));
                } else {
                    // 如果文件打开失败，只输出到 stderr
                    builder.target(Target::Stderr);
                }
            } else {
                // 没有指定文件路径，只输出到 stderr
                builder.target(Target::Stderr);
            }
        }
        
        builder.init();
    });
    
    Ok(())
}

/// 自动检测模式并初始化日志系统
pub fn auto_init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let is_mcp_mode = args.len() >= 3 && args[1] == "--mcp-request";
    
    let config = if is_mcp_mode {
        // MCP 模式：输出到文件
        let log_file_path = env::var("MCP_LOG_FILE")
            .unwrap_or_else(|_| {
                let temp_dir = env::temp_dir();
                temp_dir.join("cunzhi-mcp.log").to_string_lossy().to_string()
            });
            
        LogConfig {
            level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "warn".to_string())
                .parse::<LevelFilter>()
                .unwrap_or(LevelFilter::Warn),
            file_path: Some(log_file_path),
            is_mcp_mode: true,
        }
    } else {
        // GUI 模式：输出到 stderr
        LogConfig {
            level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string())
                .parse::<LevelFilter>()
                .unwrap_or(LevelFilter::Info),
            file_path: None,
            is_mcp_mode: false,
        }
    };
    
    init_logger(config)
}

/// 便利宏：只在重要情况下记录日志
#[macro_export]
macro_rules! log_important {
    (error, $($arg:tt)*) => {
        log::error!($($arg)*)
    };
    (warn, $($arg:tt)*) => {
        log::warn!($($arg)*)
    };
    (info, $($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

/// 便利宏：调试日志（只在 debug 级别下输出）
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

/// 便利宏：跟踪日志（只在 trace 级别下输出）
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        log::trace!($($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, LevelFilter::Warn);
        assert_eq!(config.file_path, None);
        assert_eq!(config.is_mcp_mode, false);
    }
    
    #[test]
    fn test_mcp_mode_detection() {
        // 这个测试需要在实际环境中运行
        // 这里只是展示如何测试
    }
}
