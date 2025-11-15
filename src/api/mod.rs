//! API 服务模块

use axum::{
    extract::{Path, State},
    routing::{get, put, delete},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::core::ConnectionPool;
use crate::core::connection::{ConnectionInfo, TrafficStats};
use crate::config::Config;

/// API 状态
pub struct ApiState {
    pub connection_pool: Arc<ConnectionPool>,
    pub config: Arc<Config>,
}

/// API 响应
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

/// 版本信息
#[derive(Serialize)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
}

/// 配置更新请求
#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub mode: Option<String>,
    pub allow_lan: Option<bool>,
}

/// 创建 API 路由
pub fn create_api_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/api/version", get(get_version))
        .route("/api/configs", get(get_configs))
        .route("/api/configs", put(update_configs))
        .route("/api/connections", get(get_connections))
        .route("/api/connections/:id", get(get_connection))
        .route("/api/connections", delete(close_all_connections))
        .route("/api/connections/:id", delete(close_connection))
        .route("/api/traffic", get(get_traffic))
        .with_state(state)
}

/// 获取版本信息
async fn get_version() -> Json<ApiResponse<VersionInfo>> {
    Json(ApiResponse::success(VersionInfo {
        name: crate::NAME.to_string(),
        version: crate::VERSION.to_string(),
    }))
}

/// 获取配置
async fn get_configs(
    State(state): State<Arc<ApiState>>,
) -> Json<ApiResponse<Config>> {
    Json(ApiResponse::success((*state.config).clone()))
}

/// 更新配置
async fn update_configs(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<ConfigUpdateRequest>,
) -> Json<ApiResponse<String>> {
    // TODO: 实现配置更新逻辑
    Json(ApiResponse::success("Config update queued".to_string()))
}

/// 获取所有连接
async fn get_connections(
    State(state): State<Arc<ApiState>>,
) -> Json<ApiResponse<Vec<ConnectionInfo>>> {
    let connections = state.connection_pool.get_all_connections();
    Json(ApiResponse::success(connections))
}

/// 获取单个连接
async fn get_connection(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<ConnectionInfo>> {
    match state.connection_pool.get_connection(&id) {
        Some(conn) => Json(ApiResponse::success(conn)),
        None => Json(ApiResponse::error("Connection not found".to_string())),
    }
}

/// 关闭所有连接
async fn close_all_connections(
    State(state): State<Arc<ApiState>>,
) -> Json<ApiResponse<String>> {
    state.connection_pool.close_all_connections();
    Json(ApiResponse::success("All connections closed".to_string()))
}

/// 关闭单个连接
async fn close_connection(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    state.connection_pool.remove_connection(&id);
    Json(ApiResponse::success(format!("Connection {} closed", id)))
}

/// 获取流量统计
async fn get_traffic(
    State(state): State<Arc<ApiState>>,
) -> Json<ApiResponse<TrafficStats>> {
    let stats = state.connection_pool.get_traffic_stats();
    Json(ApiResponse::success(stats))
}
