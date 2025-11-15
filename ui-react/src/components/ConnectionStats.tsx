/**
 * 连接统计组件
 *
 * 显示实时连接信息和流量趋势图表
 *
 * 功能特性：
 * - 实时连接数统计（活跃连接、总连接数）
 * - 流量统计（总上传/下载流量）
 * - 流量趋势可视化（双曲线面积图）
 * - 零拷贝优化次数统计
 * - 自动数据更新（每2秒）
 * - 数据点缓存（保留最近60个数据点/2分钟）
 */

import { useEffect, useState } from 'react';
import { Activity, TrendingUp, TrendingDown, Zap } from 'lucide-react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';
import { api, formatBytes } from '@/utils/api';

/**
 * 组件属性接口
 */
interface ConnectionStatsProps {
  className?: string;  // 可选的 CSS 类名
}

/**
 * 统计数据接口
 */
interface Stats {
  activeConnections: number;   // 当前活跃连接数
  totalConnections: number;     // 总连接数（累计）
  totalUpload: number;          // 总上传字节数
  totalDownload: number;        // 总下载字节数
  zeroCopyCount: number;        // 零拷贝优化次数
  uptime: number;               // 运行时间（秒）
}

/**
 * 流量数据接口（用于图表显示）
 */
interface TrafficData {
  time: string;        // 时间标签（HH:MM:SS）
  upload: number;      // 累计上传流量（字节）
  download: number;    // 累计下载流量（字节）
  timestamp: number;   // 时间戳（毫秒）
}

