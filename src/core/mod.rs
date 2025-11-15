use crate::config::Config;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};

pub mod proxy;
pub mod router;
pub mod connection;
pub mod health_check;
pub mod proxy_group;
pub mod performance;

pub use proxy::ProxyHandler;
pub use router::{Router, ProxySelection};
pub use connection::ConnectionPool;
pub use health_check::{HealthChecker, HealthCheckManager, HealthCheckResult};
pub use proxy_group::{ProxyGroup, ProxyGroupManager};
pub use performance::{
    zero_copy_relay, zero_copy_relay_with_stats, BufferSize, BufferPool,
    PerformanceConfig, PerformanceStats,
};

pub type Result<T> = anyhow::Result<T>;

pub struct ProxyServer {
    config: Arc<Config>,
    router: Arc<Router>,
    connection_pool: Arc<ConnectionPool>,
}

impl ProxyServer {
    pub fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let router = Arc::new(Router::new(config.clone())?);
        let connection_pool = Arc::new(ConnectionPool::new());

        Ok(Self {
            config,
            router,
            connection_pool,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Cat Proxy Server...");
        info!("Mode: {:?}", self.config.mode);
        info!("HTTP Proxy Port: {}", self.config.port);
        info!("SOCKS5 Proxy Port: {}", self.config.socks_port);

        let http_handle = self.start_http_proxy();
        let socks_handle = self.start_socks_proxy();

        tokio::try_join!(http_handle, socks_handle)?;
        Ok(())
    }

    async fn start_http_proxy(&self) -> Result<()> {
        let addr = if self.config.allow_lan {
            format!("0.0.0.0:{}", self.config.port)
        } else {
            format!("127.0.0.1:{}", self.config.port)
        };

        let listener = TcpListener::bind(&addr).await?;
        info!("HTTP Proxy listening on {}", addr);

        let router = self.router.clone();
        let pool = self.connection_pool.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let router = router.clone();
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            if let Err(e) = ProxyHandler::handle_http(stream, peer_addr, router, pool).await {
                                error!("HTTP proxy error: {}", e);
                            }
                        });
                    }
                    Err(e) => error!("Failed to accept HTTP connection: {}", e),
                }
            }
        });

        Ok(())
    }

    async fn start_socks_proxy(&self) -> Result<()> {
        let addr = if self.config.allow_lan {
            format!("0.0.0.0:{}", self.config.socks_port)
        } else {
            format!("127.0.0.1:{}", self.config.socks_port)
        };

        let listener = TcpListener::bind(&addr).await?;
        info!("SOCKS5 Proxy listening on {}", addr);

        let router = self.router.clone();
        let pool = self.connection_pool.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let router = router.clone();
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            if let Err(e) = ProxyHandler::handle_socks5(stream, peer_addr, router, pool).await {
                                error!("SOCKS5 proxy error: {}", e);
                            }
                        });
                    }
                    Err(e) => error!("Failed to accept SOCKS5 connection: {}", e),
                }
            }
        });

        Ok(())
    }
}
