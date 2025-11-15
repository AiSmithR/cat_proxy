//! Shadowsocks 协议实现

use super::ProxyProtocol;
use crate::config::ProxyConfig;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use sha2::{Sha256, Digest};

/// Shadowsocks 代理
pub struct ShadowsocksProxy {
    server: String,
    port: u16,
    password: String,
    cipher: String,
}

impl ShadowsocksProxy {
    /// 从配置创建
    pub fn from_config(config: &ProxyConfig) -> anyhow::Result<Self> {
        let password = config
            .extra
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing password"))?
            .to_string();

        let cipher = config
            .extra
            .get("cipher")
            .and_then(|v| v.as_str())
            .unwrap_or("aes-256-gcm")
            .to_string();

        Ok(Self {
            server: config.server.clone(),
            port: config.port,
            password,
            cipher,
        })
    }

    /// 派生加密密钥
    fn derive_key(&self, salt: &[u8]) -> Vec<u8> {
        // EVP_BytesToKey 简化实现
        let mut hasher = Sha256::new();
        hasher.update(self.password.as_bytes());
        hasher.update(salt);
        hasher.finalize().to_vec()[..32].to_vec()
    }

    /// 编码目标地址
    fn encode_address(&self, target: &str) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        // 解析目标地址
        let (host, port) = self.parse_target(target)?;

        // 地址类型和地址
        if let Ok(ipv4) = host.parse::<std::net::Ipv4Addr>() {
            // IPv4
            buf.push(0x01);
            buf.extend_from_slice(&ipv4.octets());
        } else if let Ok(ipv6) = host.parse::<std::net::Ipv6Addr>() {
            // IPv6
            buf.push(0x04);
            buf.extend_from_slice(&ipv6.octets());
        } else {
            // 域名
            buf.push(0x03);
            buf.push(host.len() as u8);
            buf.extend_from_slice(host.as_bytes());
        }

        // 端口
        buf.extend_from_slice(&port.to_be_bytes());

        Ok(buf)
    }

    /// 解析目标地址
    fn parse_target(&self, target: &str) -> anyhow::Result<(String, u16)> {
        if let Some(pos) = target.rfind(':') {
            let host = target[..pos].to_string();
            let port = target[pos + 1..].parse::<u16>()?;
            Ok((host, port))
        } else {
            Err(anyhow::anyhow!("Invalid target format"))
        }
    }

    /// AEAD 加密数据
    #[allow(deprecated)]
    fn encrypt_aead(&self, key: &[u8], nonce: &[u8], data: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self.cipher.as_str() {
            "aes-256-gcm" => {
                let cipher = Aes256Gcm::new_from_slice(key)
                    .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

                let nonce = Nonce::from_slice(nonce);
                let payload = Payload {
                    msg: data,
                    aad: b"",
                };

                cipher
                    .encrypt(nonce, payload)
                    .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))
            }
            "chacha20-poly1305" => {
                // TODO: 实现 ChaCha20-Poly1305
                Err(anyhow::anyhow!("ChaCha20-Poly1305 not implemented yet"))
            }
            _ => Err(anyhow::anyhow!("Unsupported cipher: {}", self.cipher)),
        }
    }
}

#[async_trait]
impl ProxyProtocol for ShadowsocksProxy {
    fn name(&self) -> &str {
        "shadowsocks"
    }

    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream> {
        debug!(
            "Connecting through Shadowsocks: {}:{} to {}",
            self.server, self.port, target
        );

        // 连接到 SS 服务器
        let mut stream = TcpStream::connect(format!("{}:{}", self.server, self.port)).await?;

        // 生成随机 salt
        let salt: [u8; 32] = rand::random();

        // 派生密钥
        let key = self.derive_key(&salt);

        // 编码目标地址
        let addr_buf = self.encode_address(target)?;

        // 发送 salt
        stream.write_all(&salt).await?;

        // 加密并发送目标地址
        let nonce: [u8; 12] = rand::random();
        let encrypted = self.encrypt_aead(&key, &nonce, &addr_buf)?;

        // 发送 nonce + 加密数据
        stream.write_all(&nonce).await?;
        stream.write_all(&encrypted).await?;

        debug!("Shadowsocks handshake completed");

        // 注意：实际的 Shadowsocks 需要包装 stream 进行后续的加密通信
        // 这里为了简化，返回原始 stream
        // 完整实现需要创建一个加密的 stream wrapper

        Ok(stream)
    }
}

// 添加 rand 依赖辅助
mod rand {
    pub fn random<T: Default + AsMut<[u8]>>() -> T {
        let mut value = T::default();
        // 简化实现：使用时间戳作为随机源
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let bytes = value.as_mut();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = ((timestamp >> (i * 8)) & 0xff) as u8;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_address() {
        let proxy = ShadowsocksProxy {
            server: "test.com".to_string(),
            port: 8388,
            password: "test".to_string(),
            cipher: "aes-256-gcm".to_string(),
        };

        // 测试 IPv4
        let encoded = proxy.encode_address("1.2.3.4:80").unwrap();
        assert_eq!(encoded[0], 0x01); // IPv4 类型
        assert_eq!(&encoded[1..5], &[1, 2, 3, 4]);

        // 测试域名
        let encoded = proxy.encode_address("example.com:443").unwrap();
        assert_eq!(encoded[0], 0x03); // 域名类型
        assert_eq!(encoded[1], 11); // 域名长度
    }

    #[test]
    fn test_derive_key() {
        let proxy = ShadowsocksProxy {
            server: "test.com".to_string(),
            port: 8388,
            password: "test_password".to_string(),
            cipher: "aes-256-gcm".to_string(),
        };

        let salt = [0u8; 32];
        let key = proxy.derive_key(&salt);
        assert_eq!(key.len(), 32);
    }
}