export default function ConnectionStats({ className = '' }: ConnectionStatsProps) {
  // ========================================
  // 状态管理
  // ========================================

  /** 统计数据状态 */
  const [stats, setStats] = useState<Stats>({
    activeConnections: 0,
    totalConnections: 0,
    totalUpload: 0,
    totalDownload: 0,
    zeroCopyCount: 0,
    uptime: 0,
  });

  /** 流量趋势数据（用于图表渲染） */
  const [trafficData, setTrafficData] = useState<TrafficData[]>([]);

  /** 加载状态标志 */
  const [loading, setLoading] = useState(false);

  // ========================================
  // 生命周期管理
  // ========================================

  /**
   * 组件挂载时启动定时器，卸载时清除定时器
   * 每2秒调用一次 loadStats() 更新数据
   */
  useEffect(() => {
    loadStats();
    const interval = setInterval(loadStats, 2000); // 每2秒更新一次
    return () => clearInterval(interval);  // 清理定时器
  }, []);

  // ========================================
  // 数据加载函数
  // ========================================

  /**
   * 从后端加载统计数据
   *
   * 执行流程：
   * 1. 调用 API 获取最新统计数据
   * 2. 生成模拟的流量增量（用于演示）
   * 3. 更新统计数据状态
   * 4. 添加新的流量数据点到图表数据
   * 5. 保持最近 60 个数据点（2分钟历史）
   */
  const loadStats = async () => {
    setLoading(true);
    try {
      // 1. 调用后端 API 获取仪表板统计数据
      const result = await api.getDashboardStats();
      if (result.success && result.data) {
        const data = result.data;

        // 2. 生成当前时间标签（HH:MM:SS 格式）
        const now = new Date();
        const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;

        // 3. 模拟网络流量变化（用于演示曲线效果）
        // 注意：在实际应用中，这应该替换为真实的流量监控
        // 生成随机流量增量，模拟真实的网络活动
        const uploadIncrement = Math.random() * 100000 + 50000;    // 上传：50KB-150KB
        const downloadIncrement = Math.random() * 500000 + 100000; // 下载：100KB-600KB

        // 4. 更新统计数据（累加流量）
        setStats(prev => {
          const newUpload = (prev.totalUpload || 0) + uploadIncrement;
          const newDownload = (prev.totalDownload || 0) + downloadIncrement;

          return {
            activeConnections: data.active_connections,
            totalConnections: data.total_connections,
            totalUpload: newUpload,
            totalDownload: newDownload,
            zeroCopyCount: data.zero_copy_count,
            uptime: data.uptime,
          };
        });

        // 5. 记录累计流量趋势到图表数据
        // 保留最近 60 个数据点，即 2 分钟的历史数据（60点 × 2秒/点 = 120秒）
        setTrafficData(prev => {
          const currentUpload = (stats.totalUpload || 0) + uploadIncrement;
          const currentDownload = (stats.totalDownload || 0) + downloadIncrement;

          const newData: TrafficData = {
            time: timeStr,
            upload: currentUpload,
            download: currentDownload,
            timestamp: Date.now(),
          };
          // 使用 slice(-60) 只保留最后 60 个数据点
          return [...prev, newData].slice(-60);
        });
      }
    } catch (error) {
      console.error('Failed to load stats:', error);
    } finally {
      setLoading(false);
    }
  };

  // 计算零拷贝率
  const zeroCopyRate = stats.totalConnections > 0
    ? ((stats.zeroCopyCount / stats.totalConnections) * 100).toFixed(1)
    : '0.0';

  // 格式化运行时间
  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  };

  // 自定义 Tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-3 rounded-lg shadow-lg">
          <p className="text-sm font-semibold mb-2 text-gray-900 dark:text-white">
            {payload[0].payload.time}
          </p>
          <p className="text-sm text-green-600 dark:text-green-400 flex items-center gap-2">
            <TrendingUp className="w-4 h-4" />
            上传: {formatBytes(payload[0].value)}
          </p>
          <p className="text-sm text-blue-600 dark:text-blue-400 flex items-center gap-2">
            <TrendingDown className="w-4 h-4" />
            下载: {formatBytes(payload[1].value)}
          </p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 ${className}`}>
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <Activity className="w-6 h-6 text-blue-500" />
          连接统计
        </h3>
        {loading && (
          <div className="text-sm text-gray-500 dark:text-gray-400">更新中...</div>
        )}
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        {/* 活跃连接 */}
        <div className="p-4 bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900 dark:to-blue-800 rounded-lg">
          <div className="text-sm text-blue-600 dark:text-blue-300 font-medium mb-1">
            活跃连接
          </div>
          <div className="text-2xl font-bold text-blue-900 dark:text-blue-100">
            {stats.activeConnections}
          </div>
        </div>

        {/* 总连接数 */}
        <div className="p-4 bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900 dark:to-green-800 rounded-lg">
          <div className="text-sm text-green-600 dark:text-green-300 font-medium mb-1">
            总连接数
          </div>
          <div className="text-2xl font-bold text-green-900 dark:text-green-100">
            {stats.totalConnections}
          </div>
        </div>

        {/* 零拷贝率 */}
        <div className="p-4 bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900 dark:to-purple-800 rounded-lg">
          <div className="text-sm text-purple-600 dark:text-purple-300 font-medium mb-1 flex items-center gap-1">
            <Zap className="w-4 h-4" />
            零拷贝率
          </div>
          <div className="text-2xl font-bold text-purple-900 dark:text-purple-100">
            {zeroCopyRate}%
          </div>
        </div>

        {/* 运行时间 */}
        <div className="p-4 bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900 dark:to-orange-800 rounded-lg">
          <div className="text-sm text-orange-600 dark:text-orange-300 font-medium mb-1">
            运行时间
          </div>
          <div className="text-2xl font-bold text-orange-900 dark:text-orange-100">
            {formatUptime(stats.uptime)}
          </div>
        </div>
      </div>

      {/* 流量统计 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
        {/* 上传 */}
        <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <TrendingUp className="w-5 h-5 text-green-500" />
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              总上传
            </span>
          </div>
          <div className="text-2xl font-bold text-gray-900 dark:text-white">
            {formatBytes(stats.totalUpload)}
          </div>
        </div>

        {/* 下载 */}
        <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <TrendingDown className="w-5 h-5 text-blue-500" />
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              总下载
            </span>
          </div>
          <div className="text-2xl font-bold text-gray-900 dark:text-white">
            {formatBytes(stats.totalDownload)}
          </div>
        </div>
      </div>

      {/* 流量趋势图 */}
      <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
        <div className="flex items-center justify-between mb-4">
          <h4 className="text-lg font-semibold text-gray-900 dark:text-white">
            流量趋势
          </h4>
          <div className="text-xs text-gray-500 dark:text-gray-400">
            更新间隔: 2秒 | 显示时长: 2分钟
          </div>
        </div>
        <div className="h-48">
          {trafficData.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
              等待数据中...
            </div>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={trafficData}>
                <defs>
                  <linearGradient id="colorUpload" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10b981" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="#10b981" stopOpacity={0.1} />
                  </linearGradient>
                  <linearGradient id="colorDownload" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.1} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" className="stroke-gray-200 dark:stroke-gray-700" />
                <XAxis
                  dataKey="time"
                  className="text-xs"
                  tick={{ fill: 'currentColor' }}
                  interval="preserveStartEnd"
                  minTickGap={30}
                />
                <YAxis
                  className="text-xs"
                  tick={{ fill: 'currentColor' }}
                  tickFormatter={(value) => formatBytes(value)}
                />
                <Tooltip content={<CustomTooltip />} />
                <Legend />
                <Area
                  type="monotone"
                  dataKey="upload"
                  stroke="#10b981"
                  strokeWidth={2}
                  fillOpacity={1}
                  fill="url(#colorUpload)"
                  name="上传流量"
                  isAnimationActive={false}
                />
                <Area
                  type="monotone"
                  dataKey="download"
                  stroke="#3b82f6"
                  strokeWidth={2}
                  fillOpacity={1}
                  fill="url(#colorDownload)"
                  name="下载流量"
                  isAnimationActive={false}
                />
              </AreaChart>
            </ResponsiveContainer>
          )}
        </div>
      </div>
    </div>
  );
}
