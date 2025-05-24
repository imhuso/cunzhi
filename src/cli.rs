use anyhow::Result;
use clap::{Arg, Command};

mod ipc;

use ipc::IpcClient;

fn print_help() {
    println!("🤖 AI Review 命令行工具帮助");
    println!("═══════════════════════════════════════");
    println!();
    println!("📖 基本用法:");
    println!("  ai-review-cli [MESSAGE] [OPTIONS]");
    println!();
    println!("📝 命令示例:");
    println!("  ai-review-cli                    # 获取默认帮助信息");
    println!("  ai-review-cli help               # 显示此帮助信息");
    println!("  ai-review-cli init               # 初始化并获取提示文本");
    println!("  ai-review-cli \"你好\"              # 发送自定义消息");
    println!("  ai-review-cli \"分析代码\" -t 60      # 发送消息并设置60秒超时");
    println!();
    println!("⚙️  选项:");
    println!("  -t, --timeout <SECONDS>          设置超时时间 (默认: 30秒)");
    println!("  -h, --help                       显示帮助信息");
    println!("  -V, --version                    显示版本信息");
    println!();
    println!("💡 特殊命令:");
    println!("  help                             显示详细帮助信息");
    println!("  init                             获取 AI Review 初始化提示");
    println!();
    println!("📋 注意事项:");
    println!("  • 确保 AI Review UI 应用正在运行");
    println!("  • 消息内容支持中文和英文");
    println!("  • 超时时间建议设置为 30-120 秒");
    println!();
    println!("🔗 更多信息:");
    println!("  项目地址: https://github.com/imhuso/ai-review");
    println!("  问题反馈: https://github.com/imhuso/ai-review/issues");
}

fn main() -> Result<()> {
    let matches = Command::new("ai-review-cli")
        .version("0.1.0")
        .about("AI Review 命令行接口")
        .arg(
            Arg::new("message")
                .value_name("MESSAGE")
                .help("发送给 UI 应用的消息 (default: 'init')")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("SECONDS")
                .help("超时时间 (default: 30)")
                .default_value("30"),
        )
        .get_matches();

    let message = matches.get_one::<String>("message")
        .map(|s| s.as_str())
        .unwrap_or("init");
    let timeout: u64 = matches
        .get_one::<String>("timeout")
        .unwrap()
        .parse()
        .unwrap_or(30);

    // 处理 help 命令
    if message == "help" {
        print_help();
        return Ok(());
    }

    match IpcClient::send_message_with_timeout(message.to_string(), timeout) {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("❌ 错误: {}", e);
            eprintln!("请确保 AI Review UI 应用正在运行。");
            std::process::exit(1);
        }
    }

    Ok(())
}
