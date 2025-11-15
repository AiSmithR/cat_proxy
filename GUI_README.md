# Cat Proxy GUI 开发指南

## 概述

Cat Proxy GUI 是基于 Tauri 框架开发的跨平台图形用户界面，提供友好的可视化操作体验。

## 架构

```
cat_proxy/
├── src/
│   ├── gui/              # GUI 后端模块
│   │   ├── mod.rs        # 应用状态和数据结构
│   │   └── commands.rs   # Tauri 命令处理
│   └── bin/
│       └── cat_proxy_gui.rs  # GUI 应用入口
├── ui/                   # 前端界面
│   └── dist/
│       └── index.html    # 主界面（包含 Chart.js）
├── icons/                # 应用图标
│   ├── icon.svg          # SVG 图标
│   └── icon.png          # PNG 图标
├── build.rs              # Tauri 构建脚本
└── tauri.conf.json       # Tauri 配置
```

## 功能特性

### 已实现功能

- ✅ 仪表板统计（连接数、流量统计）
- ✅ 代理服务启动/停止
- ✅ 系统信息显示
- ✅ 实时数据刷新
- ✅ 配置管理接口
- ✅ 代理节点管理
- ✅ 规则管理
- ✅ 系统代理设置
- ✅ 订阅管理
- ✅ **流量趋势图表**（Chart.js 实时图表）
- ✅ **代理节点延迟可视化**（进度条显示）
- ✅ **主题切换**（亮色/暗色模式）
- ✅ **响应式设计**（自适应布局）
- ✅ **内置规则编辑器**（可视化规则管理）
- ✅ **实时连接查看**（活跃连接详情）

### 后端 API

所有 GUI 命令都通过 Tauri 的 invoke API 调用：

```rust
// 获取仪表板统计
get_dashboard_stats() -> DashboardStats

// 启动/停止代理
start_proxy() -> String
stop_proxy() -> String

// 获取代理节点
get_proxy_nodes() -> Vec<ProxyNodeInfo>

// 获取代理组
get_proxy_groups() -> Vec<ProxyGroupInfo>

// 获取规则列表
get_rules() -> Vec<RuleInfo>

// 配置管理
get_config() -> Config
update_config(config: Config) -> String
save_config() -> String
export_config() -> String

// 设置管理
get_settings() -> AppSettings
update_settings(settings: AppSettings) -> String

// 系统信息
get_system_info() -> SystemInfo

// 测试代理
test_proxy_node(name: String) -> u64

// 系统代理
set_system_proxy(enable: bool, http_port: u16, socks_port: u16) -> String

// 订阅管理
import_subscription(url: String) -> String
update_subscription() -> String

// 连接管理
clear_connections() -> String

// 规则管理
add_rule(rule: String) -> String
delete_rule(index: usize) -> String
update_rule(index: usize, rule: String) -> String

// 连接查看
get_connections() -> Vec<GuiConnectionInfo>
```

## 前端界面

### 当前实现

当前提供了一个功能完整的单页面应用，包含：

1. **主题切换**
   - 亮色/暗色模式切换
   - 主题偏好自动保存到 localStorage
   - 所有组件自适应主题配色

2. **仪表板卡片**
   - 活跃连接数
   - 总连接数
   - 上传流量（动态格式化）
   - 下载流量（动态格式化）

3. **流量趋势图表**
   - 基于 Chart.js 的实时折线图
   - 显示上传/下载流量变化
   - 保留最近 20 个数据点
   - 自动更新（每 3 秒）
   - 主题自适应（图表配色跟随主题）

4. **代理节点管理**
   - 节点卡片展示
   - 延迟可视化（彩色进度条）
   - 节点状态指示器
   - 自动刷新（每 10 秒）

5. **控制按钮**
   - 启动代理
   - 停止代理
   - 刷新数据
   - 打开配置

6. **系统信息**
   - 操作系统
   - CPU 架构
   - CPU 核心数
   - 当前代理模式

