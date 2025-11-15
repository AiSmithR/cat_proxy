use super::{Router, ConnectionPool, ProxySelection, PerformanceConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};

pub struct ProxyHandler;

impl ProxyHandler {
    /// 处理 HTTP 代理请求
    pub async fn handle_http(
        mut stream: TcpStream,
        peer_addr: SocketAddr,
        router: Arc<Router>,
        pool: Arc<ConnectionPool>,
    ) -> anyhow::Result<()> {
        debug!("New HTTP connection from {}", peer_addr);

        // 读取 HTTP 请求头
        let mut buffer = vec![0u8; 8192];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        let first_line = request.lines().next().unwrap_or("");

        debug!("HTTP Request: {}", first_line);

        // 解析 HTTP 请求
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid HTTP request"));
        }

        let method = parts[0];
        let url = parts[1];

        if method == "CONNECT" {
            // HTTPS 代理（CONNECT 方法）
            Self::handle_https_connect(stream, url, peer_addr, router, pool).await
        } else {
            // HTTP 代理
            Self::handle_http_forward(stream, &buffer[..n], url, peer_addr, router, pool).await
        }
    }

    /// 处理 HTTPS CONNECT
    async fn handle_https_connect(
        mut client_stream: TcpStream,
        target: &str,
        peer_addr: SocketAddr,
        router: Arc<Router>,
        pool: Arc<ConnectionPool>,
    ) -> anyhow::Result<()> {
        debug!("HTTPS CONNECT to: {}", target);

        // 添加连接记录
        let conn_id = pool.add_connection(peer_addr.to_string(), target.to_string());

        // 根据路由规则选择代理
        let selection = router.select_proxy(target).await?;
        debug!("Route selection: {:?}", selection);

        // 连接到目标（目前只实现直连）
        let remote_stream = match selection {
            ProxySelection::Direct => TcpStream::connect(target).await?,
            ProxySelection::Reject => {
                return Err(anyhow::anyhow!("Connection rejected by rules"));
            }
            ProxySelection::Proxy(_) => {
                // TODO: 通过代理连接
                TcpStream::connect(target).await?
            }
        };

        // 发送 200 Connection Established
        client_stream
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;

        // 双向转发流量
        let result = Self::relay_traffic(client_stream, remote_stream).await;

        // 移除连接记录
        pool.remove_connection(&conn_id);

        result
    }

    /// 处理 HTTP 转发
    async fn handle_http_forward(
        client_stream: TcpStream,
        request_data: &[u8],
        url: &str,
        peer_addr: SocketAddr,
        router: Arc<Router>,
        pool: Arc<ConnectionPool>,
    ) -> anyhow::Result<()> {
        debug!("HTTP forward to: {}", url);

        // 解析目标地址
        let target = Self::parse_http_target(url)?;

        // 添加连接记录
        let conn_id = pool.add_connection(peer_addr.to_string(), target.clone());

        // 根据路由规则选择代理
        let selection = router.select_proxy(&target).await?;
        debug!("Route selection: {:?}", selection);

        // 连接到目标
        let mut remote_stream = match selection {
            ProxySelection::Direct => TcpStream::connect(&target).await?,
            ProxySelection::Reject => {
                return Err(anyhow::anyhow!("Connection rejected by rules"));
            }
            ProxySelection::Proxy(_) => {
                // TODO: 通过代理连接
                TcpStream::connect(&target).await?
            }
        };

        // 转发原始请求
        remote_stream.write_all(request_data).await?;

        // 双向转发流量
        let result = Self::relay_traffic(client_stream, remote_stream).await;

        // 移除连接记录
        pool.remove_connection(&conn_id);

        result
    }

    /// 处理 SOCKS5 代理请求
    pub async fn handle_socks5(
        mut stream: TcpStream,
        peer_addr: SocketAddr,
        router: Arc<Router>,
        pool: Arc<ConnectionPool>,
    ) -> anyhow::Result<()> {
        debug!("New SOCKS5 connection from {}", peer_addr);

        // 1. 读取客户端认证方法
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await?;

        let version = buf[0];
        let nmethods = buf[1];

        if version != 5 {
            return Err(anyhow::anyhow!("Unsupported SOCKS version: {}", version));
        }

        // 读取认证方法列表
        let mut methods = vec![0u8; nmethods as usize];
        stream.read_exact(&mut methods).await?;

        // 响应：选择无认证方法 (0x00)
        stream.write_all(&[5, 0]).await?;

        // 2. 读取客户端连接请求
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await?;

        let version = buf[0];
        let cmd = buf[1];
        // let _rsv = buf[2];
        let atyp = buf[3];

        if version != 5 {
            return Err(anyhow::anyhow!("Invalid SOCKS version in request"));
        }

        if cmd != 1 {
            // 只支持 CONNECT 命令 (0x01)
            stream.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
            return Err(anyhow::anyhow!("Unsupported SOCKS5 command: {}", cmd));
        }

        // 读取目标地址
        let target = Self::read_socks5_address(&mut stream, atyp).await?;
        debug!("SOCKS5 target: {}", target);

        // 添加连接记录
        let conn_id = pool.add_connection(peer_addr.to_string(), target.clone());

        // 根据路由规则选择代理
        let selection = router.select_proxy(&target).await?;
        debug!("Route selection: {:?}", selection);

        // 连接到目标
        let remote_stream = match selection {
            ProxySelection::Direct => TcpStream::connect(&target).await?,
            ProxySelection::Reject => {
                stream.write_all(&[5, 2, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
                return Err(anyhow::anyhow!("Connection rejected by rules"));
            }
            ProxySelection::Proxy(_) => {
                // TODO: 通过代理连接
                TcpStream::connect(&target).await?
            }
        };

        // 响应成功
        stream.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]).await?;

        // 双向转发流量
        let result = Self::relay_traffic(stream, remote_stream).await;

        // 移除连接记录
        pool.remove_connection(&conn_id);

        result
    }

    /// 读取 SOCKS5 地址
    async fn read_socks5_address(
        stream: &mut TcpStream,
        atyp: u8,
    ) -> anyhow::Result<String> {
        match atyp {
            1 => {
                // IPv4
                let mut addr = [0u8; 4];
                stream.read_exact(&mut addr).await?;
                let mut port_buf = [0u8; 2];
                stream.read_exact(&mut port_buf).await?;
                let port = u16::from_be_bytes(port_buf);
                Ok(format!("{}.{}.{}.{}:{}", addr[0], addr[1], addr[2], addr[3], port))
            }
            3 => {
                // 域名
                let mut len = [0u8; 1];
                stream.read_exact(&mut len).await?;
                let mut domain = vec![0u8; len[0] as usize];
                stream.read_exact(&mut domain).await?;
                let mut port_buf = [0u8; 2];
                stream.read_exact(&mut port_buf).await?;
                let port = u16::from_be_bytes(port_buf);
                Ok(format!("{}:{}", String::from_utf8_lossy(&domain), port))
            }
            4 => {
                // IPv6
                let mut addr = [0u8; 16];
                stream.read_exact(&mut addr).await?;
                let mut port_buf = [0u8; 2];
                stream.read_exact(&mut port_buf).await?;
                let port = u16::from_be_bytes(port_buf);
                Ok(format!(
                    "[{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}]:{}",
                    addr[0], addr[1], addr[2], addr[3], addr[4], addr[5], addr[6], addr[7],
                    addr[8], addr[9], addr[10], addr[11], addr[12], addr[13], addr[14], addr[15], port
                ))
            }
            _ => Err(anyhow::anyhow!("Unsupported address type: {}", atyp)),
        }
    }

    /// 解析 HTTP 目标地址
    fn parse_http_target(url: &str) -> anyhow::Result<String> {
        // 移除协议前缀
        let url = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        // 分离域名/IP 和路径
        let host = url.split('/').next().unwrap_or(url);

        // 如果没有端口，添加默认端口
        if host.contains(':') {
            Ok(host.to_string())
        } else {
            Ok(format!("{}:80", host))
        }
    }

    /// 双向转发流量（零拷贝优化）
    async fn relay_traffic(
        mut client_stream: TcpStream,
        mut remote_stream: TcpStream,
    ) -> anyhow::Result<()> {
        // 应用性能配置
        let perf_config = PerformanceConfig::default();
        if let Err(e) = perf_config.apply_to_tcp_stream(&client_stream) {
            debug!("Failed to apply performance config to client stream: {}", e);
        }
        if let Err(e) = perf_config.apply_to_tcp_stream(&remote_stream) {
            debug!("Failed to apply performance config to remote stream: {}", e);
        }

        // 使用零拷贝双向转发
        match tokio::io::copy_bidirectional(&mut client_stream, &mut remote_stream).await {
            Ok((upload, download)) => {
                info!("Connection closed. Upload: {} bytes, Download: {} bytes", upload, download);
                Ok(())
            }
            Err(e) => {
                error!("Relay error: {}", e);
                Err(e.into())
            }
        }
    }
}

