//! UDP 数据包处理模块
//!
//! 处理从 TUN 设备捕获的 UDP 数据包

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use anyhow::Result;

use super::packet::{IpPacket, UdpPacket};
use crate::core::Router;

/// UDP 会话信息
#[derive(Debug)]
pub struct UdpSession {
    /// 源地址和端口
    #[allow(dead_code)]
    pub src_addr: String,
    /// 目标地址和端口
    #[allow(dead_code)]
    pub dst_addr: String,
    /// UDP 套接字
    pub socket: Arc<UdpSocket>,
    /// 最后活动时间
    pub last_activity: Instant,
}

/// UDP 会话管理器
pub struct UdpSessionManager {
    /// 活动会话表
    sessions: Arc<Mutex<HashMap<String, UdpSession>>>,
    /// 路由器
    #[allow(dead_code)]
    router: Arc<Router>,
    /// 会话超时时间（秒）
    timeout: Duration,
}

impl UdpSessionManager {
    /// 创建新的 UDP 会话管理器
    pub fn new(router: Arc<Router>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            router,
            timeout: Duration::from_secs(60), // 60 秒超时
        }
    }

    /// 处理 UDP 数据包
    pub async fn handle_packet(&self, ip_packet: &IpPacket) -> Result<Option<Vec<u8>>> {
        let udp_packet = UdpPacket::parse(ip_packet.payload())?;

        let src_addr = format!("{}:{}", ip_packet.src_addr, udp_packet.src_port);
        let dst_addr = format!("{}:{}", ip_packet.dst_addr, udp_packet.dst_port);

        debug!(
            "处理 UDP 数据包: {} -> {} ({} 字节)",
            src_addr,
            dst_addr,
            udp_packet.payload().len()
        );

        // 获取或创建会话
        let session_key = format!("{}->{}", src_addr, dst_addr);
        let socket = self.get_or_create_session(&session_key, &src_addr, &dst_addr).await?;

        // 转发数据
        let payload = udp_packet.payload();
        if !payload.is_empty() {
            self.forward_data(&socket, &dst_addr, payload).await?;
        }

        Ok(None)
    }

    /// 获取或创建 UDP 会话
    async fn get_or_create_session(
        &self,
        session_key: &str,
        src_addr: &str,
        dst_addr: &str,
    ) -> Result<Arc<UdpSocket>> {
        let mut sessions = self.sessions.lock().await;

        // 检查是否存在会话
        if let Some(session) = sessions.get_mut(session_key) {
            // 更新最后活动时间
            session.last_activity = Instant::now();
            return Ok(session.socket.clone());
        }

        // 创建新会话
        info!("创建新的 UDP 会话: {} -> {}", src_addr, dst_addr);

        let socket = self.create_udp_socket().await?;
        let socket = Arc::new(socket);

        let session = UdpSession {
            src_addr: src_addr.to_string(),
            dst_addr: dst_addr.to_string(),
            socket: socket.clone(),
            last_activity: Instant::now(),
        };

        sessions.insert(session_key.to_string(), session);

        // 启动接收任务
        self.start_receive_task(session_key.to_string(), socket.clone()).await;

        Ok(socket)
    }

    /// 创建 UDP 套接字
    async fn create_udp_socket(&self) -> Result<UdpSocket> {
        // 绑定到随机端口
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        debug!("创建 UDP 套接字: {:?}", socket.local_addr()?);
        Ok(socket)
    }

    /// 转发数据到目标
    async fn forward_data(
        &self,
        socket: &UdpSocket,
        dst_addr: &str,
        data: &[u8],
    ) -> Result<()> {
        // 解析目标地址
        let addr: SocketAddr = dst_addr.parse()?;

        // 在实际实现中，这里应该：
        // 1. 通过路由器查询路由规则
        // 2. 如果需要代理，通过代理转发
        // 3. 否则直接发送

        // 这里先实现直接发送
        debug!("转发 UDP 数据: {} -> {} ({} 字节)", socket.local_addr()?, dst_addr, data.len());
        socket.send_to(data, addr).await?;

        Ok(())
    }

    /// 启动接收任务（从套接字接收数据并写回 TUN）
    async fn start_receive_task(&self, session_key: String, socket: Arc<UdpSocket>) {
        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 65535]; // UDP 最大包大小

            loop {
                match socket.recv_from(&mut buffer).await {
                    Ok((n, from_addr)) => {
                        debug!("从 {} 接收到 {} 字节数据", from_addr, n);

                        // 在实际实现中，这里应该：
                        // 1. 将数据封装成 IP/UDP 包
                        // 2. 写回 TUN 设备

                        // 更新会话活动时间
                        let mut sessions_lock = sessions.lock().await;
                        if let Some(session) = sessions_lock.get_mut(&session_key) {
                            session.last_activity = Instant::now();
                        }
                    }
                    Err(e) => {
                        error!("从 UDP 套接字接收数据失败: {}", e);
                        break;
                    }
                }
            }

            // 清理会话
            let mut sessions_lock = sessions.lock().await;
            sessions_lock.remove(&session_key);
        });
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.lock().await;
        let now = Instant::now();

        sessions.retain(|key, session| {
            let expired = now.duration_since(session.last_activity) > self.timeout;
            if expired {
                info!("清理过期的 UDP 会话: {}", key);
            }
            !expired
        });
    }

    /// 启动定期清理任务
    pub fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let timeout = self.timeout;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                let mut sessions_lock = sessions.lock().await;
                let now = Instant::now();

                sessions_lock.retain(|key, session| {
                    let expired = now.duration_since(session.last_activity) > timeout;
                    if expired {
                        debug!("清理过期的 UDP 会话: {}", key);
                    }
                    !expired
                });
            }
        });
    }

    /// 获取活动会话数
    pub async fn active_sessions(&self) -> usize {
        self.sessions.lock().await.len()
    }

    /// 关闭所有会话
    pub async fn close_all(&self) {
        let mut sessions = self.sessions.lock().await;
        info!("关闭所有 UDP 会话 ({} 个)", sessions.len());
        sessions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_udp_session_manager() {
        let config = Arc::new(Config::default());
        let router = Arc::new(Router::new(config).unwrap());
        let manager = UdpSessionManager::new(router);

        assert_eq!(manager.active_sessions().await, 0);
    }

    #[tokio::test]
    async fn test_udp_socket_creation() {
        let config = Arc::new(Config::default());
        let router = Arc::new(Router::new(config).unwrap());
        let manager = UdpSessionManager::new(router);

        let socket = manager.create_udp_socket().await;
        assert!(socket.is_ok());

        let socket = socket.unwrap();
        let local_addr = socket.local_addr();
        assert!(local_addr.is_ok());
    }
}
