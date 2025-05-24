use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 获取目标架构
    let target = env::var("TARGET").unwrap_or_else(|_| "x86_64-apple-darwin".to_string());
    
    // 源文件和目标文件路径
    let src_path = "target/release/ai-review-cli";
    let dst_path = format!("target/release/ai-review-cli-{}", target);
    
    // 如果源文件存在，复制到目标文件
    if Path::new(src_path).exists() {
        if let Err(e) = fs::copy(src_path, &dst_path) {
            println!("cargo:warning=Failed to copy CLI binary: {}", e);
        } else {
            println!("cargo:warning=Copied CLI binary to {}", dst_path);
        }
    }
    
    tauri_build::build()
}
