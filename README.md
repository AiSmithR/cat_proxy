# Cat Proxy

<div align="center">

**高性能跨平台代理软件**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

[English](README.md) | [中文](README_CN.md)

</div>

## ✨ 特性

- 🚀 **高性能**: 基于 Tokio 异步运行时，零成本抽象
- 🌍 **跨平台**: 支持 macOS、Linux、Windows
- 🔐 **多协议**: HTTP/HTTPS、SOCKS5、Shadowsocks、VMess、Trojan
- 🎯 **智能路由**: 规则匹配 + Fake-IP + DNS 缓存
- ⚖️ **负载均衡**: URL 测试、轮询、随机、故障转移
- 💚 **健康检查**: 自动检测节点状态和延迟
- 🖥️ **系统集成**: 自动配置系统代理
- 📊 **实时监控**: RESTful API + 流量统计
- ⚡ **TUN 模式**: 透明代理支持（完整实现）
- 🔄 **订阅管理**: 自动更新代理节点
- 🎨 **GUI 界面**: 基于 Tauri 的现代化图形界面（可选）

## 📦 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/yourusername/cat_proxy.git
cd cat_proxy

# 编译
cargo build --release

# 编译后的二进制文件在 target/release/cat_proxy
```

### 使用 Cargo 安装

```bash
cargo install cat_proxy
```

## 🚀 快速开始

### 1. 创建配置文件

```bash
# 复制示例配置
cp config.example.yaml config.yaml

# 编辑配置文件
vim config.yaml
```

### 2. 测试配置

```bash
cat_proxy test --config config.yaml
```

### 3. 启动服务

```bash
# 直接启动
cat_proxy

# 启动并设置系统代理
cat_proxy start --set-system-proxy

# 指定配置文件和日志级别
cat_proxy --config config.yaml --log-level debug
```

### 4. 管理系统代理

```bash
# 启用系统代理
cat_proxy set-proxy --enable --http-port 7890 --socks-port 7891

# 禁用系统代理
cat_proxy set-proxy
```

### 5. 使用 GUI 界面（可选）

```bash
# 编译 GUI 版本
cargo build --features gui --bin cat_proxy_gui --release

# 运行 GUI
./target/release/cat_proxy_gui

# 或使用 Tauri 开发模式
cargo install tauri-cli
cargo tauri dev
```

GUI 功能特性：
- 📊 实时仪表板（连接统计、流量监控）
- 📈 流量趋势图表（Chart.js 可视化）
- 🎛️ 可视化控制（启动/停止代理）
- ⚙️ 配置管理界面
- 🔍 代理节点测试和延迟显示
- 📝 内置规则编辑器（添加/删除/编辑规则）
- 🔗 实时连接查看（查看活跃连接详情）
- 🎨 主题切换（亮色/暗色模式）
- 📱 响应式设计

详细的 GUI 开发指南请参考 [GUI_README.md](GUI_README.md)

## 📝 命令行选项

```
Cat Proxy - 高性能跨平台代理软件

Usage: cat_proxy [OPTIONS] [COMMAND]

Commands:
  start      启动代理服务器
  stop       停止代理服务器
  restart    重启代理服务器
  test       测试配置文件
  update     更新订阅
  set-proxy  设置系统代理
  version    显示版本信息
  help       显示帮助信息

Options:
  -c, --config <FILE>          配置文件路径 [default: config.yaml]
  -l, --log-level <LOG_LEVEL>  日志级别 [default: info]
  -d, --daemon                 后台运行模式
  -h, --help                   打印帮助信息
  -V, --version                打印版本信息
```

## ⚙️ 配置说明

### 基础配置

```yaml
# HTTP 代理端口
port: 7890

# SOCKS5 代理端口
socks-port: 7891

# 是否允许局域网连接
allow-lan: false

# 代理模式: rule (规则) / global (全局) / direct (直连)
mode: rule

# 日志级别
log-level: info
```

### 代理节点

```yaml
proxies:
  # Shadowsocks
  - name: "SS-HK"
    type: ss
    server: server.example.com
    port: 8388
    cipher: aes-256-gcm
    password: your-password

  # VMess
  - name: "VMess-US"
    type: vmess
    server: server.example.com
    port: 443
    uuid: your-uuid-here
    alterId: 0

  # Trojan
  - name: "Trojan-JP"
    type: trojan
    server: server.example.com
    port: 443
    password: your-password
