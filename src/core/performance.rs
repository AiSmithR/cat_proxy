//! 性能优化模块
//!
//! 提供零拷贝、缓冲区优化等性能提升功能

use tokio::io::{AsyncRead, AsyncWrite};
use std::io::Result;
use tracing::{debug, error};

/// 零拷贝双向数据转发
///
/// 使用 tokio 的 copy_bidirectional 实现高效的双向数据转发
/// 相比手动实现，减少了内存拷贝和系统调用次数
pub async fn zero_copy_relay<S1, S2>(stream1: &mut S1, stream2: &mut S2) -> Result<(u64, u64)>
where
    S1: AsyncRead + AsyncWrite + Unpin,
    S2: AsyncRead + AsyncWrite + Unpin,
{
    tokio::io::copy_bidirectional(stream1, stream2).await
}

/// 带统计的零拷贝转发
///
/// 在零拷贝转发的基础上添加流量统计
pub async fn zero_copy_relay_with_stats<S1, S2>(
    stream1: &mut S1,
    stream2: &mut S2,
    connection_id: u64,
) -> Result<(u64, u64)>
where
    S1: AsyncRead + AsyncWrite + Unpin,
    S2: AsyncRead + AsyncWrite + Unpin,
{
    debug!("开始零拷贝转发，连接 ID: {}", connection_id);

    match tokio::io::copy_bidirectional(stream1, stream2).await {
        Ok((bytes_to_server, bytes_to_client)) => {
            debug!(
                "连接 {} 转发完成: 上行 {} 字节, 下行 {} 字节",
                connection_id, bytes_to_server, bytes_to_client
            );
            Ok((bytes_to_server, bytes_to_client))
        }
        Err(e) => {
            error!("连接 {} 转发错误: {}", connection_id, e);
            Err(e)
        }
    }
}

/// 优化的缓冲区大小
///
/// 根据不同场景使用不同的缓冲区大小
#[derive(Debug, Clone, Copy)]
pub enum BufferSize {
    /// 小缓冲区（适用于控制消息）
    Small,
    /// 标准缓冲区（适用于一般数据传输）
    Standard,
    /// 大缓冲区（适用于大文件传输）
    Large,
    /// 超大缓冲区（适用于高带宽场景）
    Huge,
}

impl BufferSize {
    /// 获取缓冲区大小（字节）
    pub fn bytes(&self) -> usize {
        match self {
            BufferSize::Small => 4 * 1024,      // 4 KB
            BufferSize::Standard => 16 * 1024,  // 16 KB
            BufferSize::Large => 64 * 1024,     // 64 KB
            BufferSize::Huge => 256 * 1024,     // 256 KB
        }
    }

    /// 创建对应大小的缓冲区
    pub fn create_buffer(&self) -> Vec<u8> {
        vec![0u8; self.bytes()]
    }
}

/// 缓冲区池
///
/// 复用缓冲区，减少内存分配次数
pub struct BufferPool {
    pool: crossbeam::queue::ArrayQueue<Vec<u8>>,
    buffer_size: usize,
}

impl BufferPool {
    /// 创建新的缓冲区池
    pub fn new(capacity: usize, buffer_size: BufferSize) -> Self {
        let pool = crossbeam::queue::ArrayQueue::new(capacity);
        Self {
            pool,
            buffer_size: buffer_size.bytes(),
        }
    }

    /// 获取缓冲区
    pub fn acquire(&self) -> Vec<u8> {
        self.pool
            .pop()
            .unwrap_or_else(|| vec![0u8; self.buffer_size])
    }

    /// 归还缓冲区
    pub fn release(&self, mut buffer: Vec<u8>) {
        // 清空缓冲区
        buffer.clear();
        // 重新调整大小
        buffer.resize(self.buffer_size, 0);
        // 尝试归还到池中
        let _ = self.pool.push(buffer);
    }

    /// 获取池中可用缓冲区数量
    pub fn available(&self) -> usize {
        self.pool.len()
    }

    /// 获取池容量
    pub fn capacity(&self) -> usize {
        self.pool.capacity()
    }
}

/// 性能配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// 默认缓冲区大小
    pub default_buffer_size: BufferSize,
    /// 缓冲区池大小
    pub buffer_pool_size: usize,
    /// TCP 无延迟
    pub tcp_nodelay: bool,
    /// TCP 保活
    pub tcp_keepalive: bool,
    /// 保活时间（秒）
    pub keepalive_duration: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            default_buffer_size: BufferSize::Standard,
            buffer_pool_size: 1000,
            tcp_nodelay: true,
            tcp_keepalive: true,
            keepalive_duration: 60,
        }
    }
}

