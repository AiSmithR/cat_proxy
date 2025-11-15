//! TUN 模式实现模块
//!
//! TUN 模式通过创建虚拟网卡实现透明代理，无需手动配置应用程序代理

mod packet;
mod tcp_handler;
mod udp_handler;

pub use packet::{IpPacket, TcpPacket, UdpPacket, IpProtocol};
pub use tcp_handler::TcpConnectionManager;
pub use udp_handler::UdpSessionManager;

use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use crate::core::Router;

/// TUN 设备配置
#[derive(Debug, Clone)]
pub struct TunConfig {
    /// 设备名称
    pub name: String,
    /// IP 地址
    pub address: Ipv4Addr,
    /// 子网掩码
    pub netmask: Ipv4Addr,
    /// MTU
    pub mtu: u16,
    /// 是否启用
    pub enabled: bool,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            name: "utun99".to_string(), // macOS: utun, Linux: tun, Windows: TAP
            address: Ipv4Addr::new(10, 0, 0, 1),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            mtu: 1500,
            enabled: false,
        }
    }
}

/// TUN 设备状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

/// TUN 设备
pub struct TunDevice {
    config: TunConfig,
    state: Arc<parking_lot::RwLock<TunState>>,
    router: Option<Arc<Router>>,
    tcp_manager: Option<Arc<TcpConnectionManager>>,
    udp_manager: Option<Arc<UdpSessionManager>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl TunDevice {
    /// 创建新的 TUN 设备
    pub fn new(config: TunConfig) -> anyhow::Result<Self> {
        info!("创建 TUN 设备: {}", config.name);

        Ok(Self {
            config,
            state: Arc::new(parking_lot::RwLock::new(TunState::Stopped)),
            router: None,
            tcp_manager: None,
            udp_manager: None,
            shutdown_tx: None,
        })
    }

    /// 使用默认配置创建
    pub fn with_default() -> anyhow::Result<Self> {
        Self::new(TunConfig::default())
    }

    /// 设置路由器
    pub fn set_router(&mut self, router: Arc<Router>) {
        self.tcp_manager = Some(Arc::new(TcpConnectionManager::new(router.clone())));
        self.udp_manager = Some(Arc::new(UdpSessionManager::new(router.clone())));
        self.router = Some(router);
    }

    /// 启动 TUN 设备
    pub async fn start(&mut self) -> anyhow::Result<()> {
        if !self.config.enabled {
            warn!("TUN 设备未启用");
            return Ok(());
        }

        let state = self.get_state();
        if state == TunState::Running {
            warn!("TUN 设备已经在运行");
            return Ok(());
        }

        info!("启动 TUN 设备: {}", self.config.name);
        self.set_state(TunState::Starting);

        // 创建关闭信号通道
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // 在实际实现中，这里应该：
        // 1. 创建 TUN 设备
        // 2. 配置 IP 地址和路由
        // 3. 启动数据包处理循环

        match self.start_tun_device().await {
            Ok(_) => {
                self.set_state(TunState::Running);
                info!("TUN 设备已启动");

                // 启动 UDP 清理任务
                if let Some(ref udp_manager) = self.udp_manager {
                    udp_manager.start_cleanup_task();
                }

                // 启动数据包处理任务
                let config = self.config.clone();
                let state = self.state.clone();
                let tcp_manager = self.tcp_manager.clone();
                let udp_manager = self.udp_manager.clone();

                tokio::spawn(async move {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            debug!("收到关闭信号");
                        }
                        _ = Self::process_packets(config, tcp_manager, udp_manager) => {
                            debug!("数据包处理结束");
                        }
                    }

                    *state.write() = TunState::Stopped;
                });

                Ok(())
            }
            Err(e) => {
                error!("启动 TUN 设备失败: {}", e);
                self.set_state(TunState::Error);
                Err(e)
            }
        }
    }

    /// 停止 TUN 设备
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        let state = self.get_state();
        if state != TunState::Running {
            debug!("TUN 设备未运行，无需停止");
            return Ok(());
        }

        info!("停止 TUN 设备: {}", self.config.name);
        self.set_state(TunState::Stopping);

        // 发送关闭信号
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // 在实际实现中，这里应该：
        // 1. 停止数据包处理
        // 2. 删除路由表项
        // 3. 删除 TUN 设备

        self.set_state(TunState::Stopped);
        info!("TUN 设备已停止");

        Ok(())
    }

    /// 获取设备状态
    pub fn get_state(&self) -> TunState {
        *self.state.read()
    }

    /// 设置设备状态
    fn set_state(&self, state: TunState) {
        *self.state.write() = state;
    }

    /// 启动 TUN 设备（平台特定实现）
    async fn start_tun_device(&self) -> anyhow::Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.start_tun_macos().await
        }

        #[cfg(target_os = "linux")]
        {
            self.start_tun_linux().await
        }

        #[cfg(target_os = "windows")]
        {
            self.start_tun_windows().await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(anyhow::anyhow!("当前平台不支持 TUN 模式"))
        }
    }

    #[cfg(target_os = "macos")]
    async fn start_tun_macos(&self) -> anyhow::Result<()> {
        use std::process::Command;

        info!("在 macOS 上创建 TUN 设备");

        // macOS 使用 utun 设备
        // 实际实现需要使用系统调用创建 utun 设备
        // 这里仅作为框架示例

        // 配置 IP 地址
        let output = Command::new("ifconfig")
            .args(&[
                &self.config.name,
                &self.config.address.to_string(),
                &self.config.netmask.to_string(),
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                debug!("TUN 设备 IP 配置成功");
                Ok(())
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("配置 TUN 设备失败: {}", stderr))
            }
            Err(e) => {
                Err(anyhow::anyhow!("执行 ifconfig 失败: {}", e))
            }
        }
    }

    #[cfg(target_os = "linux")]
    async fn start_tun_linux(&self) -> anyhow::Result<()> {
        use std::process::Command;

        info!("在 Linux 上创建 TUN 设备");

        // Linux 使用 ip 命令配置 TUN 设备
        // 实际实现需要使用 ioctl 创建 TUN 设备

        // 创建 TUN 设备
        let output = Command::new("ip")
            .args(&["tuntap", "add", "mode", "tun", &self.config.name])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("创建 TUN 设备失败: {}", stderr));
        }

        // 配置 IP 地址
        let addr_str = format!("{}/{}", self.config.address, 24);
        let output = Command::new("ip")
            .args(&["addr", "add", &addr_str, "dev", &self.config.name])
            .output()?;

        if !output.status.success() {
            warn!("配置 TUN 设备地址失败（可能已配置）");
        }

        // 启用设备
        let output = Command::new("ip")
            .args(&["link", "set", &self.config.name, "up"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("启用 TUN 设备失败: {}", stderr));
        }

        debug!("Linux TUN 设备配置成功");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn start_tun_windows(&self) -> anyhow::Result<()> {
        warn!("Windows TUN 模式尚未完全实现");
        // Windows 需要使用 TAP-Windows 适配器或 Wintun
        // 这是一个更复杂的实现，需要安装驱动程序
        Ok(())
    }

    /// 处理数据包（核心转发逻辑）
    async fn process_packets(
        _config: TunConfig,
        _tcp_manager: Option<Arc<TcpConnectionManager>>,
        _udp_manager: Option<Arc<UdpSessionManager>>,
    ) {
        // 实际实现中，这里应该：
        // 1. 从 TUN 设备读取 IP 数据包
        // 2. 解析数据包，提取目标地址
        // 3. 根据路由规则决定处理方式
        // 4. 转发到代理服务器或直接连接
        // 5. 将响应数据包写回 TUN 设备

        debug!("TUN 数据包处理循环开始");

        // 模拟数据包处理
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // 在实际实现中，这里会：
            // 1. 从 TUN 设备读取原始 IP 数据包
            // 例如：let packet_data = tun_device.read().await;

            // 2. 解析 IP 数据包
            // 例如：
            // if let Ok(ip_packet) = IpPacket::parse(&packet_data) {
            //     match ip_packet.protocol {
            //         IpProtocol::Tcp => {
            //             if let Some(ref tcp_mgr) = tcp_manager {
            //                 tcp_mgr.handle_packet(&ip_packet).await;
            //             }
            //         }
            //         IpProtocol::Udp => {
            //             if let Some(ref udp_mgr) = udp_manager {
            //                 udp_mgr.handle_packet(&ip_packet).await;
            //             }
            //         }
            //         _ => {
            //             debug!("不支持的协议: {:?}", ip_packet.protocol);
            //         }
            //     }
            // }
        }
    }

    /// 获取设备信息
    pub fn get_info(&self) -> TunDeviceInfo {
        TunDeviceInfo {
            name: self.config.name.clone(),
            address: self.config.address,
            netmask: self.config.netmask,
            mtu: self.config.mtu,
            state: self.get_state(),
        }
    }
}

/// TUN 设备信息
#[derive(Debug, Clone)]
pub struct TunDeviceInfo {
    pub name: String,
    pub address: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub mtu: u16,
    pub state: TunState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tun_config_default() {
        let config = TunConfig::default();
        assert_eq!(config.address, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(config.mtu, 1500);
    }

    #[test]
    fn test_tun_device_new() {
        let config = TunConfig::default();
        let device = TunDevice::new(config);
        assert!(device.is_ok());

        let device = device.unwrap();
        assert_eq!(device.get_state(), TunState::Stopped);
    }

    #[test]
    fn test_tun_device_info() {
        let config = TunConfig::default();
        let device = TunDevice::new(config).unwrap();
        let info = device.get_info();

        assert_eq!(info.state, TunState::Stopped);
        assert_eq!(info.mtu, 1500);
    }
}
