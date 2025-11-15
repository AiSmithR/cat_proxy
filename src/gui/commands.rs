//! Tauri 命令处理模块
//!
//! 处理前端发送的命令

use super::{
    AppState, CommandResponse, DashboardStats, ProxyNodeInfo,
    ProxyGroupInfo, RuleInfo, AppSettings, SystemInfo, GuiConnectionInfo,
};
use crate::config::Config;
use tauri::State;

/// 获取仪表板统计信息
#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<CommandResponse<DashboardStats>, String> {
    let is_running = *state.is_running.read();
    let traffic_stats = state.connection_pool.get_traffic_stats();
    let perf_stats = state.performance_stats.read();
    let config = state.config.read();

    let stats = DashboardStats {
        is_running,
        active_connections: traffic_stats.active_connections,
        total_connections: traffic_stats.total_connections,
        total_upload: traffic_stats.total_upload,
        total_download: traffic_stats.total_download,
        zero_copy_count: perf_stats.zero_copy_count,
        proxy_mode: format!("{:?}", config.mode),
        uptime: 0, // TODO: 实现运行时间统计
    };

    Ok(CommandResponse::success(stats))
}

/// 启动代理服务
#[tauri::command]
pub async fn start_proxy(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    let mut is_running = state.is_running.write();

    if *is_running {
        return Ok(CommandResponse::error("代理服务已在运行".to_string()));
    }

    // TODO: 实际启动代理服务
    *is_running = true;

    Ok(CommandResponse::success("代理服务已启动".to_string()))
}

/// 停止代理服务
#[tauri::command]
pub async fn stop_proxy(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    let mut is_running = state.is_running.write();

    if !*is_running {
        return Ok(CommandResponse::error("代理服务未运行".to_string()));
    }

    // TODO: 实际停止代理服务
    *is_running = false;

    Ok(CommandResponse::success("代理服务已停止".to_string()))
}

/// 获取代理节点列表
#[tauri::command]
pub async fn get_proxy_nodes(state: State<'_, AppState>) -> Result<CommandResponse<Vec<ProxyNodeInfo>>, String> {
    let config = state.config.read();
    let health_results = state.health_manager.get_results();

    let nodes: Vec<ProxyNodeInfo> = config.proxies.iter().map(|proxy| {
        // 查找对应的健康检查结果
        let health_result = health_results.iter()
            .find(|r| r.proxy_name == proxy.name);

        ProxyNodeInfo {
            name: proxy.name.clone(),
            proxy_type: format!("{:?}", proxy.proxy_type),
            server: proxy.server.clone(),
            port: proxy.port,
            is_healthy: health_result.map(|r| r.is_healthy).unwrap_or(true), // 默认假设健康
            latency: health_result.and_then(|r| r.latency_ms),
        }
    }).collect();

    Ok(CommandResponse::success(nodes))
}

/// 获取代理组列表
#[tauri::command]
pub async fn get_proxy_groups(state: State<'_, AppState>) -> Result<CommandResponse<Vec<ProxyGroupInfo>>, String> {
    let config = state.config.read();

    let groups: Vec<ProxyGroupInfo> = config.proxy_groups.iter().map(|group| {
        ProxyGroupInfo {
            name: group.name.clone(),
            group_type: format!("{:?}", group.group_type),
            proxies: group.proxies.clone(),
            selected: group.proxies.first().cloned(),
        }
    }).collect();

    Ok(CommandResponse::success(groups))
}

/// 获取规则列表
#[tauri::command]
pub async fn get_rules(state: State<'_, AppState>) -> Result<CommandResponse<Vec<RuleInfo>>, String> {
    let config = state.config.read();

    let rules: Vec<RuleInfo> = config.rules.iter().filter_map(|rule_str| {
        let parts: Vec<&str> = rule_str.split(',').collect();
        if parts.len() >= 3 {
            Some(RuleInfo {
                rule_type: parts[0].to_string(),
                content: parts[1].to_string(),
                target: parts[2].to_string(),
            })
        } else {
            None
        }
    }).collect();

    Ok(CommandResponse::success(rules))
}

/// 获取配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<CommandResponse<Config>, String> {
    let config = state.config.read().clone();
    Ok(CommandResponse::success(config))
}

