// API 响应类型
export interface CommandResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// 仪表板统计
export interface DashboardStats {
  is_running: boolean;
  active_connections: number;
  total_connections: number;
  total_upload: number;
  total_download: number;
  zero_copy_count: number;
  proxy_mode: string;
  uptime: number;
}

// 代理节点
export interface ProxyNode {
  name: string;
  proxy_type: string;
  server: string;
  port: number;
  is_healthy: boolean;
  is_alive: boolean;
  latency: number | null;
}

// 代理组
export interface ProxyGroup {
  name: string;
  group_type: string;
  proxies: string[];
  selected?: string;
}

// 规则信息
export interface Rule {
  rule_type: string;
  content: string;
  target: string;
}

// 连接信息
export interface Connection {
  id: string;
  src_addr: string;
  dst_addr: string;
  proxy: string;
  upload: number;
  download: number;
  duration: number;
}

// 系统信息
export interface SystemInfo {
  os: string;
  arch: string;
  cpu_cores: number;
  total_memory: number;
  used_memory: number;
}

// 应用设置
export interface AppSettings {
  auto_start: boolean;
  auto_connect: boolean;
  system_proxy: boolean;
  allow_lan: boolean;
  http_port: number;
  socks_port: number;
  log_level: string;
  theme: string;
  language: string;
}

// 主题类型
export type Theme = 'light' | 'dark' | 'auto';

// 规则类型
export type RuleType =
  | 'DOMAIN-SUFFIX'
  | 'DOMAIN'
  | 'DOMAIN-KEYWORD'
  | 'IP-CIDR'
  | 'GEOIP'
  | 'MATCH';

// 规则目标
export type RuleTarget = 'DIRECT' | 'PROXY' | 'REJECT';

// 订阅管理
export interface Subscription {
  name: string;
  url: string;
  enabled: boolean;
  last_update: number;
  node_count: number;
  update_interval: number;
}

// 日志管理
export interface LogEntry {
  id: number;
  timestamp: string;
  level: string;
  target: string;
  message: string;
}

export interface LogStats {
  total: number;
  trace: number;
  debug: number;
  info: number;
  warn: number;
  error: number;
}

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

// 健康检查
export interface HealthCheckResult {
  proxy_name: string;
  is_healthy: boolean;
  latency_ms: number | null;
  last_check: number;
  consecutive_failures: number;
}

// 配置管理
export interface Config {
  port: number;
  socks_port: number;
  allow_lan: boolean;
  mode: string;
  log_level: string;
  proxies: ProxyNode[];
  proxy_groups?: ProxyGroup[];
  rules?: string[];
  subscriptions?: Subscription[];
  dns?: DnsConfig;
}

export interface DnsConfig {
  enable: boolean;
  listen: string;
  nameserver: string[];
}
