//! VMess 协议实现

use super::ProxyProtocol;
use crate::config::ProxyConfig;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// VMess 版本
const VMESS_VERSION: u8 = 1;

/// VMess 命令
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum VMessCommand {
    Tcp = 0x01,
    // Udp = 0x02, // 暂不支持
}

/// 地址类型
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum AddressType {
    IPv4 = 0x01,
    Domain = 0x02,
    IPv6 = 0x03,
}

/// 安全类型
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
enum SecurityType {
    Aes128Gcm = 0x03,
    ChaCha20Poly1305 = 0x04,
    None = 0x05,
}

/// VMess 代理
pub struct VMessProxy {
    /// 服务器地址
    pub server: String,
    /// 端口
    pub port: u16,
    /// 用户ID
    pub uuid: String,
    /// Alter ID
    #[allow(dead_code)]
    alter_id: u16,
}

impl VMessProxy {
    /// 从配置创建
    pub fn from_config(config: &ProxyConfig) -> anyhow::Result<Self> {
        let uuid = config
            .extra
            .get("uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing uuid"))?
            .to_string();

        let alter_id = config
            .extra
            .get("alterId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;

        Ok(Self {
            server: config.server.clone(),
            port: config.port,
            uuid,
            alter_id,
        })
    }

    /// 解析 UUID 为字节数组
    fn parse_uuid(&self) -> anyhow::Result<[u8; 16]> {
        let uuid_str = self.uuid.replace('-', "");
        if uuid_str.len() != 32 {
            return Err(anyhow::anyhow!("Invalid UUID format"));
        }

        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&uuid_str[i * 2..i * 2 + 2], 16)?;
        }
        Ok(bytes)
    }

    /// 生成认证信息
    fn generate_auth_info(&self) -> anyhow::Result<Vec<u8>> {
        let uuid_bytes = self.parse_uuid()?;

        // 获取当前时间戳（Unix 时间，秒）
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        // 构建认证数据
        let mut auth_data = Vec::new();
        auth_data.extend_from_slice(&uuid_bytes);
        auth_data.extend_from_slice(&timestamp.to_be_bytes());

        // 使用 UUID 作为密钥计算 HMAC
        let mut mac = <HmacSha256 as Mac>::new_from_slice(&uuid_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid HMAC key"))?;
        mac.update(&auth_data);
        let hmac_result = mac.finalize().into_bytes();

        // 返回前 16 字节作为认证信息
        Ok(hmac_result[..16].to_vec())
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

        // 添加端口（大端序）
        buffer.extend_from_slice(&port.to_be_bytes());

        Ok(buffer)
    }

    /// 构建 VMess 请求头
    fn build_request_header(&self, target: &str) -> anyhow::Result<Vec<u8>> {
        let mut header = Vec::new();

        // 1. 版本号
        header.push(VMESS_VERSION);

        // 2. 数据加密 IV（16 字节随机）
        let iv: [u8; 16] = rand::random();
        header.extend_from_slice(&iv);

        // 3. 数据加密密钥（16 字节随机）
        let key: [u8; 16] = rand::random();
        header.extend_from_slice(&key);

        // 4. 响应认证 V
        let response_auth: u8 = rand::random();
        header.push(response_auth);

        // 5. 选项（P 位）
        header.push(0x01); // 标准 TCP 连接

        // 6. 余量（P & 0x0F）
        let padding_length: u8 = rand::random::<u8>() % 16;
        header.push(padding_length);

        // 7. 加密方式
        header.push(SecurityType::Aes128Gcm as u8);

        // 8. 保留字节
        header.push(0x00);

        // 9. 命令
        header.push(VMessCommand::Tcp as u8);

        // 10. 目标地址
        let addr = self.encode_address(target)?;
        header.extend_from_slice(&addr);

        // 11. 随机填充
        for _ in 0..padding_length {
            header.push(rand::random());
        }

        // 12. 校验和 F（前面所有内容的 FNV1a hash 的前 4 字节）
        let checksum = self.calculate_checksum(&header);
        header.extend_from_slice(&checksum);

        Ok(header)
    }

    /// 计算校验和（简化版FNV1a）
    fn calculate_checksum(&self, data: &[u8]) -> [u8; 4] {
        let mut hash: u32 = 0x811c9dc5;
        for &byte in data {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash.to_be_bytes()
    }

    /// 构建完整的 VMess 请求
    fn build_request(&self, target: &str) -> anyhow::Result<Vec<u8>> {
        let mut request = Vec::new();

        // 1. 认证信息（16 字节）
        let auth = self.generate_auth_info()?;
        request.extend_from_slice(&auth);

        // 2. 请求头
        let header = self.build_request_header(target)?;

        // 3. 简化实现：不对请求头进行加密
        // 完整实现需要使用 AES-128-CFB 加密
        request.extend_from_slice(&header);

        Ok(request)
    }
}

// 简化的随机数生成模块
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEED: AtomicU64 = AtomicU64::new(12345);

    fn next_u64() -> u64 {
        let old = SEED.load(Ordering::Relaxed);
        let new_value = old.wrapping_mul(6364136223846793005).wrapping_add(1);
        SEED.store(new_value, Ordering::Relaxed);
        new_value
    }

    pub fn random<T: RandomValue>() -> T {
        T::generate()
    }

    pub trait RandomValue {
        fn generate() -> Self;
    }

    impl RandomValue for u8 {
        fn generate() -> Self {
            (next_u64() >> 56) as u8
        }
    }

    impl RandomValue for [u8; 16] {
        fn generate() -> Self {
            let mut result = [0u8; 16];
            for i in 0..16 {
                result[i] = (next_u64() >> 56) as u8;
            }
            result
        }
    }
}

