#!/bin/bash

# AI Review 安装脚本
# 将编译后的二进制文件安装到系统路径

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查是否为macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}错误: 此脚本目前只支持 macOS${NC}"
    exit 1
fi

# 检查是否已编译
if [ ! -f "target/release/ai-review-ui" ] || [ ! -f "target/release/ai-review-mcp" ]; then
    echo -e "${YELLOW}警告: 未找到编译后的二进制文件${NC}"
    echo -e "${BLUE}正在编译项目...${NC}"
    cargo build --release
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}错误: 编译失败${NC}"
        exit 1
    fi
fi

# 安装目录
INSTALL_DIR="/usr/local/bin"

echo -e "${BLUE}AI Review 安装程序${NC}"
echo "================================"
echo -e "将要安装到: ${GREEN}$INSTALL_DIR${NC}"
echo -e "安装文件:"
echo -e "  • ${GREEN}ai-review-ui${NC}  - UI 弹窗程序"
echo -e "  • ${GREEN}ai-review-mcp${NC} - MCP 服务器"
echo ""

# 检查权限
if [ ! -w "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}需要管理员权限来安装到 $INSTALL_DIR${NC}"
    echo "请输入密码以继续..."
    SUDO="sudo"
else
    SUDO=""
fi

# 复制文件
echo -e "${BLUE}正在安装...${NC}"

# 如果目标文件已存在，先删除以避免"identical"错误
if [ -f "$INSTALL_DIR/ai-review-ui" ] || [ -L "$INSTALL_DIR/ai-review-ui" ]; then
    echo -e "${YELLOW}移除现有的 ai-review-ui...${NC}"
    $SUDO rm -f "$INSTALL_DIR/ai-review-ui"
fi

$SUDO cp target/release/ai-review-ui "$INSTALL_DIR/"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC} ai-review-ui 安装成功"
else
    echo -e "${RED}✗${NC} ai-review-ui 安装失败"
    exit 1
fi

if [ -f "$INSTALL_DIR/ai-review-mcp" ] || [ -L "$INSTALL_DIR/ai-review-mcp" ]; then
    echo -e "${YELLOW}移除现有的 ai-review-mcp...${NC}"
    $SUDO rm -f "$INSTALL_DIR/ai-review-mcp"
fi

$SUDO cp target/release/ai-review-mcp "$INSTALL_DIR/"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC} ai-review-mcp 安装成功"
else
    echo -e "${RED}✗${NC} ai-review-mcp 安装失败"
    exit 1
fi

# 设置执行权限
$SUDO chmod +x "$INSTALL_DIR/ai-review-ui"
$SUDO chmod +x "$INSTALL_DIR/ai-review-mcp"

echo ""
echo -e "${GREEN}🎉 安装完成！${NC}"
echo ""
echo -e "${BLUE}使用方法:${NC}"
echo -e "  • 直接运行: ${GREEN}ai-review-ui${NC}"
echo -e "  • MCP 服务器: ${GREEN}ai-review-mcp${NC}"
echo ""
echo -e "${BLUE}验证安装:${NC}"
echo -e "  ${GREEN}which ai-review-ui${NC}"
echo -e "  ${GREEN}which ai-review-mcp${NC}"
echo ""

# 验证安装
if command -v ai-review-ui >/dev/null 2>&1 && command -v ai-review-mcp >/dev/null 2>&1; then
    echo -e "${GREEN}✓ 验证成功: 所有命令都可以在系统路径中找到${NC}"
else
    echo -e "${YELLOW}⚠ 警告: 某些命令可能无法在当前 shell 中找到，请重新打开终端或运行 'source ~/.zshrc'${NC}"
fi
