use crate::config::{Config, ProxyMode};
use crate::rules::RuleMatcher;
use super::proxy_group::ProxyGroupManager;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum ProxySelection {
    Direct,
    Proxy(String),
    Reject,
}

pub struct Router {
    config: Arc<Config>,
    rule_matcher: RuleMatcher,
    group_manager: Arc<ProxyGroupManager>,
}

impl Router {
    pub fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        // 初始化规则匹配器
        let mut rule_matcher = RuleMatcher::new();

        // 加载规则
        for rule_str in &config.rules {
            match RuleMatcher::parse_rule(rule_str) {
                Ok((rule, target)) => {
                    rule_matcher.add_rule(rule, target);
                }
                Err(e) => {
                    debug!("Failed to parse rule '{}': {}", rule_str, e);
                }
            }
        }

        // 初始化代理组管理器
        let group_manager = Arc::new(ProxyGroupManager::new());

        // 从配置加载代理组
        for group_config in &config.proxy_groups {
            use super::proxy_group::ProxyGroup;
            let group = Arc::new(ProxyGroup::new(group_config.clone()));
            group_manager.add_group(group);
        }

        Ok(Self {
            config,
            rule_matcher,
            group_manager,
        })
    }

    pub async fn select_proxy(&self, target: &str) -> anyhow::Result<ProxySelection> {
        debug!("Selecting proxy for target: {}", target);

        match self.config.mode {
            ProxyMode::Direct => {
                // 直连模式
                Ok(ProxySelection::Direct)
            }
            ProxyMode::Global => {
                // 全局代理模式 - 使用第一个可用代理
                self.select_global_proxy()
            }
            ProxyMode::Rule => {
                // 规则模式 - 根据规则匹配
                self.match_rules(target).await
            }
            ProxyMode::Script => {
                // 脚本模式 - 暂未实现
                debug!("Script mode not implemented yet, using direct");
                Ok(ProxySelection::Direct)
            }
        }
    }

    /// 选择全局代理
    fn select_global_proxy(&self) -> anyhow::Result<ProxySelection> {
        // 尝试从第一个代理组中选择
        if let Some(proxy_group_config) = self.config.proxy_groups.first() {
            if let Some(proxy_name) = self.group_manager.select_from_group(&proxy_group_config.name) {
                return Ok(ProxySelection::Proxy(proxy_name));
            }
        }

        // 否则使用第一个代理
        if let Some(proxy) = self.config.proxies.first() {
            return Ok(ProxySelection::Proxy(proxy.name.clone()));
        }

        // 没有配置代理，使用直连
        Ok(ProxySelection::Direct)
    }

    /// 匹配规则
    async fn match_rules(&self, target: &str) -> anyhow::Result<ProxySelection> {
        // 解析目标地址
        let (domain, _port) = self.parse_target(target)?;

        debug!("Matching rules for domain: {}", domain);

        // 使用规则匹配器匹配域名
        if let Some(target_name) = self.rule_matcher.match_domain(&domain) {
            return self.resolve_target(&target_name);
        }

        // TODO: 添加 IP 规则匹配
        // TODO: 添加 GeoIP 匹配

        // 默认直连
        Ok(ProxySelection::Direct)
    }

    /// 解析目标（代理名称或 DIRECT/REJECT）
    fn resolve_target(&self, target_name: &str) -> anyhow::Result<ProxySelection> {
        match target_name.to_uppercase().as_str() {
            "DIRECT" => Ok(ProxySelection::Direct),
            "REJECT" => Ok(ProxySelection::Reject),
            _ => {
                // 检查是否是代理组
                if let Some(proxy_name) = self.group_manager.select_from_group(target_name) {
                    return Ok(ProxySelection::Proxy(proxy_name));
                }

                // 否则直接作为代理名称
                Ok(ProxySelection::Proxy(target_name.to_string()))
            }
        }
    }

    /// 解析目标地址
    fn parse_target(&self, target: &str) -> anyhow::Result<(String, u16)> {
        // 移除协议前缀
        let target = target
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        // 移除 IPv6 方括号
        let target = if target.starts_with('[') {
            if let Some(end) = target.find(']') {
                &target[1..end]
            } else {
                target
            }
        } else {
            target
        };

        // 分离域名和端口
        if let Some(pos) = target.rfind(':') {
            let domain = target[..pos].to_string();
            let port_str = target[pos + 1..]
                .split('/')
                .next()
                .unwrap_or("80");
            let port = port_str.parse::<u16>().unwrap_or(80);
            Ok((domain, port))
        } else {
            let domain = target.split('/').next().unwrap_or(target).to_string();
            Ok((domain, 80))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_parse_target() {
        let config = Arc::new(Config::default());
        let router = Router::new(config).unwrap();

        let (domain, port) = router.parse_target("example.com:443").unwrap();
        assert_eq!(domain, "example.com");
        assert_eq!(port, 443);

        let (domain, port) = router.parse_target("http://example.com").unwrap();
        assert_eq!(domain, "example.com");
        assert_eq!(port, 80);
    }

    #[tokio::test]
    async fn test_direct_mode() {
        let mut config = Config::default();
        config.mode = ProxyMode::Direct;
        let config = Arc::new(config);
        let router = Router::new(config).unwrap();

        let selection = router.select_proxy("example.com:443").await.unwrap();
        assert!(matches!(selection, ProxySelection::Direct));
    }
}
