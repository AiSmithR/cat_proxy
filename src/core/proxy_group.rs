//! 代理组和负载均衡模块
//!
//! 实现代理组的负载均衡策略和故障转移

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::RwLock;
use rand::Rng;
use tracing::{debug, info, warn};
use anyhow::{anyhow, Result};

use super::health_check::HealthCheckManager;
use crate::config::{ProxyGroupConfig, ProxyGroupType};

/// 代理组
pub struct ProxyGroup {
    /// 配置
    config: ProxyGroupConfig,
    /// 当前选中的代理（用于 select 类型）
    selected_proxy: Arc<RwLock<Option<String>>>,
    /// 轮询计数器（用于 load-balance 类型）
    round_robin_counter: Arc<AtomicUsize>,
    /// 健康检查管理器（用于 url-test 类型）
    health_manager: Option<Arc<HealthCheckManager>>,
}

impl ProxyGroup {
    /// 创建新的代理组
    pub fn new(config: ProxyGroupConfig) -> Self {
        Self {
            config,
            selected_proxy: Arc::new(RwLock::new(None)),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
            health_manager: None,
        }
    }

    /// 设置健康检查管理器
    pub fn set_health_manager(&mut self, manager: Arc<HealthCheckManager>) {
        self.health_manager = Some(manager);
    }

    /// 获取组名称
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// 获取组类型
    pub fn group_type(&self) -> &ProxyGroupType {
        &self.config.group_type
    }

    /// 选择代理（根据组类型和策略）
    pub fn select_proxy(&self) -> Option<String> {
        match &self.config.group_type {
            ProxyGroupType::Select => self.select_manual(),
            ProxyGroupType::UrlTest => self.select_fastest(),
            ProxyGroupType::Fallback => self.select_fallback(),
            ProxyGroupType::LoadBalance => self.select_load_balance(),
        }
    }

    /// 手动选择模式
    fn select_manual(&self) -> Option<String> {
        let selected = self.selected_proxy.read();
        if let Some(proxy_name) = selected.as_ref() {
            if self.config.proxies.contains(proxy_name) {
                return Some(proxy_name.clone());
            }
        }

        // 如果没有选中或选中的代理不在列表中，返回第一个
        self.config.proxies.first().cloned()
    }

    /// URL 测试模式（选择延迟最低的）
    fn select_fastest(&self) -> Option<String> {
        if let Some(ref health_manager) = self.health_manager {
            // 获取所有健康的代理及其延迟
            let results = health_manager.get_results();
            let available: Vec<_> = results
                .iter()
                .filter(|r| {
                    r.is_healthy
                        && r.latency_ms.is_some()
                        && self.config.proxies.contains(&r.proxy_name)
                })
                .collect();

            if !available.is_empty() {
                // 选择延迟最低的
                let fastest = available
                    .iter()
                    .min_by_key(|r| r.latency_ms.unwrap())
                    .unwrap();

                debug!(
                    "URL Test 选择最快的代理: {} ({}ms)",
                    fastest.proxy_name,
                    fastest.latency_ms.unwrap()
                );

                return Some(fastest.proxy_name.clone());
            }
        }

        // 如果没有健康检查结果，返回第一个
        warn!("没有健康的代理，使用第一个");
        self.config.proxies.first().cloned()
    }

    /// 故障转移模式（优先使用第一个可用的）
    fn select_fallback(&self) -> Option<String> {
        if let Some(ref health_manager) = self.health_manager {
            let results = health_manager.get_results();

            // 按配置顺序查找第一个健康的代理
            for proxy_name in &self.config.proxies {
                if let Some(result) = results.iter().find(|r| &r.proxy_name == proxy_name) {
                    if result.is_healthy {
                        debug!("Fallback 选择代理: {}", proxy_name);
                        return Some(proxy_name.clone());
                    }
                }
            }

            warn!("没有健康的代理，使用第一个");
        }

        // 如果没有健康的代理，返回第一个
        self.config.proxies.first().cloned()
    }

