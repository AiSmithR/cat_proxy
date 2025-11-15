//! 日志管理模块
//!
//! 提供日志收集、存储和查询功能

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::Level;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 日志 ID（用于排序和去重）
    pub id: u64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志模块/来源
    pub target: String,
    /// 日志消息
    pub message: String,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(id: u64, level: LogLevel, target: String, message: String) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            level,
            target,
            message,
        }
    }
}

/// 日志过滤器
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// 最低日志级别
    pub min_level: Option<LogLevel>,
    /// 目标过滤（模块名）
    pub target: Option<String>,
    /// 消息内容搜索
    pub search: Option<String>,
}

impl LogFilter {
    /// 检查日志条目是否匹配过滤器
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // 检查日志级别
        if let Some(min_level) = self.min_level {
            let entry_level_value = match entry.level {
                LogLevel::Trace => 0,
                LogLevel::Debug => 1,
                LogLevel::Info => 2,
                LogLevel::Warn => 3,
                LogLevel::Error => 4,
            };
            let min_level_value = match min_level {
                LogLevel::Trace => 0,
                LogLevel::Debug => 1,
                LogLevel::Info => 2,
                LogLevel::Warn => 3,
                LogLevel::Error => 4,
            };
            if entry_level_value < min_level_value {
                return false;
            }
        }

        // 检查目标
        if let Some(ref target) = self.target {
            if !entry.target.contains(target) {
                return false;
            }
        }

        // 检查消息内容
        if let Some(ref search) = self.search {
            if !entry.message.to_lowercase().contains(&search.to_lowercase()) {
                return false;
            }
        }

        true
    }
}

/// 日志收集器
pub struct LogCollector {
    /// 日志缓冲区
    logs: Arc<RwLock<VecDeque<LogEntry>>>,
    /// 最大日志数量
    max_logs: usize,
    /// 下一个日志 ID
    next_id: Arc<RwLock<u64>>,
}

impl LogCollector {
    /// 创建新的日志收集器
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(max_logs))),
            max_logs,
            next_id: Arc::new(RwLock::new(0)),
        }
    }

    /// 添加日志条目
    pub fn add_log(&self, level: LogLevel, target: String, message: String) {
        let id = {
            let mut next_id = self.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let entry = LogEntry::new(id, level, target, message);

        let mut logs = self.logs.write();

        // 如果超过最大容量，移除最旧的日志
        if logs.len() >= self.max_logs {
            logs.pop_front();
        }

        logs.push_back(entry);
    }

    /// 获取所有日志
    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.read().iter().cloned().collect()
    }

    /// 获取过滤后的日志
    pub fn get_filtered_logs(&self, filter: &LogFilter) -> Vec<LogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|entry| filter.matches(entry))
            .cloned()
            .collect()
    }

    /// 获取最新的 N 条日志
    pub fn get_latest_logs(&self, count: usize) -> Vec<LogEntry> {
        let logs = self.logs.read();
        let start = if logs.len() > count {
            logs.len() - count
        } else {
            0
        };
        logs.iter().skip(start).cloned().collect()
    }

    /// 获取分页日志
    pub fn get_paginated_logs(&self, page: usize, page_size: usize, filter: Option<&LogFilter>) -> (Vec<LogEntry>, usize) {
        let all_logs: Vec<LogEntry> = if let Some(filter) = filter {
            self.get_filtered_logs(filter)
        } else {
            self.get_logs()
        };

        let total = all_logs.len();
        let start = page * page_size;
        let end = (start + page_size).min(total);

        let logs = if start < total {
            all_logs[start..end].to_vec()
        } else {
            Vec::new()
        };

        (logs, total)
    }

    /// 清空所有日志
    pub fn clear(&self) {
        self.logs.write().clear();
        *self.next_id.write() = 0;
    }

    /// 获取日志数量
    pub fn count(&self) -> usize {
        self.logs.read().len()
    }

    /// 获取日志统计
    pub fn get_stats(&self) -> LogStats {
        let logs = self.logs.read();

        let mut stats = LogStats {
            total: logs.len(),
            trace: 0,
            debug: 0,
            info: 0,
            warn: 0,
            error: 0,
        };

        for entry in logs.iter() {
            match entry.level {
                LogLevel::Trace => stats.trace += 1,
                LogLevel::Debug => stats.debug += 1,
                LogLevel::Info => stats.info += 1,
                LogLevel::Warn => stats.warn += 1,
                LogLevel::Error => stats.error += 1,
            }
        }

        stats
    }
}

