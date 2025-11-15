//! IP 数据包解析模块
//!
//! 解析从 TUN 设备读取的 IP 数据包

use std::net::{Ipv4Addr, Ipv6Addr};
use anyhow::{anyhow, Result};

/// IP 协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpProtocol {
    Tcp,
    Udp,
    Icmp,
    Other(u8),
}

impl IpProtocol {
    /// 获取协议编号
    pub fn number(&self) -> u8 {
        match self {
            IpProtocol::Tcp => 6,
            IpProtocol::Udp => 17,
            IpProtocol::Icmp => 1,
            IpProtocol::Other(n) => *n,
        }
    }
}

impl From<u8> for IpProtocol {
    fn from(value: u8) -> Self {
        match value {
            6 => IpProtocol::Tcp,
            17 => IpProtocol::Udp,
            1 => IpProtocol::Icmp,
            other => IpProtocol::Other(other),
        }
    }
}

/// IP 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    V4,
    V6,
}

/// IP 数据包
#[derive(Debug, Clone)]
pub struct IpPacket {
    /// IP 版本
    pub version: IpVersion,
    /// 源地址
    pub src_addr: IpAddress,
    /// 目标地址
    pub dst_addr: IpAddress,
    /// 协议类型
    pub protocol: IpProtocol,
    /// 数据包总长度
    pub total_length: u16,
    /// 头部长度
    pub header_length: usize,
    /// 原始数据
    pub raw_data: Vec<u8>,
}

/// IP 地址（支持 IPv4 和 IPv6）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpAddress {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl std::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddress::V4(addr) => write!(f, "{}", addr),
            IpAddress::V6(addr) => write!(f, "{}", addr),
        }
    }
}

impl IpPacket {
    /// 从原始字节解析 IP 数据包
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(anyhow!("数据包太短，无法解析"));
        }

        // 读取版本号（前 4 位）
        let version_ihl = data[0];
        let version = (version_ihl >> 4) & 0x0F;

        match version {
            4 => Self::parse_ipv4(data),
            6 => Self::parse_ipv6(data),
            _ => Err(anyhow!("不支持的 IP 版本: {}", version)),
        }
    }

    /// 解析 IPv4 数据包
    fn parse_ipv4(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(anyhow!("IPv4 数据包太短"));
        }

        let version_ihl = data[0];
        let ihl = (version_ihl & 0x0F) as usize;
        let header_length = ihl * 4;

        if data.len() < header_length {
            return Err(anyhow!("IPv4 头部不完整"));
        }

        // 读取总长度
        let total_length = u16::from_be_bytes([data[2], data[3]]);

        // 读取协议
        let protocol = IpProtocol::from(data[9]);

        // 读取源地址
        let src_addr = Ipv4Addr::new(data[12], data[13], data[14], data[15]);

        // 读取目标地址
        let dst_addr = Ipv4Addr::new(data[16], data[17], data[18], data[19]);

        Ok(IpPacket {
            version: IpVersion::V4,
            src_addr: IpAddress::V4(src_addr),
            dst_addr: IpAddress::V4(dst_addr),
            protocol,
            total_length,
            header_length,
            raw_data: data.to_vec(),
        })
    }

    /// 解析 IPv6 数据包
    fn parse_ipv6(data: &[u8]) -> Result<Self> {
        if data.len() < 40 {
            return Err(anyhow!("IPv6 数据包太短"));
        }

        // IPv6 头部固定为 40 字节
        let header_length = 40;

        // 读取负载长度
        let payload_length = u16::from_be_bytes([data[4], data[5]]);
        let total_length = 40 + payload_length;

        // 读取下一个头部（协议类型）
        let protocol = IpProtocol::from(data[6]);

        // 读取源地址（16 字节）
        let src_bytes: [u8; 16] = data[8..24].try_into()
            .map_err(|_| anyhow!("无法读取源地址"))?;
        let src_addr = Ipv6Addr::from(src_bytes);

        // 读取目标地址（16 字节）
        let dst_bytes: [u8; 16] = data[24..40].try_into()
            .map_err(|_| anyhow!("无法读取目标地址"))?;
        let dst_addr = Ipv6Addr::from(dst_bytes);

        Ok(IpPacket {
            version: IpVersion::V6,
            src_addr: IpAddress::V6(src_addr),
            dst_addr: IpAddress::V6(dst_addr),
            protocol,
            total_length,
            header_length,
            raw_data: data.to_vec(),
        })
    }

    /// 获取数据包负载（不包括 IP 头部）
    pub fn payload(&self) -> &[u8] {
        &self.raw_data[self.header_length..]
    }

    /// 获取目标地址字符串
    pub fn dst_addr_string(&self) -> String {
        self.dst_addr.to_string()
    }

    /// 检查是否是 TCP 协议
    pub fn is_tcp(&self) -> bool {
        self.protocol == IpProtocol::Tcp
    }

    /// 检查是否是 UDP 协议
    pub fn is_udp(&self) -> bool {
        self.protocol == IpProtocol::Udp
    }

    /// 检查是否是 ICMP 协议
    pub fn is_icmp(&self) -> bool {
        self.protocol == IpProtocol::Icmp
    }
}

