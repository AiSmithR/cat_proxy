/**
 * Tauri API 工具模块
 *
 * 提供前端与 Rust 后端通信的 API 接口
 *
 * 功能模块：
 * - 仪表板统计
 * - 代理控制（启动/停止）
 * - 节点和组管理
 * - 规则管理
 * - 连接管理
 * - 系统配置
 * - 订阅管理
 * - 健康检查
 * - 日志系统
 *
 * 工具函数：
 * - formatBytes: 格式化字节数为易读形式
 * - formatDuration: 格式化时长为易读形式
 *
 * @module utils/api
 */

import { invoke } from '@tauri-apps/api/tauri';
import type {
  CommandResponse,
  DashboardStats,
  ProxyNode,
  ProxyGroup,
  Rule,
  Connection,
  SystemInfo,
  AppSettings,
  Subscription,
  LogEntry,
  LogStats,
  LogLevel,
  Config,
} from '@/types';

/**
 * 调用 Tauri 后端命令的通用函数
 *
 * 这是一个包装函数，用于统一处理所有 Tauri 命令调用：
 * - 自动错误捕获
 * - 统一的返回格式
 * - 错误日志记录
 *
 * @template T - 命令返回数据的类型
 * @param command - Tauri 命令名称（对应 Rust 中的 #[tauri::command]）
 * @param args - 命令参数对象（可选）
 * @returns Promise<CommandResponse<T>> - 统一的命令响应格式
 *
 * @example
 * ```typescript
 * // 调用无参数命令
 * const stats = await invokeCommand<DashboardStats>('get_dashboard_stats');
 *
 * // 调用带参数命令
 * const result = await invokeCommand<string>('add_rule', { rule: 'DOMAIN,google.com,PROXY' });
 * ```
 */
async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<CommandResponse<T>> {
  try {
    // 调用 Tauri 后端命令
    const result = await invoke<CommandResponse<T>>(command, args);
    return result;
  } catch (error) {
    // 错误处理：记录日志并返回错误响应
    console.error(`Command ${command} failed:`, error);
    return {
      success: false,
      error: String(error),
    };
  }
}

/**
 * API 对象
 *
 * 包含所有可用的后端 API 调用方法
 * 所有方法返回 Promise<CommandResponse<T>>
 *
 * 命名约定：
 * - get*: 获取数据
 * - set*update*: 修改数据
 * - add*delete*: 添加/删除数据
 * - start*stop*: 启动/停止服务
 * - test*: 测试功能
 */
