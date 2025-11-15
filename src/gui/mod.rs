//! GUI 应用程序模块
//!
//! 提供 Tauri GUI 应用的后端支持

pub mod commands;

use crate::core::{ConnectionPool, PerformanceStats, HealthCheckManager};
use crate::config::Config;
use crate::logging::LogCollector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// GUI 应用状态
pub struct AppState {
    /// 配置
    pub config: Arc<RwLock<Config>>,
    /// 连接池
    pub connection_pool: Arc<ConnectionPool>,
    /// 性能统计
    pub performance_stats: Arc<RwLock<PerformanceStats>>,
    /// 代理服务器是否运行
    pub is_running: Arc<RwLock<bool>>,
    /// 健康检查管理器
    pub health_manager: Arc<HealthCheckManager>,
    /// 日志收集器
    pub log_collector: Arc<LogCollector>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
            connection_pool: Arc::new(ConnectionPool::new()),
            performance_stats: Arc::new(RwLock::new(PerformanceStats::new())),
            is_running: Arc::new(RwLock::new(false)),
            health_manager: Arc::new(HealthCheckManager::with_defaults()),
            log_collector: Arc::new(LogCollector::new(10000)), // 保留 10000 条日志
        }
    }

    /// 从配置文件加载
    pub fn from_config(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            connection_pool: Arc::new(ConnectionPool::new()),
            performance_stats: Arc::new(RwLock::new(PerformanceStats::new())),
            is_running: Arc::new(RwLock::new(false)),
            health_manager: Arc::new(HealthCheckManager::with_defaults()),
            log_collector: Arc::new(LogCollector::new(10000)),
        }
    }
}

/// 仪表板统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// 是否正在运行
    pub is_running: bool,
    /// 活跃连接数
    pub active_connections: usize,
    /// 总连接数
    pub total_connections: u64,
    /// 总上传字节数
    pub total_upload: u64,
    /// 总下载字节数
    pub total_download: u64,
    /// 零拷贝次数
    pub zero_copy_count: u64,
    /// 当前代理模式
    pub proxy_mode: String,
    /// 运行时间（秒）
    pub uptime: u64,
}

/// 代理节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNodeInfo {
    /// 节点名称
    pub name: String,
    /// 节点类型
    pub proxy_type: String,
    /// 服务器地址
    pub server: String,
    /// 端口
    pub port: u16,
    /// 是否健康
    pub is_healthy: bool,
    /// 延迟（毫秒）
    pub latency: Option<u64>,
}

/// 代理组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroupInfo {
    /// 组名称
    pub name: String,
    /// 组类型
    pub group_type: String,
    /// 代理列表
    pub proxies: Vec<String>,
    /// 当前选中的代理
    pub selected: Option<String>,
}

/// 规则信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInfo {
    /// 规则类型
    pub rule_type: String,
    /// 规则内容
    pub content: String,
    /// 目标
    pub target: String,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 操作系统
    pub os: String,
    /// 架构
    pub arch: String,
    /// CPU 核心数
    pub cpu_cores: usize,
    /// 总内存（MB）
    pub total_memory: u64,
    /// 已用内存（MB）
    pub used_memory: u64,
}

impl SystemInfo {
    /// 获取系统信息
    pub fn get() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores: num_cpus::get(),
            total_memory: 0, // 需要使用 sysinfo crate
            used_memory: 0,
        }
    }
}

/// GUI 连接信息（用于前端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConnectionInfo {
    /// 连接 ID
    pub id: String,
    /// 源地址
    pub src_addr: String,
    /// 目标地址
    pub dst_addr: String,
    /// 使用的代理
    pub proxy: String,
    /// 上传字节数
    pub upload: u64,
    /// 下载字节数
    pub download: u64,
    /// 连接持续时间（秒）
    pub duration: i64,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: String,
    /// 日志级别
    pub level: String,
    /// 消息
    pub message: String,
    /// 来源
    pub source: String,
}

/// GUI 命令响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
}

impl<T> CommandResponse<T> {
    /// 成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 错误响应
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 开机启动
    pub auto_start: bool,
    /// 启动时自动连接
    pub auto_connect: bool,
    /// 系统代理
    pub system_proxy: bool,
    /// 允许局域网连接
    pub allow_lan: bool,
    /// HTTP 端口
    pub http_port: u16,
    /// SOCKS5 端口
    pub socks_port: u16,
    /// 日志级别
    pub log_level: String,
    /// 主题
    pub theme: String,
    /// 语言
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            auto_connect: false,
            system_proxy: false,
            allow_lan: false,
            http_port: 7890,
            socks_port: 7891,
            log_level: "info".to_string(),
            theme: "auto".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state() {
        let state = AppState::new();
        assert!(!*state.is_running.read());
        assert_eq!(state.connection_pool.get_connection_count(), 0);
    }

    #[test]
    fn test_command_response() {
        let success: CommandResponse<String> = CommandResponse::success("OK".to_string());
        assert!(success.success);
        assert_eq!(success.data, Some("OK".to_string()));

        let error: CommandResponse<String> = CommandResponse::error("Failed".to_string());
        assert!(!error.success);
        assert_eq!(error.error, Some("Failed".to_string()));
    }

    #[test]
    fn test_app_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.http_port, 7890);
        assert_eq!(settings.socks_port, 7891);
        assert_eq!(settings.log_level, "info");
    }

    #[test]
    fn test_system_info() {
        let info = SystemInfo::get();
        assert!(!info.os.is_empty());
        assert!(info.cpu_cores > 0);
    }
}
