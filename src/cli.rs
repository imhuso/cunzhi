use anyhow::Result;
use clap::{Arg, Command};

mod ipc;

use ipc::IpcClient;

fn main() -> Result<()> {
    let matches = Command::new("ai-review-cli")
        .version("0.1.0")
        .about("AI Review Command Line Interface")
        .arg(
            Arg::new("message")
                .value_name("MESSAGE")
                .help("Message to send to the UI application")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("SECONDS")
                .help("Timeout in seconds (default: 30)")
                .default_value("30"),
        )
        .get_matches();

    let message = matches.get_one::<String>("message").unwrap();
    let timeout: u64 = matches
        .get_one::<String>("timeout")
        .unwrap()
        .parse()
        .unwrap_or(30);

    println!("📤 发送消息到 AI Review UI: {}", message);
    println!("⏱️  等待回复中... (超时时间: {}秒)", timeout);

    match IpcClient::send_message_with_timeout(message.clone(), timeout) {
        Ok(response) => {
            println!("✅ 收到回复: {}", response);
        }
        Err(e) => {
            eprintln!("❌ 错误: {}", e);
            eprintln!("请确保 AI Review UI 应用正在运行。");
            std::process::exit(1);
        }
    }

    Ok(())
}
