# AI Review 编译指南

本文档介绍如何编译和构建 AI Review 应用。

## 快速开始

### 使用 Make (推荐)
```bash
make help          # 查看所有命令
make dev           # 开发模式
make build-cli     # 仅编译 CLI
make build         # 完整编译
```

### 开发模式
```bash
./dev.sh
```
启动开发环境，包含热重载功能。

### 快速编译
```bash
./quick-build.sh
```
快速编译发布版本。

### 完整编译
```bash
./build.sh
```
完整编译，包含清理、依赖检查等步骤。

## 编译脚本说明

### 1. `dev.sh` - 开发模式
- 自动安装前端依赖（如果需要）
- 编译 CLI 工具（开发模式）
- 启动 Tauri 开发服务器
- 支持热重载

### 2. `quick-build.sh` - 快速编译
- 编译 CLI 工具（发布模式）
- 编译 Tauri 应用（发布模式）
- 显示输出文件位置和大小

### 3. `build.sh` - 完整编译
功能最全面的编译脚本，支持多种选项：

#### 基本用法
```bash
./build.sh                # 完整编译
./build.sh --clean        # 清理后编译
./build.sh --cli-only     # 仅编译 CLI
./build.sh --app-only     # 仅编译应用
./build.sh --help         # 显示帮助
```

#### 功能特性
- ✅ 依赖检查
- 🧹 清理构建目录
- 📦 自动安装前端依赖
- 🛠️ 编译 CLI 工具
- 🖥️ 编译 Tauri 应用
- 📁 创建发布目录
- 📊 显示文件大小统计

## 手动编译

如果您不想使用脚本，也可以手动编译：

### 编译 CLI 工具
```bash
# 开发模式
cargo build --bin ai-review-cli

# 发布模式
cargo build --release --bin ai-review-cli
```

### 编译 Tauri 应用
```bash
# 安装前端依赖
npm install

# 开发模式
npm run tauri dev

# 发布模式
npm run tauri build
```

## 输出文件

### CLI 工具
- 开发模式: `target/debug/ai-review-cli`
- 发布模式: `target/release/ai-review-cli`

### Tauri 应用
- macOS: `src-tauri/target/release/bundle/macos/`
- Windows: `src-tauri/target/release/bundle/msi/`
- Linux: `src-tauri/target/release/bundle/deb/` 或 `bundle/appimage/`

### 发布目录
使用 `./build.sh` 完整编译后，所有发布文件会复制到 `release/` 目录。

## 系统要求

### 开发环境
- Rust 1.70+
- Node.js 18+
- npm 或 yarn

### macOS 额外要求
- Xcode Command Line Tools

### Linux 额外要求
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk3-devel.x86_64 \
    openssl-devel \
    curl \
    wget \
    libappindicator-gtk3 \
    librsvg2-devel

# Arch Linux
sudo pacman -S webkit2gtk \
    base-devel \
    curl \
    wget \
    openssl \
    appmenu-gtk-module \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    libvips
```

## 故障排除

### 常见问题

1. **Rust 编译错误**
   ```bash
   rustup update
   ```

2. **前端依赖问题**
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   ```

3. **Tauri 构建失败**
   ```bash
   npm run tauri info  # 检查环境
   ```

4. **权限问题**
   ```bash
   chmod +x *.sh
   ```

### 清理构建缓存
```bash
# 清理 Rust 缓存
cargo clean

# 清理前端缓存
rm -rf node_modules dist

# 使用脚本清理
./build.sh --clean
```

## 性能优化

### 编译优化
在 `Cargo.toml` 中已配置了发布模式优化：
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 减小文件大小
```bash
# 使用 strip 减小二进制文件大小
strip target/release/ai-review-cli
```

## 持续集成

可以在 CI/CD 中使用这些脚本：
```yaml
# GitHub Actions 示例
- name: Build Application
  run: ./build.sh --release
```