    /// 负载均衡模式（轮询）
    fn select_load_balance(&self) -> Option<String> {
        if self.config.proxies.is_empty() {
            return None;
        }

        // 获取健康的代理列表
        let healthy_proxies = if let Some(ref health_manager) = self.health_manager {
            let results = health_manager.get_results();
            self.config
                .proxies
                .iter()
                .filter(|p| {
                    results
                        .iter()
                        .any(|r| &r.proxy_name == *p && r.is_healthy)
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            self.config.proxies.clone()
        };

        if healthy_proxies.is_empty() {
            warn!("没有健康的代理，使用全部代理列表");
            // 使用全部代理
            let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
            return Some(self.config.proxies[index % self.config.proxies.len()].clone());
        }

        // 轮询选择
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        let selected = &healthy_proxies[index % healthy_proxies.len()];

        debug!("Load Balance 选择代理: {} (轮询索引: {})", selected, index);
        Some(selected.clone())
    }

    /// 随机负载均衡
    #[allow(dead_code)]
    fn select_random(&self) -> Option<String> {
        if self.config.proxies.is_empty() {
            return None;
        }

        // 获取健康的代理列表
        let healthy_proxies = if let Some(ref health_manager) = self.health_manager {
            let results = health_manager.get_results();
            self.config
                .proxies
                .iter()
                .filter(|p| {
                    results
                        .iter()
                        .any(|r| &r.proxy_name == *p && r.is_healthy)
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            self.config.proxies.clone()
        };

        if healthy_proxies.is_empty() {
            // 从全部代理中随机选择
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..self.config.proxies.len());
            return Some(self.config.proxies[index].clone());
        }

        // 随机选择健康的代理
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..healthy_proxies.len());
        Some(healthy_proxies[index].clone())
    }

    /// 手动设置选中的代理（仅用于 select 类型）
    pub fn set_selected_proxy(&self, proxy_name: String) -> Result<()> {
        if self.config.group_type != ProxyGroupType::Select {
            return Err(anyhow!("只有 select 类型的代理组才能手动选择"));
        }

        if !self.config.proxies.contains(&proxy_name) {
            return Err(anyhow!("代理 {} 不在组中", proxy_name));
        }

        *self.selected_proxy.write() = Some(proxy_name.clone());
        info!("代理组 {} 选择: {}", self.config.name, proxy_name);
        Ok(())
    }

    /// 获取当前选中的代理
    pub fn get_selected_proxy(&self) -> Option<String> {
        self.select_proxy()
    }

    /// 获取代理列表
    pub fn get_proxies(&self) -> &[String] {
        &self.config.proxies
    }
}

/// 代理组管理器
pub struct ProxyGroupManager {
    /// 代理组列表
    groups: Arc<RwLock<Vec<Arc<ProxyGroup>>>>,
}

impl ProxyGroupManager {
    /// 创建新的代理组管理器
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加代理组
    pub fn add_group(&self, group: Arc<ProxyGroup>) {
        info!("添加代理组: {} ({:?})", group.name(), group.group_type());
        self.groups.write().push(group);
    }

    /// 获取代理组
    pub fn get_group(&self, name: &str) -> Option<Arc<ProxyGroup>> {
        self.groups
            .read()
            .iter()
            .find(|g| g.name() == name)
            .cloned()
    }

    /// 获取所有代理组
    pub fn get_all_groups(&self) -> Vec<Arc<ProxyGroup>> {
        self.groups.read().clone()
    }

    /// 从代理组选择代理
    pub fn select_from_group(&self, group_name: &str) -> Option<String> {
        if let Some(group) = self.get_group(group_name) {
            group.select_proxy()
        } else {
            warn!("找不到代理组: {}", group_name);
            None
        }
    }
}

impl Default for ProxyGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_group_select() {
        let config = ProxyGroupConfig {
            name: "test-group".to_string(),
            group_type: ProxyGroupType::Select,
            proxies: vec!["proxy1".to_string(), "proxy2".to_string()],
            url: None,
            interval: None,
        };

        let group = ProxyGroup::new(config);

        // 默认选择第一个
        assert_eq!(group.select_proxy(), Some("proxy1".to_string()));

        // 手动选择
        group.set_selected_proxy("proxy2".to_string()).unwrap();
        assert_eq!(group.select_proxy(), Some("proxy2".to_string()));
    }

    #[test]
    fn test_proxy_group_load_balance() {
        let config = ProxyGroupConfig {
            name: "test-group".to_string(),
            group_type: ProxyGroupType::LoadBalance,
            proxies: vec![
                "proxy1".to_string(),
                "proxy2".to_string(),
                "proxy3".to_string(),
            ],
            url: None,
            interval: None,
        };

        let group = ProxyGroup::new(config);

        // 轮询测试
        let proxy1 = group.select_proxy().unwrap();
        let proxy2 = group.select_proxy().unwrap();
        let proxy3 = group.select_proxy().unwrap();
        let proxy4 = group.select_proxy().unwrap();

        assert_eq!(proxy1, "proxy1");
        assert_eq!(proxy2, "proxy2");
        assert_eq!(proxy3, "proxy3");
        assert_eq!(proxy4, "proxy1"); // 回到第一个
    }

    #[test]
    fn test_proxy_group_manager() {
        let manager = ProxyGroupManager::new();

        let config = ProxyGroupConfig {
            name: "test-group".to_string(),
            group_type: ProxyGroupType::Select,
            proxies: vec!["proxy1".to_string()],
            url: None,
            interval: None,
        };

        let group = Arc::new(ProxyGroup::new(config));
        manager.add_group(group.clone());

        assert!(manager.get_group("test-group").is_some());
        assert!(manager.get_group("non-existent").is_none());
    }
}
