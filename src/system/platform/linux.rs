//! Linux 系统代理设置实现

use tracing::{debug, error, info, warn};
use std::process::Command;

/// 代理设置状态
#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub http_enabled: bool,
    pub https_enabled: bool,
    pub socks_enabled: bool,
}

/// 桌面环境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopEnvironment {
    Gnome,
    Kde,
    Unknown,
}

/// 检测当前桌面环境
fn detect_desktop_environment() -> DesktopEnvironment {
    // 检查 XDG_CURRENT_DESKTOP 环境变量
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let desktop = desktop.to_lowercase();
        if desktop.contains("gnome") || desktop.contains("ubuntu") {
            return DesktopEnvironment::Gnome;
        } else if desktop.contains("kde") {
            return DesktopEnvironment::Kde;
        }
    }

    // 检查 DESKTOP_SESSION
    if let Ok(session) = std::env::var("DESKTOP_SESSION") {
        let session = session.to_lowercase();
        if session.contains("gnome") || session.contains("ubuntu") {
            return DesktopEnvironment::Gnome;
        } else if session.contains("kde") {
            return DesktopEnvironment::Kde;
        }
    }

    DesktopEnvironment::Unknown
}

/// 设置系统代理
pub fn set_system_proxy(enable: bool, http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    let desktop = detect_desktop_environment();
    debug!("检测到桌面环境: {:?}", desktop);

    if enable {
        info!("启用 Linux 系统代理");
        enable_linux_proxy(desktop, http_port, socks_port)?;
    } else {
        info!("禁用 Linux 系统代理");
        disable_linux_proxy(desktop)?;
    }

    Ok(())
}

/// 启用 GNOME 系统代理
fn enable_gnome_proxy(http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    // 检查 gsettings 是否可用
    if !Command::new("which")
        .arg("gsettings")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        warn!("gsettings 不可用，无法设置 GNOME 代理");
        return Ok(());
    }

    // 设置代理模式为 manual
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "'manual'"])
        .output();

    // 设置 HTTP 代理
    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy.http",
            "host", "'127.0.0.1'"
        ])
        .output();

    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy.http",
            "port", &http_port.to_string()
        ])
        .output();

    // 设置 HTTPS 代理
    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy.https",
            "host", "'127.0.0.1'"
        ])
        .output();

    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy.https",
            "port", &http_port.to_string()
        ])
        .output();

    // 设置 SOCKS 代理
    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy.socks",
            "host", "'127.0.0.1'"
        ])
        .output();

    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy.socks",
            "port", &socks_port.to_string()
        ])
        .output();

    // 设置忽略列表
    let ignore_hosts = "['localhost', '127.0.0.0/8', '10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16']";
    let _ = Command::new("gsettings")
        .args(&[
            "set", "org.gnome.system.proxy",
            "ignore-hosts", ignore_hosts
        ])
        .output();

    debug!("GNOME 代理设置已启用");
    Ok(())
}

/// 禁用 GNOME 系统代理
fn disable_gnome_proxy() -> anyhow::Result<()> {
    // 设置代理模式为 none
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "'none'"])
        .output();

    debug!("GNOME 代理设置已禁用");
    Ok(())
}

/// 启用 KDE 系统代理
fn enable_kde_proxy(http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    // 检查 kwriteconfig5 是否可用
    let kwriteconfig = if Command::new("which")
        .arg("kwriteconfig5")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "kwriteconfig5"
    } else if Command::new("which")
        .arg("kwriteconfig")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "kwriteconfig"
    } else {
        warn!("kwriteconfig 不可用，无法设置 KDE 代理");
        return Ok(());
    };

    // 启用代理
    let _ = Command::new(kwriteconfig)
        .args(&[
            "--file", "kioslaverc",
            "--group", "Proxy Settings",
            "--key", "ProxyType", "1"
        ])
        .output();

    // 设置 HTTP 代理
    let http_proxy = format!("http://127.0.0.1:{}", http_port);
    let _ = Command::new(kwriteconfig)
        .args(&[
            "--file", "kioslaverc",
            "--group", "Proxy Settings",
            "--key", "httpProxy", &http_proxy
        ])
        .output();

    // 设置 HTTPS 代理
    let _ = Command::new(kwriteconfig)
        .args(&[
            "--file", "kioslaverc",
            "--group", "Proxy Settings",
            "--key", "httpsProxy", &http_proxy
        ])
        .output();

    // 设置 SOCKS 代理
    let socks_proxy = format!("socks://127.0.0.1:{}", socks_port);
    let _ = Command::new(kwriteconfig)
        .args(&[
            "--file", "kioslaverc",
            "--group", "Proxy Settings",
            "--key", "socksProxy", &socks_proxy
        ])
        .output();

    // 设置不代理的地址
    let _ = Command::new(kwriteconfig)
        .args(&[
            "--file", "kioslaverc",
            "--group", "Proxy Settings",
            "--key", "NoProxyFor",
            "localhost,127.0.0.1,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16"
        ])
        .output();

    debug!("KDE 代理设置已启用");
    Ok(())
}