export const api = {
  // ========================================
  // 仪表板
  // ========================================

  /**
   * 获取仪表板统计信息
   * @returns 包含运行状态、连接数、流量等信息
   */
  getDashboardStats: () => invokeCommand<DashboardStats>('get_dashboard_stats'),

  // ========================================
  // 代理控制
  // ========================================

  /**
   * 启动代理服务
   * @returns 成功消息或错误信息
   */
  startProxy: () => invokeCommand<string>('start_proxy'),

  /**
   * 停止代理服务
   * @returns 成功消息或错误信息
   */
  stopProxy: () => invokeCommand<string>('stop_proxy'),

  // ========================================
  // 节点和组管理
  // ========================================

  /**
   * 获取所有代理节点列表
   * @returns 代理节点数组
   */
  getProxyNodes: () => invokeCommand<ProxyNode[]>('get_proxy_nodes'),

  /**
   * 获取所有代理组列表
   * @returns 代理组数组
   */
  getProxyGroups: () => invokeCommand<ProxyGroup[]>('get_proxy_groups'),

  /**
   * 测试单个代理节点的延迟
   * @param name - 代理节点名称
   * @returns 延迟时间（毫秒）
   */
  testProxyNode: (name: string) => invokeCommand<number>('test_proxy_node', { nodeName: name }),

  // ========================================
  // 规则管理
  // ========================================

  /**
   * 获取所有路由规则
   * @returns 规则数组
   */
  getRules: () => invokeCommand<Rule[]>('get_rules'),

  /**
   * 添加新规则
   * @param rule - 规则字符串（格式: "TYPE,PATTERN,TARGET"）
   * @returns 成功消息或错误信息
   */
  addRule: (rule: string) => invokeCommand<string>('add_rule', { rule }),

  /**
   * 删除规则
   * @param index - 规则索引
   * @returns 成功消息或错误信息
   */
  deleteRule: (index: number) => invokeCommand<string>('delete_rule', { index }),

  /**
   * 更新规则
   * @param index - 规则索引
   * @param rule - 新的规则字符串
   * @returns 成功消息或错误信息
   */
  updateRule: (index: number, rule: string) =>
    invokeCommand<string>('update_rule', { index, rule }),

  // ========================================
  // 连接管理
  // ========================================

  /**
   * 获取当前活跃连接列表
   * @returns 连接数组
   */
  getConnections: () => invokeCommand<Connection[]>('get_connections'),

  /**
   * 清空连接记录
   * @returns 成功消息或错误信息
   */
  clearConnections: () => invokeCommand<string>('clear_connections'),

  // ========================================
  // 系统和配置
  // ========================================

  /**
   * 获取系统信息
   * @returns 系统信息（操作系统、CPU、内存等）
   */
  getSystemInfo: () => invokeCommand<SystemInfo>('get_system_info'),

  /**
   * 获取应用设置
   * @returns 应用设置对象
   */
  getSettings: () => invokeCommand<AppSettings>('get_settings'),

  /**
   * 更新应用设置
   * @param settings - 新的设置对象
   * @returns 成功消息或错误信息
   */
  updateSettings: (settings: AppSettings) =>
    invokeCommand<string>('update_settings', { settings }),

  /**
   * 获取完整配置
   * @returns 配置对象
   */
  getConfig: () => invokeCommand<Config>('get_config'),

  /**
   * 更新配置
   * @param config - 新的配置对象
   * @returns 成功消息或错误信息
   */
  updateConfig: (config: Config) => invokeCommand<string>('update_config', { config }),

  /**
   * 保存配置到文件
   * @returns 成功消息或错误信息
   */
  saveConfig: () => invokeCommand<string>('save_config'),

  /**
   * 导出配置
   * @returns 配置文件内容
   */
  exportConfig: () => invokeCommand<string>('export_config'),

  // ========================================
  // 系统代理
  // ========================================

  /**
   * 设置系统代理
   * @param enable - 是否启用系统代理
   * @param httpPort - HTTP 代理端口
   * @param socksPort - SOCKS5 代理端口
   * @returns 成功消息或错误信息
   */
  setSystemProxy: (enable: boolean, httpPort: number, socksPort: number) =>
    invokeCommand<string>('set_system_proxy', {
      enable,
      httpPort,
      socksPort,
    }),

  // ========================================
  // 订阅管理
  // ========================================

  /**
   * 导入新订阅
   * @param name - 订阅名称
   * @param url - 订阅URL
   * @returns 成功消息或错误信息
   */
  importSubscription: (name: string, url: string) =>
    invokeCommand<string>('import_subscription', { name, url }),

  /**
   * 更新订阅
   * @param name - 订阅名称（可选，不传则更新所有订阅）
   * @returns 成功消息或错误信息
   */
  updateSubscription: (name?: string) =>
    invokeCommand<string>('update_subscription', { name }),

  /**
   * 获取所有订阅列表
   * @returns 订阅数组
   */
  getSubscriptions: () => invokeCommand<Subscription[]>('get_subscriptions'),

  /**
   * 删除订阅
   * @param name - 订阅名称
   * @returns 成功消息或错误信息
   */
  deleteSubscription: (name: string) =>
    invokeCommand<string>('delete_subscription', { name }),

  /**
   * 启用或禁用订阅
   * @param name - 订阅名称
   * @param enabled - 是否启用
   * @returns 成功消息或错误信息
   */
  setSubscriptionEnabled: (name: string, enabled: boolean) =>
    invokeCommand<string>('set_subscription_enabled', { name, enabled }),

  // ========================================
  // 健康检查
  // ========================================

  /**
   * 测试所有代理节点
   * @returns 节点名称和延迟的数组（延迟为 null 表示测试失败）
   */
  testAllProxyNodes: () =>
    invokeCommand<Array<[string, number | null]>>('test_all_proxy_nodes'),

  /**
   * 获取所有代理节点的延迟信息
   * @returns [节点名称, 延迟(ms), 是否健康] 的数组
   */
  getProxyLatencies: () =>
    invokeCommand<Array<[string, number | null, boolean]>>('get_proxy_latencies'),

  /**
   * 启动自动健康检查
   * @returns 成功消息或错误信息
   */
  startAutoHealthCheck: () =>
    invokeCommand<string>('start_auto_health_check'),

  /**
   * 获取带延迟信息的代理节点列表
   * @returns 包含延迟信息的代理节点数组
   */
  getProxyNodesWithLatency: () =>
    invokeCommand<ProxyNode[]>('get_proxy_nodes_with_latency'),

  // ========================================
  // 日志系统
  // ========================================

  /**
   * 获取日志（分页）
   * @param params - 查询参数
   * @param params.page - 页码（从 1 开始）
   * @param params.pageSize - 每页条数
   * @param params.minLevel - 最低日志级别
   * @param params.target - 目标模块过滤
   * @param params.search - 搜索关键词
   * @returns [日志数组, 总数] 的元组
   */
  getLogs: (params: {
    page?: number;
    pageSize?: number;
    minLevel?: LogLevel;
    target?: string;
    search?: string;
  }) =>
    invokeCommand<[LogEntry[], number]>('get_logs', {
      page: params.page,
      pageSize: params.pageSize,
      minLevel: params.minLevel,
      target: params.target,
      search: params.search,
    }),

  /**
   * 获取最新的 N 条日志
   * @param count - 日志条数
   * @returns 日志数组
   */
  getLatestLogs: (count: number) =>
    invokeCommand<LogEntry[]>('get_latest_logs', { count }),

  /**
   * 清空所有日志
   * @returns 成功消息或错误信息
   */
  clearLogs: () => invokeCommand<string>('clear_logs'),

  /**
   * 获取日志统计信息
   * @returns 日志统计对象（总数、各级别数量等）
   */
  getLogStats: () => invokeCommand<LogStats>('get_log_stats'),

  /**
   * 添加测试日志（仅用于开发调试）
   * @param level - 日志级别
   * @param message - 日志消息
   * @returns 成功消息或错误信息
   */
  addTestLog: (level: string, message: string) =>
    invokeCommand<string>('add_test_log', { level, message }),
};

