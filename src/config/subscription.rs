//! 订阅管理模块

use super::{ProxyConfig, ProxyType};
use reqwest;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn, error};

/// 订阅信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// 订阅 URL
    pub url: String,
    /// 订阅名称
    pub name: String,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 最后更新时间（Unix 时间戳）
    #[serde(default)]
    pub last_update: u64,
    /// 节点数量
    #[serde(default)]
    pub node_count: usize,
    /// 更新间隔（秒），0 表示手动更新
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_update_interval() -> u64 {
    86400 // 24 小时
}

impl Subscription {
    /// 创建新订阅
    pub fn new(name: String, url: String) -> Self {
        Self {
            url,
            name,
            enabled: true,
            last_update: 0,
            node_count: 0,
            update_interval: 86400,
        }
    }

    /// 检查是否需要更新
    pub fn needs_update(&self) -> bool {
        if !self.enabled || self.update_interval == 0 {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.last_update >= self.update_interval
    }

    /// 标记为已更新
    pub fn mark_updated(&mut self, node_count: usize) {
        self.last_update = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.node_count = node_count;
    }
}

/// 订阅管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionManager {
    subscriptions: Vec<Subscription>,
}

impl SubscriptionManager {
    /// 创建新的订阅管理器
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    /// 获取所有订阅
    pub fn get_subscriptions(&self) -> &[Subscription] {
        &self.subscriptions
    }

    /// 添加订阅
    pub fn add_subscription(&mut self, name: String, url: String) -> anyhow::Result<()> {
        // 检查名称是否重复
        if self.subscriptions.iter().any(|s| s.name == name) {
            return Err(anyhow::anyhow!("Subscription name already exists: {}", name));
        }

        // 检查 URL 是否重复
        if self.subscriptions.iter().any(|s| s.url == url) {
            return Err(anyhow::anyhow!("Subscription URL already exists"));
        }

        self.subscriptions.push(Subscription::new(name, url));
        Ok(())
    }

    /// 删除订阅
    pub fn remove_subscription(&mut self, name: &str) -> anyhow::Result<()> {
        let original_len = self.subscriptions.len();
        self.subscriptions.retain(|s| s.name != name);

        if self.subscriptions.len() == original_len {
            return Err(anyhow::anyhow!("Subscription not found: {}", name));
        }

        Ok(())
    }

    /// 启用/禁用订阅
    pub fn set_subscription_enabled(&mut self, name: &str, enabled: bool) -> anyhow::Result<()> {
        let sub = self.subscriptions
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("Subscription not found: {}", name))?;

