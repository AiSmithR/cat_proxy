import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import type { DashboardStats, SystemInfo, Theme } from '@/types';
import { api } from '@/utils/api';

interface AppContextType {
  // 主题
  theme: Theme;
  setTheme: (theme: Theme) => void;
  isDark: boolean;

  // 仪表板数据
  stats: DashboardStats | null;
  systemInfo: SystemInfo | null;
  refreshStats: () => Promise<void>;

  // 加载状态
  loading: boolean;
  error: string | null;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = localStorage.getItem('theme') as Theme;
    return saved || 'light';
  });

  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isDark = theme === 'dark' || (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);

  // 更新主题
  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme);
    localStorage.setItem('theme', newTheme);
  };

  // 刷新统计数据
  const refreshStats = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getDashboardStats();
      if (result.success && result.data) {
        setStats(result.data);
      } else {
        setError(result.error || '获取统计数据失败');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  // 获取系统信息
  const fetchSystemInfo = async () => {
    const result = await api.getSystemInfo();
    if (result.success && result.data) {
      setSystemInfo(result.data);
    }
  };

  // 初始化
  useEffect(() => {
    refreshStats();
    fetchSystemInfo();

    // 定期刷新统计
    const interval = setInterval(refreshStats, 3000);
    return () => clearInterval(interval);
  }, []);

  // 应用主题
  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [isDark]);

  const value: AppContextType = {
    theme,
    setTheme,
    isDark,
    stats,
    systemInfo,
    refreshStats,
    loading,
    error,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp() {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within AppProvider');
  }
  return context;
}
