pub mod crypto;

pub fn parse_port(s: &str) -> Option<u16> {
    s.parse().ok()
}

pub fn is_valid_domain(domain: &str) -> bool {
    !domain.is_empty() && domain.len() < 256
}
