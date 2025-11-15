//! HTTP 代理协议实现

use super::ProxyProtocol;
use crate::config::ProxyConfig;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

/// HTTP 代理
pub struct HttpProxy {
    server: String,
    port: u16,
}

impl HttpProxy {
    /// 从配置创建
    pub fn from_config(config: &ProxyConfig) -> anyhow::Result<Self> {
        Ok(Self {
            server: config.server.clone(),
            port: config.port,
        })
    }
}

#[async_trait]
impl ProxyProtocol for HttpProxy {
    fn name(&self) -> &str {
        "http"
    }

    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream> {
        debug!(
            "Connecting through HTTP proxy: {}:{} to {}",
            self.server, self.port, target
        );

        // 连接到 HTTP 代理服务器
        let mut stream = TcpStream::connect(format!("{}:{}", self.server, self.port)).await?;

        // 发送 CONNECT 请求
        let request = format!(
            "CONNECT {} HTTP/1.1\r\nHost: {}\r\n\r\n",
            target, target
        );

        stream.write_all(request.as_bytes()).await?;

        // 读取响应
        let mut response = vec![0u8; 1024];
        let n = stream.read(&mut response).await?;

        let response_str = String::from_utf8_lossy(&response[..n]);
        if !response_str.contains("200") {
            return Err(anyhow::anyhow!("HTTP proxy connection failed"));
        }

        Ok(stream)
    }
}
