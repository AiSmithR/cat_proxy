//! DNS 解析模块

use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// DNS 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsMode {
    /// 真实 IP 模式
    RealIp,
    /// Fake IP 模式
    FakeIp,
}

/// DNS 协议类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsProtocol {
    /// 传统 UDP/TCP DNS（系统默认）
    System,
    /// 自定义 DNS 服务器（UDP/TCP）
    Custom { servers: Vec<SocketAddr> },
}

impl Default for DnsProtocol {
    fn default() -> Self {
        Self::System
    }
}

/// DNS 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    ips: Vec<IpAddr>,
    expire_time: Instant,
}

/// Fake IP 池
struct FakeIpPool {
    /// Fake IP 范围起始地址 (默认 198.18.0.0/16)
    base_ip: u32,
    /// 当前分配的 IP 索引
    current_index: u32,
    /// 域名到 Fake IP 的映射
    domain_to_ip: HashMap<String, IpAddr>,
    /// Fake IP 到域名的映射
    ip_to_domain: HashMap<IpAddr, String>,
}

impl FakeIpPool {
    fn new() -> Self {
        // 使用 198.18.0.0/16 作为 Fake IP 池
        // 这个范围被 IANA 保留用于基准测试
        let base_ip = 0xC612_0000u32; // 198.18.0.0
        Self {
            base_ip,
            current_index: 1, // 从 198.18.0.1 开始
            domain_to_ip: HashMap::new(),
            ip_to_domain: HashMap::new(),
        }
    }

    /// 为域名分配一个 Fake IP
    fn allocate(&mut self, domain: &str) -> IpAddr {
        // 如果域名已经有 Fake IP，直接返回
        if let Some(&ip) = self.domain_to_ip.get(domain) {
            return ip;
        }

        // 分配新的 Fake IP
        let ip_u32 = self.base_ip + self.current_index;
        let ip = IpAddr::V4(Ipv4Addr::from(ip_u32));

        self.current_index += 1;
        // 避免超出 /16 范围
        if self.current_index >= 65535 {
            self.current_index = 1;
            warn!("Fake IP 池已满，重新从头开始分配");
        }

        self.domain_to_ip.insert(domain.to_string(), ip);
        self.ip_to_domain.insert(ip, domain.to_string());

        debug!("为域名 {} 分配 Fake IP: {}", domain, ip);
        ip
    }

    /// 根据 Fake IP 获取原始域名
    fn get_domain(&self, ip: &IpAddr) -> Option<String> {
        self.ip_to_domain.get(ip).cloned()
    }

    /// 检查 IP 是否为 Fake IP
    fn is_fake_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let ip_u32 = u32::from(*ipv4);
                ip_u32 >= self.base_ip && ip_u32 < self.base_ip + 65536
            }
            IpAddr::V6(_) => false,
        }
    }
}

/// DNS 解析器
pub struct DnsResolver {
    /// 底层解析器
    resolver: TokioAsyncResolver,
    /// DNS 模式
    mode: DnsMode,
    /// DNS 协议
    protocol: DnsProtocol,
    /// DNS 缓存
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Fake IP 池
    fake_ip_pool: Arc<RwLock<FakeIpPool>>,
    /// 缓存过期时间（秒）
    cache_ttl: Duration,
    /// 查询统计
    query_stats: Arc<RwLock<QueryStats>>,
}

impl DnsResolver {
    /// 创建新的 DNS 解析器（真实 IP 模式，系统默认 DNS）
    pub async fn new() -> anyhow::Result<Self> {
        Self::new_with_protocol(DnsMode::RealIp, DnsProtocol::System).await
    }

    /// 创建新的 DNS 解析器并指定模式
    pub async fn new_with_mode(mode: DnsMode) -> anyhow::Result<Self> {
        Self::new_with_protocol(mode, DnsProtocol::System).await
    }

    /// 创建新的 DNS 解析器并指定协议
    pub async fn new_with_protocol(mode: DnsMode, protocol: DnsProtocol) -> anyhow::Result<Self> {
        let (config, opts) = match &protocol {
            DnsProtocol::System => {
                (ResolverConfig::default(), ResolverOpts::default())
            }
            DnsProtocol::Custom { servers } => {
                info!("配置自定义 DNS 服务器: {:?}", servers);
                Self::create_custom_config(servers)?
            }
        };

        let resolver = TokioAsyncResolver::tokio(config, opts);

        info!("DNS 解析器已创建，模式: {:?}, 协议: {:?}", mode, protocol);

        Ok(Self {
            resolver,
            mode,
            protocol,
            cache: Arc::new(RwLock::new(HashMap::new())),
            fake_ip_pool: Arc::new(RwLock::new(FakeIpPool::new())),
            cache_ttl: Duration::from_secs(300), // 默认 5 分钟
            query_stats: Arc::new(RwLock::new(QueryStats {
                cache_hits: 0,
                cache_misses: 0,
                total_queries: 0,
            })),
        })
    }

