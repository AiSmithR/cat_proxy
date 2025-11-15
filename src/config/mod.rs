use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

pub mod subscription;
pub mod parser;

pub use subscription::{SubscriptionManager, Subscription};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    Direct,
    Global,
    Rule,
    Script,
}

impl Default for ProxyMode {
    fn default() -> Self {
        Self::Rule
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Silent,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DnsConfig {
    pub enable: bool,
    pub listen: Option<SocketAddr>,
    pub fake_ip_range: Option<String>,
    pub nameserver: Vec<String>,
    #[serde(default)]
    pub hosts: HashMap<String, String>,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            enable: true,
            listen: Some("0.0.0.0:53".parse().unwrap()),
            fake_ip_range: Some("198.18.0.1/16".to_string()),
            nameserver: vec!["223.5.5.5".to_string(), "8.8.8.8".to_string()],
            hosts: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    #[serde(rename = "ss")]
    Shadowsocks,
    #[serde(rename = "ssr")]
    ShadowsocksR,
    #[serde(rename = "vmess")]
    VMess,
    #[serde(rename = "vless")]
    VLESS,
    #[serde(rename = "trojan")]
    Trojan,
    #[serde(rename = "hysteria")]
    Hysteria,
    #[serde(rename = "hysteria2")]
    Hysteria2,
    #[serde(rename = "tuic")]
    TUIC,
    #[serde(rename = "wireguard")]
    WireGuard,
    #[serde(rename = "snell")]
    Snell,
    #[serde(rename = "socks5")]
    Socks5,
    #[serde(rename = "http")]
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,
    pub server: String,
    pub port: u16,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyGroupType {
    Select,
    #[serde(rename = "url-test")]
    UrlTest,
    Fallback,
    #[serde(rename = "load-balance")]
    LoadBalance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroupConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: ProxyGroupType,
    pub proxies: Vec<String>,
    pub url: Option<String>,
    pub interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_socks_port")]
    pub socks_port: u16,
    #[serde(default)]
    pub allow_lan: bool,
    #[serde(default)]
    pub mode: ProxyMode,
    #[serde(default)]
    pub log_level: LogLevel,
    #[serde(default)]
    pub dns: DnsConfig,
    #[serde(default)]
    pub proxies: Vec<ProxyConfig>,
    #[serde(default)]
    pub proxy_groups: Vec<ProxyGroupConfig>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

fn default_port() -> u16 {
    7890
}

fn default_socks_port() -> u16 {
    7891
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            socks_port: default_socks_port(),
            allow_lan: false,
            mode: ProxyMode::default(),
            log_level: LogLevel::default(),
            dns: DnsConfig::default(),
            proxies: Vec::new(),
            proxy_groups: Vec::new(),
            rules: Vec::new(),
            subscriptions: Vec::new(),
            config_path: None,
        }
    }
}

impl Config {
    /// 从 YAML 文件加载配置
    pub fn from_yaml_file(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let mut config: Self = serde_yaml::from_str(&content)?;
        config.config_path = Some(path);
        config.validate()?;
        Ok(config)
    }

    /// 从 JSON 字符串加载配置
    pub fn from_json_str(json: &str) -> anyhow::Result<Self> {
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// 保存配置到文件（使用记录的路径）
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.config_path {
            self.save_to(path)?;
        }
        Ok(())
    }

    /// 保存配置到指定路径
    pub fn save_to(&self, path: impl Into<PathBuf>) -> anyhow::Result<()> {
        let path = path.into();

        // 创建父目录（如果不存在）
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 将配置导出为 JSON 字符串
    pub fn to_json_string(&self) -> anyhow::Result<String> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        // 验证端口范围
        if self.port == 0 {
            return Err(anyhow::anyhow!("Invalid HTTP port: {}", self.port));
        }
        if self.socks_port == 0 {
            return Err(anyhow::anyhow!("Invalid SOCKS port: {}", self.socks_port));
        }
        if self.port == self.socks_port {
            return Err(anyhow::anyhow!("HTTP and SOCKS ports must be different"));
        }

        // 验证代理配置
        for proxy in &self.proxies {
            if proxy.name.is_empty() {
                return Err(anyhow::anyhow!("Proxy name cannot be empty"));
            }
            if proxy.server.is_empty() {
                return Err(anyhow::anyhow!("Proxy server cannot be empty for {}", proxy.name));
            }
            if proxy.port == 0 {
                return Err(anyhow::anyhow!("Invalid proxy port for {}: {}", proxy.name, proxy.port));
            }
        }

        // 验证代理组配置
        for group in &self.proxy_groups {
            if group.name.is_empty() {
                return Err(anyhow::anyhow!("Proxy group name cannot be empty"));
            }
            if group.proxies.is_empty() {
                return Err(anyhow::anyhow!("Proxy group {} has no proxies", group.name));
            }

            // 验证代理组中的代理是否存在
            for proxy_name in &group.proxies {
                if !self.proxies.iter().any(|p| &p.name == proxy_name) {
                    return Err(anyhow::anyhow!(
                        "Proxy '{}' in group '{}' does not exist",
                        proxy_name,
                        group.name
                    ));
                }
            }
        }

        // 验证 DNS 配置
        if self.dns.enable {
            if self.dns.nameserver.is_empty() {
                return Err(anyhow::anyhow!("DNS is enabled but no nameservers configured"));
            }
        }

        Ok(())
    }

    /// 重新加载配置文件
    pub fn reload(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.config_path {
            let new_config = Self::from_yaml_file(path.clone())?;
            *self = new_config;
        }
        Ok(())
    }

    /// 合并配置（用于热更新）
    pub fn merge(&mut self, other: Self) {
        self.port = other.port;
        self.socks_port = other.socks_port;
        self.allow_lan = other.allow_lan;
        self.mode = other.mode;
        self.log_level = other.log_level;
        self.dns = other.dns;
        self.proxies = other.proxies;
        self.proxy_groups = other.proxy_groups;
        self.rules = other.rules;
    }

    /// 获取默认配置文件路径
    pub fn default_config_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".config/cat_proxy/config.yaml")
        } else {
            PathBuf::from("config.yaml")
        }
    }

    /// 导出示例配置
    pub fn example() -> Self {
        let mut config = Self::default();

        // 添加示例代理
        config.proxies = vec![
            ProxyConfig {
                name: "Example-SS".to_string(),
                proxy_type: ProxyType::Shadowsocks,
                server: "example.com".to_string(),
                port: 8388,
                extra: {
                    let mut extra = HashMap::new();
                    extra.insert("password".to_string(), serde_json::json!("password"));
                    extra.insert("cipher".to_string(), serde_json::json!("aes-256-gcm"));
                    extra
                },
            },
            ProxyConfig {
                name: "Example-VMess".to_string(),
                proxy_type: ProxyType::VMess,
                server: "example.com".to_string(),
                port: 443,
                extra: {
                    let mut extra = HashMap::new();
                    extra.insert("uuid".to_string(), serde_json::json!("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"));
                    extra.insert("alterId".to_string(), serde_json::json!(0));
                    extra
                },
            },
        ];

        // 添加示例代理组
        config.proxy_groups = vec![
            ProxyGroupConfig {
                name: "Auto".to_string(),
                group_type: ProxyGroupType::UrlTest,
                proxies: vec!["Example-SS".to_string(), "Example-VMess".to_string()],
                url: Some("http://www.gstatic.com/generate_204".to_string()),
                interval: Some(300),
            },
        ];

        // 添加示例规则
        config.rules = vec![
            "DOMAIN-SUFFIX,google.com,Auto".to_string(),
            "DOMAIN-SUFFIX,youtube.com,Auto".to_string(),
            "GEOIP,CN,DIRECT".to_string(),
            "MATCH,Auto".to_string(),
        ];

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());

        // 测试端口冲突
        let mut bad_config = Config::default();
        bad_config.socks_port = bad_config.port;
        assert!(bad_config.validate().is_err());

        // 测试无效端口
        let mut bad_config = Config::default();
        bad_config.port = 0;
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        // 创建并保存配置
        let mut config = Config::default();
        config.port = 8080;
        config.socks_port = 8081;
        config.save_to(&config_path).unwrap();

        // 验证文件存在
        assert!(config_path.exists());

        // 加载配置
        let loaded_config = Config::from_yaml_file(&config_path).unwrap();
        assert_eq!(loaded_config.port, 8080);
        assert_eq!(loaded_config.socks_port, 8081);
    }

    #[test]
    fn test_config_to_json() {
        let config = Config::default();
        let json = config.to_json_string().unwrap();
        assert!(json.contains("port"));
        // JSON 使用 snake_case
        assert!(json.contains("\"socks-port\"") || json.contains("socksPort") || json.contains("socks_port"));
    }

    #[test]
    fn test_config_validation_with_proxies() {
        let mut config = Config::default();

        // 添加有效代理
        config.proxies.push(ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::Socks5,
            server: "127.0.0.1".to_string(),
            port: 1080,
            extra: HashMap::new(),
        });

        assert!(config.validate().is_ok());

        // 添加无效代理（空名称）
        config.proxies.push(ProxyConfig {
            name: "".to_string(),
            proxy_type: ProxyType::Socks5,
            server: "127.0.0.1".to_string(),
            port: 1080,
            extra: HashMap::new(),
        });

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_example() {
        let config = Config::example();
        assert!(!config.proxies.is_empty());
        assert!(!config.proxy_groups.is_empty());
        assert!(!config.rules.is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = Config::default();
        config1.port = 7890;

        let mut config2 = Config::default();
        config2.port = 8080;
        config2.socks_port = 8081;

        config1.merge(config2);

        assert_eq!(config1.port, 8080);
        assert_eq!(config1.socks_port, 8081);
    }

    #[test]
    fn test_default_config_path() {
        let path = Config::default_config_path();
        assert!(path.to_string_lossy().contains("cat_proxy"));
    }
}
