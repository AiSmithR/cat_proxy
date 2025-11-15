import { Activity, Globe, ArrowUp, ArrowDown } from 'lucide-react';
import { useApp } from '@/contexts/AppContext';
import { formatBytes } from '@/utils/api';

export default function Dashboard() {
  const { stats } = useApp();

  const cards = [
    {
      icon: <Activity className="w-8 h-8" />,
      title: '活跃连接',
      value: stats?.active_connections || 0,
      unit: '个连接',
      gradient: 'from-blue-500 to-blue-600',
    },
    {
      icon: <Globe className="w-8 h-8" />,
      title: '总连接数',
      value: stats?.total_connections || 0,
      unit: '累计',
      gradient: 'from-purple-500 to-purple-600',
    },
    {
      icon: <ArrowUp className="w-8 h-8" />,
      title: '上传流量',
      value: formatBytes(stats?.total_upload || 0),
      unit: '总上传',
      gradient: 'from-green-500 to-green-600',
    },
    {
      icon: <ArrowDown className="w-8 h-8" />,
      title: '下载流量',
      value: formatBytes(stats?.total_download || 0),
      unit: '总下载',
      gradient: 'from-orange-500 to-orange-600',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-6">
      {cards.map((card, index) => (
        <div
          key={index}
          className={`card p-6 bg-gradient-to-br ${card.gradient} text-white hover:scale-105`}
        >
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-medium opacity-90">{card.title}</h3>
            {card.icon}
          </div>
          <div className="text-3xl font-bold mb-1">{card.value}</div>
          <div className="text-xs opacity-80">{card.unit}</div>
        </div>
      ))}
    </div>
  );
}
