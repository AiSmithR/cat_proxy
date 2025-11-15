import { useState, useEffect } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { api, formatBytes } from '@/utils/api';

interface TrafficData {
  time: string;
  upload: number;
  download: number;
}

export default function TrafficChart() {
  const [data, setData] = useState<TrafficData[]>([]);
  const [lastUpload, setLastUpload] = useState(0);
  const [lastDownload, setLastDownload] = useState(0);

  useEffect(() => {
    const updateTraffic = async () => {
      const result = await api.getDashboardStats();
      if (result.success && result.data) {
        const now = new Date();
        const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;

        const uploadDelta = Math.max(0, result.data.total_upload - lastUpload);
        const downloadDelta = Math.max(0, result.data.total_download - lastDownload);

        setData((prev) => {
          const newData = [
            ...prev,
            {
              time: timeStr,
              upload: uploadDelta,
              download: downloadDelta,
            },
          ];
          // Keep only last 20 data points
          return newData.slice(-20);
        });

        setLastUpload(result.data.total_upload);
        setLastDownload(result.data.total_download);
      }
    };

    updateTraffic();
    const interval = setInterval(updateTraffic, 3000);
    return () => clearInterval(interval);
  }, [lastUpload, lastDownload]);

  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-3 rounded-lg shadow-lg">
          <p className="text-sm font-semibold mb-1">{payload[0].payload.time}</p>
          <p className="text-sm text-green-600 dark:text-green-400">
            上传: {formatBytes(payload[0].value)}
          </p>
          <p className="text-sm text-orange-600 dark:text-orange-400">
            下载: {formatBytes(payload[1].value)}
          </p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="glass card p-6 mb-6">
      <h2 className="text-xl font-bold mb-4">流量趋势</h2>
      <div className="h-64">
        {data.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            等待数据中...
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-gray-200 dark:stroke-gray-700" />
              <XAxis
                dataKey="time"
                className="text-xs"
                tick={{ fill: 'currentColor' }}
              />
              <YAxis
                className="text-xs"
                tick={{ fill: 'currentColor' }}
                tickFormatter={(value) => formatBytes(value)}
              />
              <Tooltip content={<CustomTooltip />} />
              <Legend />
              <Line
                type="monotone"
                dataKey="upload"
                stroke="#10b981"
                strokeWidth={2}
                name="上传"
                dot={false}
                isAnimationActive={false}
              />
              <Line
                type="monotone"
                dataKey="download"
                stroke="#f97316"
                strokeWidth={2}
                name="下载"
                dot={false}
                isAnimationActive={false}
              />
            </LineChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
