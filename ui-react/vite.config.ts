import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // 防止 vite 在 Tauri 开发时出现问题
  clearScreen: false,

  // Tauri 使用固定端口
  server: {
    port: 1420,
    strictPort: true,
  },

  // 环境变量前缀
  envPrefix: ['VITE_', 'TAURI_'],

  // 构建配置
  build: {
    // Tauri 使用 Chromium，不需要兼容旧浏览器
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // 不使用 minify 以便调试
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // 生产环境产生 sourcemap
    sourcemap: !!process.env.TAURI_DEBUG,
  },

  // 路径别名
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
