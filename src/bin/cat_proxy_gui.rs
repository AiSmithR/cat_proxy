//! Cat Proxy GUI 应用程序
//!
//! 基于 Tauri 的跨平台图形用户界面
//!
//! # 功能特性
//! - 现代化的 Web 技术栈（React + TypeScript）
//! - 原生性能（Rust 后端 + Tauri）
//! - 跨平台支持（Windows、macOS、Linux）
//! - 实时流量监控和可视化
//! - 代理节点管理和健康检查
//! - 规则配置和订阅管理

// Windows 系统上，Release 模式下不显示控制台窗口
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// 导入 GUI 相关模块
use cat_proxy::gui::{AppState, commands};
use cat_proxy::Config;

/// 应用程序主入口函数
///
/// # 执行流程
/// 1. 初始化日志系统
/// 2. 加载配置文件（如果存在）
/// 3. 创建应用状态
/// 4. 注册所有 Tauri 命令处理器
/// 5. 启动 GUI 应用程序
fn main() {
    // ========================================
    // 1. 初始化日志系统
    // ========================================
    // 配置 tracing 日志记录器
    // - 日志级别：info（可通过环境变量 RUST_LOG 覆盖）
    // - 不显示日志来源模块名（简化输出）
    tracing_subscriber::fmt()
        .with_env_filter("cat_proxy=info")  // 只记录 cat_proxy 模块的 info 及以上级别日志
        .with_target(false)                  // 不显示日志目标（模块名）
        .init();

    // ========================================
    // 2. 加载配置文件
    // ========================================
    // 尝试从 config.yaml 加载配置
    // 如果文件不存在或解析失败，则使用默认配置
    let config = Config::from_yaml_file("config.yaml")
        .unwrap_or_else(|_| {
            // 配置文件加载失败时的提示
            println!("配置文件不存在或无法加载，使用默认配置");
            Config::default()  // 返回默认配置
        });

    // ========================================
    // 3. 创建应用状态
    // ========================================
    // AppState 包含：
    // - 配置信息（Arc<RwLock<Config>>）
    // - 连接池（Arc<ConnectionPool>）
    // - 性能统计（Arc<RwLock<PerformanceStats>>）
    // - 运行状态（Arc<RwLock<bool>>）
    // - 健康检查管理器（Arc<HealthCheckManager>）
    // - 日志收集器（Arc<LogCollector>）
    let app_state = AppState::from_config(config);

    // ========================================
    // 4. 启动 Tauri 应用程序
    // ========================================
    // 注意：系统托盘功能已移除，需要完整的图标文件才能启用
    tauri::Builder::default()
        // 将应用状态注入到 Tauri 的状态管理中
        // 所有命令处理器都可以通过 State<AppState> 访问
        .manage(app_state)

        // 注册所有命令处理器
        // 这些命令可以从前端通过 invoke() 调用
        .invoke_handler(tauri::generate_handler![
            // === 仪表板相关 ===
            commands::get_dashboard_stats,        // 获取仪表板统计信息

            // === 代理控制 ===
            commands::start_proxy,                // 启动代理服务
            commands::stop_proxy,                 // 停止代理服务

            // === 代理节点管理 ===
            commands::get_proxy_nodes,            // 获取代理节点列表
            commands::get_proxy_groups,           // 获取代理组列表
            commands::test_proxy_node,            // 测试单个代理节点
            commands::test_all_proxy_nodes,       // 测试所有代理节点
            commands::get_proxy_latencies,        // 获取代理延迟信息
            commands::get_proxy_nodes_with_latency, // 获取带延迟信息的节点列表
            commands::start_auto_health_check,    // 启动自动健康检查

            // === 规则管理 ===
            commands::get_rules,                  // 获取路由规则列表
            commands::add_rule,                   // 添加新规则
            commands::delete_rule,                // 删除规则
            commands::update_rule,                // 更新规则

            // === 配置管理 ===
            commands::get_config,                 // 获取当前配置
            commands::update_config,              // 更新配置
            commands::save_config,                // 保存配置到默认文件
            commands::save_config_to,             // 保存配置到指定文件
            commands::reload_config,              // 重新加载配置
            commands::validate_config,            // 验证配置有效性
            commands::import_config_file,         // 导入配置文件
            commands::export_config,              // 导出配置
            commands::export_config_json,         // 导出配置为 JSON
            commands::get_example_config,         // 获取示例配置
            commands::get_default_config_path,    // 获取默认配置路径

            // === 订阅管理 ===
            commands::import_subscription,        // 导入订阅
            commands::update_subscription,        // 更新订阅
            commands::get_subscriptions,          // 获取订阅列表
            commands::delete_subscription,        // 删除订阅
            commands::set_subscription_enabled,   // 启用/禁用订阅

            // === 连接管理 ===
            commands::get_connections,            // 获取活跃连接列表
            commands::clear_connections,          // 清空连接记录

            // === 系统相关 ===
            commands::get_settings,               // 获取应用设置
            commands::update_settings,            // 更新应用设置
            commands::get_system_info,            // 获取系统信息
            commands::set_system_proxy,           // 设置系统代理

            // === 日志管理 ===
            commands::get_logs,                   // 获取日志列表
            commands::get_latest_logs,            // 获取最新日志
            commands::clear_logs,                 // 清空日志
            commands::get_log_stats,              // 获取日志统计
            commands::add_test_log,               // 添加测试日志（开发用）
        ])
        // 运行 Tauri 应用
        // generate_context!() 宏会读取 tauri.conf.json 生成应用上下文
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
