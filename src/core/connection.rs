//! 连接池管理模块

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;
use serde::Serialize;

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConnectionState {
    Active,
    Closed,
}

/// 连接信息
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub upload: u64,
    pub download: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub state: ConnectionState,
    #[serde(skip)]
    pub rule: Option<String>,
    #[serde(skip)]
    pub proxy: Option<String>,
}

/// 流量统计
#[derive(Debug, Clone, Serialize)]
pub struct TrafficStats {
    pub total_upload: u64,
    pub total_download: u64,
    pub active_connections: usize,
    pub total_connections: u64,
}

/// 连接池
pub struct ConnectionPool {
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    total_upload: Arc<AtomicU64>,
    total_download: Arc<AtomicU64>,
    total_connections: Arc<AtomicU64>,
}

impl ConnectionPool {
    /// 创建新的连接池
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            total_upload: Arc::new(AtomicU64::new(0)),
            total_download: Arc::new(AtomicU64::new(0)),
            total_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 添加连接
    pub fn add_connection(&self, source: String, destination: String) -> String {
        let id = Uuid::new_v4().to_string();
        let info = ConnectionInfo {
            id: id.clone(),
            source,
            destination,
            upload: 0,
            download: 0,
            start_time: chrono::Utc::now(),
            state: ConnectionState::Active,
            rule: None,
            proxy: None,
        };

        self.connections.write().insert(id.clone(), info);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        id
    }

    /// 添加连接（带规则和代理信息）
    pub fn add_connection_with_info(
        &self,
        source: String,
        destination: String,
        rule: Option<String>,
        proxy: Option<String>,
    ) -> String {
        let id = Uuid::new_v4().to_string();
        let info = ConnectionInfo {
            id: id.clone(),
            source,
            destination,
            upload: 0,
            download: 0,
            start_time: chrono::Utc::now(),
            state: ConnectionState::Active,
            rule,
            proxy,
        };

        self.connections.write().insert(id.clone(), info);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        id
    }

    /// 移除连接
    pub fn remove_connection(&self, id: &str) {
        if let Some(conn) = self.connections.write().remove(id) {
            // 更新总流量
            self.total_upload.fetch_add(conn.upload, Ordering::Relaxed);
            self.total_download.fetch_add(conn.download, Ordering::Relaxed);
        }
    }

    /// 更新流量统计
    pub fn update_traffic(&self, id: &str, upload: u64, download: u64) {
        if let Some(conn) = self.connections.write().get_mut(id) {
            conn.upload += upload;
            conn.download += download;
        }
    }

    /// 获取所有连接
    pub fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        self.connections.read().values().cloned().collect()
    }

    /// 获取活跃连接
    pub fn get_active_connections(&self) -> Vec<ConnectionInfo> {
        self.connections
            .read()
            .values()
            .filter(|c| c.state == ConnectionState::Active)
            .cloned()
            .collect()
    }

    /// 获取连接信息
    pub fn get_connection(&self, id: &str) -> Option<ConnectionInfo> {
        self.connections.read().get(id).cloned()
    }

    /// 获取连接数量
    pub fn get_connection_count(&self) -> usize {
        self.connections.read().len()
    }

    /// 获取活跃连接数量
    pub fn get_active_count(&self) -> usize {
        self.connections
            .read()
            .values()
            .filter(|c| c.state == ConnectionState::Active)
            .count()
    }

    /// 获取流量统计
    pub fn get_traffic_stats(&self) -> TrafficStats {
        let connections = self.connections.read();

        // 计算当前活跃连接的流量
        let (current_up, current_down) = connections.values().fold((0u64, 0u64), |(up, down), conn| {
            (up + conn.upload, down + conn.download)
        });

        // 加上历史流量
        let total_upload = self.total_upload.load(Ordering::Relaxed) + current_up;
        let total_download = self.total_download.load(Ordering::Relaxed) + current_down;

        TrafficStats {
            total_upload,
            total_download,
            active_connections: self.get_active_count(),
            total_connections: self.total_connections.load(Ordering::Relaxed),
        }
    }

    /// 关闭所有连接
    pub fn close_all_connections(&self) {
        let mut connections = self.connections.write();

        // 更新总流量
        for conn in connections.values() {
            self.total_upload.fetch_add(conn.upload, Ordering::Relaxed);
            self.total_download.fetch_add(conn.download, Ordering::Relaxed);
        }

        connections.clear();
    }

    /// 清空所有连接和统计
    pub fn clear(&self) {
        self.connections.write().clear();
        self.total_upload.store(0, Ordering::Relaxed);
        self.total_download.store(0, Ordering::Relaxed);
        self.total_connections.store(0, Ordering::Relaxed);
    }

    /// 获取连接持续时间（秒）
    pub fn get_connection_duration(&self, id: &str) -> Option<i64> {
        self.connections.read().get(id).map(|conn| {
            (chrono::Utc::now() - conn.start_time).num_seconds()
        })
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool() {
        let pool = ConnectionPool::new();
        let id = pool.add_connection("127.0.0.1:1234".to_string(), "example.com:443".to_string());

        assert_eq!(pool.get_connection_count(), 1);
        assert_eq!(pool.get_active_count(), 1);

        pool.update_traffic(&id, 100, 200);
        let conns = pool.get_all_connections();
        assert_eq!(conns[0].upload, 100);
        assert_eq!(conns[0].download, 200);

        pool.remove_connection(&id);
        assert_eq!(pool.get_connection_count(), 0);

        // 检查流量统计
        let stats = pool.get_traffic_stats();
        assert_eq!(stats.total_upload, 100);
        assert_eq!(stats.total_download, 200);
        assert_eq!(stats.total_connections, 1);
    }

    #[test]
    fn test_connection_with_info() {
        let pool = ConnectionPool::new();
        let id = pool.add_connection_with_info(
            "127.0.0.1:1234".to_string(),
            "example.com:443".to_string(),
            Some("DOMAIN-SUFFIX".to_string()),
            Some("PROXY".to_string()),
        );

        let conn = pool.get_connection(&id).unwrap();
        assert_eq!(conn.rule, Some("DOMAIN-SUFFIX".to_string()));
        assert_eq!(conn.proxy, Some("PROXY".to_string()));
    }

    #[test]
    fn test_traffic_stats() {
        let pool = ConnectionPool::new();

        // 添加多个连接
        let id1 = pool.add_connection("127.0.0.1:1".to_string(), "a.com:443".to_string());
        let id2 = pool.add_connection("127.0.0.1:2".to_string(), "b.com:443".to_string());

        pool.update_traffic(&id1, 100, 200);
        pool.update_traffic(&id2, 150, 250);

        let stats = pool.get_traffic_stats();
        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.total_upload, 250);
        assert_eq!(stats.total_download, 450);
    }
}
