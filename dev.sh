#!/bin/bash

# AI Review 开发模式脚本
# 用于启动开发环境

set -e

echo "🔧 启动 AI Review 开发环境..."

# 检查是否需要安装依赖
if [ ! -d "node_modules" ]; then
    echo "📦 安装前端依赖..."
    npm install
fi

# 编译 CLI (开发模式)
echo "🛠️  编译 CLI (开发模式)..."
cargo build --bin ai-review-cli

echo "🚀 启动 Tauri 开发服务器..."
npm run tauri dev
