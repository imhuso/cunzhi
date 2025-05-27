# AI Review

AI Review 是一个基于 Tauri 的桌面应用程序，提供 AI 代码审查和交互功能。

## ✨ 特性

- 🎯 **MCP 标准兼容**: 完全符合 MCP 2024-11-05 协议规范
- 🎨 **美观的用户界面**: 使用 Vue 3 + Ant Design Vue + Tailwind CSS 构建
- 🔒 **安全的 Markdown 渲染**: 防止 XSS 攻击的安全渲染
- 🖥️ **原生应用体验**: 基于 Tauri 的跨平台桌面应用
- ⚡ **高效通信**: 使用进程间通信，不依赖文件监听
- 🛠️ **灵活配置**: 支持预定义选项和自由文本输入
- 🧠 **全局记忆管理**: 智能存储和管理开发规范、用户偏好和最佳实践

## 🚀 快速开始

### 方式一：全局安装（推荐）

```bash
# 1. 构建项目
cargo build --release

# 2. 全局安装（需要管理员权限）
./install.sh

# 3. 验证安装
which ai-review-ui
which ai-review-mcp

# 4. 启动 MCP 服务器
ai-review-mcp
```

### 方式二：本地运行

```bash
# 1. 构建项目
cargo build --release

# 2. 构建前端（可选，用于 Tauri 版本）
pnpm install
pnpm build

# 3. 启动 MCP 服务器
./target/release/ai-review-mcp
```

### 测试系统

```bash
# 完整测试
python3 test_mcp.py

# 只测试弹窗
python3 test_mcp.py --popup-only

# 只测试 MCP 服务器
python3 test_mcp.py --mcp-only
```

### 卸载

```bash
# 卸载全局安装的文件
./uninstall.sh
```

## 📋 工作流程

### 1. 弹窗系统架构

```
AI IDE/Claude Desktop
        ↓ (MCP 调用)
AI Review MCP 服务器
        ↓ (直接调用)
Tauri 应用 (Vue + Ant Design)
        ↓ (用户交互)
响应返回给 MCP 服务器
```

### 2. 通信方式

- **MCP 协议**: JSON-RPC 2.0 over stdio
- **弹窗启动**: 直接调用 Tauri 应用
- **用户界面**: 原生 Tauri 应用 (Vue + Ant Design)
- **响应传递**: 进程间通信，无文件依赖

### 3. 支持的功能

#### ai_review_chat 工具

```json
{
  "name": "ai_review_chat",
  "description": "AI Review 智能代码审查交互工具，支持预定义选项和自由文本输入",
  "inputSchema": {
    "type": "object",
    "properties": {
      "message": {
        "type": "string",
        "description": "要显示给用户的消息"
      },
      "predefined_options": {
        "type": "array",
        "items": {"type": "string"},
        "description": "预定义的选项列表（可选，支持多选）"
      },
      "is_markdown": {
        "type": "boolean",
        "description": "消息是否为Markdown格式",
        "default": false
      }
    },
    "required": ["message"]
  }
}
```

**功能特性**：
- ✅ 预定义选项多选支持
- ✅ 自由文本输入
- ✅ Markdown 格式消息渲染
- ✅ 附加消息功能
- ✅ 美观的现代化UI界面
- ✅ 响应式设计，适配不同屏幕尺寸

### 2. memory_manager - 全局记忆管理工具

```json
{
  "name": "memory_manager",
  "description": "全局记忆管理工具，用于存储和管理重要的开发规范、用户偏好和最佳实践",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action": {
        "type": "string",
        "enum": ["add", "get_project_info"],
        "description": "操作类型：add(添加记忆), get_project_info(获取项目信息)"
      },
      "content": {
        "type": "string",
        "description": "记忆内容（add操作时必需）"
      },
      "category": {
        "type": "string",
        "enum": ["rule", "preference", "pattern", "context"],
        "description": "记忆分类：rule(规范规则), preference(用户偏好), pattern(最佳实践), context(项目上下文)"
      },
      "project_path": {
        "type": "string",
        "description": "项目路径（必需）"
      }
    },
    "required": ["action", "project_path"]
  }
}
```

