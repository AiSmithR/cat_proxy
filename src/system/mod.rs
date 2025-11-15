//! 系统集成模块

pub mod platform;

// 重新导出平台相关的类型和函数
#[cfg(target_os = "macos")]
pub use platform::macos::{set_system_proxy, get_current_proxy_settings, check_admin_privileges, ProxySettings};

#[cfg(target_os = "windows")]
pub use platform::windows::{set_system_proxy, get_current_proxy_settings, check_admin_privileges, ProxySettings};

#[cfg(target_os = "linux")]
pub use platform::linux::{set_system_proxy, get_current_proxy_settings, check_admin_privileges, ProxySettings};

// 如果不是支持的平台，提供默认实现
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn set_system_proxy(_enable: bool, _http_port: u16, _socks_port: u16) -> anyhow::Result<()> {
    anyhow::bail!("当前平台不支持系统代理设置")
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn check_admin_privileges() -> bool {
    false
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub http_enabled: bool,
    pub https_enabled: bool,
    pub socks_enabled: bool,
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn get_current_proxy_settings(_service: &str) -> anyhow::Result<ProxySettings> {
    anyhow::bail!("当前平台不支持获取代理设置")
}
