# 浏览器环境兼容方案

## 概述

让应用在浏览器中（`npm run dev`）也能正常操作 Redis，需要为每个 Tauri 后端命令提供浏览器端的替代实现。

## 当前状态

- 项目共 **25 个 Tauri invoke 命令**，全部通过 Rust 后端执行
- 浏览器环境下 `safeInvoke` 直接抛错 `"当前不在 Tauri 环境中"`
- 另有 2 个非 invoke 的 Tauri API（文件对话框、路径解析）
- 服务器配置存储在本地文件系统（Tauri 端）

## 核心思路

创建一个**浏览器端适配层**，在 `safeInvoke` 中根据环境分流：
- **Tauri 环境** → 走原有 `invoke` 调用 Rust 后端
- **浏览器环境** → 走浏览器适配层（WebSocket/HTTP 连接 Redis）

## 需要解决的关键问题

### 1. Redis 连接（核心）

浏览器无法直接连接 Redis（TCP 协议），需要一个中间代理。方案：

| 方案 | 说明 | 优点 | 缺点 |
|------|------|------|------|
| **A. 内嵌 WebSocket 代理** | 在 `dev` 模式下启动一个 Node.js WebSocket 服务器作为 Redis 代理 | 开发体验好，无需额外服务 | 需要维护代理代码 |
| **B. 独立代理服务** | 提供一个独立的 Redis HTTP/WebSocket 代理服务 | 解耦清晰 | 需要用户额外启动 |
| **C. 纯前端 mock** | 浏览器端用 mock 数据模拟 | 最简单 | 无法真正操作 Redis |

**推荐方案 A**：在 `vite.config.ts` 中通过插件启动一个 WebSocket 代理，开发时自动可用。

### 2. 服务器配置持久化

| Tauri 环境 | 浏览器环境 |
|-----------|-----------|
| Rust 端读写本地 JSON 文件 | `localStorage` 存储 |

### 3. 文件对话框

| Tauri 环境 | 浏览器环境 |
|-----------|-----------|
| `@tauri-apps/plugin-dialog` 的 `open()` | `<input type="file">` + `<input type="folder">` 或下载到本地 |

### 4. 废键箱持久化

| Tauri 环境 | 浏览器环境 |
|-----------|-----------|
| Rust 端读写本地 JSON 文件 | `localStorage` 存储 |

## 实施计划

### 第一步：创建 WebSocket 代理服务

**新建文件**：`src/frontend/utils/redis-proxy.ts`

- 启动一个 WebSocket 服务器（开发模式）
- 接收前端命令，转发到 Redis
- 返回结果给前端

**新建文件**：`server/ws-proxy.js`

- 独立的 Node.js WebSocket 代理服务器
- 使用 `ws` 和 `ioredis` 库
- 支持 12 个 Redis 操作命令

### 第二步：创建浏览器适配层

**新建文件**：`src/frontend/adapters/browser-adapter.ts`

为每个 Tauri 命令提供浏览器端实现：

#### Redis 操作（通过 WebSocket 代理）
- `connect` / `get_databases` / `get_keys` / `get_key_value`
- `set_key_value` / `delete_key` / `search_keys`
- `create_database` / `delete_database` / `flush_database`
- `generate_test_data`

#### 服务器配置（localStorage）
- `get_servers` / `add_server` / `edit_server` / `delete_server`
- `test_connection`（通过 WebSocket 代理测试）

#### 废键箱（localStorage）
- `move_to_trash` / `batch_move_to_trash` / `get_trash_items`
- `restore_from_trash` / `batch_restore_from_trash`
- `permanent_delete_trash` / `clear_expired_trash`

#### 导入导出（浏览器 API）
- `export_data` → `Blob` + `URL.createObjectURL` 下载
- `import_data` → `<input type="file">` 读取

### 第三步：修改 safeInvoke 分流

**修改文件**：`src/frontend/utils/tauri.ts`

```
safeInvoke(cmd, args)
  ├── Tauri 环境 → invoke(cmd, args)
  └── 浏览器环境 → browserAdapter.execute(cmd, args)
```

### 第四步：处理文件对话框

**修改文件**：`src/frontend/views/MainView.vue`

- `selectExportFolder`：浏览器环境下使用下载方式（无需选择文件夹）
- `selectImportFile`：浏览器环境下使用已有的 `fileInput` 元素

### 第五步：移除环境限制

**修改文件**：`src/frontend/views/MainView.vue`

- 移除 `onMounted` 中的 `if (isRunningInTauri.value)` 条件，让浏览器环境也加载服务器列表

## 修改文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `server/ws-proxy.js` | 新建 | WebSocket 代理服务器 |
| `src/frontend/adapters/browser-adapter.ts` | 新建 | 浏览器端命令适配层 |
| `src/frontend/utils/tauri.ts` | 修改 | safeInvoke 增加浏览器分流 |
| `src/frontend/views/MainView.vue` | 修改 | 文件对话框兼容 + 移除环境限制 |
| `package.json` | 修改 | 添加 `ws`、`ioredis` 依赖和代理启动脚本 |
| `vite.config.ts` | 修改 | 开发模式自动启动代理（可选） |

## 验证步骤

1. `npm run dev` 启动后，代理服务器自动运行
2. 浏览器中添加 Redis 服务器配置
3. 测试连接、浏览 keys、查看/编辑值
4. 测试导入导出
5. 测试废键箱功能
6. 确认 Tauri 环境下功能不受影响
