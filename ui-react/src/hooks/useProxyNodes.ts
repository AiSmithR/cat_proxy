import { useState, useEffect } from 'react';
import { api } from '@/utils/api';
import type { ProxyNode } from '@/types';

export function useProxyNodes() {
  const [nodes, setNodes] = useState<ProxyNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchNodes = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getProxyNodes();
      if (result.success && result.data) {
        setNodes(result.data);
      } else {
        setError(result.error || '获取节点失败');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchNodes();
    const interval = setInterval(fetchNodes, 10000);
    return () => clearInterval(interval);
  }, []);

  return { nodes, loading, error, refresh: fetchNodes };
}
