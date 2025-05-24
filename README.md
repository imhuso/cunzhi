# AI Review - 智能代码审查助手

一个基于 Rust + Tauri 构建的跨平台聊天窗口应用，支持命令行与UI实时交互。

## ✨ 主要功能

- 🖥️ **跨平台桌面应用** - 基于 Tauri 框架，支持 Windows、macOS、Linux
- 📡 **命令行与UI通信** - 通过 IPC (进程间通信) 实现实时消息传递
- 🔔 **系统级通知** - 收到新消息时显示系统通知
- 🪟 **智能弹窗** - 自动创建置顶的快速回复窗口
- ⏱️ **可配置超时** - 支持自定义等待回复的最大时间
- ⌨️ **快捷键支持** - Ctrl/Cmd+Enter 发送，Escape 取消
- 🎯 **实时倒计时** - 显示剩余回复时间

### 🎨 Vue 3 界面新特性

- ✨ **现代化设计** - 渐变背景、卡片式布局、流畅动画
- 📱 **响应式界面** - 适配不同窗口大小和设备
- 🚨 **智能提醒** - 剩余时间不足时红色闪烁警告
- 📝 **字符计数** - 实时显示输入长度，超长提醒
- 🎮 **完整快捷键** - 全键盘操作支持
- 🔄 **状态管理** - 清晰的等待、处理、紧急状态显示

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Node.js (可选，用于前端开发)
- 系统依赖：
  - **macOS**: 无需额外依赖
  - **Linux**: `webkit2gtk-4.0-dev`, `libappindicator3-dev`
  - **Windows**: 无需额外依赖

### 编译项目

```bash
# 克隆项目
git clone <repository-url>
cd ai-review

# 安装前端依赖
npm install

# 构建前端
npm run build

# 编译Rust后端
cargo build --release
```

### 运行应用

#### 1. 启动UI应用

```bash
# 开发模式
cargo run --bin ai-review-ui

# 或使用编译后的二进制文件
./target/release/ai-review-ui
```

#### 2. 使用命令行工具发送消息

```bash
# 基本用法 (默认30秒超时)
./target/release/ai-review-cli "请帮我审查这段代码"

# 自定义超时时间
./target/release/ai-review-cli "这是一个测试消息" --timeout 60

# 查看帮助
./target/release/ai-review-cli --help
```

## 📖 使用说明

### 命令行工具参数

```bash
ai-review-cli [OPTIONS] <MESSAGE>

参数:
  <MESSAGE>          要发送给UI应用的消息

选项:
  -t, --timeout <SECONDS>  超时时间（秒），默认30秒
  -h, --help              显示帮助信息
  -V, --version           显示版本信息
```

### UI界面操作

1. **接收消息**: 当命令行发送消息时，会：
   - 显示系统通知
   - 弹出置顶的快速回复窗口
   - 显示消息内容和剩余时间

2. **回复消息**:
   - 在文本框中输入回复
   - 点击"发送回复"或按 `Ctrl/Cmd + Enter`
   - 窗口会自动关闭

3. **取消操作**:
   - 点击"取消"按钮或按 `Escape` 键
   - 命令行会收到取消消息

### 快捷键

- `Ctrl/Cmd + Enter`: 发送回复
- `Escape`: 取消当前请求

## 🏗️ 项目结构

```
ai-review/
├── src/
│   ├── main.rs          # UI应用主程序
│   ├── cli.rs           # 命令行工具
│   └── ipc.rs           # 进程间通信模块
├── dist/
│   └── index.html       # 前端界面
├── Cargo.toml           # Rust项目配置
├── tauri.conf.json      # Tauri应用配置
├── test.sh              # 测试脚本
└── README.md            # 项目说明
```

## 🧪 测试

运行测试脚本来验证功能：

```bash
./test.sh
```

测试脚本会：
1. 自动编译项目（如果需要）
2. 运行多个测试用例
3. 验证不同超时时间的功能

## 🔧 技术实现

### 核心技术栈

- **后端**: Rust + Tauri
- **前端**: Vue 3 + Vite (现代化前端框架)
- **通信**: Unix Domain Socket (IPC)
- **通知**: notify-rust + Tauri 通知系统

### 关键特性

1. **IPC通信**: 使用 Unix Domain Socket 实现命令行与UI的双向通信
2. **异步处理**: 基于 Tokio 的异步运行时
3. **窗口管理**: 动态创建和管理弹窗窗口
4. **状态管理**: 使用 Tauri 的状态管理系统
5. **超时控制**: 支持自定义超时时间和实时倒计时

## 📝 开发说明

### 添加新功能

1. **后端功能**: 在 `src/main.rs` 中添加新的 Tauri 命令
2. **前端功能**: 在 `dist/index.html` 中添加 JavaScript 代码
3. **IPC扩展**: 在 `src/ipc.rs` 中扩展消息类型

### 调试模式

```bash
# 启用详细日志
RUST_LOG=debug cargo run --bin ai-review-ui

# 启用 Rust 回溯
RUST_BACKTRACE=1 cargo run --bin ai-review-ui
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 🔗 相关链接

- [Tauri 官方文档](https://tauri.app/)
- [Rust 官方网站](https://www.rust-lang.org/)
- [项目仓库](https://github.com/your-username/ai-review)

---

**注意**: 确保在使用命令行工具之前先启动UI应用，否则会出现连接错误。