#[async_trait]
impl ProxyProtocol for VMessProxy {
    fn name(&self) -> &str {
        "vmess"
    }

    async fn connect(&self, target: &str) -> anyhow::Result<TcpStream> {
        debug!(
            "Connecting through VMess: {}:{} to {}",
            self.server, self.port, target
        );

        // 连接到 VMess 服务器
        let mut stream = TcpStream::connect(format!("{}:{}", self.server, self.port)).await?;

        // 构建并发送 VMess 请求
        let request = self.build_request(target)?;
        stream.write_all(&request).await?;

        debug!("VMess handshake sent for target: {}", target);

        // VMess 服务器会返回响应头，但在简化实现中我们直接使用连接
        // 完整实现需要读取并验证响应头

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("uuid".to_string(), serde_json::Value::String("b831381d-6324-4d53-ad4f-8cda48b30811".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::VMess,
            server: "127.0.0.1".to_string(),
            port: 10086,
            extra,
        };

        let proxy = VMessProxy::from_config(&config).unwrap();
        let uuid_bytes = proxy.parse_uuid().unwrap();

        assert_eq!(uuid_bytes.len(), 16);
        println!("UUID bytes: {:02x?}", uuid_bytes);
    }

    #[test]
    fn test_encode_address_domain() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("uuid".to_string(), serde_json::Value::String("b831381d-6324-4d53-ad4f-8cda48b30811".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::VMess,
            server: "127.0.0.1".to_string(),
            port: 10086,
            extra,
        };

        let proxy = VMessProxy::from_config(&config).unwrap();
        let addr = proxy.encode_address("example.com:443").unwrap();

        // 类型(1) + 长度(1) + 域名(11) + 端口(2) = 15 字节
        assert_eq!(addr.len(), 15);
        assert_eq!(addr[0], AddressType::Domain as u8);
        assert_eq!(addr[1], 11); // "example.com".len()
    }

    #[test]
    fn test_build_request_header() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("uuid".to_string(), serde_json::Value::String("b831381d-6324-4d53-ad4f-8cda48b30811".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::VMess,
            server: "127.0.0.1".to_string(),
            port: 10086,
            extra,
        };

        let proxy = VMessProxy::from_config(&config).unwrap();
        let header = proxy.build_request_header("example.com:443").unwrap();

        // VMess 请求头应该至少包含固定字段
        assert!(header.len() > 40);
        assert_eq!(header[0], VMESS_VERSION);

        println!("Request header size: {} bytes", header.len());
    }

    #[test]
    fn test_generate_auth_info() {
        use crate::config::ProxyType;
        use std::collections::HashMap;

        let mut extra = HashMap::new();
        extra.insert("uuid".to_string(), serde_json::Value::String("b831381d-6324-4d53-ad4f-8cda48b30811".to_string()));

        let config = ProxyConfig {
            name: "test".to_string(),
            proxy_type: ProxyType::VMess,
            server: "127.0.0.1".to_string(),
            port: 10086,
            extra,
        };

        let proxy = VMessProxy::from_config(&config).unwrap();
        let auth = proxy.generate_auth_info().unwrap();

        assert_eq!(auth.len(), 16);
        println!("Auth info: {:02x?}", auth);
    }
}
