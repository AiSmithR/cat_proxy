# Cat Proxy 跨平台构建指南

本指南将帮助您在 Windows、macOS 和 Linux 三个平台上构建 Cat Proxy 应用程序。

## 📋 目录

- [前提条件](#前提条件)
- [macOS 构建](#macos-构建)
- [Windows 构建](#windows-构建)
- [Linux 构建](#linux-构建)
- [常见问题](#常见问题)

---

## 🔧 前提条件

### 所有平台共同要求

1. **Rust 工具链** (1.70+)
   ```bash
   # 安装 Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # 验证安装
   rustc --version
   cargo --version
   ```

2. **Node.js 和 npm** (18+)
   ```bash
   # 验证安装
   node --version
   npm --version
   ```

3. **项目源代码**
   ```bash
   git clone <your-repo-url>
   cd cat_proxy
   ```

---

## 🍎 macOS 构建

### 系统要求
- macOS 10.13 (High Sierra) 或更高版本
- Xcode Command Line Tools

### 步骤 1: 安装依赖

```bash
# 安装 Xcode Command Line Tools (如果还没安装)
xcode-select --install

# 验证安装
xcode-select -p
```

### 步骤 2: 构建前端

```bash
cd ui-react
npm install
npm run build
cd ..
```

### 步骤 3: 构建后端

```bash
# 构建 release 版本
cargo build --release --bin cat_proxy_gui --features gui

# 生成的可执行文件位于:
# ./target/release/cat_proxy_gui
```

### 步骤 4: 运行应用

```bash
./target/release/cat_proxy_gui
```

### 可选: 创建 .app 包

```bash
# 安装 Tauri CLI
npm install -g @tauri-apps/cli

# 或使用项目本地的 Tauri
cd ui-react
npm install

# 创建 .app 和 .dmg 包
npm run tauri build

# 生成的包位于:
# ./target/release/bundle/macos/Cat Proxy.app
# ./target/release/bundle/dmg/Cat Proxy_0.1.0_aarch64.dmg
```

---

## 🪟 Windows 构建

### 系统要求
- Windows 10/11
- Visual Studio 2019/2022 (Build Tools 或完整版)
- WebView2 (Windows 11 已预装)

### 步骤 1: 安装 Visual Studio Build Tools

1. 下载 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
2. 安装时选择 "C++ build tools"
3. 确保包含以下组件：
   - MSVC v142+ (x64/x86)
   - Windows SDK 10/11

### 步骤 2: 安装 WebView2

```powershell
# Windows 11 通常已预装
# Windows 10 需要下载安装
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

### 步骤 3: 构建前端

```powershell
cd ui-react
npm install
npm run build
cd ..
```

### 步骤 4: 构建后端

```powershell
# 构建 release 版本
cargo build --release --bin cat_proxy_gui --features gui

# 生成的可执行文件位于:
# .\target\release\cat_proxy_gui.exe
```

### 步骤 5: 运行应用

```powershell
.\target\release\cat_proxy_gui.exe
```

### 可选: 创建 MSI 安装包

```powershell
# 安装 WiX Toolset
# 下载地址: https://wixtoolset.org/releases/

# 使用 Tauri 创建安装包
cd ui-react
npm run tauri build

# 生成的包位于:
# .\target\release\bundle\msi\Cat Proxy_0.1.0_x64.msi
```

---

## 🐧 Linux 构建

### 支持的发行版
- Ubuntu 20.04+
- Debian 11+
- Fedora 35+
- Arch Linux

### 步骤 1: 安装系统依赖

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### Fedora

```bash
sudo dnf install \
    webkit2gtk3-devel \
    openssl-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

#### Arch Linux

```bash
sudo pacman -S \
    webkit2gtk \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    gtk3 \
    libappindicator-gtk3 \
    librsvg
```

### 步骤 2: 构建前端

```bash
cd ui-react
npm install
npm run build
cd ..
```

### 步骤 3: 构建后端

```bash
# 构建 release 版本
cargo build --release --bin cat_proxy_gui --features gui

# 生成的可执行文件位于:
# ./target/release/cat_proxy_gui
```

### 步骤 4: 运行应用

```bash
./target/release/cat_proxy_gui
```

### 可选: 创建 DEB/AppImage 包

```bash
cd ui-react
npm run tauri build

# 生成的包位于:
# ./target/release/bundle/deb/cat-proxy_0.1.0_amd64.deb
# ./target/release/bundle/appimage/cat-proxy_0.1.0_amd64.AppImage
```

---

## 🔍 常见问题

### Q1: 编译时出现 "linker 'cc' not found"

**Windows:**
- 确保安装了 Visual Studio Build Tools
- 重新运行安装程序，确保选择了 C++ 工具

**Linux:**
```bash
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf groupinstall "Development Tools"  # Fedora
```

**macOS:**
```bash
xcode-select --install
```

### Q2: 前端构建失败

```bash
# 清理并重新安装依赖
cd ui-react
rm -rf node_modules package-lock.json
npm install
npm run build
```

### Q3: Tauri 版本不匹配错误

确保前端和后端的 Tauri 版本一致：

```bash
# 检查版本
cat ui-react/package.json | grep "@tauri-apps"
cat Cargo.toml | grep "tauri"

# 应该都是 1.5.x 或 2.x.x (但不能混用)
```

### Q4: macOS 上出现"无法打开应用"的提示

```bash
# 移除隔离属性
xattr -cr "./target/release/bundle/macos/Cat Proxy.app"

# 或者在系统偏好设置 > 安全性与隐私中允许
```

### Q5: Linux 上缺少 WebKit2GTK

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.0-37

# Fedora
sudo dnf install webkit2gtk3

# Arch
sudo pacman -S webkit2gtk
```

### Q6: 构建时内存不足

```bash
# 限制并行编译任务数
cargo build --release --jobs 2
```

### Q7: Windows 上找不到 WebView2

下载并安装: https://go.microsoft.com/fwlink/p/?LinkId=2124703

---

## 📦 构建产物说明

### macOS
- **二进制文件**: `target/release/cat_proxy_gui`
- **应用包**: `target/release/bundle/macos/Cat Proxy.app`
- **DMG 安装镜像**: `target/release/bundle/dmg/Cat Proxy_*.dmg`

### Windows
- **可执行文件**: `target\release\cat_proxy_gui.exe`
- **MSI 安装包**: `target\release\bundle\msi\Cat Proxy_*.msi`

### Linux
- **二进制文件**: `target/release/cat_proxy_gui`
- **DEB 包**: `target/release/bundle/deb/cat-proxy_*.deb`
- **AppImage**: `target/release/bundle/appimage/cat-proxy_*.AppImage`

---

## 🚀 快速构建命令总结

### macOS
```bash
cd ui-react && npm install && npm run build && cd ..
cargo build --release --bin cat_proxy_gui --features gui
./target/release/cat_proxy_gui
```

### Windows
```powershell
cd ui-react; npm install; npm run build; cd ..
cargo build --release --bin cat_proxy_gui --features gui
.\target\release\cat_proxy_gui.exe
```

### Linux
```bash
cd ui-react && npm install && npm run build && cd ..
cargo build --release --bin cat_proxy_gui --features gui
./target/release/cat_proxy_gui
```

---

## 📝 注意事项

1. **首次构建时间较长**: Rust release 构建需要 5-10 分钟，请耐心等待
2. **磁盘空间**: 确保至少有 5GB 可用空间用于依赖和构建产物
3. **网络连接**: 首次构建需要下载大量依赖，确保网络畅通
4. **跨平台限制**: 某些平台特定的包只能在对应平台上构建:
   - **.dmg** 只能在 macOS 上构建
   - **.msi** 只能在 Windows 上构建
   - **.deb/.rpm** 只能在 Linux 上构建

---

## 🆘 获取帮助

如果遇到问题:

1. 查看项目 Issues: [GitHub Issues](https://github.com/your-repo/issues)
2. 查看 Tauri 文档: https://tauri.app/
3. Rust 文档: https://doc.rust-lang.org/

---

**祝您构建顺利！** 🎉
