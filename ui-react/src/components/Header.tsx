import { Moon, Sun } from 'lucide-react';
import { useApp } from '@/contexts/AppContext';

export default function Header() {
  const { setTheme, isDark, stats } = useApp();

  const toggleTheme = () => {
    setTheme(isDark ? 'light' : 'dark');
  };

  return (
    <header className="glass card p-6 mb-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h1 className="text-3xl font-bold bg-gradient-to-r from-primary-500 to-secondary-500 bg-clip-text text-transparent">
            🐱 Cat Proxy
          </h1>
          <div
            className={`px-3 py-1 rounded-full text-sm font-semibold ${
              stats?.is_running
                ? 'bg-green-500 text-white'
                : 'bg-red-500 text-white'
            }`}
          >
            {stats?.is_running ? '运行中' : '已停止'}
          </div>
        </div>

        <button
          onClick={toggleTheme}
          className="btn btn-secondary p-3 rounded-full"
          aria-label="切换主题"
        >
          {isDark ? <Sun size={20} /> : <Moon size={20} />}
        </button>
      </div>
    </header>
  );
}
