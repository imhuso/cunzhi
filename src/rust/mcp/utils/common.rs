/// MCP 通用工具函数模块
///
/// 包含 MCP 相关的通用工具函数和辅助方法

use anyhow::Result;
use std::path::Path;
use percent_encoding;

/// 解码 URL 编码的路径
///
/// 在 Windows 下，路径中的冒号可能会被编码为 %3A，需要先解码
fn decode_path(path: &str) -> String {
    // 使用 percent_encoding 库进行 URL 解码
    match percent_encoding::percent_decode_str(path).decode_utf8() {
        Ok(decoded) => decoded.to_string(),
        Err(_) => {
            // 如果解码失败，返回原始路径
            path.to_string()
        }
    }
}

/// 验证项目路径是否存在
pub fn validate_project_path(path: &str) -> Result<()> {
    // 先对路径进行 URL 解码，处理 Windows 下的编码问题
    let decoded_path = decode_path(path);

    if !Path::new(&decoded_path).exists() {
        anyhow::bail!("项目路径不存在: {}", decoded_path);
    }
    Ok(())
}

/// 生成唯一的请求 ID
pub fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}




