#!/bin/bash

# AI Review 安装脚本 - macOS App Bundle 方式

set -e

# 检查是否只构建不安装
BUILD_ONLY=false
if [[ "$1" == "--build-only" ]]; then
    BUILD_ONLY=true
    echo "🚀 只构建不安装 AI Review..."
else
    echo "🚀 开始安装 AI Review (macOS App Bundle)..."
fi

# 检查是否为 macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ 此安装脚本仅支持 macOS"
    exit 1
fi

echo "🔨 构建 Tauri 应用包..."

# 首先确保前端已构建
echo "📦 构建前端资源..."
pnpm build

# 构建 Tauri 应用包，如果失败则重试
MAX_RETRIES=3
RETRY_COUNT=0

while [[ $RETRY_COUNT -lt $MAX_RETRIES ]]; do
    echo "🔄 尝试构建 Tauri 应用包 (第 $((RETRY_COUNT + 1)) 次)..."

    if cargo tauri build; then
        echo "✅ Tauri 构建成功"
        break
    else
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [[ $RETRY_COUNT -lt $MAX_RETRIES ]]; then
            echo "⚠️  构建失败，等待 5 秒后重试..."
            sleep 5
        else
            echo "❌ Tauri 构建失败，已达到最大重试次数"

            # 检查是否有部分构建产物
            if [[ -f "target/release/ai-review-ui" ]] && [[ -f "target/release/ai-review-mcp" ]]; then
                echo "🔧 检测到二进制文件，尝试手动创建 App Bundle..."

                # 手动创建 App Bundle
                APP_BUNDLE="target/release/bundle/macos/AI Review.app"
                mkdir -p "$APP_BUNDLE/Contents/MacOS"
                mkdir -p "$APP_BUNDLE/Contents/Resources"

                # 复制二进制文件
                cp "target/release/ai-review-ui" "$APP_BUNDLE/Contents/MacOS/"
                cp "target/release/ai-review-mcp" "$APP_BUNDLE/Contents/MacOS/"

                # 复制图标（如果存在）
                if [[ -f "icons/icon.icns" ]]; then
                    cp "icons/icon.icns" "$APP_BUNDLE/Contents/Resources/"
                fi

                # 创建 Info.plist
                cat > "$APP_BUNDLE/Contents/Info.plist" << 'PLIST_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>AI Review</string>
    <key>CFBundleExecutable</key>
    <string>ai-review-ui</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>com.imhuso.ai-review</string>
    <key>CFBundleName</key>
    <string>AI Review</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
PLIST_EOF

                echo "✅ 手动创建 App Bundle 成功"
                break
            else
                echo "❌ 无法找到构建产物，请检查构建错误"
                exit 1
            fi
        fi
    fi
done

# 检查构建结果
APP_BUNDLE="target/release/bundle/macos/AI Review.app"
if [[ ! -d "$APP_BUNDLE" ]]; then
    echo "❌ 应用包构建失败: $APP_BUNDLE"
    exit 1
fi

echo "✅ 应用包构建成功: $APP_BUNDLE"

# 如果只构建不安装，则在这里退出
if [[ "$BUILD_ONLY" == "true" ]]; then
    echo ""
    echo "🎉 AI Review 构建完成！"
    echo ""
    echo "📋 应用包位置: $APP_BUNDLE"
    echo ""
    echo "如需安装，请手动复制应用包到 Applications 目录。"
    exit 0
fi

# 安装应用到 Applications 目录
echo "📋 安装应用到 Applications 目录..."

# 移除旧版本（如果存在）
if [[ -d "/Applications/AI Review.app" ]]; then
    echo "🗑️  移除旧版本..."
    sudo rm -rf "/Applications/AI Review.app"
fi

# 复制新版本
sudo cp -R "$APP_BUNDLE" "/Applications/"
echo "✅ 应用已安装到 /Applications/AI Review.app"

# 运行 postinstall 脚本
echo "⚙️  配置命令行工具..."
if [[ -f "scripts/postinstall.sh" ]]; then
    bash scripts/postinstall.sh
else
    echo "❌ 未找到 postinstall.sh 脚本"
    exit 1
fi

echo ""
echo "🎉 AI Review 安装完成！"
echo ""
echo "📋 使用方法："
echo "  🖥️  GUI模式: 在 Applications 中打开 'AI Review'"
echo "  💻 命令行模式:"
echo "    ai-review-ui                    - 启动 UI 界面"
echo "    ai-review-ui --mcp-request file - MCP 弹窗模式"
echo "    ai-review-mcp                   - 启动 MCP 服务器"
echo ""
echo "📝 配置 MCP 客户端："
echo "将以下内容添加到您的 MCP 客户端配置中："
echo ""
cat << 'EOF'
{
  "mcpServers": {
    "ai-review": {
      "command": "ai-review-mcp",
      "args": []
    }
  }
}
EOF
echo ""
echo "🔗 命令行工具已链接到 /usr/local/bin/"
echo "📁 应用位置: /Applications/AI Review.app"
