# Cat Proxy 快速开始指南

## 立即运行

### 1. 构建项目

```bash
cargo build --release
```

### 2. 创建配置文件

```bash
cp config.example.yaml config.yaml
```

### 3. 运行代理服务器

```bash
# 开发模式
cargo run

# 或使用 release 版本
./target/release/cat_proxy
```

你应该会看到类似这样的输出：

```
Starting Cat Proxy Server...
Mode: Rule
HTTP Proxy Port: 7890
SOCKS5 Proxy Port: 7891
HTTP Proxy listening on 127.0.0.1:7890
SOCKS5 Proxy listening on 127.0.0.1:7891
```

## 测试代理功能

### 方式 1: 使用 curl 测试

#### 测试 HTTP 代理

```bash
# 通过 HTTP 代理访问网站
curl -x http://127.0.0.1:7890 https://www.google.com
curl -x http://127.0.0.1:7890 https://www.github.com

# 显示响应头
curl -i -x http://127.0.0.1:7890 https://httpbin.org/ip
```

#### 测试 SOCKS5 代理

```bash
# 通过 SOCKS5 代理访问网站
curl -x socks5://127.0.0.1:7891 https://www.google.com

# 测试不同的网站
curl -x socks5://127.0.0.1:7891 https://api.github.com
```

### 方式 2: 配置浏览器代理

#### Chrome/Edge

1. 打开设置 → 系统 → 打开代理设置
2. 设置 HTTP 代理: `127.0.0.1:7890`
3. 设置 HTTPS 代理: `127.0.0.1:7890`
4. 设置 SOCKS 代理: `127.0.0.1:7891`

#### Firefox

1. 设置 → 常规 → 网络设置
2. 手动代理配置
3. HTTP 代理: `127.0.0.1` 端口 `7890`
4. 勾选 "也将此代理用于 HTTPS"
5. SOCKS v5: `127.0.0.1` 端口 `7891`

### 方式 3: 使用系统代理

#### macOS

```bash
# 设置 HTTP 代理
networksetup -setwebproxy "Wi-Fi" 127.0.0.1 7890

# 设置 HTTPS 代理
networksetup -setsecurewebproxy "Wi-Fi" 127.0.0.1 7890

# 设置 SOCKS 代理
networksetup -setsocksfirewallproxy "Wi-Fi" 127.0.0.1 7891

# 取消代理
networksetup -setwebproxystate "Wi-Fi" off
networksetup -setsecurewebproxystate "Wi-Fi" off
networksetup -setsocksfirewallproxystate "Wi-Fi" off
```

#### Windows (PowerShell)

```powershell
# 设置代理
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 1 /f
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyServer /t REG_SZ /d "127.0.0.1:7890" /f

# 取消代理
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f
```

#### Linux (GNOME)

```bash
# 设置代理
gsettings set org.gnome.system.proxy mode 'manual'
gsettings set org.gnome.system.proxy.http host '127.0.0.1'
gsettings set org.gnome.system.proxy.http port 7890
gsettings set org.gnome.system.proxy.https host '127.0.0.1'
gsettings set org.gnome.system.proxy.https port 7890
gsettings set org.gnome.system.proxy.socks host '127.0.0.1'
gsettings set org.gnome.system.proxy.socks port 7891

# 取消代理
gsettings set org.gnome.system.proxy mode 'none'
```

## 配置规则

编辑 `config.yaml` 文件：

```yaml
# 代理模式: direct, global, rule
mode: rule

# 规则列表
rules:
  # Google 服务走代理
  - DOMAIN-SUFFIX,google.com,PROXY
  - DOMAIN-SUFFIX,googleapis.com,PROXY
  - DOMAIN-SUFFIX,youtube.com,PROXY
  
  # GitHub 走代理
  - DOMAIN-SUFFIX,github.com,PROXY
  - DOMAIN-SUFFIX,githubusercontent.com,PROXY
  
  # 国内网站直连
  - DOMAIN-SUFFIX,cn,DIRECT
  - DOMAIN-SUFFIX,baidu.com,DIRECT
  - DOMAIN-SUFFIX,qq.com,DIRECT
  - DOMAIN-SUFFIX,taobao.com,DIRECT
  
  # 本地网络直连
  - IP-CIDR,192.168.0.0/16,DIRECT
  - IP-CIDR,10.0.0.0/8,DIRECT
  
  # 默认规则
  - MATCH,DIRECT
```

## 日志和调试

### 查看日志

日志会输出到控制台，显示所有连接和路由信息：

```
INFO  Starting Cat Proxy Server...
DEBUG New SOCKS5 connection from 127.0.0.1:xxxxx
DEBUG SOCKS5 target: www.google.com:443
DEBUG Selecting proxy for target: www.google.com:443
DEBUG Matching rules for domain: www.google.com
INFO  Connection closed. Upload: 1234 bytes, Download: 5678 bytes
```

### 启用详细日志

设置环境变量：

```bash
# 启用 debug 级别日志
RUST_LOG=cat_proxy=debug cargo run

# 启用 trace 级别日志（非常详细）
RUST_LOG=cat_proxy=trace cargo run
```

## 常见问题

### Q: 为什么代理不工作？

A: 检查以下几点：
1. 代理服务器是否已启动
2. 端口是否被占用（7890/7891）
3. 配置文件格式是否正确
4. 规则是否正确匹配

### Q: 如何验证规则是否生效？

A: 查看日志输出，会显示：
```
DEBUG Matching rules for domain: www.google.com
DEBUG Route selection: Proxy("PROXY")
```

### Q: 支持哪些代理协议？

A: 当前版本：
- ✅ 直连 (DIRECT)
- ✅ SOCKS5 (出站，完整实现)
- 🔨 Shadowsocks (框架就绪，需完善加密)
- 🔨 VMess (框架就绪)
- 🔨 Trojan (框架就绪)

### Q: 如何添加上游代理服务器？

A: 编辑 `config.yaml`:

```yaml
proxies:
  # SOCKS5 代理（完整支持）
  - name: "my-socks5"
    type: socks5
    server: proxy.example.com
    port: 1080

# 在代理组中使用
proxy-groups:
  - name: "PROXY"
    type: select
    proxies:
      - my-socks5

# 在规则中引用
rules:
  - DOMAIN-SUFFIX,google.com,PROXY
```

## 性能测试

### 测试并发连接

```bash
# 安装 Apache Bench
# macOS: brew install apache2
# Ubuntu: apt-get install apache2-utils

# 测试并发性能
ab -n 1000 -c 100 -X 127.0.0.1:7890 http://httpbin.org/ip
```

### 测试延迟

```bash
# 使用 curl 测量延迟
time curl -x http://127.0.0.1:7890 https://www.google.com > /dev/null
```

## 下一步

1. 参考 [DEVELOPMENT.md](./DEVELOPMENT.md) 了解项目开发进度
2. 查看 [README.md](./README.md) 了解项目详细信息
3. 阅读 [config.example.yaml](./config.example.yaml) 了解所有配置选项

## 反馈和问题

如果遇到问题或有建议，欢迎：
- 提交 Issue
- 查看日志输出
- 阅读开发文档

---

享受使用 Cat Proxy！🐱