/// 更新配置
#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: Config,
) -> Result<CommandResponse<String>, String> {
    // 验证配置
    if let Err(e) = config.validate() {
        return Ok(CommandResponse::error(format!("配置验证失败: {}", e)));
    }

    *state.config.write() = config;
    Ok(CommandResponse::success("配置已更新".to_string()))
}

/// 保存配置到文件
#[tauri::command]
pub async fn save_config(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    let config = state.config.read();

    match config.save() {
        Ok(_) => Ok(CommandResponse::success("配置已保存".to_string())),
        Err(e) => Ok(CommandResponse::error(format!("保存配置失败: {}", e))),
    }
}

/// 保存配置到指定路径
#[tauri::command]
pub async fn save_config_to(
    state: State<'_, AppState>,
    path: String,
) -> Result<CommandResponse<String>, String> {
    let config = state.config.read();

    match config.save_to(&path) {
        Ok(_) => Ok(CommandResponse::success(format!("配置已保存到: {}", path))),
        Err(e) => Ok(CommandResponse::error(format!("保存配置失败: {}", e))),
    }
}

/// 重新加载配置
#[tauri::command]
pub async fn reload_config(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();

    match config.reload() {
        Ok(_) => Ok(CommandResponse::success("配置已重新加载".to_string())),
        Err(e) => Ok(CommandResponse::error(format!("重新加载配置失败: {}", e))),
    }
}

/// 验证配置
#[tauri::command]
pub async fn validate_config(
    config: Config,
) -> Result<CommandResponse<String>, String> {
    match config.validate() {
        Ok(_) => Ok(CommandResponse::success("配置验证通过".to_string())),
        Err(e) => Ok(CommandResponse::error(format!("配置验证失败: {}", e))),
    }
}

/// 导入配置文件
#[tauri::command]
pub async fn import_config_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<CommandResponse<String>, String> {
    match Config::from_yaml_file(&path) {
        Ok(new_config) => {
            let mut config = state.config.write();
            *config = new_config;
            Ok(CommandResponse::success(format!("配置已从 {} 导入", path)))
        }
        Err(e) => Ok(CommandResponse::error(format!("导入配置失败: {}", e))),
    }
}

/// 导出配置为 JSON
#[tauri::command]
pub async fn export_config_json(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    let config = state.config.read();

    match config.to_json_string() {
        Ok(json) => Ok(CommandResponse::success(json)),
        Err(e) => Ok(CommandResponse::error(format!("导出配置失败: {}", e))),
    }
}

/// 获取示例配置
#[tauri::command]
pub async fn get_example_config() -> Result<CommandResponse<Config>, String> {
    let config = Config::example();
    Ok(CommandResponse::success(config))
}

/// 获取默认配置路径
#[tauri::command]
pub async fn get_default_config_path() -> Result<CommandResponse<String>, String> {
    let path = Config::default_config_path();
    Ok(CommandResponse::success(path.to_string_lossy().to_string()))
}

/// 获取应用设置
#[tauri::command]
pub async fn get_settings() -> Result<CommandResponse<AppSettings>, String> {
    let settings = AppSettings::default();
    Ok(CommandResponse::success(settings))
}

/// 更新应用设置
#[tauri::command]
pub async fn update_settings(_settings: AppSettings) -> Result<CommandResponse<String>, String> {
    // TODO: 持久化设置
    Ok(CommandResponse::success("设置已更新".to_string()))
}

/// 获取系统信息
#[tauri::command]
pub async fn get_system_info() -> Result<CommandResponse<SystemInfo>, String> {
    let info = SystemInfo::get();
    Ok(CommandResponse::success(info))
}

