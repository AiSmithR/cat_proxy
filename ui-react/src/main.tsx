/**
 * Cat Proxy 前端应用入口文件
 *
 * 这是 React 应用的启动点，负责：
 * 1. 初始化 React 根节点
 * 2. 启用严格模式（StrictMode）
 * 3. 渲染主应用组件
 * 4. 加载全局样式
 *
 * @module main
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles/index.css';

/**
 * 应用初始化
 *
 * 流程：
 * 1. 获取 HTML 中 id 为 'root' 的 DOM 元素
 * 2. 创建 React 18 的根节点
 * 3. 在严格模式下渲染 App 组件
 *
 * React.StrictMode 的作用：
 * - 检测不安全的生命周期方法
 * - 警告使用过时的 API
 * - 检测意外的副作用
 * - 确保可复用状态
 *
 * 注意：StrictMode 仅在开发模式下有效，不影响生产构建
 */
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
