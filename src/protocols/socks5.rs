//! SOCKS5 代理协议实现

use super::ProxyProtocol;
use crate::config::ProxyConfig;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

/// SOCKS5 代理
pub struct Socks5Proxy {
    #[allow(dead_code)]
    server: String,
    port: u16,
    #[allow(dead_code)]
    username: Option<String>,
    #[allow(dead_code)]
    password: Option<String>,
}

impl Socks5Proxy {
    /// 从配置创建
    pub fn from_config(config: &ProxyConfig) -> anyhow::Result<Self> {
        let username = config
            .extra
            .get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let password = config
            .extra
            .get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Self {
            server: config.server.clone(),
            port: config.port,
            username,
            password,
        })
    }
}

#[async_trait]
impl ProxyProtocol for Socks5Proxy {
    fn name(&self) -> &str {
        "socks5"
    }

    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream> {
        debug!(
            "Connecting through SOCKS5: {}:{} to {}",
            self.server, self.port, target
        );

        // 连接到 SOCKS5 服务器
        let mut stream = TcpStream::connect(format!("{}:{}", self.server, self.port)).await?;

        // SOCKS5 握手
        // 1. 发送认证方法
        let auth_method = if self.username.is_some() { 0x02 } else { 0x00 };
        stream.write_all(&[0x05, 0x01, auth_method]).await?;

        // 2. 读取服务器选择的认证方法
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await?;

        if buf[0] != 0x05 {
            return Err(anyhow::anyhow!("Invalid SOCKS5 version"));
        }

        // TODO: 实现用户名密码认证

        // 3. 发送连接请求
        // 解析目标地址
        let (host, port) = Self::parse_target(target)?;
        
        // 构造请求
        let mut request = vec![0x05, 0x01, 0x00]; // VER, CMD=CONNECT, RSV
        
        // 地址类型和地址
        request.push(0x03); // ATYP=DOMAIN
        request.push(host.len() as u8);
        request.extend_from_slice(host.as_bytes());
        request.extend_from_slice(&port.to_be_bytes());

        stream.write_all(&request).await?;

        // 4. 读取响应
        let mut response = [0u8; 10];
        stream.read_exact(&mut response[..4]).await?;

        if response[1] != 0x00 {
            return Err(anyhow::anyhow!("SOCKS5 connection failed: {}", response[1]));
        }

        // 读取剩余的地址信息
        match response[3] {
            0x01 => {
                stream.read_exact(&mut response[4..10]).await?; // IPv4
            }
            0x03 => {
                let mut len = [0u8; 1];
                stream.read_exact(&mut len).await?;
                let mut addr = vec![0u8; len[0] as usize + 2];
                stream.read_exact(&mut addr).await?;
            }
            0x04 => {
                let mut addr = [0u8; 18];
                stream.read_exact(&mut addr).await?; // IPv6
            }
            _ => return Err(anyhow::anyhow!("Unknown address type")),
        }

        Ok(stream)
    }
}

impl Socks5Proxy {
    fn parse_target(target: &str) -> anyhow::Result<(String, u16)> {
        if let Some(pos) = target.rfind(':') {
            let host = target[..pos].to_string();
            let port = target[pos + 1..].parse::<u16>()?;
            Ok((host, port))
        } else {
            Err(anyhow::anyhow!("Invalid target format"))
        }
    }
}