/// 测试代理节点
#[tauri::command]
pub async fn test_proxy_node(
    state: State<'_, AppState>,
    node_name: String,
) -> Result<CommandResponse<u64>, String> {
    // 查找代理配置
    let proxy_config = {
        let config = state.config.read();
        config.proxies.iter()
            .find(|p| p.name == node_name)
            .cloned()
    };

    match proxy_config {
        Some(proxy) => {
            // 创建临时健康检查器进行测试
            let checker = crate::core::HealthChecker::with_defaults();

            match checker.check_proxy(&proxy).await {
                Ok(result) => {
                    if result.is_healthy {
                        if let Some(latency) = result.latency_ms {
                            Ok(CommandResponse::success(latency))
                        } else {
                            Ok(CommandResponse::error("节点健康但无延迟数据".to_string()))
                        }
                    } else {
                        Ok(CommandResponse::error("节点不可达".to_string()))
                    }
                }
                Err(e) => Ok(CommandResponse::error(format!("测试失败: {}", e))),
            }
        }
        None => Ok(CommandResponse::error(format!("找不到节点: {}", node_name))),
    }
}

/// 设置系统代理
#[tauri::command]
pub async fn set_system_proxy(
    enable: bool,
    http_port: u16,
    socks_port: u16,
) -> Result<CommandResponse<String>, String> {
    match crate::system::set_system_proxy(enable, http_port, socks_port) {
        Ok(_) => {
            let msg = if enable {
                "系统代理已启用"
            } else {
                "系统代理已禁用"
            };
            Ok(CommandResponse::success(msg.to_string()))
        }
        Err(e) => Ok(CommandResponse::error(format!("设置系统代理失败: {}", e))),
    }
}

/// 导入订阅
#[tauri::command]
pub async fn import_subscription(
    state: State<'_, AppState>,
    name: String,
    url: String,
) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();

    // 添加订阅到配置
    match config.subscriptions.iter_mut().find(|s| s.name == name || s.url == url) {
        Some(_) => {
            return Ok(CommandResponse::error("订阅名称或 URL 已存在".to_string()));
        }
        None => {
            config.subscriptions.push(crate::config::Subscription::new(name.clone(), url.clone()));
        }
    }

    // 保存配置
    drop(config);
    let config = state.config.read();
    if let Err(e) = config.save() {
        return Ok(CommandResponse::error(format!("保存配置失败: {}", e)));
    }

    Ok(CommandResponse::success(format!("订阅 {} 已添加", name)))
}

/// 更新订阅
#[tauri::command]
pub async fn update_subscription(
    state: State<'_, AppState>,
    name: Option<String>,
) -> Result<CommandResponse<String>, String> {
    // 复制订阅列表
    let subscriptions = {
        let config = state.config.read();
        config.subscriptions.clone()
    };

    // 创建管理器并添加订阅
    let mut manager = crate::config::SubscriptionManager::new();
    for sub in &subscriptions {
        manager.add_subscription(sub.name.clone(), sub.url.clone()).ok();
    }

    // 更新订阅
    let result = if let Some(name) = name {
        manager.update_subscription(&name).await
    } else {
        manager.update_all().await
    };

    match result {
        Ok(proxies) => {
            // 将新节点合并到配置中
            let mut config = state.config.write();
            for proxy in proxies {
                if !config.proxies.iter().any(|p| p.name == proxy.name) {
                    config.proxies.push(proxy);
                }
            }

            // 更新订阅的最后更新时间
            for sub in &subscriptions {
                if let Some(config_sub) = config.subscriptions.iter_mut().find(|s| s.name == sub.name) {
                    if let Some(manager_sub) = manager.get_subscriptions().iter().find(|s| s.name == sub.name) {
                        config_sub.last_update = manager_sub.last_update;
                        config_sub.node_count = manager_sub.node_count;
                    }
                }
            }

            // 保存配置
            if let Err(e) = config.save() {
                return Ok(CommandResponse::error(format!("保存配置失败: {}", e)));
            }

            Ok(CommandResponse::success("订阅更新成功".to_string()))
        }
        Err(e) => Ok(CommandResponse::error(format!("订阅更新失败: {}", e))),
    }
}

/// 获取订阅列表
#[tauri::command]
pub async fn get_subscriptions(
    state: State<'_, AppState>,
) -> Result<CommandResponse<Vec<crate::config::Subscription>>, String> {
    let config = state.config.read();
    Ok(CommandResponse::success(config.subscriptions.clone()))
}

