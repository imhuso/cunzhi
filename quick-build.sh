#!/bin/bash

# AI Review 快速编译脚本
# 用于快速编译和测试

set -e

echo "🚀 快速编译 AI Review..."

# 编译 CLI
echo "📦 编译 CLI..."
cargo build --release --bin ai-review-cli

# 编译 Tauri 应用
echo "🖥️  编译 Tauri 应用..."
npm run tauri build

echo "✅ 编译完成！"
echo ""
echo "📁 输出文件："
echo "  CLI: target/release/ai-review-cli"
echo "  App: src-tauri/target/release/bundle/"

# 显示文件大小
if [ -f "target/release/ai-review-cli" ]; then
    cli_size=$(du -h target/release/ai-review-cli | cut -f1)
    echo "  CLI 大小: $cli_size"
fi
