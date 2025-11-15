import { useEffect, useState } from 'react';
import { X, Plus, RefreshCw, Trash2, Check, AlertCircle, Clock } from 'lucide-react';
import { api } from '@/utils/api';
import type { Subscription } from '@/types';

interface SubscriptionManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SubscriptionManager({ isOpen, onClose }: SubscriptionManagerProps) {
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [loading, setLoading] = useState(false);
  const [updating, setUpdating] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newSub, setNewSub] = useState({ name: '', url: '' });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadSubscriptions();
    }
  }, [isOpen]);

  const loadSubscriptions = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getSubscriptions();
      if (result.success && result.data) {
        setSubscriptions(result.data);
      } else {
        setError(result.error || '加载订阅失败');
      }
    } catch (err) {
      setError('加载订阅失败: ' + String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleAddSubscription = async () => {
    if (!newSub.name || !newSub.url) {
      setError('请填写订阅名称和 URL');
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await api.importSubscription(newSub.name, newSub.url);
      if (result.success) {
        setNewSub({ name: '', url: '' });
        setShowAddForm(false);
        await loadSubscriptions();
      } else {
        setError(result.error || '添加订阅失败');
      }
    } catch (err) {
      setError('添加订阅失败: ' + String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateSubscription = async (name: string) => {
    setUpdating(name);
    setError(null);
    try {
      const result = await api.updateSubscription(name);
      if (result.success) {
        await loadSubscriptions();
      } else {
        setError(result.error || '更新订阅失败');
      }
    } catch (err) {
      setError('更新订阅失败: ' + String(err));
    } finally {
      setUpdating(null);
    }
  };

  const handleUpdateAllSubscriptions = async () => {
    setUpdating('all');
    setError(null);
    try {
      const result = await api.updateSubscription();
      if (result.success) {
        await loadSubscriptions();
      } else {
        setError(result.error || '更新全部订阅失败');
      }
    } catch (err) {
      setError('更新全部订阅失败: ' + String(err));
    } finally {
      setUpdating(null);
    }
  };

  const handleDeleteSubscription = async (name: string) => {
    if (!confirm(`确定要删除订阅 "${name}" 吗？`)) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await api.deleteSubscription(name);
      if (result.success) {
        await loadSubscriptions();
      } else {
        setError(result.error || '删除订阅失败');
      }
    } catch (err) {
      setError('删除订阅失败: ' + String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleToggleEnabled = async (name: string, enabled: boolean) => {
    setError(null);
    try {
      const result = await api.setSubscriptionEnabled(name, enabled);
      if (result.success) {
        await loadSubscriptions();
      } else {
        setError(result.error || '更新订阅状态失败');
      }
    } catch (err) {
      setError('更新订阅状态失败: ' + String(err));
    }
  };

  const formatLastUpdate = (timestamp: number) => {
    if (timestamp === 0) return '从未更新';
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('zh-CN');
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            订阅管理
          </h2>
          <div className="flex items-center gap-2">
            <button
              onClick={handleUpdateAllSubscriptions}
              disabled={updating === 'all' || subscriptions.length === 0}
              className="px-3 py-1.5 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white rounded-lg text-sm flex items-center gap-1 transition-colors"
            >
              <RefreshCw className={`w-4 h-4 ${updating === 'all' ? 'animate-spin' : ''}`} />
              更新全部
            </button>
            <button
              onClick={() => setShowAddForm(true)}
              className="px-3 py-1.5 bg-green-500 hover:bg-green-600 text-white rounded-lg text-sm flex items-center gap-1 transition-colors"
            >
              <Plus className="w-4 h-4" />
              添加订阅
            </button>
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-4 overflow-y-auto max-h-[calc(80vh-80px)]">
          {error && (
            <div className="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg flex items-center gap-2">
              <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-300" />
              <span className="text-red-800 dark:text-red-200">{error}</span>
            </div>
          )}

          {/* Add Form */}
          {showAddForm && (
            <div className="mb-4 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
              <h3 className="text-lg font-medium mb-3 text-gray-900 dark:text-white">
                添加新订阅
              </h3>
              <div className="space-y-3">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    订阅名称
                  </label>
                  <input
                    type="text"
                    value={newSub.name}
                    onChange={(e) => setNewSub({ ...newSub, name: e.target.value })}
                    placeholder="例如：我的订阅"
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    订阅 URL
                  </label>
                  <input
                    type="text"
                    value={newSub.url}
                    onChange={(e) => setNewSub({ ...newSub, url: e.target.value })}
                    placeholder="https://example.com/subscription"
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={handleAddSubscription}
                    disabled={loading}
                    className="px-4 py-2 bg-green-500 hover:bg-green-600 disabled:bg-gray-400 text-white rounded-lg transition-colors"
                  >
                    确定
                  </button>
                  <button
                    onClick={() => {
                      setShowAddForm(false);
                      setNewSub({ name: '', url: '' });
                      setError(null);
                    }}
                    className="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg transition-colors"
                  >
                    取消
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Subscriptions List */}
          {loading && subscriptions.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              加载中...
            </div>
          ) : subscriptions.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              暂无订阅，点击"添加订阅"开始
            </div>
          ) : (
            <div className="space-y-3">
              {subscriptions.map((sub) => (
                <div
                  key={sub.name}
                  className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                          {sub.name}
                        </h3>
                        {sub.enabled ? (
                          <span className="px-2 py-0.5 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 text-xs rounded-full flex items-center gap-1">
                            <Check className="w-3 h-3" />
                            已启用
                          </span>
                        ) : (
                          <span className="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 text-xs rounded-full">
                            已禁用
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-2 break-all">
                        {sub.url}
                      </p>
                      <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                        <span className="flex items-center gap-1">
                          <Clock className="w-4 h-4" />
                          {formatLastUpdate(sub.last_update)}
                        </span>
                        <span>节点数: {sub.node_count}</span>
                        <span>更新间隔: {sub.update_interval / 3600} 小时</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleToggleEnabled(sub.name, !sub.enabled)}
                        className={`px-3 py-1.5 rounded-lg text-sm transition-colors ${
                          sub.enabled
                            ? 'bg-gray-500 hover:bg-gray-600 text-white'
                            : 'bg-green-500 hover:bg-green-600 text-white'
                        }`}
                      >
                        {sub.enabled ? '禁用' : '启用'}
                      </button>
                      <button
                        onClick={() => handleUpdateSubscription(sub.name)}
                        disabled={updating === sub.name}
                        className="p-2 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white rounded-lg transition-colors"
                      >
                        <RefreshCw
                          className={`w-4 h-4 ${updating === sub.name ? 'animate-spin' : ''}`}
                        />
                      </button>
                      <button
                        onClick={() => handleDeleteSubscription(sub.name)}
                        className="p-2 bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