/// 删除订阅
#[tauri::command]
pub async fn delete_subscription(
    state: State<'_, AppState>,
    name: String,
) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();
    let original_len = config.subscriptions.len();

    config.subscriptions.retain(|s| s.name != name);

    if config.subscriptions.len() == original_len {
        return Ok(CommandResponse::error(format!("订阅不存在: {}", name)));
    }

    // 保存配置
    if let Err(e) = config.save() {
        return Ok(CommandResponse::error(format!("保存配置失败: {}", e)));
    }

    Ok(CommandResponse::success(format!("订阅 {} 已删除", name)))
}

/// 启用/禁用订阅
#[tauri::command]
pub async fn set_subscription_enabled(
    state: State<'_, AppState>,
    name: String,
    enabled: bool,
) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();

    match config.subscriptions.iter_mut().find(|s| s.name == name) {
        Some(sub) => {
            sub.enabled = enabled;

            // 保存配置
            if let Err(e) = config.save() {
                return Ok(CommandResponse::error(format!("保存配置失败: {}", e)));
            }

            let status = if enabled { "启用" } else { "禁用" };
            Ok(CommandResponse::success(format!("订阅 {} 已{}", name, status)))
        }
        None => Ok(CommandResponse::error(format!("订阅不存在: {}", name))),
    }
}

/// 导出配置
#[tauri::command]
pub async fn export_config(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    let config = state.config.read();

    match serde_yaml::to_string(&*config) {
        Ok(yaml) => Ok(CommandResponse::success(yaml)),
        Err(e) => Ok(CommandResponse::error(format!("导出配置失败: {}", e))),
    }
}

/// 清空所有连接
#[tauri::command]
pub async fn clear_connections(state: State<'_, AppState>) -> Result<CommandResponse<String>, String> {
    state.connection_pool.close_all_connections();
    Ok(CommandResponse::success("已清空所有连接".to_string()))
}

/// 添加规则
#[tauri::command]
pub async fn add_rule(
    state: State<'_, AppState>,
    rule: String,
) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();
    config.rules.push(rule.clone());
    Ok(CommandResponse::success(format!("规则已添加: {}", rule)))
}

/// 删除规则
#[tauri::command]
pub async fn delete_rule(
    state: State<'_, AppState>,
    index: usize,
) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();

    if index < config.rules.len() {
        let removed = config.rules.remove(index);
        Ok(CommandResponse::success(format!("规则已删除: {}", removed)))
    } else {
        Ok(CommandResponse::error("规则索引超出范围".to_string()))
    }
}

/// 更新规则
#[tauri::command]
pub async fn update_rule(
    state: State<'_, AppState>,
    index: usize,
    rule: String,
) -> Result<CommandResponse<String>, String> {
    let mut config = state.config.write();

    if index < config.rules.len() {
        config.rules[index] = rule.clone();
        Ok(CommandResponse::success(format!("规则已更新: {}", rule)))
    } else {
        Ok(CommandResponse::error("规则索引超出范围".to_string()))
    }
}

/// 获取活跃连接列表
#[tauri::command]
pub async fn get_connections(state: State<'_, AppState>) -> Result<CommandResponse<Vec<GuiConnectionInfo>>, String> {
    let connections = state.connection_pool.get_all_connections()
        .into_iter()
        .map(|conn| {
            let duration = (chrono::Utc::now() - conn.start_time).num_seconds();
            GuiConnectionInfo {
                id: conn.id,
                src_addr: conn.source,
                dst_addr: conn.destination,
                proxy: conn.proxy.unwrap_or_else(|| "DIRECT".to_string()),
                upload: conn.upload,
                download: conn.download,
                duration,
            }
        })
        .collect();

    Ok(CommandResponse::success(connections))
}

/// 测试所有代理节点
#[tauri::command]
pub async fn test_all_proxy_nodes(
    state: State<'_, AppState>,
) -> Result<CommandResponse<Vec<(String, Option<u64>)>>, String> {
    // 获取所有代理配置
    let proxies = {
        let config = state.config.read();
        config.proxies.clone()
    };

    if proxies.is_empty() {
        return Ok(CommandResponse::success(Vec::new()));
    }

    // 创建健康检查器
    let checker = crate::core::HealthChecker::with_defaults();

    // 批量测试
    let results = checker.check_proxies(&proxies).await;

    // 提取节点名称和延迟
    let latencies: Vec<(String, Option<u64>)> = results
        .into_iter()
        .map(|r| (r.proxy_name, r.latency_ms))
        .collect();

    Ok(CommandResponse::success(latencies))
}

