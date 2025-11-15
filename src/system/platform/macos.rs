//! macOS 系统代理设置实现

use std::process::Command;
use tracing::{debug, error, info};

/// 设置系统代理
pub fn set_system_proxy(enable: bool, http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    let services = get_network_services()?;

    for service in services {
        if enable {
            info!("启用系统代理: {}", service);
            enable_proxy_for_service(&service, http_port, socks_port)?;
        } else {
            info!("禁用系统代理: {}", service);
            disable_proxy_for_service(&service)?;
        }
    }

    Ok(())
}

/// 获取所有网络服务
fn get_network_services() -> anyhow::Result<Vec<String>> {
    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("获取网络服务失败: {}", error_msg);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let services: Vec<String> = output_str
        .lines()
        .skip(1) // 跳过第一行提示信息
        .filter(|line| !line.starts_with('*')) // 跳过禁用的服务
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    debug!("发现 {} 个网络服务", services.len());
    Ok(services)
}

/// 为指定服务启用代理
fn enable_proxy_for_service(service: &str, http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    let localhost = "127.0.0.1";

    // 启用 HTTP 代理
    let status = Command::new("networksetup")
        .arg("-setwebproxy")
        .arg(service)
        .arg(localhost)
        .arg(http_port.to_string())
        .status()?;

    if !status.success() {
        error!("设置 HTTP 代理失败: {}", service);
    }

    // 启用 HTTPS 代理
    let status = Command::new("networksetup")
        .arg("-setsecurewebproxy")
        .arg(service)
        .arg(localhost)
        .arg(http_port.to_string())
        .status()?;

    if !status.success() {
        error!("设置 HTTPS 代理失败: {}", service);
    }

    // 启用 SOCKS 代理
    let status = Command::new("networksetup")
        .arg("-setsocksfirewallproxy")
        .arg(service)
        .arg(localhost)
        .arg(socks_port.to_string())
        .status()?;

    if !status.success() {
        error!("设置 SOCKS 代理失败: {}", service);
    }

    // 设置代理绕过列表
    set_proxy_bypass_domains(service)?;

    Ok(())
}

/// 为指定服务禁用代理
fn disable_proxy_for_service(service: &str) -> anyhow::Result<()> {
    // 禁用 HTTP 代理
    let status = Command::new("networksetup")
        .arg("-setwebproxystate")
        .arg(service)
        .arg("off")
        .status()?;

    if !status.success() {
        error!("禁用 HTTP 代理失败: {}", service);
    }

    // 禁用 HTTPS 代理
    let status = Command::new("networksetup")
        .arg("-setsecurewebproxystate")
        .arg(service)
        .arg("off")
        .status()?;

    if !status.success() {
        error!("禁用 HTTPS 代理失败: {}", service);
    }

    // 禁用 SOCKS 代理
    let status = Command::new("networksetup")
        .arg("-setsocksfirewallproxystate")
        .arg(service)
        .arg("off")
        .status()?;

    if !status.success() {
        error!("禁用 SOCKS 代理失败: {}", service);
    }

    Ok(())
}

/// 设置代理绕过域名列表
fn set_proxy_bypass_domains(service: &str) -> anyhow::Result<()> {
    let bypass_domains = vec![
        "localhost",
        "127.0.0.1",
        "*.local",
        "169.254.0.0/16",
        "192.168.0.0/16",
        "10.0.0.0/8",
        "172.16.0.0/12",
    ];

    let bypass_str = bypass_domains.join(" ");

    let status = Command::new("networksetup")
        .arg("-setproxybypassdomains")
        .arg(service)
        .args(bypass_domains)
        .status()?;

    if !status.success() {
        error!("设置代理绕过列表失败: {}", service);
    } else {
        debug!("设置代理绕过列表: {}", bypass_str);
    }

    Ok(())
}

/// 获取当前代理设置
pub fn get_current_proxy_settings(service: &str) -> anyhow::Result<ProxySettings> {
    let http_output = Command::new("networksetup")
        .arg("-getwebproxy")
        .arg(service)
        .output()?;

    let https_output = Command::new("networksetup")
        .arg("-getsecurewebproxy")
        .arg(service)
        .output()?;

    let socks_output = Command::new("networksetup")
        .arg("-getsocksfirewallproxy")
        .arg(service)
        .output()?;

    Ok(ProxySettings {
        http_enabled: parse_proxy_status(&http_output.stdout),
        https_enabled: parse_proxy_status(&https_output.stdout),
        socks_enabled: parse_proxy_status(&socks_output.stdout),
    })
}

/// 代理设置状态
#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub http_enabled: bool,
    pub https_enabled: bool,
    pub socks_enabled: bool,
}

/// 解析代理状态输出
fn parse_proxy_status(output: &[u8]) -> bool {
    let output_str = String::from_utf8_lossy(output);
    output_str.contains("Enabled: Yes")
}

/// 检查是否有管理员权限
pub fn check_admin_privileges() -> bool {
    // 在 macOS 上，networksetup 命令可能需要 sudo 权限
    // 尝试执行一个读取命令来检查
    Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_network_services() {
        // 这个测试只在 macOS 上运行
        if cfg!(target_os = "macos") {
            let services = get_network_services();
            assert!(services.is_ok());
            if let Ok(services) = services {
                println!("网络服务: {:?}", services);
            }
        }
    }

    #[test]
    fn test_check_admin_privileges() {
        if cfg!(target_os = "macos") {
            let has_privileges = check_admin_privileges();
            println!("是否有管理员权限: {}", has_privileges);
        }
    }
}
