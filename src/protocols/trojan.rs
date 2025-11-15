//! Trojan 协议实现

use super::ProxyProtocol;
use crate::config::ProxyConfig;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use sha2::{Sha224, Digest};

/// Trojan 命令类型
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum TrojanCommand {
    Connect = 0x01,
    // UdpAssociate = 0x03, // 暂不支持 UDP
}

/// 地址类型
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum AddressType {
    IPv4 = 0x01,
    Domain = 0x03,
    IPv6 = 0x04,
}

/// Trojan 代理
pub struct TrojanProxy {
    server: String,
    port: u16,
    #[allow(dead_code)]
    password: String,
    password_hash: String,
}

impl TrojanProxy {
    /// 从配置创建
    pub fn from_config(config: &ProxyConfig) -> anyhow::Result<Self> {
        let password = config
            .extra
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing password"))?
            .to_string();

        let password_hash = Self::hash_password_str(&password);

        Ok(Self {
            server: config.server.clone(),
            port: config.port,
            password,
            password_hash,
        })
    }

    /// 生成密码哈希（SHA224）
    fn hash_password_str(password: &str) -> String {
        let mut hasher = Sha224::new();
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// 编码目标地址
    fn encode_address(&self, target: &str) -> anyhow::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // 解析目标地址
        let (host, port) = if let Some(pos) = target.rfind(':') {
            let host = &target[..pos];
            let port: u16 = target[pos + 1..]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid port"))?;
            (host, port)
        } else {
            return Err(anyhow::anyhow!("Invalid target format"));
        };

        // 判断地址类型并编码
        if let Ok(ipv4) = host.parse::<std::net::Ipv4Addr>() {
            // IPv4 地址
            buffer.push(AddressType::IPv4 as u8);
            buffer.extend_from_slice(&ipv4.octets());
        } else if let Ok(ipv6) = host.parse::<std::net::Ipv6Addr>() {
            // IPv6 地址
            buffer.push(AddressType::IPv6 as u8);
            buffer.extend_from_slice(&ipv6.octets());
        } else {
            // 域名
            buffer.push(AddressType::Domain as u8);
            if host.len() > 255 {
                return Err(anyhow::anyhow!("Domain name too long"));
            }
            buffer.push(host.len() as u8);
            buffer.extend_from_slice(host.as_bytes());
        }

        // 添加端口
        buffer.extend_from_slice(&port.to_be_bytes());

        Ok(buffer)
    }

    /// 构建 Trojan 请求
    fn build_request(&self, target: &str) -> anyhow::Result<Vec<u8>> {
        let mut request = Vec::new();

        // 1. 密码哈希（56 字节的十六进制字符串）
        request.extend_from_slice(self.password_hash.as_bytes());

        // 2. CRLF
        request.extend_from_slice(b"\r\n");

        // 3. 命令（1 字节）
        request.push(TrojanCommand::Connect as u8);

        // 4. 目标地址
        let addr = self.encode_address(target)?;
        request.extend_from_slice(&addr);

        // 5. CRLF
        request.extend_from_slice(b"\r\n");

        Ok(request)
    }
}

#[async_trait]
impl ProxyProtocol for TrojanProxy {
    fn name(&self) -> &str {
        "trojan"
    }

    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream> {
        debug!(
            "Connecting through Trojan: {}:{} to {}",
            self.server, self.port, target
        );

        // 连接到 Trojan 服务器
        let mut stream = TcpStream::connect(format!("{}:{}", self.server, self.port)).await?;

        // 构建并发送 Trojan 请求
        let request = self.build_request(target)?;
        stream.write_all(&request).await?;

        debug!("Trojan handshake sent for target: {}", target);

        // Trojan 协议没有握手响应，直接返回流
        // 后续数据可以直接通过这个流转发
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let hash = TrojanProxy::hash_password_str("password123");
        // SHA224 应该产生 56 个十六进制字符（28 字节）
        assert_eq!(hash.len(), 56);
        println!("Password hash: {}", hash);
    }

    #[test]
    fn test_encode_address_ipv4() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("password".to_string(), serde_json::Value::String("test".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::Trojan,
            server: "127.0.0.1".to_string(),
            port: 443,
            extra,
        };

        let proxy = TrojanProxy::from_config(&config).unwrap();
        let addr = proxy.encode_address("8.8.8.8:443").unwrap();

        // 类型(1) + IPv4(4) + 端口(2) = 7 字节
        assert_eq!(addr.len(), 7);
        assert_eq!(addr[0], AddressType::IPv4 as u8);
    }

    #[test]
    fn test_encode_address_domain() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("password".to_string(), serde_json::Value::String("test".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::Trojan,
            server: "127.0.0.1".to_string(),
            port: 443,
            extra,
        };

        let proxy = TrojanProxy::from_config(&config).unwrap();
        let addr = proxy.encode_address("example.com:443").unwrap();

        // 类型(1) + 长度(1) + 域名(11) + 端口(2) = 15 字节
        assert_eq!(addr.len(), 15);
        assert_eq!(addr[0], AddressType::Domain as u8);
        assert_eq!(addr[1], 11); // "example.com".len()
    }

    #[test]
    fn test_build_request() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("password".to_string(), serde_json::Value::String("test123".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::Trojan,
            server: "127.0.0.1".to_string(),
            port: 443,
            extra,
        };

        let proxy = TrojanProxy::from_config(&config).unwrap();
        let request = proxy.build_request("example.com:443").unwrap();

        // 请求应该包含：密码哈希 + CRLF + 命令 + 地址 + CRLF
        assert!(request.len() > 56); // 至少包含密码哈希

        // 验证密码哈希部分
        let hash_part = String::from_utf8_lossy(&request[..56]);
        assert_eq!(hash_part.len(), 56);

        println!("Request size: {} bytes", request.len());
    }
}
