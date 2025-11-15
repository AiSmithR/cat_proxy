# Cat Proxy 快速开始

## 简介

Cat Proxy 是一个现代化的跨平台代理软件，提供强大的订阅管理、节点健康监控、DNS 增强和日志系统。

## 系统要求

- Rust 1.70+
- Node.js 18+
- macOS / Linux / Windows

## 安装

### 1. 克隆项目
```bash
git clone <repository-url>
cd cat_proxy
```

### 2. 安装依赖
```bash
# Rust 依赖会在构建时自动安装

# 前端依赖（如果需要 GUI）
cd src-ui
npm install
cd ..
```

### 3. 构建项目
```bash
# CLI 版本
cargo build --release --bin cat_proxy

# GUI 版本
cargo build --release --bin cat_proxy_gui --features gui
```

## 快速使用

### 命令行版本

#### 1. 创建配置文件

创建 `config.yaml`:
```yaml
port: 7890
socks-port: 7891
allow-lan: false
mode: rule
log-level: info

dns:
  enable: true
  listen: 0.0.0.0:53
  nameserver:
    - 223.5.5.5
    - 8.8.8.8

proxies:
  - name: "Example-SS"
    type: ss
    server: example.com
    port: 8388
    password: password
    cipher: aes-256-gcm

proxy-groups:
  - name: "Auto"
    type: url-test
    proxies:
      - Example-SS
    url: http://www.gstatic.com/generate_204
    interval: 300

rules:
  - DOMAIN-SUFFIX,google.com,Auto
  - GEOIP,CN,DIRECT
  - MATCH,Auto
```

#### 2. 运行代理

```bash
cargo run --bin cat_proxy -- --config config.yaml
```

### GUI 版本

#### 1. 启动 GUI

```bash
# 开发模式
cargo tauri dev

# 或运行构建后的版本
cargo run --bin cat_proxy_gui --features gui
```

#### 2. 基本操作

**使用 GUI 界面**:

GUI 提供了友好的图形界面，包含以下功能：

1. **控制面板**:
   - 启动/停止代理
   - 查看实时统计（连接数、流量等）
   - 刷新数据

2. **订阅管理** ⭐ 新功能:
   - 点击"订阅管理"按钮
   - 添加新订阅（输入名称和 URL）
   - 更新订阅（单个或全部）
   - 启用/禁用订阅
   - 查看订阅详情（节点数、更新时间等）

3. **日志查看** ⭐ 新功能:
   - 点击"日志查看"按钮
   - 按级别过滤日志（Trace/Debug/Info/Warn/Error）
   - 按模块名称过滤
   - 搜索日志内容
   - 分页浏览日志
   - 查看日志统计

4. **节点管理**:
   - 查看所有代理节点
   - 测试节点延迟
   - 查看节点健康状态

5. **规则管理**:
   - 添加、编辑、删除规则
   - 查看当前规则列表

6. **连接查看**:
   - 查看活跃连接
   - 监控流量统计

**通过 API 调用**:
```typescript
await invoke('import_subscription', {
  name: 'My Subscription',
  url: 'https://example.com/subscription'
});
```

**更新订阅**:
```typescript
await invoke('update_subscription', {
  name: 'My Subscription'
});
```

**测试节点延迟**:
```typescript
// 测试单个节点
await invoke('test_proxy_node', {
  nodeName: 'Example-SS'
});

// 测试所有节点
await invoke('test_all_proxy_nodes');
```

**启动自动健康检查**:
```typescript
await invoke('start_auto_health_check');
```

**获取日志**:
```typescript
// 获取最新 100 条日志
await invoke('get_latest_logs', { count: 100 });

// 分页查询日志
await invoke('get_logs', {
  page: 0,
  pageSize: 50,
  minLevel: 'info',
  search: 'error'
});
```

## 常用功能

### 配置管理

```typescript
// 获取配置
const config = await invoke('get_config');

// 更新配置
await invoke('update_config', { config: newConfig });

// 保存配置
await invoke('save_config');

// 重新加载配置
await invoke('reload_config');
```

### 订阅管理

