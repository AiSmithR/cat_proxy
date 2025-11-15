# Cat Proxy - React Frontend

基于 React + TypeScript + Vite + Tauri 构建的现代化代理管理界面。

## 技术栈

- **React 18**: 使用函数式组件和 Hooks
- **TypeScript**: 完整的类型安全
- **Vite**: 快速的开发服务器和构建工具
- **Tailwind CSS**: 实用优先的 CSS 框架
- **Recharts**: 数据可视化图表库
- **Lucide React**: 现代图标库
- **Tauri**: 轻量级桌面应用框架

## 功能特性

### 1. 仪表板
- 实时显示活跃连接数
- 总连接数统计
- 上传/下载流量统计
- 渐变色卡片设计

### 2. 控制面板
- 启动/停止代理服务
- 刷新统计数据
- 打开规则管理器
- 打开连接查看器

### 3. 代理节点管理
- 网格布局展示所有节点
- 实时延迟监控（自动每 10 秒刷新）
- 节点在线状态显示
- 根据延迟显示不同颜色
  - 绿色: < 100ms
  - 黄色: 100-300ms
  - 红色: >= 300ms

### 4. 流量趋势图
- 实时上传/下载速率曲线
- 自动每 3 秒更新
- 保留最近 20 个数据点
- 交互式工具提示

### 5. 规则编辑器（模态框）
- 查看所有路由规则
- 添加新规则
  - 规则类型: DOMAIN-SUFFIX, DOMAIN, DOMAIN-KEYWORD, IP-CIDR, GEOIP, MATCH
  - 规则目标: DIRECT, PROXY, REJECT
- 删除规则
- 保存到配置文件

### 6. 连接查看器（模态框）
- 表格展示所有活跃连接
- 显示源地址、目标地址
- 显示使用的代理
- 上传/下载流量统计
- 连接持续时间
- 自动每 2 秒刷新

### 7. 主题系统
- 支持亮色/暗色/自动模式
- 完整的暗色模式适配
- 主题偏好持久化
- 平滑过渡动画

## 项目结构

```
ui-react/
├── src/
│   ├── components/          # React 组件
│   │   ├── Header.tsx       # 应用头部
│   │   ├── Dashboard.tsx    # 仪表板卡片
│   │   ├── ControlPanel.tsx # 控制按钮
│   │   ├── ProxyNodes.tsx   # 代理节点网格
│   │   ├── TrafficChart.tsx # 流量图表
│   │   ├── RulesEditor.tsx  # 规则编辑器（模态框）
│   │   └── ConnectionsViewer.tsx # 连接查看器（模态框）
│   ├── contexts/            # React Context
│   │   └── AppContext.tsx   # 全局应用状态
│   ├── hooks/               # 自定义 Hooks
│   │   └── useProxyNodes.ts # 代理节点数据
│   ├── styles/              # 样式文件
│   │   └── index.css        # Tailwind + 自定义样式
│   ├── types/               # TypeScript 类型定义
│   │   └── index.ts         # 所有类型定义
│   ├── utils/               # 工具函数
│   │   └── api.ts           # Tauri API 调用封装
│   ├── App.tsx              # 主应用组件
│   └── main.tsx             # React 入口
├── index.html               # HTML 入口
├── package.json             # NPM 依赖
├── tsconfig.json            # TypeScript 配置
├── vite.config.ts           # Vite 配置
├── tailwind.config.js       # Tailwind CSS 配置
└── postcss.config.js        # PostCSS 配置
```

## 开发指南

### 安装依赖

```bash
npm install
```

### 开发模式

从项目根目录运行（会自动启动 Vite dev server）:

```bash
cargo tauri dev
```

或者分别启动前端和后端：

```bash
# 终端 1: 启动 Vite dev server
npm run dev

# 终端 2: 启动 Tauri（从项目根目录）
cargo tauri dev
```

### 生产构建

从项目根目录运行:

```bash
cargo tauri build
```

### 类型检查

```bash
npm run build  # 会自动运行 tsc
# 或者单独运行
npx tsc --noEmit
```

## 状态管理

使用 React Context API 管理全局状态：

- **AppContext**: 主题、仪表板统计、系统信息
- 自动每 3 秒刷新统计数据
- 主题偏好存储在 localStorage

## API 调用

所有 Tauri 命令调用都封装在 `utils/api.ts` 中：

- `getDashboardStats()`: 获取仪表板统计
- `getProxyNodes()`: 获取代理节点列表
- `getRules()`: 获取路由规则
- `getConnections()`: 获取活跃连接
- `startProxy()`: 启动代理
- `stopProxy()`: 停止代理
- `addRule()`: 添加规则
- `deleteRule()`: 删除规则
- `saveConfig()`: 保存配置

## 样式系统

### 自定义 CSS 类

- `.glass`: 毛玻璃效果
- `.card`: 卡片样式
- `.btn`, `.btn-primary`, `.btn-danger`, `.btn-secondary`: 按钮样式
- `.modal-overlay`, `.modal-content`: 模态框样式
- `.input`, `.select`: 表单控件样式

### 主题颜色

在 `tailwind.config.js` 中定义：

- Primary: `#667eea` (紫蓝色)
- Secondary: `#764ba2` (紫色)

## 开发注意事项

1. **TypeScript 严格模式**: 所有代码必须通过类型检查
2. **响应式设计**: 使用 Tailwind 断点 (md:, lg:, xl:)
3. **暗色模式**: 所有组件都支持 dark: 前缀
4. **性能优化**:
   - 合理使用 useEffect 清理定时器
   - 避免不必要的重渲染
5. **错误处理**: 所有 API 调用都有错误处理

## 待优化项

- [ ] 添加代码分割（dynamic import）以减小 bundle 大小
- [ ] 添加错误边界（Error Boundary）
- [ ] 添加加载骨架屏
- [ ] 优化图表性能
- [ ] 添加单元测试