impl Default for LogCollector {
    fn default() -> Self {
        Self::new(10000) // 默认保留 10000 条日志
    }
}

/// 日志统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total: usize,
    pub trace: usize,
    pub debug: usize,
    pub info: usize,
    pub warn: usize,
    pub error: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_collector() {
        let collector = LogCollector::new(100);

        collector.add_log(LogLevel::Info, "test".to_string(), "Test message 1".to_string());
        collector.add_log(LogLevel::Warn, "test".to_string(), "Test message 2".to_string());
        collector.add_log(LogLevel::Error, "test".to_string(), "Test message 3".to_string());

        assert_eq!(collector.count(), 3);

        let logs = collector.get_logs();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "Test message 1");
        assert_eq!(logs[1].message, "Test message 2");
        assert_eq!(logs[2].message, "Test message 3");
    }

    #[test]
    fn test_log_filter() {
        let collector = LogCollector::new(100);

        collector.add_log(LogLevel::Debug, "module1".to_string(), "Debug message".to_string());
        collector.add_log(LogLevel::Info, "module2".to_string(), "Info message".to_string());
        collector.add_log(LogLevel::Warn, "module1".to_string(), "Warning message".to_string());
        collector.add_log(LogLevel::Error, "module2".to_string(), "Error message".to_string());

        // 过滤 Info 级别以上
        let filter = LogFilter {
            min_level: Some(LogLevel::Info),
            target: None,
            search: None,
        };
        let filtered = collector.get_filtered_logs(&filter);
        assert_eq!(filtered.len(), 3); // Info, Warn, Error

        // 过滤特定模块
        let filter = LogFilter {
            min_level: None,
            target: Some("module1".to_string()),
            search: None,
        };
        let filtered = collector.get_filtered_logs(&filter);
        assert_eq!(filtered.len(), 2);

        // 搜索消息内容
        let filter = LogFilter {
            min_level: None,
            target: None,
            search: Some("error".to_string()),
        };
        let filtered = collector.get_filtered_logs(&filter);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_log_pagination() {
        let collector = LogCollector::new(100);

        for i in 0..25 {
            collector.add_log(
                LogLevel::Info,
                "test".to_string(),
                format!("Message {}", i),
            );
        }

        // 第一页，每页 10 条
        let (logs, total) = collector.get_paginated_logs(0, 10, None);
        assert_eq!(logs.len(), 10);
        assert_eq!(total, 25);
        assert_eq!(logs[0].message, "Message 0");

        // 第二页
        let (logs, total) = collector.get_paginated_logs(1, 10, None);
        assert_eq!(logs.len(), 10);
        assert_eq!(total, 25);
        assert_eq!(logs[0].message, "Message 10");

        // 最后一页
        let (logs, total) = collector.get_paginated_logs(2, 10, None);
        assert_eq!(logs.len(), 5);
        assert_eq!(total, 25);
    }

    #[test]
    fn test_log_stats() {
        let collector = LogCollector::new(100);

        collector.add_log(LogLevel::Info, "test".to_string(), "Message 1".to_string());
        collector.add_log(LogLevel::Info, "test".to_string(), "Message 2".to_string());
        collector.add_log(LogLevel::Warn, "test".to_string(), "Message 3".to_string());
        collector.add_log(LogLevel::Error, "test".to_string(), "Message 4".to_string());

        let stats = collector.get_stats();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.info, 2);
        assert_eq!(stats.warn, 1);
        assert_eq!(stats.error, 1);
    }

    #[test]
    fn test_max_logs_capacity() {
        let collector = LogCollector::new(5); // 最多保留 5 条日志

        for i in 0..10 {
            collector.add_log(
                LogLevel::Info,
                "test".to_string(),
                format!("Message {}", i),
            );
        }

        assert_eq!(collector.count(), 5);

        let logs = collector.get_logs();
        // 应该保留最新的 5 条（Message 5-9）
        assert_eq!(logs[0].message, "Message 5");
        assert_eq!(logs[4].message, "Message 9");
    }
}
