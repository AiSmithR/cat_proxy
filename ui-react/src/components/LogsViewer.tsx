import { useEffect, useState } from 'react';
import { X, RefreshCw, Trash2, AlertCircle, Info, AlertTriangle, XCircle, Bug, Search, ChevronLeft, ChevronRight } from 'lucide-react';
import { api } from '@/utils/api';
import type { LogEntry, LogStats, LogLevel } from '@/types';

interface LogsViewerProps {
  isOpen: boolean;
  onClose: () => void;
}

const LOG_LEVEL_COLORS = {
  trace: 'text-gray-500 bg-gray-100 dark:bg-gray-700',
  debug: 'text-blue-600 bg-blue-100 dark:bg-blue-900',
  info: 'text-green-600 bg-green-100 dark:bg-green-900',
  warn: 'text-yellow-600 bg-yellow-100 dark:bg-yellow-900',
  error: 'text-red-600 bg-red-100 dark:bg-red-900',
};

const LOG_LEVEL_ICONS = {
  trace: Bug,
  debug: Info,
  info: Info,
  warn: AlertTriangle,
  error: XCircle,
};

export default function LogsViewer({ isOpen, onClose }: LogsViewerProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [stats, setStats] = useState<LogStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [minLevel, setMinLevel] = useState<LogLevel | ''>('');
  const [search, setSearch] = useState('');
  const [target, setTarget] = useState('');

  // Pagination
  const [page, setPage] = useState(0);
  const [pageSize] = useState(50);
  const [totalLogs, setTotalLogs] = useState(0);

  useEffect(() => {
    if (isOpen) {
      loadLogs();
      loadStats();
    }
  }, [isOpen, page, minLevel, search, target]);

  const loadLogs = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getLogs({
        page,
        pageSize,
        minLevel: minLevel || undefined,
        search: search || undefined,
        target: target || undefined,
      });

      if (result.success && result.data) {
        const [logData, total] = result.data;
        setLogs(logData);
        setTotalLogs(total);
      } else {
        setError(result.error || '加载日志失败');
      }
    } catch (err) {
      setError('加载日志失败: ' + String(err));
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const result = await api.getLogStats();
      if (result.success && result.data) {
        setStats(result.data);
      }
    } catch (err) {
      console.error('Failed to load log stats:', err);
    }
  };

  const handleClearLogs = async () => {
    if (!confirm('确定要清空所有日志吗？')) {
      return;
    }

    try {
      const result = await api.clearLogs();
      if (result.success) {
        setLogs([]);
        setTotalLogs(0);
        setPage(0);
        await loadStats();
      } else {
        setError(result.error || '清空日志失败');
      }
    } catch (err) {
      setError('清空日志失败: ' + String(err));
    }
  };

  const handleRefresh = () => {
    setPage(0);
    loadLogs();
    loadStats();
  };

  const handleFilterChange = () => {
    setPage(0);
  };

  const totalPages = Math.ceil(totalLogs / pageSize);

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      const hours = date.getHours().toString().padStart(2, '0');
      const minutes = date.getMinutes().toString().padStart(2, '0');
      const seconds = date.getSeconds().toString().padStart(2, '0');
      const ms = date.getMilliseconds().toString().padStart(3, '0');
      return `${hours}:${minutes}:${seconds}.${ms}`;
    } catch {
      return timestamp;
    }
  };

  const getLevelIcon = (level: string) => {
    const Icon = LOG_LEVEL_ICONS[level.toLowerCase() as LogLevel] || Info;
    return <Icon className="w-4 h-4" />;
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-7xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              日志查看器
            </h2>
            {stats && (
              <div className="flex items-center gap-2 text-sm">
                <span className="text-gray-600 dark:text-gray-400">总计: {stats.total}</span>
                {stats.error > 0 && (
                  <span className="px-2 py-0.5 bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-200 rounded-full">
                    错误: {stats.error}
                  </span>
                )}
                {stats.warn > 0 && (
                  <span className="px-2 py-0.5 bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200 rounded-full">
                    警告: {stats.warn}
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleRefresh}
              disabled={loading}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              <RefreshCw className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`} />
            </button>
            <button
              onClick={handleClearLogs}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              <Trash2 className="w-5 h-5 text-red-500" />
            </button>
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Filters */}
        <div className="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                最低级别
              </label>
              <select
                value={minLevel}
                onChange={(e) => {
                  setMinLevel(e.target.value as LogLevel | '');
                  handleFilterChange();
                }}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="">全部</option>
                <option value="trace">Trace</option>
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warn</option>
                <option value="error">Error</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                模块过滤
              </label>
              <input
                type="text"
                value={target}
                onChange={(e) => {
                  setTarget(e.target.value);
                  handleFilterChange();
                }}
                placeholder="例如：cat_proxy"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                搜索内容
              </label>
              <div className="relative">
                <input
                  type="text"
                  value={search}
                  onChange={(e) => {
                    setSearch(e.target.value);
                    handleFilterChange();
                  }}
                  placeholder="搜索日志消息"
                  className="w-full px-3 py-2 pl-10 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                />
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              </div>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {error && (
            <div className="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg flex items-center gap-2">
              <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-300" />
              <span className="text-red-800 dark:text-red-200">{error}</span>
            </div>
          )}

          {loading && logs.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              加载中...
            </div>
          ) : logs.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              暂无日志
            </div>
          ) : (
            <div className="space-y-1 font-mono text-sm">
              {logs.map((log) => (
                <div
                  key={log.id}
                  className="p-2 border border-gray-200 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors"
                >
                  <div className="flex items-start gap-3">
                    <span className="text-gray-500 dark:text-gray-400 text-xs whitespace-nowrap">
                      {formatTimestamp(log.timestamp)}
                    </span>
                    <span
                      className={`px-2 py-0.5 rounded-full text-xs font-semibold uppercase flex items-center gap-1 whitespace-nowrap ${
                        LOG_LEVEL_COLORS[log.level.toLowerCase() as LogLevel] ||
                        'text-gray-600 bg-gray-100'
                      }`}
                    >
                      {getLevelIcon(log.level)}
                      {log.level}
                    </span>
                    {log.target && (
                      <span className="text-blue-600 dark:text-blue-400 text-xs whitespace-nowrap">
                        [{log.target}]
                      </span>
                    )}
                    <span className="text-gray-900 dark:text-gray-100 flex-1 break-words">
                      {log.message}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
            <div className="flex items-center justify-between">
              <div className="text-sm text-gray-600 dark:text-gray-400">
                显示 {page * pageSize + 1} - {Math.min((page + 1) * pageSize, totalLogs)} / 共 {totalLogs} 条
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setPage(Math.max(0, page - 1))}
                  disabled={page === 0}
                  className="p-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  第 {page + 1} / {totalPages} 页
                </span>
                <button
                  onClick={() => setPage(Math.min(totalPages - 1, page + 1))}
                  disabled={page >= totalPages - 1}
                  className="p-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