/// 获取所有代理节点延迟信息
#[tauri::command]
pub async fn get_proxy_latencies(
    state: State<'_, AppState>,
) -> Result<CommandResponse<Vec<(String, Option<u64>, bool)>>, String> {
    // 从健康检查管理器获取结果
    let results = state.health_manager.get_results();

    // 提取节点名称、延迟和健康状态
    let latencies: Vec<(String, Option<u64>, bool)> = results
        .into_iter()
        .map(|r| (r.proxy_name, r.latency_ms, r.is_healthy))
        .collect();

    Ok(CommandResponse::success(latencies))
}

/// 启动自动健康检查
#[tauri::command]
pub async fn start_auto_health_check(
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    // 获取所有代理配置
    let proxies = {
        let config = state.config.read();
        config.proxies.clone()
    };

    if proxies.is_empty() {
        return Ok(CommandResponse::error("没有配置代理节点".to_string()));
    }

    // 启动健康检查
    state.health_manager.start(proxies);

    Ok(CommandResponse::success(format!("自动健康检查已启动")))
}

/// 更新代理节点信息（添加延迟数据）
#[tauri::command]
pub async fn get_proxy_nodes_with_latency(
    state: State<'_, AppState>,
) -> Result<CommandResponse<Vec<ProxyNodeInfo>>, String> {
    let config = state.config.read();
    let health_results = state.health_manager.get_results();

    let nodes: Vec<ProxyNodeInfo> = config.proxies.iter().map(|proxy| {
        // 查找对应的健康检查结果
        let health_result = health_results.iter()
            .find(|r| r.proxy_name == proxy.name);

        ProxyNodeInfo {
            name: proxy.name.clone(),
            proxy_type: format!("{:?}", proxy.proxy_type),
            server: proxy.server.clone(),
            port: proxy.port,
            is_healthy: health_result.map(|r| r.is_healthy).unwrap_or(false),
            latency: health_result.and_then(|r| r.latency_ms),
        }
    }).collect();

    Ok(CommandResponse::success(nodes))
}

/// 获取日志列表
#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    page: Option<usize>,
    page_size: Option<usize>,
    min_level: Option<String>,
    target: Option<String>,
    search: Option<String>,
) -> Result<CommandResponse<(Vec<crate::logging::LogEntry>, usize)>, String> {
    use crate::logging::{LogFilter, LogLevel};

    // 构建过滤器
    let filter = LogFilter {
        min_level: min_level.and_then(|level| match level.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }),
        target,
        search,
    };

    // 获取分页日志
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(50);
    let (logs, total) = state.log_collector.get_paginated_logs(page, page_size, Some(&filter));

    Ok(CommandResponse::success((logs, total)))
}

/// 获取最新日志
#[tauri::command]
pub async fn get_latest_logs(
    state: State<'_, AppState>,
    count: usize,
) -> Result<CommandResponse<Vec<crate::logging::LogEntry>>, String> {
    let logs = state.log_collector.get_latest_logs(count);
    Ok(CommandResponse::success(logs))
}

/// 清空日志
#[tauri::command]
pub async fn clear_logs(
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    state.log_collector.clear();
    Ok(CommandResponse::success("日志已清空".to_string()))
}

/// 获取日志统计
#[tauri::command]
pub async fn get_log_stats(
    state: State<'_, AppState>,
) -> Result<CommandResponse<crate::logging::LogStats>, String> {
    let stats = state.log_collector.get_stats();
    Ok(CommandResponse::success(stats))
}

/// 添加测试日志（用于测试）
#[tauri::command]
pub async fn add_test_log(
    state: State<'_, AppState>,
    level: String,
    message: String,
) -> Result<CommandResponse<String>, String> {
    use crate::logging::LogLevel;

    let log_level = match level.to_lowercase().as_str() {
        "trace" => LogLevel::Trace,
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };

    state.log_collector.add_log(log_level, "test".to_string(), message.clone());
    Ok(CommandResponse::success(format!("已添加测试日志: {}", message)))
}

