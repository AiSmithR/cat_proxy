use async_trait::async_trait;
use crate::config::{ProxyConfig, ProxyType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;

pub mod socks5;
pub mod http;
pub mod shadowsocks;
pub mod vmess;
pub mod trojan;

/// 代理协议 trait
#[async_trait]
pub trait ProxyProtocol: Send + Sync {
    /// 协议名称
    fn name(&self) -> &str;

    /// 连接到目标地址（通过此代理）
    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream>;
}

/// 直连实现
pub struct DirectProxy;

#[async_trait]
impl ProxyProtocol for DirectProxy {
    fn name(&self) -> &str {
        "direct"
    }

    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream> {
        Ok(TcpStream::connect(target).await?)
    }
}

/// 代理管理器
pub struct ProxyManager {
    proxies: HashMap<String, Arc<dyn ProxyProtocol>>,
}

impl ProxyManager {
    /// 创建新的代理管理器
    pub fn new() -> Self {
        Self {
            proxies: HashMap::new(),
        }
    }

    /// 从配置加载代理
    pub fn load_from_configs(&mut self, configs: &[ProxyConfig]) -> anyhow::Result<()> {
        for config in configs {
            let proxy: Arc<dyn ProxyProtocol> = match config.proxy_type {
                ProxyType::Shadowsocks => {
                    Arc::new(shadowsocks::ShadowsocksProxy::from_config(config)?)
                }
                ProxyType::VMess => {
                    Arc::new(vmess::VMessProxy::from_config(config)?)
                }
                ProxyType::Trojan => {
                    Arc::new(trojan::TrojanProxy::from_config(config)?)
                }
                ProxyType::Socks5 => {
                    Arc::new(socks5::Socks5Proxy::from_config(config)?)
                }
                ProxyType::Http => {
                    Arc::new(http::HttpProxy::from_config(config)?)
                }
                // 新协议类型 - 订阅解析支持，实际连接功能待实现
                ProxyType::ShadowsocksR | ProxyType::VLESS | ProxyType::Hysteria |
                ProxyType::Hysteria2 | ProxyType::TUIC | ProxyType::WireGuard | ProxyType::Snell => {
                    // 暂时跳过这些协议，它们已经被解析但尚未实现连接功能
                    tracing::warn!("Protocol {:?} is parsed but connection not yet implemented for node: {}",
                        config.proxy_type, config.name);
                    continue;
                }
            };

            self.proxies.insert(config.name.clone(), proxy);
        }

        Ok(())
    }

    /// 获取代理
    pub fn get_proxy(&self, name: &str) -> Option<Arc<dyn ProxyProtocol>> {
        self.proxies.get(name).cloned()
    }

    /// 列出所有代理名称
    pub fn list_proxies(&self) -> Vec<String> {
        self.proxies.keys().cloned().collect()
    }
}

impl Default for ProxyManager {
    fn default() -> Self {
        Self::new()
    }
}
