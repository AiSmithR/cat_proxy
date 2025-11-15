//! 代理健康检查模块
//!
//! 定期检查代理节点的健康状态和延迟

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;
use parking_lot::RwLock;
use reqwest::Client;
use tracing::{debug, error, info, warn};
use anyhow::Result;

use crate::config::ProxyConfig;

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// 代理名称
    pub proxy_name: String,
    /// 是否健康
    pub is_healthy: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 最后检查时间
    pub last_check: Instant,
    /// 连续失败次数
    pub consecutive_failures: u32,
}

impl HealthCheckResult {
    /// 创建新的健康检查结果
    pub fn new(proxy_name: String) -> Self {
        Self {
            proxy_name,
            is_healthy: false,
            latency_ms: None,
            last_check: Instant::now(),
            consecutive_failures: 0,
        }
    }

    /// 标记为健康
    pub fn mark_healthy(&mut self, latency_ms: u64) {
        self.is_healthy = true;
        self.latency_ms = Some(latency_ms);
        self.last_check = Instant::now();
        self.consecutive_failures = 0;
    }

    /// 标记为不健康
    pub fn mark_unhealthy(&mut self) {
        self.is_healthy = false;
        self.latency_ms = None;
        self.last_check = Instant::now();
        self.consecutive_failures += 1;
    }
}

/// 健康检查器
pub struct HealthChecker {
    /// 检查的 URL
    test_url: String,
    /// 超时时间
    #[allow(dead_code)]
    timeout: Duration,
    /// 检查间隔
    interval: Duration,
    /// 最大失败次数（超过此次数标记为不健康）
    #[allow(dead_code)]
    max_failures: u32,
    /// HTTP 客户端
    client: Client,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(test_url: String, timeout: Duration, interval: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            test_url,
            timeout,
            interval,
            max_failures: 3,
            client,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(
            "http://www.gstatic.com/generate_204".to_string(),
            Duration::from_secs(5),
            Duration::from_secs(300), // 5 分钟
        )
    }

    /// 检查单个代理的健康状态
    pub async fn check_proxy(&self, _proxy: &ProxyConfig) -> Result<HealthCheckResult> {
        let proxy_name = _proxy.name.clone();
        let mut result = HealthCheckResult::new(proxy_name.clone());

        debug!("开始健康检查: {}", proxy_name);

        // 记录开始时间
        let start = Instant::now();

        // 发送 HTTP 请求
        match self.client.get(&self.test_url).send().await {
            Ok(response) => {
                let latency = start.elapsed();
                let latency_ms = latency.as_millis() as u64;

                if response.status().is_success() || response.status().as_u16() == 204 {
                    result.mark_healthy(latency_ms);
                    debug!("代理 {} 健康，延迟: {}ms", proxy_name, latency_ms);
                } else {
                    result.mark_unhealthy();
                    warn!("代理 {} 返回错误状态码: {}", proxy_name, response.status());
                }
            }
            Err(e) => {
                result.mark_unhealthy();
                warn!("代理 {} 健康检查失败: {}", proxy_name, e);
            }
        }

        Ok(result)
    }

    /// 批量检查多个代理
    pub async fn check_proxies(&self, proxies: &[ProxyConfig]) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();

        for proxy in proxies {
            match self.check_proxy(proxy).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("检查代理 {} 时出错: {}", proxy.name, e);
                    let mut result = HealthCheckResult::new(proxy.name.clone());
                    result.mark_unhealthy();
                    results.push(result);
                }
            }
        }

        results
    }

    /// 启动定期健康检查任务
    pub fn start_periodic_check(
        self: Arc<Self>,
        proxies: Vec<ProxyConfig>,
        results: Arc<RwLock<Vec<HealthCheckResult>>>,
    ) {
        tokio::spawn(async move {
            let mut ticker = interval(self.interval);

            loop {
                ticker.tick().await;

                info!("执行定期健康检查，共 {} 个代理", proxies.len());

                let check_results = self.check_proxies(&proxies).await;

                // 更新结果
                let mut results_lock = results.write();
                *results_lock = check_results.clone();

                // 打印统计信息
                let healthy_count = check_results.iter().filter(|r| r.is_healthy).count();
                info!(
                    "健康检查完成: {}/{} 个代理健康",
                    healthy_count,
                    proxies.len()
                );

                // 打印详细信息
                for result in &check_results {
                    if let Some(latency) = result.latency_ms {
                        debug!("  {} - 健康 ({}ms)", result.proxy_name, latency);
                    } else {
                        debug!("  {} - 不健康", result.proxy_name);
                    }
                }
            }
        });
    }
}

/// 健康检查管理器
pub struct HealthCheckManager {
    /// 健康检查器
    checker: Arc<HealthChecker>,
    /// 检查结果
    results: Arc<RwLock<Vec<HealthCheckResult>>>,
}

impl HealthCheckManager {
    /// 创建新的健康检查管理器
    pub fn new(test_url: String, timeout: Duration, interval: Duration) -> Self {
        let checker = Arc::new(HealthChecker::new(test_url, timeout, interval));
        let results = Arc::new(RwLock::new(Vec::new()));

        Self { checker, results }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        let checker = Arc::new(HealthChecker::with_defaults());
        let results = Arc::new(RwLock::new(Vec::new()));

        Self { checker, results }
    }

    /// 启动健康检查
    pub fn start(&self, proxies: Vec<ProxyConfig>) {
        info!("启动健康检查管理器，监控 {} 个代理", proxies.len());

        // 初始化结果
        {
            let mut results_lock = self.results.write();
            *results_lock = proxies
                .iter()
                .map(|p| HealthCheckResult::new(p.name.clone()))
                .collect();
        }

        // 启动定期检查任务
        self.checker
            .clone()
            .start_periodic_check(proxies, self.results.clone());
    }

    /// 获取所有健康检查结果
    pub fn get_results(&self) -> Vec<HealthCheckResult> {
        self.results.read().clone()
    }

    /// 获取指定代理的健康状态
    pub fn get_proxy_status(&self, proxy_name: &str) -> Option<HealthCheckResult> {
        self.results
            .read()
            .iter()
            .find(|r| r.proxy_name == proxy_name)
            .cloned()
    }

    /// 获取所有健康的代理
    pub fn get_healthy_proxies(&self) -> Vec<String> {
        self.results
            .read()
            .iter()
            .filter(|r| r.is_healthy)
            .map(|r| r.proxy_name.clone())
            .collect()
    }

    /// 获取延迟最低的代理
    pub fn get_fastest_proxy(&self) -> Option<String> {
        self.results
            .read()
            .iter()
            .filter(|r| r.is_healthy && r.latency_ms.is_some())
            .min_by_key(|r| r.latency_ms.unwrap())
            .map(|r| r.proxy_name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result() {
        let mut result = HealthCheckResult::new("test-proxy".to_string());
        assert!(!result.is_healthy);
        assert_eq!(result.consecutive_failures, 0);

        result.mark_healthy(100);
        assert!(result.is_healthy);
        assert_eq!(result.latency_ms, Some(100));
        assert_eq!(result.consecutive_failures, 0);

        result.mark_unhealthy();
        assert!(!result.is_healthy);
        assert_eq!(result.consecutive_failures, 1);
    }

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::with_defaults();
        assert_eq!(checker.test_url, "http://www.gstatic.com/generate_204");
        assert_eq!(checker.max_failures, 3);
    }

    #[test]
    fn test_health_check_manager() {
        let manager = HealthCheckManager::with_defaults();
        assert_eq!(manager.get_results().len(), 0);
    }
}
