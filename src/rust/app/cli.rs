use crate::config::load_standalone_telegram_config;
use crate::telegram::handle_telegram_only_mcp_request;
use crate::log_important;
use crate::app::builder::run_tauri_app;
use anyhow::Result;

/// 处理命令行参数
pub fn handle_cli_args() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        // 无参数：正常启动GUI
        1 => {
            run_tauri_app();
        }
        // 单参数：帮助或版本
        2 => {
            match args[1].as_str() {
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                "--version" | "-v" => {
                    print_version();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("未知参数: {}", args[1]);
                    print_help();
                    std::process::exit(1);
                }
            }
        }
        // 多参数：MCP请求模式
        _ => {
            if args[1] == "--mcp-request" && args.len() >= 3 {
                handle_mcp_request(&args[2])?;
            } else {
                eprintln!("无效的命令行参数");
                print_help();
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// 处理MCP请求
fn handle_mcp_request(request_file: &str) -> Result<()> {
    // 先记录会话信息（无论是哪种模式）
    record_session_from_request(request_file);

    // 检查Telegram配置，决定是否启用纯Telegram模式
    match load_standalone_telegram_config() {
        Ok(telegram_config) => {
            if telegram_config.enabled && telegram_config.hide_frontend_popup {
                // 纯Telegram模式：不启动GUI，直接处理
                if let Err(e) = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(handle_telegram_only_mcp_request(request_file))
                {
                    log_important!(error, "处理Telegram请求失败: {}", e);
                    std::process::exit(1);
                }
            } else {
                // 正常模式：启动GUI处理弹窗
                run_tauri_app();
            }
        }
        Err(e) => {
            log_important!(warn, "加载Telegram配置失败: {}，使用默认GUI模式", e);
            // 配置加载失败时，使用默认行为（启动GUI）
            run_tauri_app();
        }
    }
    Ok(())
}

/// 从请求文件中记录会话信息
fn record_session_from_request(request_file: &str) {
    use crate::config::{load_standalone_config, save_standalone_config};
    use crate::mcp::types::PopupRequest;

    log_important!(info, "开始记录会话，请求文件: {}", request_file);

    // 读取请求文件
    match std::fs::read_to_string(request_file) {
        Ok(request_json) => {
            log_important!(info, "成功读取请求文件");
            match serde_json::from_str::<PopupRequest>(&request_json) {
                Ok(request) => {
                    log_important!(info, "成功解析请求，session_id: {:?}", request.session_id);
                    // 如果请求中包含 session_id，记录会话
                    if let Some(session_id) = request.session_id {
                        match load_standalone_config() {
                            Ok(mut app_config) => {
                                log_important!(info, "成功加载配置，开始记录会话");
                                app_config.telegram_config.record_session_request(&session_id);
                                // 保存配置
                                match save_standalone_config(&app_config) {
                                    Ok(_) => {
                                        log_important!(info, "✅ 已记录会话: {}", session_id);
                                    }
                                    Err(e) => {
                                        log_important!(warn, "保存会话记录失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log_important!(warn, "加载配置失败: {}", e);
                            }
                        }
                    } else {
                        log_important!(warn, "请求中没有 session_id");
                    }
                }
                Err(e) => {
                    log_important!(warn, "解析请求失败: {}", e);
                }
            }
        }
        Err(e) => {
            log_important!(warn, "读取请求文件失败: {}", e);
        }
    }
}

/// 显示帮助信息
fn print_help() {
    use std::io::{self, Write};
    println!("寸止 - 智能代码审查工具");
    println!();
    println!("用法:");
    println!("  等一下                    启动设置界面");
    println!("  等一下 --mcp-request <文件>  处理 MCP 请求");
    println!("  等一下 --help             显示此帮助信息");
    println!("  等一下 --version          显示版本信息");
    let _ = io::stdout().flush();
}

/// 显示版本信息
fn print_version() {
    use std::io::{self, Write};
    println!("寸止 v{}", env!("CARGO_PKG_VERSION"));
    let _ = io::stdout().flush();
}
