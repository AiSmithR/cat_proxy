# Cat Proxy GUI 启动指南

## 🚀 三种启动方式

---

## 方法一：直接运行（推荐 ⭐）

**最简单快速的方式，适合日常使用**

### 启动步骤

打开终端，执行以下命令：

```bash
cd /Users/coocit/workspace/rust_workspace/cat_proxy
./target/release/cat_proxy_gui
```

就这么简单！应用程序窗口会自动弹出。

### 特点
- ✅ 启动速度最快（约 1 秒）
- ✅ 性能最优化
- ✅ 内存占用低（~100 MB）
- ✅ 无需额外配置

### 首次运行可能需要的操作

**如果遇到"无法验证开发者"提示：**
```bash
xattr -cr ./target/release/cat_proxy_gui
```

**如果提示"权限被拒绝"：**
```bash
chmod +x ./target/release/cat_proxy_gui
```

---

## 方法二：开发模式（带热更新）

**适合需要修改代码并实时查看效果**

### 启动步骤

需要打开**两个终端窗口**：

#### 终端 1 - 启动前端开发服务器

```bash
cd /Users/coocit/workspace/rust_workspace/cat_proxy/ui-react
npm run dev
```

等待看到：
```
VITE v5.4.21  ready in 104 ms
➜  Local:   http://localhost:1420/
```

#### 终端 2 - 启动后端应用

```bash
cd /Users/coocit/workspace/rust_workspace/cat_proxy
cargo run --bin cat_proxy_gui --features gui
```

等待编译完成，应用程序窗口会自动打开。

### 特点
- ✅ 支持热更新（修改前端代码即时生效）
- ✅ 可以看到实时日志
- ✅ 适合开发和调试
- ⚠️ 首次启动较慢（编译需要 30-60 秒）
- ⚠️ 内存占用较高（~360 MB）

---

## 方法三：后台运行

**适合需要程序在后台持续运行**

### 启动命令

```bash
cd /Users/coocit/workspace/rust_workspace/cat_proxy
./target/release/cat_proxy_gui &
```

### 查看后台进程

```bash
ps aux | grep cat_proxy_gui | grep -v grep
```

输出示例：
```
coocit  37070  0.0  0.6  cat_proxy_gui
```

### 停止后台程序

```bash
pkill -f cat_proxy_gui
```

---

## 🛠️ 常用管理命令

### 检查应用是否在运行

```bash
ps aux | grep cat_proxy_gui | grep -v grep
```

### 停止所有相关进程

```bash
# 停止 GUI 应用
pkill -f cat_proxy_gui

# 停止开发服务器（如果使用了开发模式）
pkill -f vite
```

### 查看应用占用的端口

```bash
lsof -i :1420  # 前端开发服务器端口
```

---

## 📱 应用程序界面说明

启动后，您会看到一个 1200x800 像素的窗口，包含：

### 主要功能区

1. **顶部导航栏**
   - Dashboard（仪表板）
   - Proxies（代理节点）
   - Rules（规则）
   - Connections（连接）
   - Logs（日志）
   - Settings（设置）

2. **连接统计卡片**
   - 活跃连接数
   - 总连接数
   - 总上传流量
   - 总下载流量

3. **流量趋势图**
   - 绿色曲线：上传流量
   - 蓝色曲线：下载流量
   - 实时更新（每 2 秒）
   - 显示最近 2 分钟数据

4. **代理控制**
   - 启动/停止代理按钮
   - 节点选择
   - 延迟测试

---

## ⚙️ 配置文件（可选）

应用程序会尝试加载配置文件。如果不存在，会使用默认配置。

### 创建配置文件

```bash
cd /Users/coocit/workspace/rust_workspace/cat_proxy
cp config.example.yaml config.yaml
```

### 默认配置

如果没有 config.yaml，应用会使用以下默认值：

- **HTTP 代理端口**: 7890
- **SOCKS5 代理端口**: 7891
- **代理模式**: Rule（规则模式）
- **允许局域网**: 否
- **日志级别**: Info

---

## 🔧 故障排除

### 问题 1: 应用启动后立即退出

**查看错误日志：**
```bash
./target/release/cat_proxy_gui 2>&1 | tee app.log
cat app.log
```

### 问题 2: 窗口没有出现

**确保前端已构建：**
```bash
cd ui-react
npm run build
ls -la dist/  # 应该能看到文件
```

### 问题 3: 开发模式端口被占用

**查找占用 1420 端口的进程：**
```bash
lsof -i :1420
```

**结束占用进程：**
```bash
kill -9 <PID>
```

### 问题 4: 前端资源加载失败

**检查 dist 目录：**
```bash
ls -la ui-react/dist/
```

**如果为空，重新构建：**
```bash
cd ui-react
rm -rf dist
npm run build
```

### 问题 5: macOS "应用已损坏"提示

**移除隔离属性：**
```bash
xattr -cr ./target/release/cat_proxy_gui
```

或者在"系统偏好设置" → "安全性与隐私"中允许该应用。

---

## 📊 性能参考

### Release 版本（方法一）
- 启动时间: ~1 秒
- 内存占用: ~100 MB
- CPU 占用: <1%（空闲时）
- 文件大小: 7.1 MB

### 开发模式（方法二）
- 启动时间: ~30-60 秒（首次编译）
- 内存占用: ~360 MB（前端 260MB + 后端 100MB）
- CPU 占用: ~1%（空闲时）
- 支持热更新

---

## 🎯 快速命令参考

### 最常用的命令

```bash
# 进入项目目录
cd /Users/coocit/workspace/rust_workspace/cat_proxy

# 启动应用（Release）
./target/release/cat_proxy_gui

# 停止应用
pkill -f cat_proxy_gui

# 查看是否在运行
ps aux | grep cat_proxy_gui | grep -v grep
```

### 开发模式命令

```bash
# 终端 1 - 前端
cd /Users/coocit/workspace/rust_workspace/cat_proxy/ui-react
npm run dev

# 终端 2 - 后端
cd /Users/coocit/workspace/rust_workspace/cat_proxy
cargo run --bin cat_proxy_gui --features gui
```

---

## 💡 使用建议

### 日常使用
- 推荐使用**方法一**（直接运行 Release 版本）
- 启动快、性能好、资源占用少

### 开发调试
- 使用**方法二**（开发模式）
- 可以实时看到代码修改效果
- 便于查看日志和调试

### 后台服务
- 使用**方法三**（后台运行）
- 不占用终端窗口
- 适合长时间运行

---

## 📝 下一步

1. 启动应用后，查看 Dashboard 了解流量统计
2. 在 Proxies 页面添加和管理代理节点
3. 在 Rules 页面配置路由规则
4. 在 Settings 页面调整应用设置

---

## 🆘 获取帮助

- **构建指南**: 查看 `BUILD_GUIDE.md`
- **项目说明**: 查看 `README.md`
- **构建总结**: 查看 `BUILD_SUMMARY.md`

---

**祝使用愉快！** 🎉
