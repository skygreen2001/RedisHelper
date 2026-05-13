# 新建窗口、新建标签功能实现方案

## 概述

以连接为基础，为 Redis 小助手添加**应用内标签页**和**操作系统多窗口**功能。每个标签/窗口都是完整独立会话。同时在 macOS 顶部菜单新增"文件"菜单，并在"窗口"菜单中添加"显示标签栏"子菜单。

## 当前状态

- 单页面应用，无 vue-router，`MainView` 是唯一主视图
- `MainView` 有 42 个 ref 状态变量，全部是单例
- `redisStore` state 仅有 `isConnected`，操作通过 actions 封装 IPC 调用
- macOS 菜单：Redis小助手、窗口、帮助（无"文件"菜单）
- Tauri 仅配置单个窗口，无多窗口 API 使用
- capabilities 无窗口创建权限

## 设计方案

### 核心思路：会话状态外提

将 `MainView` 中的会话相关状态（selectedServer、selectedDb、databases、keys 等）提取到独立的 `Session` 类中。标签页/窗口各自持有独立的 Session 实例，实现会话隔离。

```
Session 类
├── id: string
├── title: string
├── selectedServer / selectedDb / databases / keys
├── selectedKey / keyValue / keyType
├── searchPattern / sortOrder / caseSensitive
├── isTrashView / trashSelectedIds
└── 方法：connect / loadDatabases / loadKeys / ...
```

### 标签页方案

在菜单栏下方添加标签栏组件，管理多个 Session 实例。

### 多窗口方案

使用 Tauri `WebviewWindow` API 创建新窗口。每个窗口天然拥有独立的 JS 运行时和 Pinia store，天然隔离。通过 URL query 参数 `?window=true` 标识独立窗口模式（隐藏标签栏）。

## 实施计划

### 第一步：创建 Session 会话类

**新建文件**：`src/frontend/sessions/Session.ts`

提取 `MainView` 中的会话状态为独立类：

```ts
class Session {
  id: string
  title: string
  selectedServer: any
  selectedDb: number | null
  databases: Array<[number, number]>
  keys: string[]
  selectedKey: string
  keyValue: string
  keyType: string
  searchPattern: string
  sortOrder: 'none' | 'asc' | 'desc'
  isTrashView: boolean
  // ... 其他会话状态

  async connect(server) { ... }
  async loadDatabases() { ... }
  async loadKeys() { ... }
  async getKeyValue(key) { ... }
  // ... 其他操作方法（复用 redisStore/trashStore 的 safeInvoke 调用）
}
```

### 第二步：创建标签管理器

**新建文件**：`src/frontend/sessions/SessionManager.ts`

```ts
class SessionManager {
  sessions: Session[]
  activeSessionId: string

  createSession(): Session
  closeSession(id: string): void
  switchSession(id: string): void
  getActiveSession(): Session
}
```

导出单例供全局使用。

### 第三步：创建标签栏组件

**新建文件**：`src/frontend/components/TabBar.vue`

- 水平标签栏，显示在菜单栏下方
- 每个标签显示 `服务器名 - db编号`
- 支持：新建标签（+按钮）、关闭标签（×按钮）、点击切换
- 标签过多时水平滚动
- 可通过"显示标签栏"菜单项控制显示/隐藏

### 第四步：重构 MainView 使用 Session

**修改文件**：`src/frontend/views/MainView.vue`

- 将 42 个会话相关 ref 替换为从 `SessionManager.getActiveSession()` 读取
- 所有操作函数改为调用 session 实例的方法
- 在 template 中插入 `<TabBar />` 组件
- 添加 `showTabBar` ref 控制标签栏显示

### 第五步：macOS 菜单更新

**修改文件**：`src-tauri/src/main.rs`

新增"文件"菜单，更新"窗口"菜单：

```
文件
├── 新建标签    Ctrl+T
├── 新建窗口    Ctrl+N
├── ---
├── 关闭标签    Ctrl+W
└── 关闭窗口    Ctrl+Shift+W

窗口
├── 最小化
├── 关闭窗口
├── ---
├── 全屏
└── 显示标签栏  Ctrl+Shift+T    (勾选/取消)
```

菜单项通过 Tauri 事件机制通知前端执行对应操作。

### 第六步：多窗口支持

**修改文件**：
- `src-tauri/src/main.rs` — 注册 `create_new_window` 命令
- `src-tauri/capabilities/main.json` — 添加 `core:window:allow-create` 权限
- `src/frontend/views/MainView.vue` — 监听菜单事件，调用窗口创建

**窗口隔离策略**：
- 新窗口 URL：`index.html?window=true`
- `window=true` 时隐藏标签栏，显示完整独立界面
- 每个窗口有独立的 SessionManager 实例（天然隔离）

### 第七步：快捷键绑定

**修改文件**：`src/frontend/views/MainView.vue`

| 快捷键 | 功能 |
|--------|------|
| Ctrl+T | 新建标签 |
| Ctrl+N | 新建窗口 |
| Ctrl+W | 关闭当前标签 |
| Ctrl+Shift+W | 关闭窗口 |
| Ctrl+Shift+T | 切换标签栏显示 |

## 修改文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/frontend/sessions/Session.ts` | 新建 | 会话类，封装独立会话的所有状态和操作 |
| `src/frontend/sessions/SessionManager.ts` | 新建 | 标签管理器，维护多个 Session 实例 |
| `src/frontend/components/TabBar.vue` | 新建 | 标签栏 UI 组件 |
| `src/frontend/views/MainView.vue` | 修改 | 重构为使用 Session，插入 TabBar，监听菜单事件 |
| `src-tauri/src/main.rs` | 修改 | 新增"文件"菜单 + "显示标签栏" + create_new_window 命令 |
| `src-tauri/capabilities/main.json` | 修改 | 添加窗口创建权限 |

## 验证步骤

1. **标签页**：新建标签 → 切换标签 → 状态独立 → 关闭标签
2. **多窗口**：新建窗口 → 独立操作 → 关闭窗口
3. **macOS 菜单**：文件菜单四个子项均可用，窗口菜单"显示标签栏"可切换
4. **快捷键**：Ctrl+T/N/W/Shift+W/Shift+T 均正常
5. **浏览器兼容**：`npm run dev` 下标签页正常，新建窗口使用 window.open
6. **Tauri 兼容**：`npm run tauri:dev` 下标签页和多窗口均正常
