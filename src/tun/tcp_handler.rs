//! TCP 连接处理模块
//!
//! 处理从 TUN 设备捕获的 TCP 连接

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use anyhow::Result;

use super::packet::{IpPacket, TcpPacket};
use crate::core::Router;

/// TCP 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpConnectionState {
    /// 已建立连接
    Established,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
}

/// TCP 连接信息
#[derive(Debug)]
pub struct TcpConnection {
    /// 源地址和端口
    #[allow(dead_code)]
    pub src_addr: String,
    /// 目标地址和端口
    #[allow(dead_code)]
    pub dst_addr: String,
    /// 连接状态
    pub state: TcpConnectionState,
    /// 上游连接（到目标服务器或代理）
    pub upstream: Option<TcpStream>,
    /// 序列号
    #[allow(dead_code)]
    pub seq_number: u32,
    /// 确认号
    #[allow(dead_code)]
    pub ack_number: u32,
}

/// TCP 连接管理器
pub struct TcpConnectionManager {
    /// 活动连接表
    connections: Arc<Mutex<HashMap<String, TcpConnection>>>,
    /// 路由器
    #[allow(dead_code)]
    router: Arc<Router>,
}

impl TcpConnectionManager {
    /// 创建新的 TCP 连接管理器
    pub fn new(router: Arc<Router>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            router,
        }
    }

    /// 处理 TCP 数据包
    pub async fn handle_packet(&self, ip_packet: &IpPacket) -> Result<Option<Vec<u8>>> {
        let tcp_packet = TcpPacket::parse(ip_packet.payload())?;

        let src_addr = format!("{}:{}", ip_packet.src_addr, tcp_packet.src_port);
        let dst_addr = format!("{}:{}", ip_packet.dst_addr, tcp_packet.dst_port);

        debug!(
            "处理 TCP 数据包: {} -> {} (SEQ: {}, ACK: {}, Flags: SYN={} ACK={} FIN={} RST={})",
            src_addr,
            dst_addr,
            tcp_packet.seq_number,
            tcp_packet.ack_number,
            tcp_packet.flags.syn,
            tcp_packet.flags.ack,
            tcp_packet.flags.fin,
            tcp_packet.flags.rst
        );

        // 处理不同类型的 TCP 包
        if tcp_packet.is_syn() && !tcp_packet.flags.ack {
            // SYN 包：新连接
            self.handle_syn_packet(&src_addr, &dst_addr, &tcp_packet).await?;
        } else if tcp_packet.is_rst() {
            // RST 包：重置连接
            self.handle_rst_packet(&src_addr).await?;
        } else if tcp_packet.is_fin() {
            // FIN 包：关闭连接
            self.handle_fin_packet(&src_addr).await?;
        } else {
            // 数据包：转发数据
            self.handle_data_packet(&src_addr, &tcp_packet).await?;
        }

        Ok(None)
    }

    /// 处理 SYN 包（建立新连接）
    async fn handle_syn_packet(
        &self,
        src_addr: &str,
        dst_addr: &str,
        tcp_packet: &TcpPacket,
    ) -> Result<()> {
        info!("建立新的 TCP 连接: {} -> {}", src_addr, dst_addr);

        // 通过路由器决定如何连接
        let target = dst_addr.to_string();

        // 建立到目标的连接（可能通过代理）
        match self.connect_to_target(&target).await {
            Ok(upstream) => {
                let connection = TcpConnection {
                    src_addr: src_addr.to_string(),
                    dst_addr: dst_addr.to_string(),
                    state: TcpConnectionState::Established,
                    upstream: Some(upstream),
                    seq_number: tcp_packet.seq_number,
                    ack_number: tcp_packet.ack_number,
                };

                let mut connections = self.connections.lock().await;
                connections.insert(src_addr.to_string(), connection);

                debug!("TCP 连接已建立: {}", src_addr);
            }
            Err(e) => {
                error!("连接目标失败 {}: {}", dst_addr, e);
                // 在实际实现中，这里应该发送 RST 包
            }
        }

        Ok(())
    }

    /// 处理 RST 包（重置连接）
    async fn handle_rst_packet(&self, src_addr: &str) -> Result<()> {
        debug!("收到 RST 包，关闭连接: {}", src_addr);

        let mut connections = self.connections.lock().await;
        if let Some(mut connection) = connections.remove(src_addr) {
            connection.state = TcpConnectionState::Closed;
            // 关闭上游连接
            if let Some(mut upstream) = connection.upstream {
                let _ = upstream.shutdown().await;
            }
        }

        Ok(())
    }

    /// 处理 FIN 包（关闭连接）
    async fn handle_fin_packet(&self, src_addr: &str) -> Result<()> {
        debug!("收到 FIN 包，关闭连接: {}", src_addr);

        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.get_mut(src_addr) {
            connection.state = TcpConnectionState::Closing;

            // 关闭上游连接
            if let Some(mut upstream) = connection.upstream.take() {
                let _ = upstream.shutdown().await;
            }
        }

        Ok(())
    }

    /// 处理数据包（转发数据）
    async fn handle_data_packet(&self, src_addr: &str, tcp_packet: &TcpPacket) -> Result<()> {
        let payload = tcp_packet.payload();
        if payload.is_empty() {
            // 纯 ACK 包，无需处理
            return Ok(());
        }

        debug!("转发数据: {} ({} 字节)", src_addr, payload.len());

        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.get_mut(src_addr) {
            if let Some(ref mut upstream) = connection.upstream {
                // 将数据转发到上游连接
                if let Err(e) = upstream.write_all(payload).await {
                    error!("转发数据失败: {}", e);
                    connection.state = TcpConnectionState::Closed;
                } else {
                    debug!("数据转发成功: {} 字节", payload.len());
                }
            }
        } else {
            warn!("找不到连接: {}", src_addr);
        }

        Ok(())
    }

    /// 连接到目标服务器（可能通过代理）
    async fn connect_to_target(&self, target: &str) -> Result<TcpStream> {
        // 解析目标地址
        let addr: SocketAddr = target.parse()?;

        // 在实际实现中，这里应该：
        // 1. 通过路由器查询路由规则
        // 2. 如果需要代理，通过代理连接
        // 3. 否则直接连接

        // 这里先实现直接连接
        debug!("直接连接到目标: {}", target);
        let stream = TcpStream::connect(addr).await?;

        Ok(stream)
    }

    /// 启动连接读取任务（从上游读取数据并写回 TUN）
    pub async fn start_read_task(
        &self,
        src_addr: String,
    ) -> Result<()> {
        let connections = self.connections.clone();

        tokio::spawn(async move {
            loop {
                // 获取连接
                let upstream = {
                    let mut conns = connections.lock().await;
                    if let Some(connection) = conns.get_mut(&src_addr) {
                        if connection.state == TcpConnectionState::Closed {
                            break;
                        }
                        connection.upstream.as_mut().map(|s| s as *mut TcpStream)
                    } else {
                        break;
                    }
                };

                if let Some(upstream_ptr) = upstream {
                    // 从上游读取数据
                    let mut buffer = vec![0u8; 4096];
                    let upstream = unsafe { &mut *upstream_ptr };

                    match upstream.read(&mut buffer).await {
                        Ok(0) => {
                            // 连接关闭
                            debug!("上游连接关闭: {}", src_addr);
                            break;
                        }
                        Ok(n) => {
                            debug!("从上游读取 {} 字节: {}", n, src_addr);
                            // 在实际实现中，这里应该将数据封装成 IP/TCP 包
                            // 并写回 TUN 设备
                        }
                        Err(e) => {
                            error!("从上游读取失败: {}", e);
                            break;
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            // 清理连接
            let mut conns = connections.lock().await;
            if let Some(mut connection) = conns.remove(&src_addr) {
                connection.state = TcpConnectionState::Closed;
                if let Some(mut upstream) = connection.upstream {
                    let _ = upstream.shutdown().await;
                }
            }
        });

        Ok(())
    }

    /// 获取活动连接数
    pub async fn active_connections(&self) -> usize {
        self.connections.lock().await.len()
    }

    /// 关闭所有连接
    pub async fn close_all(&self) {
        let mut connections = self.connections.lock().await;
        for (_, mut connection) in connections.drain() {
            connection.state = TcpConnectionState::Closed;
            if let Some(mut upstream) = connection.upstream {
                let _ = upstream.shutdown().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_tcp_connection_manager() {
        let config = Arc::new(Config::default());
        let router = Arc::new(Router::new(config).unwrap());
        let manager = TcpConnectionManager::new(router);

        assert_eq!(manager.active_connections().await, 0);
    }
}