**功能特性**：
- ✅ 智能分类存储（规范、偏好、模式、上下文）
- ✅ 项目级别记忆隔离
- ✅ Markdown格式存储
- ✅ 压缩显示项目记忆

**记忆分类说明**：
- **rule**: 开发规范和规则（如代码风格、命名约定）
- **preference**: 用户偏好设置（如技术选择、UI偏好）
- **pattern**: 常用模式和最佳实践（如设计模式、架构模式）
- **context**: 项目上下文信息（如项目背景、特殊要求）

**使用示例**：
```bash
# 添加开发规范
memory_manager(action="add", content="使用中文回复，代码使用英文", category="rule", project_path="/path/to/project")

# 获取项目记忆信息
memory_manager(action="get_project_info", project_path="/path/to/project")

# 添加用户偏好
memory_manager(action="add", content="用户偏好使用Vue框架", category="preference", project_path="/path/to/project")
```

**智能记忆处理**：
当用户输入包含"请记住："时，AI会自动总结内容并调用memory_manager添加记忆。

## 🔧 配置

### Claude Desktop 配置

#### 全局安装后（推荐）

在 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "ai-review": {
      "command": "ai-review-mcp",
      "args": []
    }
  }
}
```

#### 本地运行

在 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "ai-review": {
      "command": "/path/to/ai-review/target/release/ai-review-mcp",
      "args": []
    }
  }
}
```

### Cursor 配置

#### 全局安装后（推荐）

在 Cursor 的 MCP 设置中添加：

```json
{
  "name": "ai-review",
  "command": "ai-review-mcp"
}
```

#### 本地运行

在 Cursor 的 MCP 设置中添加：

```json
{
  "name": "ai-review",
  "command": "/path/to/ai-review/target/release/ai-review-mcp"
}
```

## 📁 项目结构

```
ai-review/
├── src/
│   ├── main.rs              # Tauri 主应用
│   ├── mcp_server_bin.rs    # MCP 服务器实现
│   ├── components/
│   │   ├── McpPopup.vue     # MCP 弹窗组件
│   │   └── SettingsModal.vue # 设置弹窗
│   └── App.vue              # 主应用组件
├── install.sh              # 全局安装脚本
├── uninstall.sh            # 卸载脚本
├── claude_desktop_config.json # Claude Desktop 配置示例
├── Cargo.toml              # Rust 依赖
├── package.json            # 前端依赖
└── README.md               # 项目文档
```

## 🧪 测试

### 手动测试

```bash
# 创建测试请求
echo '{
  "id": "test",
  "message": "# 🎉 代码审查完成\n\n## 发现的问题\n\n1. **安全问题**: 已修复XSS漏洞\n2. **性能问题**: 已优化文件监听\n3. **代码质量**: 已重构重复逻辑\n\n## 建议的下一步\n\n请选择您希望的操作：",
  "predefined_options": ["✅ 立即部署", "📝 查看详情", "⏰ 稍后处理", "❌ 取消"],
  "is_markdown": true
}' > test_request.json

# 启动弹窗
./target/release/ai-review-ui --mcp-request test_request.json
```

## 🔍 故障排除

### 常见问题

1. **弹窗无法启动**
   - 确保 `popup.html` 文件存在
   - 检查 Python 3 是否安装
   - 确认浏览器可以打开本地文件

2. **MCP 服务器无响应**
   - 检查可执行文件是否存在：`./target/release/ai-review-mcp`
   - 运行 `cargo build --release` 重新构建
   - 查看错误日志

3. **依赖问题**
   - 弹窗使用 CDN 依赖，需要网络连接
   - 如果网络受限，可以下载依赖到本地

### 调试模式

```bash
# 启用详细日志
RUST_LOG=debug ./target/release/ai-review-mcp
```

## 🛠️ 开发

### 添加新功能

1. **修改 MCP 服务器**: 编辑 `src/mcp_server_bin.rs`
2. **更新弹窗界面**: 编辑 `src/components/McpPopup.vue`
3. **测试更改**: 运行 `cargo build --release` 重新编译

### 构建发布版本

```bash
# 构建优化版本
cargo build --release

# 构建 Tauri 应用
npm run tauri:build
```

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如有问题，请创建 GitHub Issue 或联系维护者。
