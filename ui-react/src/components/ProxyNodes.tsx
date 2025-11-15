import { Server, Wifi, WifiOff, Radio } from 'lucide-react';
import { useProxyNodes } from '@/hooks/useProxyNodes';

export default function ProxyNodes() {
  const { nodes, loading, error } = useProxyNodes();

  const getLatencyBg = (latency: number | null): string => {
    if (latency === null) return 'from-gray-400 to-gray-500';
    if (latency < 100) return 'from-green-500 to-green-600';
    if (latency < 300) return 'from-yellow-500 to-yellow-600';
    return 'from-red-500 to-red-600';
  };

  if (error) {
    return (
      <div className="glass card p-6 mb-6">
        <h2 className="text-xl font-bold mb-4">代理节点</h2>
        <div className="text-center py-8 text-red-500">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="glass card p-6 mb-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold">代理节点</h2>
        {loading && (
          <span className="text-sm text-gray-500">刷新中...</span>
        )}
      </div>

      {nodes.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          暂无代理节点
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {nodes.map((node) => (
            <div
              key={node.name}
              className={`card p-4 bg-gradient-to-br ${getLatencyBg(node.latency)} text-white hover:scale-105`}
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-2">
                  {node.latency === null ? (
                    <WifiOff size={20} className="opacity-80" />
                  ) : (
                    <Wifi size={20} className="opacity-80" />
                  )}
                  <h3 className="font-semibold text-sm">{node.name}</h3>
                </div>
                {node.is_alive && (
                  <Radio size={16} className="opacity-80 animate-pulse" />
                )}
              </div>

              <div className="space-y-2">
                <div className="flex items-center gap-2 text-xs opacity-90">
                  <Server size={14} />
                  <span className="font-mono truncate">{node.server}</span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-xs opacity-80">延迟</span>
                  <span className="text-lg font-bold">
                    {node.latency !== null ? `${node.latency}ms` : 'N/A'}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-xs opacity-80">类型</span>
                  <span className="text-xs font-semibold bg-white/20 px-2 py-1 rounded">
                    {node.proxy_type}
                  </span>
                </div>

                {!node.is_alive && (
                  <div className="mt-2 text-xs bg-red-900/50 px-2 py-1 rounded text-center">
                    节点离线
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
