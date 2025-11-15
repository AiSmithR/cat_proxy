/**
 * Cat Proxy 主应用组件
 *
 * 这是应用的根组件，负责：
 * - 布局管理和页面结构
 * - 全局状态管理（通过 Context Providers）
 * - 错误边界处理
 * - 模态框状态管理
 * - 组件组合和渲染
 *
 * 架构层次：
 * App (根组件)
 *  └─ ErrorBoundary (错误边界)
 *      └─ AppProvider (应用状态)
 *          └─ ToastProvider (通知提示)
 *              └─ AppContent (主要内容)
 *
 * @module App
 */

import { useState } from 'react';
import { AppProvider } from '@/contexts/AppContext';
import { ToastProvider } from '@/contexts/ToastContext';
import ErrorBoundary from '@/components/ErrorBoundary';
import Header from '@/components/Header';
import Dashboard from '@/components/Dashboard';
import ControlPanel from '@/components/ControlPanel';
import ProxyNodes from '@/components/ProxyNodes';
import RulesEditor from '@/components/RulesEditor';
import ConnectionsViewer from '@/components/ConnectionsViewer';
import SubscriptionManager from '@/components/SubscriptionManager';
import LogsViewer from '@/components/LogsViewer';
import ConfigEditor from '@/components/ConfigEditor';
import ConnectionStats from '@/components/ConnectionStats';

/**
 * 应用主内容组件
 *
 * 包含所有主要的 UI 组件和模态框
 *
 * 组件结构：
 * - Header: 顶部标题栏
 * - Dashboard: 仪表板（运行状态、快速操作）
 * - ControlPanel: 控制面板（功能按钮组）
 * - ConnectionStats: 连接统计和流量趋势图
 * - ProxyNodes: 代理节点列表
 * - 5个模态框组件（规则、连接、订阅、日志、配置）
 *
 * 模态框管理：
 * - 使用 useState 管理每个模态框的打开/关闭状态
 * - 通过 ControlPanel 的回调函数打开相应的模态框
 * - 通过模态框的 onClose 回调关闭模态框
 */
function AppContent() {
  // ========================================
  // 模态框状态管理
  // ========================================

  /** 规则编辑器模态框打开状态 */
  const [isRulesOpen, setIsRulesOpen] = useState(false);

  /** 连接查看器模态框打开状态 */
  const [isConnectionsOpen, setIsConnectionsOpen] = useState(false);

  /** 订阅管理器模态框打开状态 */
  const [isSubscriptionsOpen, setIsSubscriptionsOpen] = useState(false);

  /** 日志查看器模态框打开状态 */
  const [isLogsOpen, setIsLogsOpen] = useState(false);

  /** 配置编辑器模态框打开状态 */
  const [isConfigOpen, setIsConfigOpen] = useState(false);

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 transition-colors duration-200">
      <div className="container mx-auto px-4 py-6 max-w-7xl">
        {/* 顶部标题栏 */}
        <Header />

        {/* 仪表板 - 显示运行状态和快速操作 */}
        <Dashboard />

        {/* 控制面板 - 提供各种功能按钮 */}
        <ControlPanel
          onOpenRules={() => setIsRulesOpen(true)}              // 打开规则编辑器
          onOpenConnections={() => setIsConnectionsOpen(true)}  // 打开连接查看器
          onOpenSubscriptions={() => setIsSubscriptionsOpen(true)} // 打开订阅管理器
          onOpenLogs={() => setIsLogsOpen(true)}                // 打开日志查看器
          onOpenConfig={() => setIsConfigOpen(true)}            // 打开配置编辑器
        />

        {/* 连接统计和流量趋势图 */}
        <ConnectionStats className="mb-6" />

        {/* 代理节点列表 */}
        <ProxyNodes />

        {/* ========================================
            模态框组件
            ======================================== */}

        {/* 规则编辑器 - 管理路由规则 */}
        <RulesEditor
          isOpen={isRulesOpen}
          onClose={() => setIsRulesOpen(false)}
        />

        {/* 连接查看器 - 查看活跃连接 */}
        <ConnectionsViewer
          isOpen={isConnectionsOpen}
          onClose={() => setIsConnectionsOpen(false)}
        />

        {/* 订阅管理器 - 管理代理订阅 */}
        <SubscriptionManager
          isOpen={isSubscriptionsOpen}
          onClose={() => setIsSubscriptionsOpen(false)}
        />

        {/* 日志查看器 - 查看应用日志 */}
        <LogsViewer
          isOpen={isLogsOpen}
          onClose={() => setIsLogsOpen(false)}
        />

        {/* 配置编辑器 - 编辑应用配置 */}
        <ConfigEditor
          isOpen={isConfigOpen}
          onClose={() => setIsConfigOpen(false)}
        />
      </div>
    </div>
  );
}

/**
 * 应用根组件
 *
 * 提供应用所需的所有 Context Providers 和错误处理
 *
 * Provider 层次结构（从外到内）：
 * 1. ErrorBoundary - 捕获和显示运行时错误
 * 2. AppProvider - 提供全局应用状态
 * 3. ToastProvider - 提供通知提示功能
 * 4. AppContent - 实际的应用内容
 *
 * 设计原则：
 * - 错误边界在最外层，确保任何错误都能被捕获
 * - AppProvider 提供全局状态（配置、代理状态等）
 * - ToastProvider 提供用户反馈机制
 * - 内容组件专注于 UI 渲染和交互
 */
export default function App() {
  return (
    <ErrorBoundary>
      <AppProvider>
        <ToastProvider>
          <AppContent />
        </ToastProvider>
      </AppProvider>
    </ErrorBoundary>
  );
}