```typescript
// 获取订阅列表
const subs = await invoke('get_subscriptions');

// 启用/禁用订阅
await invoke('set_subscription_enabled', {
  name: 'My Subscription',
  enabled: true
});

// 删除订阅
await invoke('delete_subscription', {
  name: 'My Subscription'
});
```

### 节点管理

```typescript
// 获取节点列表（带延迟信息）
const nodes = await invoke('get_proxy_nodes_with_latency');

// 获取代理组列表
const groups = await invoke('get_proxy_groups');
```

### 系统代理

```typescript
// 设置系统代理
await invoke('set_system_proxy', {
  enable: true,
  httpPort: 7890,
  socksPort: 7891
});
```

## DNS 配置

### 使用自定义 DNS

```rust
use cat_proxy::dns::{DnsResolver, DnsMode, DnsProtocol};
use std::net::SocketAddr;

// 使用 Cloudflare DNS
let resolver = DnsResolver::new_with_cloudflare(DnsMode::RealIp).await?;

// 使用自定义 DNS 服务器
let servers = vec![
    "1.1.1.1:53".parse::<SocketAddr>()?,
    "8.8.8.8:53".parse::<SocketAddr>()?,
];
let resolver = DnsResolver::new_with_protocol(
    DnsMode::RealIp,
    DnsProtocol::Custom { servers }
).await?;

// 解析域名
let ips = resolver.resolve("example.com").await?;

// 获取 DNS 统计
let stats = resolver.get_cache_stats();
println!("缓存命中率: {:.2}%",
    100.0 * stats.cache_hits as f64 / stats.total_queries as f64
);
```

## 日志系统

### 查看日志

```typescript
// 获取日志统计
const stats = await invoke('get_log_stats');
console.log(`总日志: ${stats.total}`);
console.log(`错误: ${stats.error}`);
console.log(`警告: ${stats.warn}`);

// 过滤日志
const { data } = await invoke('get_logs', {
  page: 0,
  pageSize: 50,
  minLevel: 'warn',      // 只看警告和错误
  target: 'cat_proxy',   // 过滤模块
  search: 'connection'   // 搜索关键词
});

// 清空日志
await invoke('clear_logs');
```

## 开发模式

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test config::
cargo test dns::
cargo test logging::

# 显示测试输出
cargo test -- --nocapture
```

### 代码检查

```bash
# 检查代码
cargo check

# 运行 Clippy
cargo clippy

# 格式化代码
cargo fmt
```

## 故障排除

### 1. 编译错误

**问题**: 找不到依赖
```bash
# 更新依赖
cargo update
cargo clean
cargo build
```

### 2. 权限问题

**问题**: TUN 模式需要管理员权限
```bash
# macOS/Linux
sudo cargo run --bin cat_proxy
```

### 3. 端口被占用

**问题**: 默认端口 7890/7891 被占用
```yaml
# 修改配置文件
port: 8890
socks-port: 8891
```

### 4. DNS 解析失败

**问题**: 无法解析域名
```yaml
# 使用其他 DNS 服务器
dns:
  nameserver:
    - 1.1.1.1
    - 8.8.8.8
```

## 性能优化

### 1. 调整缓存大小

```rust
// 增加 DNS 缓存 TTL
let resolver = DnsResolver::new_with_config(
    DnsMode::RealIp,
    config,
    Duration::from_secs(600), // 10 分钟
)?;
```

### 2. 调整日志容量

```rust
// 增加日志容量
let log_collector = LogCollector::new(50000); // 50,000 条日志
```

### 3. 优化连接池

```yaml
# 在配置中调整连接池大小
# （功能待实现）
```

## API 文档

完整的 API 文档请参考：
- [配置管理 API](./docs/api/config.md)
- [订阅管理 API](./docs/api/subscription.md)
- [健康监控 API](./docs/api/health.md)
- [日志系统 API](./docs/api/logging.md)

## 更多资源

- [开发总结](./DEVELOPMENT_SUMMARY.md)
- [架构设计](./docs/architecture.md)
- [贡献指南](./CONTRIBUTING.md)
- [常见问题](./docs/faq.md)

## 联系和支持

- GitHub Issues: 报告问题和提交功能请求
- 文档: 查看完整文档

---

祝您使用愉快！ 🚀
