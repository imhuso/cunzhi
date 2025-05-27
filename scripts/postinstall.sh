#!/bin/bash

# macOS 安装后脚本
# 此脚本在 AI Review.app 安装后运行，自动设置命令行工具

set -e

echo "🚀 AI Review 安装后配置..."

# 获取应用程序路径
APP_PATH="/Applications/AI Review.app"
MAIN_BINARY="$APP_PATH/Contents/MacOS/ai-review-ui"

# 检查应用是否存在
if [[ ! -d "$APP_PATH" ]]; then
    echo "❌ 错误: 未找到 AI Review.app"
    exit 1
fi

# 检查主程序是否存在
if [[ ! -f "$MAIN_BINARY" ]]; then
    echo "❌ 错误: 未找到主程序: $MAIN_BINARY"
    exit 1
fi

# 创建 /usr/local/bin 目录（如果不存在）
if [[ ! -d "/usr/local/bin" ]]; then
    mkdir -p "/usr/local/bin"
fi

# 定义命令目标
UI_TARGET="/usr/local/bin/ai-review-ui"
MCP_TARGET="/usr/local/bin/ai-review-mcp"

# 移除旧的符号链接（如果存在）
for target in "$UI_TARGET" "$MCP_TARGET"; do
    if [[ -L "$target" ]] || [[ -f "$target" ]]; then
        rm -f "$target"
    fi
done

# 创建 ai-review-ui 符号链接
ln -s "$MAIN_BINARY" "$UI_TARGET"
chmod +x "$UI_TARGET"
echo "✓ ai-review-ui 命令已创建 -> $MAIN_BINARY"

# 检查是否有独立的MCP服务器二进制文件
MCP_BINARY="$APP_PATH/Contents/MacOS/ai-review-mcp"
if [[ -f "$MCP_BINARY" ]]; then
    ln -s "$MCP_BINARY" "$MCP_TARGET"
    chmod +x "$MCP_TARGET"
    echo "✓ ai-review-mcp 命令已创建 -> $MCP_BINARY"
else
    # 如果没有独立的MCP二进制文件，创建一个指向主程序的链接
    # 主程序应该能够检测到它是以 ai-review-mcp 名称运行的
    ln -s "$MAIN_BINARY" "$MCP_TARGET"
    chmod +x "$MCP_TARGET"
    echo "✓ ai-review-mcp 命令已创建 -> $MAIN_BINARY (共享)"
fi

echo ""
echo "🎉 AI Review 安装完成！"
echo ""
echo "📋 使用方法:"
echo "  ai-review-ui                    - 启动UI界面"
echo "  ai-review-ui --mcp-request file - MCP弹窗模式"
echo "  ai-review-mcp                   - 启动MCP服务器"
echo ""
echo "📁 应用位置: /Applications/AI Review.app"
echo "🔗 命令链接: /usr/local/bin/ai-review-*"
