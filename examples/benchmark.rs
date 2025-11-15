//! 性能基准测试
//!
//! 用于测试和比较不同优化策略的性能

use cat_proxy::core::{
    BufferSize, BufferPool, PerformanceConfig, PerformanceStats,
};

fn main() {
    println!("Cat Proxy 性能基准测试\n");

    // 测试缓冲区性能
    benchmark_buffer_sizes();

    // 测试缓冲区池性能
    benchmark_buffer_pool();

    // 测试性能配置
    benchmark_performance_configs();

    println!("\n性能基准测试完成！");
}

/// 测试不同缓冲区大小的性能
fn benchmark_buffer_sizes() {
    println!("=== 缓冲区大小测试 ===");

    let sizes = [
        BufferSize::Small,
        BufferSize::Standard,
        BufferSize::Large,
        BufferSize::Huge,
    ];

    for size in &sizes {
        let buffer = size.create_buffer();
        println!("{:?}: {} 字节 ({} KB)", size, buffer.len(), buffer.len() / 1024);
    }
    println!();
}

/// 测试缓冲区池性能
fn benchmark_buffer_pool() {
    println!("=== 缓冲区池性能测试 ===");

    let pool = BufferPool::new(100, BufferSize::Standard);
    println!("缓冲区池容量: {}", pool.capacity());

    // 测试获取和归还
    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let buffer = pool.acquire();
        pool.release(buffer);
    }

    let duration = start.elapsed();
    println!(
        "完成 {} 次获取/归还操作: {:?}",
        iterations, duration
    );
    println!("平均每次操作: {:?}", duration / iterations);
    println!("池中可用缓冲区: {}", pool.available());
    println!();
}

/// 测试性能配置
fn benchmark_performance_configs() {
    println!("=== 性能配置测试 ===");

    let configs = vec![
        ("默认配置", PerformanceConfig::default()),
        ("高性能配置", PerformanceConfig::high_performance()),
        ("低内存配置", PerformanceConfig::low_memory()),
    ];

    for (name, config) in configs {
        println!("{}: ", name);
        println!("  缓冲区大小: {:?} ({} KB)",
            config.default_buffer_size,
            config.default_buffer_size.bytes() / 1024
        );
        println!("  缓冲区池大小: {}", config.buffer_pool_size);
        println!("  TCP NoDelay: {}", config.tcp_nodelay);
        println!("  TCP KeepAlive: {} ({}秒)",
            config.tcp_keepalive,
            config.keepalive_duration
        );
    }
    println!();
}

/// 测试性能统计
#[allow(dead_code)]
fn benchmark_performance_stats() {
    println!("=== 性能统计测试 ===");

    let mut stats = PerformanceStats::new();

    // 模拟多个连接
    for i in 0..100 {
        stats.increment_connection();
        stats.add_bytes((i + 1) * 1000);
        stats.increment_zero_copy();

        if i % 10 == 0 {
            stats.decrement_connection();
        }
    }

    println!("总连接数: {}", stats.total_connections);
    println!("活动连接数: {}", stats.active_connections);
    println!("总传输字节: {} ({} MB)",
        stats.total_bytes,
        stats.total_bytes / 1024 / 1024
    );
    println!("零拷贝次数: {}", stats.zero_copy_count);
    println!("平均每连接字节数: {:.2}", stats.avg_bytes_per_connection());
    println!();
}
