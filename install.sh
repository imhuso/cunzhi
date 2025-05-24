#!/bin/bash

set -e

echo "🚀 开始安装 AI Review..."

# 检查必要的目录
BUNDLE_DIR="target/release/bundle"
APP_PATH="$BUNDLE_DIR/macos/AI Review.app"
CLI_PATH="$APP_PATH/Contents/MacOS/ai-review-cli"

if [[ ! -d "$APP_PATH" ]]; then
    echo "❌ 未找到应用程序包: $APP_PATH"
    echo "请先运行: cargo tauri build"
    exit 1
fi

if [[ ! -f "$CLI_PATH" ]]; then
    echo "❌ 未找到CLI二进制文件: $CLI_PATH"
    exit 1
fi

# 安装GUI应用
echo "📱 安装GUI应用到 /Applications..."
if [[ -d "/Applications/AI Review.app" ]]; then
    echo "⚠️  应用已存在，将覆盖安装"
    rm -rf "/Applications/AI Review.app"
fi

cp -R "$APP_PATH" "/Applications/"
echo "✅ GUI应用安装完成"

# 安装CLI命令
echo "⚙️  安装CLI命令..."

# 检查 /usr/local/bin 是否存在
if [[ ! -d "/usr/local/bin" ]]; then
    echo "📁 创建 /usr/local/bin 目录..."
    sudo mkdir -p /usr/local/bin
fi

CLI_TARGET="/usr/local/bin/ai-review-cli"
CLI_SOURCE="/Applications/AI Review.app/Contents/MacOS/ai-review-cli"

# 移除旧的符号链接（如果存在）
if [[ -L "$CLI_TARGET" ]] || [[ -f "$CLI_TARGET" ]]; then
    echo "🗑️  移除旧的CLI命令"
    sudo rm -f "$CLI_TARGET"
fi

# 创建符号链接
sudo ln -s "$CLI_SOURCE" "$CLI_TARGET"
sudo chmod +x "$CLI_TARGET"
echo "✅ CLI命令安装完成"

# 验证安装
echo "🔍 验证安装..."

if [[ -d "/Applications/AI Review.app" ]]; then
    echo "✅ GUI应用安装成功"
else
    echo "❌ GUI应用安装失败"
fi

if command -v ai-review-cli >/dev/null 2>&1; then
    echo "✅ CLI命令安装成功"
    echo "📋 CLI版本: $(ai-review-cli --version 2>/dev/null || echo '无法获取版本')"
else
    echo "❌ CLI命令安装失败或不在PATH中"
    echo "💡 请确保 /usr/local/bin 在您的PATH中"
fi

echo ""
echo "🎉 安装完成！"
echo ""
echo "📖 使用方法:"
echo "  GUI应用: 在启动台或应用程序文件夹中找到 'AI Review'"
echo "  CLI命令: 在终端中运行 'ai-review-cli help'"
echo ""
echo "💡 提示: 如果CLI命令无法使用，请重新启动终端" 