/// TCP 数据包
#[derive(Debug, Clone)]
pub struct TcpPacket {
    /// 源端口
    pub src_port: u16,
    /// 目标端口
    pub dst_port: u16,
    /// 序列号
    pub seq_number: u32,
    /// 确认号
    pub ack_number: u32,
    /// 数据偏移（头部长度）
    pub data_offset: u8,
    /// 标志位
    pub flags: TcpFlags,
    /// 窗口大小
    pub window_size: u16,
    /// 校验和
    pub checksum: u16,
    /// 紧急指针
    pub urgent_pointer: u16,
    /// 原始数据
    pub raw_data: Vec<u8>,
}

/// TCP 标志位
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl TcpPacket {
    /// 从 IP 数据包的负载解析 TCP 数据包
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(anyhow!("TCP 数据包太短"));
        }

        // 读取源端口
        let src_port = u16::from_be_bytes([data[0], data[1]]);

        // 读取目标端口
        let dst_port = u16::from_be_bytes([data[2], data[3]]);

        // 读取序列号
        let seq_number = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        // 读取确认号
        let ack_number = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        // 读取数据偏移和标志位
        let data_offset = (data[12] >> 4) & 0x0F;
        let flags_byte = data[13];

        let flags = TcpFlags {
            fin: (flags_byte & 0x01) != 0,
            syn: (flags_byte & 0x02) != 0,
            rst: (flags_byte & 0x04) != 0,
            psh: (flags_byte & 0x08) != 0,
            ack: (flags_byte & 0x10) != 0,
            urg: (flags_byte & 0x20) != 0,
        };

        // 读取窗口大小
        let window_size = u16::from_be_bytes([data[14], data[15]]);

        // 读取校验和
        let checksum = u16::from_be_bytes([data[16], data[17]]);

        // 读取紧急指针
        let urgent_pointer = u16::from_be_bytes([data[18], data[19]]);

        Ok(TcpPacket {
            src_port,
            dst_port,
            seq_number,
            ack_number,
            data_offset,
            flags,
            window_size,
            checksum,
            urgent_pointer,
            raw_data: data.to_vec(),
        })
    }

    /// 获取 TCP 负载数据
    pub fn payload(&self) -> &[u8] {
        let header_length = (self.data_offset as usize) * 4;
        if self.raw_data.len() > header_length {
            &self.raw_data[header_length..]
        } else {
            &[]
        }
    }

    /// 检查是否是 SYN 包
    pub fn is_syn(&self) -> bool {
        self.flags.syn && !self.flags.ack
    }

    /// 检查是否是 SYN-ACK 包
    pub fn is_syn_ack(&self) -> bool {
        self.flags.syn && self.flags.ack
    }

    /// 检查是否是 FIN 包
    pub fn is_fin(&self) -> bool {
        self.flags.fin
    }

    /// 检查是否是 RST 包
    pub fn is_rst(&self) -> bool {
        self.flags.rst
    }
}