// ========================================
// 工具函数
// ========================================

/**
 * 格式化字节数为人类可读格式
 *
 * 自动选择合适的单位（B、KB、MB、GB、TB）
 * 保留两位小数
 *
 * @param bytes - 字节数
 * @returns 格式化的字符串（例如: "1.23 MB"）
 *
 * @example
 * ```typescript
 * formatBytes(0)          // "0 B"
 * formatBytes(1024)       // "1.00 KB"
 * formatBytes(1536)       // "1.50 KB"
 * formatBytes(1048576)    // "1.00 MB"
 * formatBytes(1073741824) // "1.00 GB"
 * ```
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';

  const k = 1024;  // 单位进制（1KB = 1024B）
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];  // 单位数组

  // 计算合适的单位索引
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  // 转换并保留两位小数
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

/**
 * 格式化时长为人类可读格式
 *
 * 根据时长自动选择格式：
 * - < 60秒: "Xs"
 * - < 1小时: "Xm Ys"
 * - >= 1小时: "Xh Ym"
 *
 * @param seconds - 秒数
 * @returns 格式化的字符串
 *
 * @example
 * ```typescript
 * formatDuration(30)     // "30s"
 * formatDuration(90)     // "1m 30s"
 * formatDuration(3661)   // "1h 1m"
 * formatDuration(7200)   // "2h 0m"
 * ```
 */
export function formatDuration(seconds: number): string {
  // 小于 1 分钟
  if (seconds < 60) return `${seconds}s`;

  // 小于 1 小时
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;

  // 1 小时或更长
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${minutes}m`;
}
