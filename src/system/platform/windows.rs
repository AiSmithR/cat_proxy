//! Windows 系统代理设置实现

use tracing::{debug, error, info, warn};
use std::process::Command;

/// 代理设置状态
#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub http_enabled: bool,
    pub https_enabled: bool,
    pub socks_enabled: bool,
}

/// 设置系统代理
pub fn set_system_proxy(enable: bool, http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;

        if enable {
            info!("启用 Windows 系统代理");
            enable_windows_proxy(http_port, socks_port)?;
        } else {
            info!("禁用 Windows 系统代理");
            disable_windows_proxy()?;
        }

        // 通知系统代理设置已更改
        refresh_windows_proxy_settings()?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (enable, http_port, socks_port);
        warn!("当前不是 Windows 系统，无法设置 Windows 代理");
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn enable_windows_proxy(http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    // 使用 reg 命令设置注册表
    // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings

    // 1. 设置 ProxyEnable = 1
    let output = Command::new("reg")
        .args(&[
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v", "ProxyEnable",
            "/t", "REG_DWORD",
            "/d", "1",
            "/f"
        ])
        .output()?;

    if !output.status.success() {
        error!("设置 ProxyEnable 失败: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Failed to enable proxy"));
    }

    // 2. 设置 ProxyServer
    let proxy_server = format!(
        "http=127.0.0.1:{};https=127.0.0.1:{};socks=127.0.0.1:{}",
        http_port, http_port, socks_port
    );

    let output = Command::new("reg")
        .args(&[
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v", "ProxyServer",
            "/t", "REG_SZ",
            "/d", &proxy_server,
            "/f"
        ])
        .output()?;

    if !output.status.success() {
        error!("设置 ProxyServer 失败: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Failed to set proxy server"));
    }

    // 3. 设置 ProxyOverride（绕过代理的地址）
    let proxy_override = "localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*;<local>";

    let output = Command::new("reg")
        .args(&[
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v", "ProxyOverride",
            "/t", "REG_SZ",
            "/d", proxy_override,
            "/f"
        ])
        .output()?;

    if !output.status.success() {
        error!("设置 ProxyOverride 失败: {}", String::from_utf8_lossy(&output.stderr));
    }

    debug!("Windows 代理已启用: {}", proxy_server);
    Ok(())
}

#[cfg(target_os = "windows")]
fn disable_windows_proxy() -> anyhow::Result<()> {
    // 设置 ProxyEnable = 0
    let output = Command::new("reg")
        .args(&[
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v", "ProxyEnable",
            "/t", "REG_DWORD",
            "/d", "0",
            "/f"
        ])
        .output()?;

    if !output.status.success() {
        error!("禁用代理失败: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Failed to disable proxy"));
    }

    debug!("Windows 代理已禁用");
    Ok(())
}

#[cfg(target_os = "windows")]
fn refresh_windows_proxy_settings() -> anyhow::Result<()> {
    // 使用 PowerShell 刷新 Internet Explorer 设置
    // 这会通知所有程序代理设置已更改
    let script = r#"
        $signature = @'
[DllImport("wininet.dll", SetLastError = true, CharSet=CharSet.Auto)]
public static extern bool InternetSetOption(IntPtr hInternet, int dwOption, IntPtr lpBuffer, int dwBufferLength);
'@
        $type = Add-Type -MemberDefinition $signature -Name WinInet -Namespace InternetSettings -PassThru
        $type::InternetSetOption(0, 39, 0, 0) | Out-Null
        $type::InternetSetOption(0, 37, 0, 0) | Out-Null
    "#;

    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", script])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                debug!("代理设置已刷新");
            } else {
                warn!("刷新代理设置失败，但不影响功能");
            }
        }
        Err(e) => {
            warn!("无法执行 PowerShell 刷新命令: {}", e);
        }
    }

    Ok(())
}

/// 获取当前代理设置
pub fn get_current_proxy_settings(_service: &str) -> anyhow::Result<ProxySettings> {
    #[cfg(target_os = "windows")]
    {
        // 查询 ProxyEnable 的值
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v", "ProxyEnable"
            ])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let enabled = output_str.contains("0x1");

        Ok(ProxySettings {
            http_enabled: enabled,
            https_enabled: enabled,
            socks_enabled: enabled,
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(ProxySettings {
            http_enabled: false,
            https_enabled: false,
            socks_enabled: false,
        })
    }
}

/// 检查是否有管理员权限
pub fn check_admin_privileges() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 尝试查询注册表来检查权限
        // 如果能成功查询，说明有基本权限
        Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings"
            ])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_admin_privileges() {
        if cfg!(target_os = "windows") {
            let has_privileges = check_admin_privileges();
            println!("Windows 权限检查: {}", has_privileges);
        }
    }

    #[test]
    fn test_get_current_settings() {
        if cfg!(target_os = "windows") {
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