/// UDP 数据包
#[derive(Debug, Clone)]
pub struct UdpPacket {
    /// 源端口
    pub src_port: u16,
    /// 目标端口
    pub dst_port: u16,
    /// 数据包长度
    pub length: u16,
    /// 校验和
    pub checksum: u16,
    /// 原始数据
    pub raw_data: Vec<u8>,
}

impl UdpPacket {
    /// 从 IP 数据包的负载解析 UDP 数据包
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(anyhow!("UDP 数据包太短"));
        }

        // 读取源端口
        let src_port = u16::from_be_bytes([data[0], data[1]]);

        // 读取目标端口
        let dst_port = u16::from_be_bytes([data[2], data[3]]);

        // 读取长度
        let length = u16::from_be_bytes([data[4], data[5]]);

        // 读取校验和
        let checksum = u16::from_be_bytes([data[6], data[7]]);

        Ok(UdpPacket {
            src_port,
            dst_port,
            length,
            checksum,
            raw_data: data.to_vec(),
        })
    }

    /// 获取 UDP 负载数据
    pub fn payload(&self) -> &[u8] {
        if self.raw_data.len() > 8 {
            &self.raw_data[8..]
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4_packet() {
        // 构造一个简单的 IPv4 数据包
        let mut packet = vec![0u8; 28];

        // 版本 (4) + IHL (5) = 0x45
        packet[0] = 0x45;

        // 总长度 28 字节
        packet[2..4].copy_from_slice(&28u16.to_be_bytes());

        // 协议：TCP (6)
        packet[9] = 6;

        // 源地址：192.168.1.1
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]);

        // 目标地址：8.8.8.8
        packet[16..20].copy_from_slice(&[8, 8, 8, 8]);

        let ip_packet = IpPacket::parse(&packet).unwrap();

        assert_eq!(ip_packet.version, IpVersion::V4);
        assert_eq!(ip_packet.protocol, IpProtocol::Tcp);
        assert_eq!(ip_packet.dst_addr, IpAddress::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert_eq!(ip_packet.header_length, 20);
    }

    #[test]
    fn test_parse_tcp_packet() {
        // 构造一个简单的 TCP 数据包
        let mut packet = vec![0u8; 20];

        // 源端口：8080
        packet[0..2].copy_from_slice(&8080u16.to_be_bytes());

        // 目标端口：443
        packet[2..4].copy_from_slice(&443u16.to_be_bytes());

        // 序列号
        packet[4..8].copy_from_slice(&12345u32.to_be_bytes());

        // 确认号
        packet[8..12].copy_from_slice(&67890u32.to_be_bytes());

        // 数据偏移 (5) + 保留位
        packet[12] = 0x50;

        // 标志位：SYN
        packet[13] = 0x02;

        let tcp_packet = TcpPacket::parse(&packet).unwrap();

        assert_eq!(tcp_packet.src_port, 8080);
        assert_eq!(tcp_packet.dst_port, 443);
        assert!(tcp_packet.is_syn());
        assert!(!tcp_packet.is_syn_ack());
    }

    #[test]
    fn test_parse_udp_packet() {
        // 构造一个简单的 UDP 数据包
        let mut packet = vec![0u8; 16];

        // 源端口：53
        packet[0..2].copy_from_slice(&53u16.to_be_bytes());

        // 目标端口：12345
        packet[2..4].copy_from_slice(&12345u16.to_be_bytes());

        // 长度：16
        packet[4..6].copy_from_slice(&16u16.to_be_bytes());

        // 校验和
        packet[6..8].copy_from_slice(&0u16.to_be_bytes());

        // 负载数据
        packet[8..16].copy_from_slice(b"testdata");

        let udp_packet = UdpPacket::parse(&packet).unwrap();

        assert_eq!(udp_packet.src_port, 53);
        assert_eq!(udp_packet.dst_port, 12345);
        assert_eq!(udp_packet.length, 16);
        assert_eq!(udp_packet.payload(), b"testdata");
    }
}