    /// 创建自定义 DNS 配置
    fn create_custom_config(servers: &[SocketAddr]) -> anyhow::Result<(ResolverConfig, ResolverOpts)> {
        if servers.is_empty() {
            return Err(anyhow::anyhow!("自定义 DNS 服务器列表不能为空"));
        }

        let mut config = ResolverConfig::new();

        for &server in servers {
            let name_server = NameServerConfig {
                socket_addr: server,
                protocol: Protocol::Udp,
                tls_dns_name: None,
                trust_negative_responses: true,
                bind_addr: None,
            };
            config.add_name_server(name_server);
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(5);
        opts.attempts = 2;

        Ok((config, opts))
    }

    /// 创建带自定义配置的 DNS 解析器
    pub fn new_with_config(
        mode: DnsMode,
        config: ResolverConfig,
        cache_ttl: Duration,
    ) -> anyhow::Result<Self> {
        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());

        info!("DNS 解析器已创建，模式: {:?}, 缓存 TTL: {:?}", mode, cache_ttl);

        Ok(Self {
            resolver,
            mode,
            protocol: DnsProtocol::System,
            cache: Arc::new(RwLock::new(HashMap::new())),
            fake_ip_pool: Arc::new(RwLock::new(FakeIpPool::new())),
            cache_ttl,
            query_stats: Arc::new(RwLock::new(QueryStats {
                cache_hits: 0,
                cache_misses: 0,
                total_queries: 0,
            })),
        })
    }

    /// 解析域名
    pub async fn resolve(&self, domain: &str) -> anyhow::Result<Vec<IpAddr>> {
        // 记录总查询数
        {
            let mut stats = self.query_stats.write();
            stats.total_queries += 1;
        }

        match self.mode {
            DnsMode::RealIp => self.resolve_real(domain).await,
            DnsMode::FakeIp => self.resolve_fake(domain).await,
        }
    }

    /// 真实 IP 解析（带缓存）
    async fn resolve_real(&self, domain: &str) -> anyhow::Result<Vec<IpAddr>> {
        // 检查缓存
        {
            let cache = self.cache.read();
            if let Some(entry) = cache.get(domain) {
                if entry.expire_time > Instant::now() {
                    debug!("DNS 缓存命中: {}", domain);
                    // 记录缓存命中
                    {
                        let mut stats = self.query_stats.write();
                        stats.cache_hits += 1;
                    }
                    return Ok(entry.ips.clone());
                }
            }
        }

        // 记录缓存未命中
        {
            let mut stats = self.query_stats.write();
            stats.cache_misses += 1;
        }

        // 缓存未命中，执行真实解析
        debug!("DNS 查询: {}", domain);
        let response = self.resolver.lookup_ip(domain).await?;
        let ips: Vec<IpAddr> = response.iter().collect();

        // 更新缓存
        {
            let mut cache = self.cache.write();
            cache.insert(
                domain.to_string(),
                CacheEntry {
                    ips: ips.clone(),
                    expire_time: Instant::now() + self.cache_ttl,
                },
            );
        }

        Ok(ips)
    }

    /// Fake IP 解析
    async fn resolve_fake(&self, domain: &str) -> anyhow::Result<Vec<IpAddr>> {
        let mut pool = self.fake_ip_pool.write();
        let ip = pool.allocate(domain);
        Ok(vec![ip])
    }

    /// 根据 IP 获取原始域名（仅 Fake IP 模式）
    pub fn get_domain_by_ip(&self, ip: &IpAddr) -> Option<String> {
        let pool = self.fake_ip_pool.read();
        pool.get_domain(ip)
    }

    /// 检查 IP 是否为 Fake IP
    pub fn is_fake_ip(&self, ip: &IpAddr) -> bool {
        let pool = self.fake_ip_pool.read();
        pool.is_fake_ip(ip)
    }

    /// 清空 DNS 缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        info!("DNS 缓存已清空");
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let pool = self.fake_ip_pool.read();
        let stats = self.query_stats.read();

        let now = Instant::now();
        let valid_entries = cache.values().filter(|entry| entry.expire_time > now).count();

