use std::net::IpAddr;
use std::str::FromStr;

/// 规则类型
#[derive(Debug, Clone)]
pub enum Rule {
    /// 域名完全匹配
    Domain(String),
    /// 域名后缀匹配
    DomainSuffix(String),
    /// 域名关键字匹配
    DomainKeyword(String),
    /// IP CIDR 匹配
    IpCidr(String),
    /// GeoIP 匹配
    GeoIp(String),
    /// 进程名匹配
    ProcessName(String),
    /// 匹配所有
    Match,
}

/// 规则匹配器
pub struct RuleMatcher {
    rules: Vec<(Rule, String)>,
}

impl RuleMatcher {
    /// 创建新的规则匹配器
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Rule, target: String) {
        self.rules.push((rule, target));
    }

    /// 匹配域名
    pub fn match_domain(&self, domain: &str) -> Option<String> {
        for (rule, target) in &self.rules {
            let matched = match rule {
                Rule::Domain(d) => {
                    // 完全匹配
                    domain.eq_ignore_ascii_case(d)
                }
                Rule::DomainSuffix(suffix) => {
                    // 后缀匹配
                    let domain_lower = domain.to_lowercase();
                    let suffix_lower = suffix.to_lowercase();

                    // 完全匹配或以 ".suffix" 结尾
                    domain_lower == suffix_lower ||
                    domain_lower.ends_with(&format!(".{}", suffix_lower))
                }
                Rule::DomainKeyword(keyword) => {
                    // 关键字匹配
                    domain.to_lowercase().contains(&keyword.to_lowercase())
                }
                Rule::Match => {
                    // 匹配所有
                    true
                }
                _ => false,
            };

            if matched {
                return Some(target.clone());
            }
        }
        None
    }

    /// 匹配 IP
    pub fn match_ip(&self, ip: IpAddr) -> Option<String> {
        for (rule, target) in &self.rules {
            let matched = match rule {
                Rule::IpCidr(cidr) => {
                    // TODO: 实现 CIDR 匹配
                    // 简单实现：检查是否是完全匹配
                    if let Ok(rule_ip) = IpAddr::from_str(cidr.split('/').next().unwrap_or("")) {
                        ip == rule_ip
                    } else {
                        false
                    }
                }
                Rule::GeoIp(_country) => {
                    // TODO: 实现 GeoIP 匹配
                    false
                }
                Rule::Match => true,
                _ => false,
            };

            if matched {
                return Some(target.clone());
            }
        }
        None
    }

    /// 解析规则字符串
    pub fn parse_rule(rule_str: &str) -> anyhow::Result<(Rule, String)> {
        let parts: Vec<&str> = rule_str.split(',').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty rule"));
        }

        // 获取规则类型
        let rule_type = parts[0];

        // 对于 MATCH 规则，只需要类型和目标
        if rule_type.eq_ignore_ascii_case("MATCH") {
            let target = parts.get(1).unwrap_or(&"DIRECT").to_string();
            return Ok((Rule::Match, target));
        }

        // 其他规则需要至少 3 部分：类型,值,目标
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Invalid rule format: {}", rule_str));
        }

        let rule_value = parts[1];
        let target = parts[2].to_string();

        let rule = match rule_type.to_uppercase().as_str() {
            "DOMAIN" => Rule::Domain(rule_value.to_string()),
            "DOMAIN-SUFFIX" => Rule::DomainSuffix(rule_value.to_string()),
            "DOMAIN-KEYWORD" => Rule::DomainKeyword(rule_value.to_string()),
            "IP-CIDR" => Rule::IpCidr(rule_value.to_string()),
            "GEOIP" => Rule::GeoIp(rule_value.to_string()),
            "PROCESS-NAME" => Rule::ProcessName(rule_value.to_string()),
            _ => return Err(anyhow::anyhow!("Unknown rule type: {}", rule_type)),
        };

        Ok((rule, target))
    }

    /// 获取规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for RuleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule() {
        // 测试域名后缀规则
        let (rule, target) = RuleMatcher::parse_rule("DOMAIN-SUFFIX,google.com,PROXY").unwrap();
        assert!(matches!(rule, Rule::DomainSuffix(_)));
        assert_eq!(target, "PROXY");

        // 测试域名完全匹配规则
        let (rule, target) = RuleMatcher::parse_rule("DOMAIN,example.com,DIRECT").unwrap();
        assert!(matches!(rule, Rule::Domain(_)));
        assert_eq!(target, "DIRECT");

        // 测试 MATCH 规则
        let (rule, target) = RuleMatcher::parse_rule("MATCH,PROXY").unwrap();
        assert!(matches!(rule, Rule::Match));
        assert_eq!(target, "PROXY");

        // 测试 IP-CIDR 规则
        let (rule, target) = RuleMatcher::parse_rule("IP-CIDR,192.168.0.0/16,DIRECT").unwrap();
        assert!(matches!(rule, Rule::IpCidr(_)));
        assert_eq!(target, "DIRECT");
    }

    #[test]
    fn test_match_domain() {
        let mut matcher = RuleMatcher::new();

        // 添加域名后缀规则
        matcher.add_rule(Rule::DomainSuffix("google.com".to_string()), "PROXY".to_string());
        matcher.add_rule(Rule::DomainSuffix("cn".to_string()), "DIRECT".to_string());
        matcher.add_rule(Rule::DomainKeyword("youtube".to_string()), "PROXY".to_string());
        matcher.add_rule(Rule::Domain("example.com".to_string()), "DIRECT".to_string());

        // 测试域名后缀匹配
        assert_eq!(matcher.match_domain("www.google.com"), Some("PROXY".to_string()));
        assert_eq!(matcher.match_domain("google.com"), Some("PROXY".to_string()));
        assert_eq!(matcher.match_domain("mail.google.com"), Some("PROXY".to_string()));

        // 测试域名关键字匹配
        assert_eq!(matcher.match_domain("youtube.com"), Some("PROXY".to_string()));
        assert_eq!(matcher.match_domain("m.youtube.com"), Some("PROXY".to_string()));

        // 测试完全匹配
        assert_eq!(matcher.match_domain("example.com"), Some("DIRECT".to_string()));

        // 测试不匹配的情况
        assert_eq!(matcher.match_domain("notmatch.org"), None);

        // 测试 .cn 后缀
        assert_eq!(matcher.match_domain("baidu.cn"), Some("DIRECT".to_string()));
        assert_eq!(matcher.match_domain("qq.com.cn"), Some("DIRECT".to_string()));
    }

    #[test]
    fn test_match_all() {
        let mut matcher = RuleMatcher::new();
        matcher.add_rule(Rule::DomainSuffix("google.com".to_string()), "PROXY".to_string());
        matcher.add_rule(Rule::Match, "DIRECT".to_string());

        // 匹配具体规则
        assert_eq!(matcher.match_domain("www.google.com"), Some("PROXY".to_string()));

        // 匹配 MATCH 规则
        assert_eq!(matcher.match_domain("anything.com"), Some("DIRECT".to_string()));
    }
}