/// 禁用 KDE 系统代理
fn disable_kde_proxy() -> anyhow::Result<()> {
    let kwriteconfig = if Command::new("which")
        .arg("kwriteconfig5")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "kwriteconfig5"
    } else {
        "kwriteconfig"
    };

    // 禁用代理（ProxyType = 0）
    let _ = Command::new(kwriteconfig)
        .args(&[
            "--file", "kioslaverc",
            "--group", "Proxy Settings",
            "--key", "ProxyType", "0"
        ])
        .output();

    debug!("KDE 代理设置已禁用");
    Ok(())
}

/// 启用 Linux 代理（根据桌面环境）
fn enable_linux_proxy(
    desktop: DesktopEnvironment,
    http_port: u16,
    socks_port: u16,
) -> anyhow::Result<()> {
    match desktop {
        DesktopEnvironment::Gnome => {
            enable_gnome_proxy(http_port, socks_port)?;
        }
        DesktopEnvironment::Kde => {
            enable_kde_proxy(http_port, socks_port)?;
        }
        DesktopEnvironment::Unknown => {
            warn!("未识别的桌面环境，将仅提示设置环境变量");
            info!("请手动设置以下环境变量:");
            info!("export HTTP_PROXY=http://127.0.0.1:{}", http_port);
            info!("export HTTPS_PROXY=http://127.0.0.1:{}", http_port);
            info!("export SOCKS_PROXY=socks5://127.0.0.1:{}", socks_port);
            info!("export NO_PROXY=localhost,127.0.0.1,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16");
        }
    }

    Ok(())
}

/// 禁用 Linux 代理
fn disable_linux_proxy(desktop: DesktopEnvironment) -> anyhow::Result<()> {
    match desktop {
        DesktopEnvironment::Gnome => {
            disable_gnome_proxy()?;
        }
        DesktopEnvironment::Kde => {
            disable_kde_proxy()?;
        }
        DesktopEnvironment::Unknown => {
            info!("请手动取消设置环境变量:");
            info!("unset HTTP_PROXY HTTPS_PROXY SOCKS_PROXY NO_PROXY");
        }
    }

    Ok(())
}

/// 获取当前代理设置
pub fn get_current_proxy_settings(_service: &str) -> anyhow::Result<ProxySettings> {
    let desktop = detect_desktop_environment();

    match desktop {
        DesktopEnvironment::Gnome => {
            // 查询 GNOME 代理设置
            let output = Command::new("gsettings")
                .args(&["get", "org.gnome.system.proxy", "mode"])
                .output();

            if let Ok(output) = output {
                let mode = String::from_utf8_lossy(&output.stdout);
                let enabled = mode.contains("manual");

                Ok(ProxySettings {
                    http_enabled: enabled,
                    https_enabled: enabled,
                    socks_enabled: enabled,
                })
            } else {
                Ok(ProxySettings {
                    http_enabled: false,
                    https_enabled: false,
                    socks_enabled: false,
                })
            }
        }
        DesktopEnvironment::Kde => {
            // 查询 KDE 代理设置
            let kreadconfig = if Command::new("which")
                .arg("kreadconfig5")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                "kreadconfig5"
            } else {
                "kreadconfig"
            };

            let output = Command::new(kreadconfig)
                .args(&[
                    "--file", "kioslaverc",
                    "--group", "Proxy Settings",
                    "--key", "ProxyType"
                ])
                .output();

            if let Ok(output) = output {
                let proxy_type = String::from_utf8_lossy(&output.stdout);
                let enabled = proxy_type.trim() == "1";

                Ok(ProxySettings {
                    http_enabled: enabled,
                    https_enabled: enabled,
                    socks_enabled: enabled,
                })
            } else {
                Ok(ProxySettings {
                    http_enabled: false,
                    https_enabled: false,
                    socks_enabled: false,
                })
            }
        }
        DesktopEnvironment::Unknown => {
            // 检查环境变量
            let http_enabled = std::env::var("HTTP_PROXY").is_ok();
            let https_enabled = std::env::var("HTTPS_PROXY").is_ok();
            let socks_enabled = std::env::var("SOCKS_PROXY").is_ok();

            Ok(ProxySettings {
                http_enabled,
                https_enabled,
                socks_enabled,
            })
        }
    }
}

/// 检查是否有管理员权限
pub fn check_admin_privileges() -> bool {
    // 在 Linux 上检查是否是 root 用户或有 sudo 权限
    if let Ok(user) = std::env::var("USER") {
        if user == "root" {
            return true;
        }
    }

    // 检查是否可以使用 sudo
    Command::new("sudo")
        .args(&["-n", "true"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_desktop_environment() {
        let desktop = detect_desktop_environment();
        println!("检测到的桌面环境: {:?}", desktop);
    }

    #[test]
    fn test_check_admin_privileges() {
        if cfg!(target_os = "linux") {
            let has_privileges = check_admin_privileges();
            println!("是否有管理员权限: {}", has_privileges);
        }
    }

    #[test]
    fn test_get_current_settings() {
        if cfg!(target_os = "linux") {
            match get_current_proxy_settings("") {
                Ok(settings) => {
                    println!("当前代理设置: {:?}", settings);
                }
                Err(e) => {
                    println!("获取代理设置失败: {}", e);
                }
            }
        }
    }
}
