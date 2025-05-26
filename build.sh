#!/bin/bash

# AI Review 应用编译脚本
# 用于编译 Tauri 应用和 CLI 工具

set -e  # 遇到错误时退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_step() {
    echo -e "${BLUE}🔧 $1${NC}"
}

# 检查依赖
check_dependencies() {
    print_step "检查依赖..."

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo 未安装"
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        print_error "Node.js/npm 未安装"
        exit 1
    fi

    print_success "依赖检查完成"
}

# 清理构建目录
clean_build() {
    print_step "清理构建目录..."

    if [ -d "target" ]; then
        rm -rf target
        print_info "已清理 target 目录"
    fi

    if [ -d "node_modules" ]; then
        print_info "保留 node_modules 目录"
    fi

    if [ -d "dist" ]; then
        rm -rf dist
        print_info "已清理 dist 目录"
    fi

    print_success "清理完成"
}

# 安装前端依赖
install_frontend_deps() {
    print_step "安装前端依赖..."
    npm install
    print_success "前端依赖安装完成"
}

# 编译 Rust 二进制文件
build_rust() {
    print_step "编译 Rust 二进制文件..."
    cargo build --release
    print_success "Rust 编译完成"

    # 检查二进制文件
    if [ -f "target/release/ai-review-ui" ] && [ -f "target/release/ai-review-mcp" ]; then
        print_info "二进制文件位置: target/release/"

        # 显示文件大小
        ui_size=$(du -h target/release/ai-review-ui | cut -f1)
        mcp_size=$(du -h target/release/ai-review-mcp | cut -f1)
        print_info "UI 文件大小: $ui_size"
        print_info "MCP 文件大小: $mcp_size"
    else
        print_error "编译失败"
        exit 1
    fi
}

# 编译 Tauri 应用
build_tauri() {
    print_step "编译 Tauri 应用..."
    npm run tauri build
    print_success "Tauri 应用编译完成"

    # 检查构建产物
    if [ -d "src-tauri/target/release/bundle" ]; then
        print_info "应用包位置: src-tauri/target/release/bundle/"

        # 列出构建产物
        print_info "构建产物:"
        find src-tauri/target/release/bundle -name "*.app" -o -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" | while read file; do
            size=$(du -h "$file" | cut -f1)
            print_info "  - $(basename "$file") ($size)"
        done
    fi
}

# 创建发布目录
create_release_dir() {
    print_step "创建发布目录..."

    RELEASE_DIR="release"
    mkdir -p "$RELEASE_DIR"

    # 复制 Rust 二进制文件
    if [ -f "target/release/ai-review-ui" ]; then
        cp target/release/ai-review-ui "$RELEASE_DIR/"
        print_info "已复制 UI 工具到 $RELEASE_DIR/"
    fi

    if [ -f "target/release/ai-review-mcp" ]; then
        cp target/release/ai-review-mcp "$RELEASE_DIR/"
        print_info "已复制 MCP 服务器到 $RELEASE_DIR/"
    fi

    # 复制 Tauri 应用包
    if [ -d "target/release/bundle" ]; then
        find target/release/bundle -name "*.app" -o -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" | while read file; do
            cp -r "$file" "$RELEASE_DIR/"
            print_info "已复制 $(basename "$file") 到 $RELEASE_DIR/"
        done
    fi

    print_success "发布目录创建完成: $RELEASE_DIR/"
}

# 显示帮助信息
show_help() {
    echo "AI Review 编译脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help     显示此帮助信息"
    echo "  -c, --clean    清理构建目录"
    echo "  --rust-only    仅编译 Rust 二进制文件"
    echo "  --app-only     仅编译 Tauri 应用"
    echo "  --dev          开发模式编译"
    echo "  --release      发布模式编译 (默认)"
    echo ""
    echo "示例:"
    echo "  $0                # 完整编译"
    echo "  $0 --rust-only    # 仅编译 Rust 二进制文件"
    echo "  $0 --clean        # 清理后编译"
}

# 主函数
main() {
    echo "🚀 AI Review 编译脚本"
    echo "===================="

    # 解析命令行参数
    CLEAN=false
    RUST_ONLY=false
    APP_ONLY=false
    DEV_MODE=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            --rust-only)
                RUST_ONLY=true
                shift
                ;;
            --app-only)
                APP_ONLY=true
                shift
                ;;
            --dev)
                DEV_MODE=true
                shift
                ;;
            --release)
                DEV_MODE=false
                shift
                ;;
            *)
                print_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # 检查依赖
    check_dependencies

    # 清理构建目录
    if [ "$CLEAN" = true ]; then
        clean_build
    fi

    # 安装前端依赖
    if [ "$RUST_ONLY" = false ]; then
        install_frontend_deps
    fi

    # 编译
    if [ "$APP_ONLY" = false ]; then
        build_rust
    fi

    if [ "$RUST_ONLY" = false ]; then
        build_tauri
    fi

    # 创建发布目录
    if [ "$RUST_ONLY" = false ] && [ "$APP_ONLY" = false ]; then
        create_release_dir
    fi

    print_success "编译完成！"

    if [ "$DEV_MODE" = false ]; then
        print_info "发布文件位于 release/ 目录"
    fi
}

# 运行主函数
main "$@"
