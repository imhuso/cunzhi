/// MCP 通用工具函数模块
///
/// 包含 MCP 相关的通用工具函数和辅助方法

use anyhow::Result;
use std::path::Path;
use percent_encoding;
use regex::Regex;

/// 解码并规范化路径
///
/// 处理 URL 编码、Windows 路径格式转换等问题
pub fn decode_and_normalize_path(path: &str) -> Result<String> {
    // 1. 先进行 URL 解码
    let decoded = decode_url_path(path);

    // 2. 规范化路径格式
    let normalized = normalize_path_format(&decoded)?;

    Ok(normalized)
}

/// 解码 URL 编码的路径
///
/// 在 Windows 下，路径中的冒号可能会被编码为 %3A，需要先解码
fn decode_url_path(path: &str) -> String {
    // 使用 percent_encoding 库进行 URL 解码
    match percent_encoding::percent_decode_str(path).decode_utf8() {
        Ok(decoded) => decoded.to_string(),
        Err(_) => {
            // 如果解码失败，返回原始路径
            path.to_string()
        }
    }
}

/// 规范化路径格式
///
/// 处理 Windows 下的路径格式问题，如 /c:/ -> C:\
fn normalize_path_format(path: &str) -> Result<String> {
    let path = path.trim();

    // 检查是否为 Windows 风格的路径（以 /盘符:/ 开头）
    if let Some(normalized) = normalize_windows_path(path) {
        return Ok(normalized);
    }

    // 检查是否为标准的 Windows 路径（C:\ 或 C:/）
    if is_windows_absolute_path(path) {
        return Ok(path.replace('/', "\\"));
    }

    // 其他情况直接返回
    Ok(path.to_string())
}

/// 规范化 Windows 路径格式
///
/// 将 /c:/path 或 /C:/path 格式转换为 C:\path
fn normalize_windows_path(path: &str) -> Option<String> {
    // 匹配 /盘符:/ 格式的路径
    let re = Regex::new(r"^/([a-zA-Z]):(.*)$").ok()?;

    if let Some(captures) = re.captures(path) {
        let drive = captures.get(1)?.as_str().to_uppercase();
        let rest = captures.get(2)?.as_str();

        // 转换为 Windows 格式
        let windows_path = format!("{}:{}", drive, rest.replace('/', "\\"));
        return Some(windows_path);
    }

    None
}

/// 检查是否为 Windows 绝对路径
fn is_windows_absolute_path(path: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z]:[/\\]").unwrap();
    re.is_match(path)
}

/// 验证项目路径是否存在
pub fn validate_project_path(path: &str) -> Result<()> {
    // 先对路径进行解码和规范化
    let normalized_path = decode_and_normalize_path(path)?;

    // 验证路径格式
    validate_path_format(&normalized_path)?;

    // 检查路径是否存在
    let path_obj = Path::new(&normalized_path);
    if !path_obj.exists() {
        anyhow::bail!("项目路径不存在: {}", normalized_path);
    }

    // 检查是否为目录
    if !path_obj.is_dir() {
        anyhow::bail!("项目路径不是目录: {}", normalized_path);
    }

    Ok(())
}

/// 验证路径格式是否合法
fn validate_path_format(path: &str) -> Result<()> {
    // 检查路径是否包含非法字符
    let illegal_chars = ['<', '>', '"', '|', '?', '*'];
    for ch in illegal_chars.iter() {
        if path.contains(*ch) {
            anyhow::bail!("路径包含非法字符 '{}': {}", ch, path);
        }
    }

    // 检查路径长度（Windows 限制）
    if cfg!(windows) && path.len() > 260 {
        anyhow::bail!("路径过长（超过260字符）: {}", path);
    }

    Ok(())
}

/// 生成唯一的请求 ID
pub fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}




