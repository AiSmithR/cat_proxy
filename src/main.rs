//! Cat Proxy - 跨平台代理软件主程序

use cat_proxy::{Config, ProxyServer, NAME, VERSION};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber;

/// Cat Proxy - 高性能跨平台代理软件
#[derive(Parser, Debug)]
#[command(name = NAME)]
#[command(version = VERSION)]
#[command(about = "A cross-platform proxy software like Clash Verge", long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE", default_value = "config.yaml")]
    config: PathBuf,

    /// 日志级别 (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// 后台运行模式
    #[arg(short, long)]
    daemon: bool,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 启动代理服务器（默认命令）
    Start {
        /// 是否启用系统代理
        #[arg(long)]
        set_system_proxy: bool,

        /// 是否启用 TUN 模式
        #[arg(long)]
        tun: bool,
    },

    /// 停止代理服务器
    Stop,

    /// 重启代理服务器
    Restart,

    /// 测试配置文件
    Test,

    /// 更新订阅
    Update {
        /// 订阅名称（可选，不指定则更新全部）
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },

    /// 设置系统代理
    SetProxy {
        /// 是否启用
        #[arg(long)]
        enable: bool,

        /// HTTP 端口
        #[arg(long, default_value = "7890")]
        http_port: u16,

        /// SOCKS 端口
        #[arg(long, default_value = "7891")]
        socks_port: u16,
    },

    /// 显示版本信息
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 初始化日志系统
    init_logging(&cli.log_level)?;

    info!("{} v{} starting...", NAME, VERSION);

    // 处理子命令
    match cli.command {
        Some(Commands::Start {
            set_system_proxy,
            tun,
        }) => {
            start_server(&cli.config, set_system_proxy, tun).await?;
        }
        Some(Commands::Stop) => {
            stop_server()?;
        }
        Some(Commands::Restart) => {
            restart_server(&cli.config).await?;
        }
        Some(Commands::Test) => {
            test_config(&cli.config)?;
        }
        Some(Commands::Update { name }) => {
            update_subscription(&cli.config, name).await?;
        }
        Some(Commands::SetProxy {
            enable,
            http_port,
            socks_port,
        }) => {
            set_system_proxy(enable, http_port, socks_port)?;
        }
        Some(Commands::Version) => {
            print_version();
        }
        None => {
            // 默认启动服务器
            start_server(&cli.config, false, false).await?;
        }
    }

    Ok(())
}

/// 初始化日志系统
fn init_logging(level: &str) -> anyhow::Result<()> {
    let env_filter = match level.to_lowercase().as_str() {
        "trace" => "cat_proxy=trace",
        "debug" => "cat_proxy=debug",
        "info" => "cat_proxy=info",
        "warn" => "cat_proxy=warn",
        "error" => "cat_proxy=error",
        _ => {
            warn!("无效的日志级别: {}, 使用默认级别 info", level);
            "cat_proxy=info"
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    Ok(())
}

/// 启动代理服务器
async fn start_server(
    config_path: &PathBuf,
    enable_system_proxy: bool,
    _enable_tun: bool,
) -> anyhow::Result<()> {
    info!("加载配置文件: {:?}", config_path);

    // 加载配置
    let config = if config_path.exists() {
        Config::from_yaml_file(config_path.to_str().unwrap())?
    } else {
        warn!("配置文件不存在，使用默认配置");
        Config::default()
    };

    info!("配置加载成功");
    info!("HTTP 端口: {}", config.port);
    info!("SOCKS 端口: {}", config.socks_port);
    info!("代理模式: {:?}", config.mode);

    // 设置系统代理
    if enable_system_proxy {
        info!("设置系统代理...");
        match cat_proxy::system::set_system_proxy(true, config.port, config.socks_port) {
            Ok(_) => info!("系统代理设置成功"),
            Err(e) => warn!("系统代理设置失败: {}", e),
        }
    }

    // 创建代理服务器
    let server = ProxyServer::new(config)?;

    // 注册信号处理
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    setup_signal_handlers(tx)?;

    info!("代理服务器启动中...");

    // 在后台任务中启动服务器
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            error!("服务器启动失败: {}", e);
        }
    });

    // 等待关闭信号
    tokio::select! {
        _ = rx.recv() => {
            info!("收到关闭信号，正在停止服务器...");
        }
        _ = server_handle => {
            info!("服务器已停止");
        }
    }

    // 清理系统代理
    if enable_system_proxy {
        info!("清理系统代理...");
        let _ = cat_proxy::system::set_system_proxy(false, 0, 0);
    }

    info!("代理服务器已关闭");
    Ok(())
}