7. **快速开始指南**

### 前端技术栈建议

可以使用以下技术栈升级前端：

- **Vue 3 + TypeScript**: 响应式框架，类型安全
- **React + TypeScript**: 组件化开发
- **Svelte**: 轻量级，性能优异
- **Tailwind CSS**: 实用优先的 CSS 框架
- **shadcn/ui**: 优雅的组件库

## 开发流程

### 1. 安装依赖

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 如果使用 npm/yarn 开发前端
cd ui
npm install  # 或 yarn install
```

### 2. 开发模式

```bash
# 启动开发服务器
cargo tauri dev

# 或者只编译后端
cargo build --features gui --bin cat_proxy_gui
```

### 3. 构建发布版本

```bash
# 构建跨平台应用
cargo tauri build

# 构建产物在 target/release/ 目录
```

## 数据结构

### DashboardStats

```rust
pub struct DashboardStats {
    pub is_running: bool,
    pub active_connections: usize,
    pub total_connections: u64,
    pub total_upload: u64,
    pub total_download: u64,
    pub zero_copy_count: u64,
    pub proxy_mode: String,
    pub uptime: u64,
}
```

### ProxyNodeInfo

```rust
pub struct ProxyNodeInfo {
    pub name: String,
    pub proxy_type: String,
    pub server: String,
    pub port: u16,
    pub is_healthy: bool,
    pub latency: Option<u64>,
}
```

### CommandResponse<T>

所有命令返回统一的响应格式：

```rust
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

## 自定义开发

### 添加新命令

1. 在 `src/gui/commands.rs` 中添加新的命令处理函数：

```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<CommandResponse<String>, String> {
    // 处理逻辑
    Ok(CommandResponse::success("OK".to_string()))
}
```

2. 在 `src/bin/cat_proxy_gui.rs` 中注册命令：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    commands::my_command,
])
```

3. 在前端调用：

```javascript
const result = await invoke('my_command', { param: 'value' });
```

### 添加新的状态数据

在 `src/gui/mod.rs` 的 `AppState` 中添加：

```rust
pub struct AppState {
    // 现有字段...
    pub my_data: Arc<RwLock<MyData>>,
}
```

## 打包分发

### macOS

```bash
cargo tauri build --target universal-apple-darwin
# DMG 文件在 target/release/bundle/dmg/
```

### Windows

```bash
cargo tauri build --target x86_64-pc-windows-msvc
# MSI 安装包在 target/release/bundle/msi/
```

### Linux

```bash
cargo tauri build --target x86_64-unknown-linux-gnu
# AppImage 在 target/release/bundle/appimage/
# Deb 包在 target/release/bundle/deb/
```

## 注意事项

1. **功能开关**: GUI 功能通过 `gui` feature 控制，默认不启用
2. **运行时依赖**: Tauri 需要系统 WebView（macOS/Linux 内置，Windows 需要 Edge WebView2）
3. **权限管理**: 系统代理设置等功能需要管理员权限
4. **跨平台**: 确保测试所有目标平台的兼容性

## 未来改进

- [ ] 完整的前端框架（Vue/React）
- [x] 更多可视化图表（流量趋势、连接分布）✅
- [ ] 拖拽排序代理节点
- [x] 内置规则编辑器 ✅
- [x] 主题切换（亮色/暗色）✅
- [ ] 多语言支持
- [ ] 托盘菜单（需要图标文件配置）
- [ ] 开机自启动
- [ ] 自动更新
- [ ] 规则导入/导出
- [ ] 连接过滤和搜索
- [ ] 配置向导

## 技术栈

### 当前使用
- **后端**: Rust + Tauri 1.5
- **前端**: HTML5 + CSS3 + Vanilla JavaScript
- **图表库**: Chart.js 4.4.0
- **构建工具**: Tauri CLI

### CSS 特性
- CSS Variables 实现主题切换
- Flexbox + Grid 布局
- 渐变背景和卡片效果
- 响应式设计
- 动画过渡效果
