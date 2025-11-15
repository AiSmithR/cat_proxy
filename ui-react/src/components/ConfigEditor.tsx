import { useEffect, useState } from 'react';
import { X, Save, Plus, Trash2, AlertCircle, Settings } from 'lucide-react';
import { api } from '@/utils/api';
import type { ProxyNode } from '@/types';

interface Config {
  port: number;
  socks_port: number;
  allow_lan: boolean;
  mode: string;
  log_level: string;
  proxies: ProxyNode[];
}

interface ConfigEditorProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ConfigEditor({ isOpen, onClose }: ConfigEditorProps) {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAddProxy, setShowAddProxy] = useState(false);

  // 新代理表单
  const [newProxy, setNewProxy] = useState({
    name: '',
    proxy_type: 'ss',
    server: '',
    port: 443,
    password: '',
    cipher: 'aes-256-gcm',
  });

  useEffect(() => {
    if (isOpen) {
      loadConfig();
    }
  }, [isOpen]);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getConfig();
      if (result.success && result.data) {
        setConfig({
          port: result.data.port,
          socks_port: result.data.socks_port || 7891,
          allow_lan: result.data.allow_lan || false,
          mode: result.data.mode || 'Rule',
          log_level: result.data.log_level || 'info',
          proxies: result.data.proxies || [],
        });
      } else {
        setError(result.error || '加载配置失败');
      }
    } catch (err) {
      setError('加载配置失败: ' + String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;

    setLoading(true);
    setError(null);
    try {
      const result = await api.updateConfig(config);
      if (result.success) {
        // 保存到文件
        const saveResult = await api.saveConfig();
        if (saveResult.success) {
          alert('配置已保存');
          onClose();
        } else {
          setError(saveResult.error || '保存配置到文件失败');
        }
      } else {
        setError(result.error || '更新配置失败');
      }
    } catch (err) {
      setError('保存配置失败: ' + String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleAddProxy = () => {
    if (!config) return;
    if (!newProxy.name || !newProxy.server) {
      setError('请填写代理名称和服务器地址');
      return;
    }

    const proxy: ProxyNode = {
      name: newProxy.name,
      proxy_type: newProxy.proxy_type,
      server: newProxy.server,
      port: newProxy.port,
      is_healthy: false,
      is_alive: false,
      latency: null,
    };

    setConfig({
      ...config,
      proxies: [...config.proxies, proxy],
    });

    setNewProxy({
      name: '',
      proxy_type: 'ss',
      server: '',
      port: 443,
      password: '',
      cipher: 'aes-256-gcm',
    });
    setShowAddProxy(false);
  };

  const handleDeleteProxy = (index: number) => {
    if (!config) return;
    if (!confirm('确定要删除此代理节点吗？')) return;

    const updatedProxies = [...config.proxies];
    updatedProxies.splice(index, 1);
    setConfig({
      ...config,
      proxies: updatedProxies,
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2">
            <Settings className="w-6 h-6 text-blue-500" />
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              配置编辑器
            </h2>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleSaveConfig}
              disabled={loading || !config}
              className="px-4 py-2 bg-green-500 hover:bg-green-600 disabled:bg-gray-400 text-white rounded-lg flex items-center gap-2 transition-colors"
            >
              <Save className="w-4 h-4" />
              保存配置
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
        <div className="flex-1 overflow-y-auto p-4">
          {error && (
            <div className="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg flex items-center gap-2">
              <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-300" />
              <span className="text-red-800 dark:text-red-200">{error}</span>
            </div>
          )}

          {loading && !config ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              加载中...
            </div>
          ) : config ? (
            <div className="space-y-6">
              {/* 基本设置 */}
              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <h3 className="text-lg font-medium mb-4 text-gray-900 dark:text-white">
                  基本设置
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      HTTP 端口
                    </label>
                    <input
                      type="number"
                      value={config.port}
                      onChange={(e) =>
                        setConfig({ ...config, port: parseInt(e.target.value) || 7890 })
                      }
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      SOCKS5 端口
                    </label>
                    <input
                      type="number"
                      value={config.socks_port}
                      onChange={(e) =>
                        setConfig({ ...config, socks_port: parseInt(e.target.value) || 7891 })
                      }
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      代理模式
                    </label>
                    <select
                      value={config.mode}
                      onChange={(e) => setConfig({ ...config, mode: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    >
                      <option value="Rule">规则模式</option>
                      <option value="Global">全局模式</option>
                      <option value="Direct">直连模式</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      日志级别
                    </label>
                    <select
                      value={config.log_level}
                      onChange={(e) => setConfig({ ...config, log_level: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    >
                      <option value="trace">Trace</option>
                      <option value="debug">Debug</option>
                      <option value="info">Info</option>
                      <option value="warn">Warn</option>
                      <option value="error">Error</option>
                    </select>
                  </div>
                  <div className="flex items-center">
                    <input
                      type="checkbox"
                      checked={config.allow_lan}
                      onChange={(e) => setConfig({ ...config, allow_lan: e.target.checked })}
                      className="w-4 h-4 text-blue-600 rounded"
                    />
                    <label className="ml-2 text-sm text-gray-700 dark:text-gray-300">
                      允许局域网连接
                    </label>
                  </div>
                </div>
              </div>

              {/* 代理节点 */}
              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                    代理节点 ({config.proxies.length})
                  </h3>
                  <button
                    onClick={() => setShowAddProxy(true)}
                    className="px-3 py-1.5 bg-blue-500 hover:bg-blue-600 text-white rounded-lg text-sm flex items-center gap-1 transition-colors"
                  >
                    <Plus className="w-4 h-4" />
                    添加节点
                  </button>
                </div>

                {/* 添加节点表单 */}
                {showAddProxy && (
                  <div className="mb-4 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-600">
                    <h4 className="text-md font-medium mb-3 text-gray-900 dark:text-white">
                      添加新节点
                    </h4>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          节点名称
                        </label>
                        <input
                          type="text"
                          value={newProxy.name}
                          onChange={(e) => setNewProxy({ ...newProxy, name: e.target.value })}
                          placeholder="例如：香港节点"
                          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          协议类型
                        </label>
                        <select
                          value={newProxy.proxy_type}
                          onChange={(e) => setNewProxy({ ...newProxy, proxy_type: e.target.value })}
                          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        >
                          <option value="ss">Shadowsocks</option>
                          <option value="vmess">VMess</option>
                          <option value="trojan">Trojan</option>
                          <option value="socks5">SOCKS5</option>
                        </select>
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          服务器地址
                        </label>
                        <input
                          type="text"
                          value={newProxy.server}
                          onChange={(e) => setNewProxy({ ...newProxy, server: e.target.value })}
                          placeholder="例如：example.com"
                          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          端口
                        </label>
                        <input
                          type="number"
                          value={newProxy.port}
                          onChange={(e) =>
                            setNewProxy({ ...newProxy, port: parseInt(e.target.value) || 443 })
                          }
                          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        />
                      </div>
                    </div>
                    <div className="flex gap-2 mt-3">
                      <button
                        onClick={handleAddProxy}
                        className="px-4 py-2 bg-green-500 hover:bg-green-600 text-white rounded-lg transition-colors"
                      >
                        确定
                      </button>
                      <button
                        onClick={() => setShowAddProxy(false)}
                        className="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg transition-colors"
                      >
                        取消
                      </button>
                    </div>
                  </div>
                )}

                {/* 节点列表 */}
                <div className="space-y-2">
                  {config.proxies.length === 0 ? (
                    <div className="text-center py-4 text-gray-500 dark:text-gray-400">
                      暂无代理节点，点击"添加节点"开始
                    </div>
                  ) : (
                    config.proxies.map((proxy, index) => (
                      <div
                        key={index}
                        className="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-600 flex items-center justify-between"
                      >
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="font-medium text-gray-900 dark:text-white">
                              {proxy.name}
                            </span>
                            <span className="px-2 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded-full">
                              {proxy.proxy_type}
                            </span>
                          </div>
                          <div className="text-sm text-gray-600 dark:text-gray-400">
                            {proxy.server}:{proxy.port}
                          </div>
                        </div>
                        <button
                          onClick={() => handleDeleteProxy(index)}
                          className="p-2 bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