        CacheStats {
            total_entries: cache.len(),
            valid_entries,
            fake_ip_mappings: pool.domain_to_ip.len(),
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            total_queries: stats.total_queries,
        }
    }

    /// 设置 DNS 模式
    pub fn set_mode(&mut self, mode: DnsMode) {
        info!("DNS 模式已切换: {:?} -> {:?}", self.mode, mode);
        self.mode = mode;
    }

    /// 获取当前 DNS 模式
    pub fn mode(&self) -> DnsMode {
        self.mode
    }

    /// 获取当前 DNS 协议
    pub fn protocol(&self) -> &DnsProtocol {
        &self.protocol
    }

    /// 切换 DNS 协议（需要重新创建解析器）
    pub async fn switch_protocol(&mut self, protocol: DnsProtocol) -> anyhow::Result<()> {
        info!("切换 DNS 协议: {:?} -> {:?}", self.protocol, protocol);

        let (config, opts) = match &protocol {
            DnsProtocol::System => {
                (ResolverConfig::default(), ResolverOpts::default())
            }
            DnsProtocol::Custom { servers } => {
                Self::create_custom_config(servers)?
            }
        };

        self.resolver = TokioAsyncResolver::tokio(config, opts);
        self.protocol = protocol;

        // 清空缓存（因为协议变更可能导致结果不同）
        self.clear_cache();

        Ok(())
    }

    /// 使用 Cloudflare DNS (1.1.1.1) 创建解析器
    pub async fn new_with_cloudflare(mode: DnsMode) -> anyhow::Result<Self> {
        let servers = vec![
            "1.1.1.1:53".parse()?,
            "1.0.0.1:53".parse()?,
        ];
        Self::new_with_protocol(mode, DnsProtocol::Custom { servers }).await
    }

    /// 使用 Google DNS (8.8.8.8) 创建解析器
    pub async fn new_with_google(mode: DnsMode) -> anyhow::Result<Self> {
        let servers = vec![
            "8.8.8.8:53".parse()?,
            "8.8.4.4:53".parse()?,
        ];
        Self::new_with_protocol(mode, DnsProtocol::Custom { servers }).await
    }

    /// 使用 Quad9 DNS (9.9.9.9) 创建解析器
    pub async fn new_with_quad9(mode: DnsMode) -> anyhow::Result<Self> {
        let servers = vec![
            "9.9.9.9:53".parse()?,
            "149.112.112.112:53".parse()?,
        ];
        Self::new_with_protocol(mode, DnsProtocol::Custom { servers }).await
    }

    /// 使用阿里 DNS (223.5.5.5) 创建解析器
    pub async fn new_with_ali(mode: DnsMode) -> anyhow::Result<Self> {
        let servers = vec![
            "223.5.5.5:53".parse()?,
            "223.6.6.6:53".parse()?,
        ];
        Self::new_with_protocol(mode, DnsProtocol::Custom { servers }).await
    }
}

/// DNS 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub fake_ip_mappings: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_queries: u64,
}

/// DNS 查询统计
#[derive(Debug, Clone)]
struct QueryStats {
    cache_hits: u64,
    cache_misses: u64,
    total_queries: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_resolve_real() {
        let resolver = DnsResolver::new().await.unwrap();
        let ips = resolver.resolve("google.com").await.unwrap();
        assert!(!ips.is_empty());
        println!("google.com 解析结果: {:?}", ips);
    }

    #[tokio::test]
    async fn test_dns_resolve_fake() {
        let resolver = DnsResolver::new_with_mode(DnsMode::FakeIp).await.unwrap();

        // 第一次解析
        let ips1 = resolver.resolve("example.com").await.unwrap();
        assert_eq!(ips1.len(), 1);

        // 第二次解析应该返回相同的 Fake IP
        let ips2 = resolver.resolve("example.com").await.unwrap();
        assert_eq!(ips1, ips2);

        // 验证反向查询
        let domain = resolver.get_domain_by_ip(&ips1[0]);
        assert_eq!(domain, Some("example.com".to_string()));

        println!("example.com 分配的 Fake IP: {:?}", ips1);
    }

    #[tokio::test]
    async fn test_dns_cache() {
        let resolver = DnsResolver::new().await.unwrap();

        // 第一次查询
        let start = Instant::now();
        let ips1 = resolver.resolve("google.com").await.unwrap();
        let duration1 = start.elapsed();

        // 第二次查询（应该从缓存读取）
        let start = Instant::now();
        let ips2 = resolver.resolve("google.com").await.unwrap();
        let duration2 = start.elapsed();

        assert_eq!(ips1, ips2);
        assert!(duration2 < duration1, "缓存查询应该更快");

        println!("首次查询耗时: {:?}, 缓存查询耗时: {:?}", duration1, duration2);
    }

    #[test]
    fn test_fake_ip_pool() {
        let mut pool = FakeIpPool::new();

        // 分配 Fake IP
        let ip1 = pool.allocate("example.com");
        let ip2 = pool.allocate("google.com");
        let ip3 = pool.allocate("example.com"); // 应该返回相同的 IP

        assert_eq!(ip1, ip3);
        assert_ne!(ip1, ip2);

        // 测试反向查询
        assert_eq!(pool.get_domain(&ip1), Some("example.com".to_string()));
        assert_eq!(pool.get_domain(&ip2), Some("google.com".to_string()));

        // 测试 Fake IP 检测
        assert!(pool.is_fake_ip(&ip1));
        assert!(pool.is_fake_ip(&ip2));
        assert!(!pool.is_fake_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let resolver = DnsResolver::new().await.unwrap();

        // 执行一些查询
        let _ = resolver.resolve("google.com").await;
        let _ = resolver.resolve("github.com").await;

        let stats = resolver.get_cache_stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 2);

        println!("缓存统计: {:?}", stats);
    }
}