```

### 规则配置

```yaml
rules:
  # 直连规则
  - DOMAIN-SUFFIX,cn,DIRECT
  - IP-CIDR,192.168.0.0/16,DIRECT
  
  # 代理规则
  - DOMAIN-SUFFIX,google.com,Proxy
  - DOMAIN-KEYWORD,youtube,Proxy
  
  # 拦截规则
  - DOMAIN-SUFFIX,ad.com,REJECT
  
  # 默认规则
  - MATCH,Proxy
```

## 🔌 API 接口

Cat Proxy 提供 RESTful API 用于监控和管理：

```bash
# 获取版本信息
GET /api/version

# 获取配置
GET /api/configs

# 获取所有连接
GET /api/connections

# 获取流量统计
GET /api/traffic

# 关闭所有连接
DELETE /api/connections
```

## 🏗️ 架构设计

```
cat_proxy/
├── src/
│   ├── api/           # RESTful API 模块
│   ├── config/        # 配置管理
│   ├── core/          # 核心代理逻辑
│   ├── dns/           # DNS 解析器
│   ├── protocols/     # 协议实现
│   │   ├── http.rs
│   │   ├── socks5.rs
│   │   ├── shadowsocks.rs
│   │   ├── vmess.rs
│   │   └── trojan.rs
│   ├── rules/         # 规则引擎
│   ├── system/        # 系统集成
│   ├── tun/           # TUN 模式
│   │   ├── mod.rs         # TUN 设备管理
│   │   ├── packet.rs      # IP 数据包解析
│   │   ├── tcp_handler.rs # TCP 连接处理
│   │   └── udp_handler.rs # UDP 会话管理
│   └── main.rs        # 主程序入口
├── config.example.yaml
└── README.md
```

## 🔧 技术栈

- **语言**: Rust 2021 Edition
- **异步运行时**: Tokio
- **HTTP 框架**: Axum + Tower
- **DNS**: Hickory Resolver
- **加密**: AES-GCM, ChaCha20-Poly1305, HMAC-SHA256
- **序列化**: Serde (JSON/YAML)
- **日志**: tracing + tracing-subscriber

## 📊 性能

- **并发连接**: 支持数万并发连接
- **内存占用**: < 50MB (空闲状态)
- **CPU 使用**: 极低（异步 I/O + 零拷贝）
- **延迟**: 毫秒级转发延迟
- **吞吐量**: 零拷贝优化，接近原生网络性能

### 性能优化特性

- ✅ **零拷贝数据转发**: 使用 `tokio::io::copy_bidirectional` 实现高效数据转发
- ✅ **缓冲区池化**: 复用缓冲区，减少内存分配开销
- ✅ **TCP 优化**: TCP_NODELAY + TCP_KEEPALIVE
- ✅ **智能缓冲区**: 根据场景自动选择缓冲区大小（4KB-256KB）
- ✅ **连接池管理**: 高效的连接跟踪和流量统计

## 🛣️ 开发路线图

- [x] 基础代理功能（HTTP/SOCKS5）
- [x] 协议支持（SS/VMess/Trojan）
- [x] 规则引擎
- [x] DNS 优化（Fake-IP + 缓存）
- [x] 系统代理集成
- [x] RESTful API
- [x] TUN 模式完整实现（IP 数据包解析、TCP/UDP 处理）
- [x] 负载均衡（URL 测试、轮询、故障转移）
- [x] 健康检查和自动故障转移
- [x] 性能优化（零拷贝、缓冲区池化、TCP 优化）
- [x] GUI 界面基础框架（Tauri + HTML/CSS/JavaScript）
- [x] GUI 高级功能（流量图表、主题切换、节点可视化）
- [ ] GUI 完整前端框架（Vue/React）
- [ ] 移动端支持（iOS/Android）

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 🙏 致谢

- [Clash](https://github.com/Dreamacro/clash) - 项目灵感来源
- [Tokio](https://tokio.rs/) - 异步运行时
- [Axum](https://github.com/tokio-rs/axum) - Web 框架

## 📮 联系方式

- Issues: [GitHub Issues](https://github.com/yourusername/cat_proxy/issues)
- Email: your.email@example.com

---

<div align="center">
Made with ❤️ using Rust
</div>