impl PerformanceConfig {
    /// 创建高性能配置
    pub fn high_performance() -> Self {
        Self {
            default_buffer_size: BufferSize::Large,
            buffer_pool_size: 2000,
            tcp_nodelay: true,
            tcp_keepalive: true,
            keepalive_duration: 30,
        }
    }

    /// 创建低内存配置
    pub fn low_memory() -> Self {
        Self {
            default_buffer_size: BufferSize::Small,
            buffer_pool_size: 100,
            tcp_nodelay: true,
            tcp_keepalive: true,
            keepalive_duration: 120,
        }
    }

    /// 应用到 TCP 流
    pub fn apply_to_tcp_stream(&self, stream: &tokio::net::TcpStream) -> Result<()> {
        if self.tcp_nodelay {
            stream.set_nodelay(true)?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            use socket2::TcpKeepalive;
            use std::time::Duration;

            if self.tcp_keepalive {
                let sock_ref = socket2::SockRef::from(stream);
                let keepalive = TcpKeepalive::new()
                    .with_time(Duration::from_secs(self.keepalive_duration));
                sock_ref.set_tcp_keepalive(&keepalive)?;
            }
        }

        Ok(())
    }
}

/// 性能统计
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// 总连接数
    pub total_connections: u64,
    /// 当前活动连接数
    pub active_connections: u64,
    /// 总传输字节数
    pub total_bytes: u64,
    /// 零拷贝次数
    pub zero_copy_count: u64,
    /// 缓冲区池命中率
    pub buffer_pool_hit_rate: f64,
}

impl PerformanceStats {
    /// 创建新的性能统计
    pub fn new() -> Self {
        Self::default()
    }

    /// 增加连接计数
    pub fn increment_connection(&mut self) {
        self.total_connections += 1;
        self.active_connections += 1;
    }

    /// 减少活动连接计数
    pub fn decrement_connection(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    /// 添加传输字节数
    pub fn add_bytes(&mut self, bytes: u64) {
        self.total_bytes += bytes;
    }

    /// 增加零拷贝计数
    pub fn increment_zero_copy(&mut self) {
        self.zero_copy_count += 1;
    }

    /// 更新缓冲区池命中率
    pub fn update_buffer_pool_hit_rate(&mut self, hit_rate: f64) {
        self.buffer_pool_hit_rate = hit_rate;
    }

    /// 获取平均每连接传输字节数
    pub fn avg_bytes_per_connection(&self) -> f64 {
        if self.total_connections > 0 {
            self.total_bytes as f64 / self.total_connections as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_size() {
        assert_eq!(BufferSize::Small.bytes(), 4 * 1024);
        assert_eq!(BufferSize::Standard.bytes(), 16 * 1024);
        assert_eq!(BufferSize::Large.bytes(), 64 * 1024);
        assert_eq!(BufferSize::Huge.bytes(), 256 * 1024);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(10, BufferSize::Standard);
        assert_eq!(pool.capacity(), 10);
        assert_eq!(pool.available(), 0);

        // 获取缓冲区
        let buffer1 = pool.acquire();
        assert_eq!(buffer1.len(), BufferSize::Standard.bytes());

        // 归还缓冲区
        pool.release(buffer1);
        assert_eq!(pool.available(), 1);

        // 再次获取应该复用
        let buffer2 = pool.acquire();
        assert_eq!(buffer2.len(), BufferSize::Standard.bytes());
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn test_performance_config() {
        let config = PerformanceConfig::default();
        assert!(config.tcp_nodelay);
        assert!(config.tcp_keepalive);

        let high_perf = PerformanceConfig::high_performance();
        assert_eq!(high_perf.buffer_pool_size, 2000);

        let low_mem = PerformanceConfig::low_memory();
        assert_eq!(low_mem.buffer_pool_size, 100);
    }

    #[test]
    fn test_performance_stats() {
        let mut stats = PerformanceStats::new();

        stats.increment_connection();
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);

        stats.add_bytes(1000);
        assert_eq!(stats.total_bytes, 1000);

        stats.decrement_connection();
        assert_eq!(stats.active_connections, 0);

        assert_eq!(stats.avg_bytes_per_connection(), 1000.0);
    }
}
