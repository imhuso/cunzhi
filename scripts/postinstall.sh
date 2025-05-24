#!/bin/bash

# macOS 安装后脚本
# 此脚本在 AI Review.app 安装后运行，自动设置 CLI 命令

set -e

# 获取应用程序路径
APP_PATH="/Applications/AI Review.app"
CLI_SOURCE="$APP_PATH/Contents/MacOS/ai-review-cli"
CLI_TARGET="/usr/local/bin/ai-review-cli"

# 检查应用是否存在
if [[ ! -d "$APP_PATH" ]]; then
    echo "错误: 未找到 AI Review.app"
    exit 1
fi

# 检查CLI二进制文件是否存在
if [[ ! -f "$CLI_SOURCE" ]]; then
    echo "错误: 未找到 CLI 二进制文件: $CLI_SOURCE"
    exit 1
fi

# 创建 /usr/local/bin 目录（如果不存在）
if [[ ! -d "/usr/local/bin" ]]; then
    mkdir -p "/usr/local/bin"
fi

# 移除旧的符号链接（如果存在）
if [[ -L "$CLI_TARGET" ]] || [[ -f "$CLI_TARGET" ]]; then
    rm -f "$CLI_TARGET"
fi

# 创建符号链接
ln -s "$CLI_SOURCE" "$CLI_TARGET"

# 设置执行权限
chmod +x "$CLI_TARGET"

echo "AI Review CLI 命令已安装到: $CLI_TARGET"
echo "您现在可以在终端中使用 'ai-review-cli' 命令" 
