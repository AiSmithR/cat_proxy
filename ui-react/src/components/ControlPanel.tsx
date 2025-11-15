import { useState } from 'react';
import { Play, Square, RefreshCw, FileText, Link, Rss, FileJson, Settings } from 'lucide-react';
import { useApp } from '@/contexts/AppContext';
import { useToast } from '@/contexts/ToastContext';
import { api } from '@/utils/api';

interface ControlPanelProps {
  onOpenRules: () => void;
  onOpenConnections: () => void;
  onOpenSubscriptions: () => void;
  onOpenLogs: () => void;
  onOpenConfig: () => void;
}

export default function ControlPanel({
  onOpenRules,
  onOpenConnections,
  onOpenSubscriptions,
  onOpenLogs,
  onOpenConfig
}: ControlPanelProps) {
  const { stats, refreshStats } = useApp();
  const toast = useToast();
  const [loading, setLoading] = useState(false);

  const handleStart = async () => {
    setLoading(true);
    const result = await api.startProxy();
    setLoading(false);
    if (result.success) {
      await refreshStats();
      toast.success('代理已启动');
    } else {
      toast.error(result.error || '启动失败');
    }
  };

  const handleStop = async () => {
    setLoading(true);
    const result = await api.stopProxy();
    setLoading(false);
    if (result.success) {
      await refreshStats();
      toast.success('代理已停止');
    } else {
      toast.error(result.error || '停止失败');
    }
  };

  return (
    <div className="glass card p-6 mb-6">
      <div className="flex flex-wrap gap-3">
        <button
          onClick={handleStart}
          disabled={loading || stats?.is_running}
          className="btn btn-primary flex items-center gap-2"
        >
          <Play size={18} />
          启动代理
        </button>

        <button
          onClick={handleStop}
          disabled={loading || !stats?.is_running}
          className="btn btn-danger flex items-center gap-2"
        >
          <Square size={18} />
          停止代理
        </button>

        <button
          onClick={refreshStats}
          disabled={loading}
          className="btn btn-secondary flex items-center gap-2"
        >
          <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
          刷新数据
        </button>

        <button
          onClick={onOpenRules}
          className="btn btn-secondary flex items-center gap-2"
        >
          <FileText size={18} />
          规则管理
        </button>

        <button
          onClick={onOpenConnections}
          className="btn btn-secondary flex items-center gap-2"
        >
          <Link size={18} />
          连接查看
        </button>

        <button
          onClick={onOpenSubscriptions}
          className="btn btn-secondary flex items-center gap-2"
        >
          <Rss size={18} />
          订阅管理
        </button>

        <button
          onClick={onOpenLogs}
          className="btn btn-secondary flex items-center gap-2"
        >
          <FileJson size={18} />
          日志查看
        </button>

        <button
          onClick={onOpenConfig}
          className="btn btn-secondary flex items-center gap-2"
        >
          <Settings size={18} />
          配置编辑
        </button>
      </div>
    </div>
  );
}