/// 设置信号处理
fn setup_signal_handlers(tx: tokio::sync::mpsc::Sender<()>) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use signal_hook::consts::signal::*;
        use signal_hook_tokio::Signals;

        let signals = Signals::new(&[SIGINT, SIGTERM, SIGQUIT])?;
        let handle = signals.handle();

        tokio::spawn(async move {
            let mut signals = signals;
            while let Some(signal) = futures::StreamExt::next(&mut signals).await {
                match signal {
                    SIGINT | SIGTERM | SIGQUIT => {
                        info!("收到信号: {}", signal);
                        let _ = tx.send(()).await;
                        break;
                    }
                    _ => {}
                }
            }
            handle.close();
        });
    }

    #[cfg(windows)]
    {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            info!("收到 Ctrl-C 信号");
            let _ = tx.send(()).await;
        });
    }

    Ok(())
}

/// 停止代理服务器
fn stop_server() -> anyhow::Result<()> {
    info!("停止代理服务器...");
    // TODO: 实现进程间通信停止服务器
    warn!("stop 命令尚未完全实现，请使用 Ctrl-C 或 kill 命令");
    Ok(())
}

/// 重启代理服务器
async fn restart_server(config_path: &PathBuf) -> anyhow::Result<()> {
    info!("重启代理服务器...");
    stop_server()?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    start_server(config_path, false, false).await
}

/// 测试配置文件
fn test_config(config_path: &PathBuf) -> anyhow::Result<()> {
    info!("测试配置文件: {:?}", config_path);

    if !config_path.exists() {
        error!("配置文件不存在: {:?}", config_path);
        return Err(anyhow::anyhow!("配置文件不存在"));
    }

    match Config::from_yaml_file(config_path.to_str().unwrap()) {
        Ok(config) => {
            info!("✓ 配置文件格式正确");
            info!("  - HTTP 端口: {}", config.port);
            info!("  - SOCKS 端口: {}", config.socks_port);
            info!("  - 代理模式: {:?}", config.mode);
            info!("  - 代理数量: {}", config.proxies.len());
            info!("  - 规则数量: {}", config.rules.len());
            Ok(())
        }
        Err(e) => {
            error!("✗ 配置文件解析失败: {}", e);
            Err(e)
        }
    }
}

/// 更新订阅
async fn update_subscription(
    _config_path: &PathBuf,
    name: Option<String>,
) -> anyhow::Result<()> {
    if let Some(name) = name {
        info!("更新订阅: {}", name);
    } else {
        info!("更新所有订阅");
    }

    // TODO: 实现订阅更新逻辑
    warn!("订阅更新功能尚未完全实现");

    Ok(())
}

/// 设置系统代理
fn set_system_proxy(enable: bool, http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    if enable {
        info!(
            "启用系统代理 - HTTP: {}, SOCKS: {}",
            http_port, socks_port
        );
    } else {
        info!("禁用系统代理");
    }

    cat_proxy::system::set_system_proxy(enable, http_port, socks_port)?;

    if enable {
        info!("✓ 系统代理已启用");
    } else {
        info!("✓ 系统代理已禁用");
    }

    Ok(())
}

/// 打印版本信息
fn print_version() {
    println!("{} v{}", NAME, VERSION);
    println!("Rust 编写的跨平台代理软件");
    println!();
    println!("支持的协议:");
    println!("  - HTTP/HTTPS (CONNECT)");
    println!("  - SOCKS5");
    println!("  - Shadowsocks (AEAD)");
    println!("  - VMess");
    println!("  - Trojan");
    println!();
    println!("支持的平台:");
    println!("  - macOS");
    println!("  - Linux");
    println!("  - Windows");
}
