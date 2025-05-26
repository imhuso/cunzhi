#!/bin/bash

# AI Review 卸载脚本
# 从系统路径中移除 AI Review 相关文件

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

# 安装目录
INSTALL_DIR="/usr/local/bin"

echo -e "${BLUE}AI Review 卸载程序${NC}"
echo "================================"
echo -e "将要从以下位置移除文件: ${YELLOW}$INSTALL_DIR${NC}"
echo ""

# 检查文件是否存在
FILES_TO_REMOVE=()
if [ -f "$INSTALL_DIR/ai-review-ui" ]; then
    FILES_TO_REMOVE+=("ai-review-ui")
fi
if [ -f "$INSTALL_DIR/ai-review-mcp" ]; then
    FILES_TO_REMOVE+=("ai-review-mcp")
fi

if [ ${#FILES_TO_REMOVE[@]} -eq 0 ]; then
    echo -e "${YELLOW}没有找到需要卸载的文件${NC}"
    exit 0
fi

echo -e "找到以下文件:"
for file in "${FILES_TO_REMOVE[@]}"; do
    echo -e "  • ${RED}$file${NC}"
done
echo ""

# 确认卸载
read -p "确定要卸载这些文件吗? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}取消卸载${NC}"
    exit 0
fi

# 检查权限
if [ ! -w "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}需要管理员权限来从 $INSTALL_DIR 删除文件${NC}"
    echo "请输入密码以继续..."
    SUDO="sudo"
else
    SUDO=""
fi

# 删除文件
echo -e "${BLUE}正在卸载...${NC}"

for file in "${FILES_TO_REMOVE[@]}"; do
    $SUDO rm -f "$INSTALL_DIR/$file"
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $file 已移除"
    else
        echo -e "${RED}✗${NC} $file 移除失败"
    fi
done

echo ""
echo -e "${GREEN}🗑️  卸载完成！${NC}"
echo ""

# 验证卸载
REMAINING_FILES=()
for file in "${FILES_TO_REMOVE[@]}"; do
    if command -v "$file" >/dev/null 2>&1; then
        REMAINING_FILES+=("$file")
    fi
done

if [ ${#REMAINING_FILES[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ 验证成功: 所有文件都已从系统路径中移除${NC}"
else
    echo -e "${YELLOW}⚠ 警告: 以下文件仍然可以在系统路径中找到:${NC}"
    for file in "${REMAINING_FILES[@]}"; do
        echo -e "  • ${YELLOW}$file${NC}"
    done
fi