        sub.enabled = enabled;
        Ok(())
    }

    /// 设置订阅更新间隔
    pub fn set_update_interval(&mut self, name: &str, interval: u64) -> anyhow::Result<()> {
        let sub = self.subscriptions
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("Subscription not found: {}", name))?;

        sub.update_interval = interval;
        Ok(())
    }

    /// 更新所有订阅
    pub async fn update_all(&mut self) -> anyhow::Result<Vec<ProxyConfig>> {
        let mut all_proxies = Vec::new();

        // 收集需要更新的订阅 URL
        let subscriptions_to_update: Vec<(String, String)> = self.subscriptions
            .iter()
            .filter(|s| s.enabled)
            .map(|s| (s.name.clone(), s.url.clone()))
            .collect();

        for (name, url) in subscriptions_to_update {
            info!("Updating subscription: {}", name);
            match self.fetch_subscription(&url).await {
                Ok(mut proxies) => {
                    info!("Got {} proxies from {}", proxies.len(), name);

                    // 更新订阅信息
                    if let Some(sub) = self.subscriptions.iter_mut().find(|s| s.name == name) {
                        sub.mark_updated(proxies.len());
                    }

                    // 为节点名称添加订阅前缀
                    for proxy in &mut proxies {
                        proxy.name = format!("[{}] {}", name, proxy.name);
                    }

                    all_proxies.append(&mut proxies);
                }
                Err(e) => {
                    error!("Failed to update subscription {}: {}", name, e);
                }
            }
        }

        // 去重（根据 server:port 组合）
        all_proxies = self.deduplicate_proxies(all_proxies);

        Ok(all_proxies)
    }

    /// 更新单个订阅
    pub async fn update_subscription(&mut self, name: &str) -> anyhow::Result<Vec<ProxyConfig>> {
        // 查找订阅并获取 URL
        let url = self.subscriptions
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("Subscription not found: {}", name))?
            .url
            .clone();

        // 检查是否启用
        let enabled = self.subscriptions
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.enabled)
            .unwrap_or(false);

        if !enabled {
            return Err(anyhow::anyhow!("Subscription is disabled: {}", name));
        }

        info!("Updating subscription: {}", name);
        let mut proxies = self.fetch_subscription(&url).await?;

        // 更新订阅信息
        if let Some(sub) = self.subscriptions.iter_mut().find(|s| s.name == name) {
            sub.mark_updated(proxies.len());
        }

        // 为节点名称添加订阅前缀
        for proxy in &mut proxies {
            proxy.name = format!("[{}] {}", name, proxy.name);
        }

        Ok(proxies)
    }

    /// 去重代理节点
    fn deduplicate_proxies(&self, proxies: Vec<ProxyConfig>) -> Vec<ProxyConfig> {
        let mut seen = HashMap::new();
        let mut result = Vec::new();

        for proxy in proxies {
            let key = format!("{}:{}", proxy.server, proxy.port);
            if !seen.contains_key(&key) {
                seen.insert(key, ());
                result.push(proxy);
            } else {
                debug!("Duplicate proxy removed: {} ({}:{})", proxy.name, proxy.server, proxy.port);
            }
        }

        result
    }

    /// 获取订阅内容
    async fn fetch_subscription(&self, url: &str) -> anyhow::Result<Vec<ProxyConfig>> {
        debug!("Fetching subscription from: {}", url);

        // 下载订阅内容
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("ClashX/1.0")
            .build()?;

        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let content = response.text().await?;

        // 尝试识别订阅格式
        if content.trim().starts_with('{') || content.trim().starts_with("proxies:") {
            // Clash 格式
            self.parse_clash_subscription(&content)
        } else {
            // Base64 编码的节点列表
            self.parse_base64_subscription(&content)
        }
    }

    /// 解析 Clash 格式订阅
    fn parse_clash_subscription(&self, _content: &str) -> anyhow::Result<Vec<ProxyConfig>> {
        // 简化实现：解析 YAML 格式的 Clash 配置
        debug!("Parsing Clash subscription");

        // TODO: 实现完整的 Clash 配置解析
        // 这里先返回空列表
        warn!("Clash subscription parsing not fully implemented");
        Ok(Vec::new())
    }

    /// 解析 Base64 编码的订阅
    fn parse_base64_subscription(&self, content: &str) -> anyhow::Result<Vec<ProxyConfig>> {
        // Base64 解码
        let decoded = BASE64.decode(content.trim())
            .or_else(|_| Ok::<Vec<u8>, anyhow::Error>(content.as_bytes().to_vec()))?;

        let text = String::from_utf8_lossy(&decoded);

        // 解析节点
        self.parse_proxies(&text)
    }

    /// 解析代理节点
    fn parse_proxies(&self, content: &str) -> anyhow::Result<Vec<ProxyConfig>> {
        let mut proxies = Vec::new();
        let mut parse_errors = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 尝试解析不同格式的节点
            let result = if line.starts_with("ss://") {
                self.parse_shadowsocks(line)
            } else if line.starts_with("ssr://") {
                self.parse_shadowsocksr(line)
            } else if line.starts_with("vmess://") {
                self.parse_vmess(line)
            } else if line.starts_with("vless://") {
                self.parse_vless(line)
            } else if line.starts_with("trojan://") {
                self.parse_trojan(line)
            } else if line.starts_with("hysteria://") || line.starts_with("hy://") {
                self.parse_hysteria(line)
            } else if line.starts_with("hysteria2://") || line.starts_with("hy2://") {
                self.parse_hysteria2(line)
            } else if line.starts_with("tuic://") {
                self.parse_tuic(line)
            } else if line.starts_with("wireguard://") || line.starts_with("wg://") {
                self.parse_wireguard(line)
            } else if line.starts_with("snell://") {
                self.parse_snell(line)
            } else {
                debug!("Unknown protocol in line: {}", line);
                continue;
            };

            match result {
                Ok(proxy) => proxies.push(proxy),
                Err(e) => {
                    warn!("Failed to parse node: {} - Error: {}", line, e);
                    parse_errors += 1;
                }
            }
        }

        if proxies.is_empty() {
            if parse_errors > 0 {
                return Err(anyhow::anyhow!(
                    "No valid proxy nodes found. {} parsing error(s) occurred",
                    parse_errors
                ));
            } else {
                return Err(anyhow::anyhow!("No proxy nodes found in subscription"));
            }
        }

        info!("Successfully parsed {} proxies ({} errors)", proxies.len(), parse_errors);
        Ok(proxies)
    }

    /// 解析 Shadowsocks 链接
    /// 格式: ss://BASE64(method:password)@server:port#name
    fn parse_shadowsocks(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("ss://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "SS Node".to_string())
        };

        // 分离服务器地址
        let (info, server_port) = url
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("Invalid SS URL format"))?;

        // Base64 解码用户信息
        let decoded = BASE64.decode(info)?;
        let user_info = String::from_utf8(decoded)?;

        // 解析 method:password
        let (cipher, password) = user_info
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid user info format"))?;

        // 解析服务器和端口
        let (server, port) = server_port
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();
        extra.insert("cipher".to_string(), serde_json::Value::String(cipher.to_string()));
        extra.insert("password".to_string(), serde_json::Value::String(password.to_string()));

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::Shadowsocks,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 VMess 链接
    /// 格式: vmess://BASE64(JSON)
    fn parse_vmess(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("vmess://");

        // Base64 解码
        let decoded = BASE64.decode(url)?;
        let json_str = String::from_utf8(decoded)?;

        // 解析 JSON
        let json: serde_json::Value = serde_json::from_str(&json_str)?;

        let name = json.get("ps")
            .and_then(|v| v.as_str())
            .unwrap_or("VMess Node")
            .to_string();

        let server = json.get("add")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing server"))?
            .to_string();

        let port = json.get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing port"))? as u16;

        let uuid = json.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing uuid"))?
            .to_string();

        let alter_id = json.get("aid")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let mut extra = serde_json::Map::new();
        extra.insert("uuid".to_string(), serde_json::Value::String(uuid));
        extra.insert("alterId".to_string(), serde_json::Value::Number(alter_id.into()));

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::VMess,
            server,
            port,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 Trojan 链接
    /// 格式: trojan://password@server:port#name
    fn parse_trojan(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("trojan://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "Trojan Node".to_string())
        };

        // 分离密码和服务器
        let (password, server_port) = url
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("Invalid Trojan URL format"))?;

        // 解析服务器和端口
        let (server, port) = server_port
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();
        extra.insert("password".to_string(), serde_json::Value::String(password.to_string()));

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::Trojan,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 ShadowsocksR 链接
    /// 格式: ssr://BASE64(server:port:protocol:method:obfs:BASE64(password)/?obfsparam=BASE64(param)&protoparam=BASE64(param)&remarks=BASE64(remarks))
    fn parse_shadowsocksr(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("ssr://");

        // Base64 解码
        let decoded = BASE64.decode(url)?;
        let text = String::from_utf8(decoded)?;

        // 分离主要部分和参数
        let (main_part, params) = if let Some(pos) = text.find("/?") {
            (&text[..pos], Some(&text[pos + 2..]))
        } else {
            (text.as_str(), None)
        };

        // 解析主要部分: server:port:protocol:method:obfs:BASE64(password)
        let parts: Vec<&str> = main_part.split(':').collect();
        if parts.len() < 6 {
            return Err(anyhow::anyhow!("Invalid SSR URL format"));
        }

        let server = parts[0].to_string();
        let port: u16 = parts[1].parse()?;
        let protocol = parts[2].to_string();
        let method = parts[3].to_string();
        let obfs = parts[4].to_string();
        let password_b64 = parts[5..].join(":");

        // 解码密码
        let password_bytes = BASE64.decode(password_b64.as_bytes())?;
        let password = String::from_utf8(password_bytes)?;

        // 解析参数
        let mut name = "SSR Node".to_string();
        let mut extra = serde_json::Map::new();

        if let Some(params_str) = params {
            for param in params_str.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "remarks" => {
                            if let Ok(decoded) = BASE64.decode(value) {
                                name = String::from_utf8_lossy(&decoded).to_string();
                            }
                        }
                        "obfsparam" => {
                            if let Ok(decoded) = BASE64.decode(value) {
                                let param = String::from_utf8_lossy(&decoded).to_string();
                                extra.insert("obfs-param".to_string(), serde_json::Value::String(param));
                            }
                        }
                        "protoparam" => {
                            if let Ok(decoded) = BASE64.decode(value) {
                                let param = String::from_utf8_lossy(&decoded).to_string();
                                extra.insert("protocol-param".to_string(), serde_json::Value::String(param));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        extra.insert("password".to_string(), serde_json::Value::String(password));
        extra.insert("cipher".to_string(), serde_json::Value::String(method));
        extra.insert("protocol".to_string(), serde_json::Value::String(protocol));
        extra.insert("obfs".to_string(), serde_json::Value::String(obfs));

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::ShadowsocksR,
            server,
            port,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 VLESS 链接
    /// 格式: vless://uuid@server:port?parameters#name
    fn parse_vless(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("vless://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "VLESS Node".to_string())
        };

        // 分离 UUID 和服务器地址
        let (uuid_part, server_params) = url
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("Invalid VLESS URL format"))?;

        let uuid = uuid_part.to_string();

        // 分离服务器地址和参数
        let (server_port, params_str) = if let Some(pos) = server_params.find('?') {
            (&server_params[..pos], Some(&server_params[pos + 1..]))
        } else {
            (server_params, None)
        };

        // 解析服务器和端口
        let (server, port) = server_port
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();
        extra.insert("uuid".to_string(), serde_json::Value::String(uuid));

        // 解析参数
        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    let value = urlencoding::decode(value)?.to_string();
                    extra.insert(key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::VLESS,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 Hysteria 链接
    /// 格式: hysteria://server:port?parameters#name
    fn parse_hysteria(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("hysteria://").trim_start_matches("hy://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "Hysteria Node".to_string())
        };

        // 分离服务器地址和参数
        let (server_port, params_str) = if let Some(pos) = url.find('?') {
            (&url[..pos], Some(&url[pos + 1..]))
        } else {
            (url, None)
        };

        // 解析服务器和端口
        let (server, port) = server_port
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();

        // 解析参数
        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    let value = urlencoding::decode(value)?.to_string();
                    extra.insert(key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::Hysteria,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 Hysteria2 链接
    /// 格式: hysteria2://password@server:port?parameters#name 或 hy2://...
    fn parse_hysteria2(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("hysteria2://").trim_start_matches("hy2://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "Hysteria2 Node".to_string())
        };

        // 分离密码和服务器
        let (password, server_params) = url
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("Invalid Hysteria2 URL format"))?;

        // 分离服务器地址和参数
        let (server_port, params_str) = if let Some(pos) = server_params.find('?') {
            (&server_params[..pos], Some(&server_params[pos + 1..]))
        } else {
            (server_params, None)
        };

        // 解析服务器和端口
        let (server, port) = server_port
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();
        extra.insert("password".to_string(), serde_json::Value::String(password.to_string()));

        // 解析参数
        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    let value = urlencoding::decode(value)?.to_string();
                    extra.insert(key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::Hysteria2,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 TUIC 链接
    /// 格式: tuic://uuid:password@server:port?parameters#name
    fn parse_tuic(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("tuic://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "TUIC Node".to_string())
        };

        // 分离认证信息和服务器
        let (auth, server_params) = url
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("Invalid TUIC URL format"))?;

        // 解析 UUID 和密码
        let (uuid, password) = auth
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid auth format"))?;

        // 分离服务器地址和参数
        let (server_port, params_str) = if let Some(pos) = server_params.find('?') {
            (&server_params[..pos], Some(&server_params[pos + 1..]))
        } else {
            (server_params, None)
        };

        // 解析服务器和端口
        let (server, port) = server_port
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();
        extra.insert("uuid".to_string(), serde_json::Value::String(uuid.to_string()));
        extra.insert("password".to_string(), serde_json::Value::String(password.to_string()));

        // 解析参数
        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    let value = urlencoding::decode(value)?.to_string();
                    extra.insert(key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::TUIC,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 WireGuard 链接
    /// 格式: wireguard://privatekey@server:port?parameters#name 或 wg://...
    fn parse_wireguard(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("wireguard://").trim_start_matches("wg://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "WireGuard Node".to_string())
        };

        // 分离私钥和服务器
        let (private_key, server_params) = url
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("Invalid WireGuard URL format"))?;

        // 分离服务器地址和参数
        let (server_port, params_str) = if let Some(pos) = server_params.find('?') {
            (&server_params[..pos], Some(&server_params[pos + 1..]))
        } else {
            (server_params, None)
        };

        // 解析服务器和端口
        let (server, port) = server_port
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();
        extra.insert("private-key".to_string(), serde_json::Value::String(private_key.to_string()));

        // 解析参数
        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    let value = urlencoding::decode(value)?.to_string();
                    extra.insert(key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::WireGuard,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }

    /// 解析 Snell 链接
    /// 格式: snell://server:port?parameters#name
    fn parse_snell(&self, url: &str) -> anyhow::Result<ProxyConfig> {
        let url = url.trim_start_matches("snell://");

        // 分离 name
        let (url, name) = if let Some(pos) = url.find('#') {
            let name = urlencoding::decode(&url[pos + 1..])?.to_string();
            (&url[..pos], name)
        } else {
            (url, "Snell Node".to_string())
        };

        // 分离服务器地址和参数
        let (server_port, params_str) = if let Some(pos) = url.find('?') {
            (&url[..pos], Some(&url[pos + 1..]))
        } else {
            (url, None)
        };

        // 解析服务器和端口
        let (server, port) = server_port
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid server:port format"))?;

        let mut extra = serde_json::Map::new();

        // 解析参数
        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    let value = urlencoding::decode(value)?.to_string();
                    extra.insert(key.to_string(), serde_json::Value::String(value));
                }
            }
        }

        Ok(ProxyConfig {
            name,
            proxy_type: ProxyType::Snell,
            server: server.to_string(),
            port: port.parse()?,
            extra: extra.into_iter().collect(),
        })
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shadowsocks() {
        let manager = SubscriptionManager::new();

        // 测试 SS 链接解析
        let ss_url = "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ=@example.com:8388#TestNode";
        let proxy = manager.parse_shadowsocks(ss_url).unwrap();

        assert_eq!(proxy.name, "TestNode");
        assert_eq!(proxy.server, "example.com");
        assert_eq!(proxy.port, 8388);
        assert!(matches!(proxy.proxy_type, ProxyType::Shadowsocks));
    }

    #[test]
    fn test_parse_trojan() {
        let manager = SubscriptionManager::new();

        let trojan_url = "trojan://password123@example.com:443#TestTrojan";
        let proxy = manager.parse_trojan(trojan_url).unwrap();

        assert_eq!(proxy.name, "TestTrojan");
        assert_eq!(proxy.server, "example.com");
        assert_eq!(proxy.port, 443);
        assert!(matches!(proxy.proxy_type, ProxyType::Trojan));
    }
}
