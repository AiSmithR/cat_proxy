//! 配置解析辅助模块

use super::ProxyConfig;

/// 解析配置文件中的代理列表
pub fn parse_proxy_list(_content: &str) -> anyhow::Result<Vec<ProxyConfig>> {
    // TODO: 实现通用的代理配置解析
    Ok(Vec::new())
}
