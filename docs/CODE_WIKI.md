# Redis 小助手 — 代码 Wiki

> 本文档为当前版本（Tauri 2.0 + Vue 3 + Rust）的代码结构说明。
> 旧版 PHP 版本文档见 [CODE_WIKI.old.md](CODE_WIKI.old.md)（如有）。

---

## 1. 项目概述

| 属性 | 值 |
|------|-----|
| 项目名称 | Redis小助手 (redis-helper) |
| 版本 | 1.3.0 |
| 技术栈 | Tauri 2.0 + Vue 3 + TypeScript + Pinia + Element Plus + Rust |
| 定位 | 跨平台 Redis 管理工具（支持桌面端和Web浏览器） |
| 前后端通信 | Tauri Commands（invoke），共 50 个命令 |

---

## 2. 项目目录结构

```
redis-helper/
├── src/frontend/                    # 前端代码
│   ├── main.ts                      # 应用入口
│   ├── App.vue                      # 根组件
│   ├── views/                       # 页面视图
│   │   ├── MainView.vue             # 主界面（核心，约 2600 行）
│   │   ├── HomeView.vue             # 监控面板页面
│   │   └── ServerConfigView.vue     # 服务器配置管理
│   ├── components/                  # 公共组件
│   │   ├── TabBar.vue               # 标签栏组件（多会话标签切换）
│   │   ├── MemoryDialog.vue         # 内存分析对话框
│   │   └── LogDialog.vue            # 慢日志对话框
│   ├── sessions/                    # 会话管理
│   │   ├── Session.ts               # 单个 Redis 会话封装
│   │   └── SessionManager.ts        # 多会话管理器
│   ├── stores/                      # Pinia 状态管理
│   │   ├── redisStore.ts            # Redis 操作
│   │   ├── serverStore.ts           # 服务器配置
│   │   ├── trashStore.ts            # 废键箱
│   │   ├── configStore.ts           # 应用配置状态
│   │   ├── logStore.ts              # 日志状态管理
│   │   └── auditStore.ts            # 审计日志状态管理
│   ├── server/                      # Web 服务器（浏览器模式）
│   │   └── proxy.ts                 # 代理服务器，转发请求到 Tauri 后端
│   └── help/                        # 帮助资源
│       └── ...                      # 帮助文档/图片等资源
├── src-tauri/                       # Rust 后端代码
│   ├── src/
│   │   ├── main.rs                  # Rust 入口，注册 50 个 Tauri 命令
│   │   ├── commands/                # 命令层（Tauri Command）
│   │   │   ├── mod.rs               # 模块声明
│   │   │   ├── server.rs            # 服务器配置命令（10 个）
│   │   │   ├── redis.rs             # Redis 操作命令（25 个）
│   │   │   ├── export.rs            # 导入导出命令（2 个）
│   │   │   ├── trash.rs             # 废键箱命令（7 个）
│   │   │   └── audit.rs             # 审计日志命令（6 个）
│   │   ├── redis/                   # Redis 连接模块
│   │   │   ├── mod.rs               # 模块声明
│   │   │   ├── connection.rs        # Redis 连接封装
│   │   │   └── audit.rs             # 审计日志模块
│   │   └── storage/                 # 本地存储模块
│   │       ├── mod.rs               # 模块声明
│   │       ├── config.rs            # 服务器配置持久化
│   │       └── trash.rs             # 废键箱数据持久化
│   ├── Cargo.toml                   # Rust 依赖
│   └── tauri.conf.json              # Tauri 配置
├── server/                            # WebSocket 代理服务（浏览器模式）
│   └── ws-proxy.js                    # WebSocket Redis 代理 + 静态文件服务
├── docs/                            # 设计文档
├── index.html                       # HTML 模板
├── package.json                     # 前端依赖
├── vite.config.ts                   # Vite 构建配置
└── README.md                        # 项目说明
```

---

## 3. 模块依赖关系

```
index.html
  └── main.ts
        ├── vue / pinia / element-plus
        └── App.vue
              ├── HomeView.vue（监控面板）
              └── MainView.vue
                    ├── TabBar ──> SessionManager ──> Session
                    │                                    └── redisStore / serverStore
                    ├── MemoryDialog ──> redisStore ──invoke──> commands::redis ──> redis::connection
                    ├── LogDialog ──> logStore ──invoke──> commands::redis ──> redis::connection
                    │             ──> auditStore ──invoke──> commands::audit ──> redis::audit
                    ├── configStore ──> 应用配置持久化
                    ├── logStore ──> 日志状态管理
                    ├── serverStore ──invoke──> commands::server ──> storage::config
                    │                                        └──> redis::connection
                    ├── redisStore  ──invoke──> commands::redis  ──> redis::connection
                    │                         ──invoke──> commands::export ──> redis::connection
                    ├── trashStore  ──invoke──> commands::trash  ──> storage::trash
                    │                                        └──> redis::connection
                    └── ServerConfigView
                          └── serverStore (同上)

浏览器环境（browser-adapter）：
  main.ts ──> browser-adapter.ts
        ├── 检测运行环境（Tauri / Browser）
        ├── Browser 模式：通过 server/ws-proxy.js WebSocket 代理请求到后端
        └── Tauri 模式：直接使用 @tauri-apps/api invoke
```

**后端模块关系**：

```
main.rs (注册所有命令)
  ├── commands::server   ──> storage::config (ConfigManager)
  │                       ──> redis::connection (RedisConnection)
  ├── commands::redis    ──> redis::connection
  ├── commands::export   ──> redis::connection
  ├── commands::trash    ──> storage::trash (TrashManager)
  │                       ──> redis::connection
  ├── commands::audit    ──> redis::audit (AuditEntry)
  └── server config import/export
        ├── export_server_config ──> storage::config (读取服务器配置并导出)
        └── import_server_config ──> storage::config (导入服务器配置)
```

---

## 4. 前端模块详解

### 4.1 视图层

#### MainView.vue

主界面视图，是项目最核心的前端文件（约 2600 行）。

**功能模块**：

| 模块 | 说明 |
|------|------|
| 标签栏 | TabBar 组件，多会话标签切换，支持拖拽排序 |
| 菜单栏 | 连接/DB/更多/多选/排序 |
| 状态栏 | 当前路径导航 |
| 搜索栏 | 刷新按钮 + 搜索框 + 搜索按钮 + 添加按钮(+) |
| 内存分析入口 | 打开 MemoryDialog 查看内存使用详情 |
| 慢日志入口 | 打开 LogDialog 查看慢查询日志 |
| 键列表 | el-tree 展示，分页加载（每次 100 个），支持排序 |
| 多选面板 | 全选/取消/删除废键箱（向上展开） |
| 值展示区 | 类型标签 + 编辑器 + 删除/修改按钮 |
| 元素级编辑 | 复杂类型（List/Set/ZSet/Hash）支持覆盖模式和元素编辑模式切换，表格化增删改查 |
| 废键箱视图 | el-table 展示，恢复/永久删除 |
| 对话框 | 添加键、修改键、服务器配置、新增DB、删除DB、导出、清空 |
| 可拖拽分隔条 | 键列表与值展示区之间的可拖拽分隔条 |

**核心方法**：

| 方法 | 功能 |
|------|------|
| `handleServerChange()` | 切换服务器，退出多选，加载数据库 |
| `handleDbChange()` | 切换 DB，退出多选，重置分页 |
| `loadKeys(reset?)` | 加载键列表（前端模拟分页） |
| `handleLoadMore()` | 加载更多（追加 100 个） |
| `handleLoadAll()` | 加载所有（分批，显示进度条） |
| `loadKeyValue(key)` | 加载键值（JSON 自动格式化） |
| `searchKeys()` | 模糊搜索（`*keyword*`） |
| `handleSortCommand()` | 排序键列表（升序/降序/取消） |
| `addKey()` | 添加新键 |
| `updateKey()` | 修改键值 |
| `deleteKey()` | 删除键（删除废键箱） |
| `batchMoveToTrash()` | 批量删除废键箱 |
| `exportData()` / `importData()` | 导入导出 JSON |
| `toggleMultiSelectMode()` | 切换多选模式 |
| `selectAllLoaded()` | 全选当前已加载 |
| `selectAllKeys()` | 全选所有 |
| `clearSelection()` | 取消选择 |
| `handleKeyDown(event)` | 快捷键处理 |
| `switchEditMode(mode)` | 切换编辑模式（覆盖模式/元素编辑模式） |
| `saveElement(index)` | 保存元素编辑（元素编辑模式） |
| `deleteElement(index)` | 删除单个元素（元素编辑模式） |
| `confirmAddElement()` | 通过对话框新增元素 |
| `filterElements()` | 元素搜索过滤（元素编辑模式） |

**快捷键**：

| 快捷键 | 功能 |
|--------|------|
| Ctrl+A | 全选当前已加载 |
| Ctrl+Shift+A | 全选所有 |
| Esc | 退出多选模式 |

#### ServerConfigView.vue

服务器配置管理页面，嵌入 MainView 的对话框中使用。

**功能**：服务器列表表格、添加/编辑/删除服务器、测试连接。

#### HomeView.vue

监控面板页面，作为应用的首页展示 Redis 服务器概览信息。

**功能**：服务器状态概览、内存使用监控、慢日志统计、快速连接入口。

#### MemoryDialog.vue

内存分析对话框组件，用于查看 Redis 服务器的内存使用详情。

**功能**：展示内存使用概况、键类型分布、内存占用排行、内存分析图表。

#### LogDialog.vue

慢日志与操作审计对话框组件，包含"慢日志"和"操作审计"两个 Tab 视图。

**功能**：慢日志 Tab 展示慢查询日志列表、查询耗时详情、慢日志筛选与分页；操作审计 Tab 展示 Redis 命令审计日志、命令统计、审计配置管理。

#### TabBar.vue

标签栏组件，支持多会话标签的切换和管理。

**功能**：多标签展示、标签切换、标签关闭、拖拽排序（基于 sortablejs）、右键菜单操作。

### 4.2 状态管理层

#### redisStore.ts

Redis 操作状态管理。

**类型定义**：

| 接口 | 字段 |
|------|------|
| `ConnectRequest` | host, port, username?, password?, db |
| `KeyRequest` | host, port, username?, password?, db, key |
| `KeyValueRequest` | host, port, username?, password?, db, key, value, key_type |
| `SearchRequest` | host, port, username?, password?, db, pattern |
| `ExportRequest` | host, port, username?, password?, db, file_path |
| `ElementRequest` | host, port, username?, password?, db, key, value/member/score/field |
| `KeyValueResponse` | key, value, key_type |

**Actions**：

| Action | 后端命令 | 功能 |
|--------|---------|------|
| `connect()` | `connect` | 连接 Redis |
| `getDatabases()` | `get_databases` | 获取数据库列表 |
| `getKeys()` | `get_keys` | 获取所有键 |
| `getKeyValue()` | `get_key_value` | 获取键值 |
| `setKeyValue()` | `set_key_value` | 设置键值 |
| `deleteKey()` | `delete_key` | 删除键 |
| `searchKeys()` | `search_keys` | 搜索键 |
| `exportData()` | `export_data` | 导出数据 |
| `importData()` | `import_data` | 导入数据 |
| `createDatabase()` | `create_database` | 创建数据库 |
| `deleteDatabase()` | `delete_database` | 删除数据库 |
| `flushDatabase()` | `flush_database` | 清空数据库 |
| `listRpush()` | `list_rpush` | List 添加元素（RPUSH） |
| `listLset()` | `list_lset` | List 修改元素（LSET） |
| `listLrem()` | `list_lrem` | List 删除元素（LREM） |
| `setSadd()` | `set_sadd` | Set 添加成员（SADD） |
| `setSrem()` | `set_srem` | Set 删除成员（SREM） |
| `zsetZadd()` | `zset_zadd` | ZSet 添加成员（ZADD） |
| `zsetZrem()` | `zset_zrem` | ZSet 删除成员（ZREM） |
| `hashHset()` | `hash_hset` | Hash 设置字段（HSET） |
| `hashHdel()` | `hash_hdel` | Hash 删除字段（HDEL） |

#### serverStore.ts

服务器配置状态管理。

**类型定义**：

| 接口 | 字段 |
|------|------|
| `Server` | id, name, host, port, username?, password?, db, created, updated |
| `TestConnectionRequest` | host, port, username?, password? |
| `TestConnectionResponse` | success, message |

**Actions**：

| Action | 后端命令 | 功能 |
|--------|---------|------|
| `loadServers()` | `get_servers` | 加载服务器列表 |
| `addServer()` | `add_server` | 添加服务器 |
| `editServer()` | `edit_server` | 编辑服务器 |
| `deleteServer()` | `delete_server` | 删除服务器 |
| `testConnection()` | `test_connection` | 测试连接 |
| `saveServerOrder()` | `save_server_order` | 保存服务器排序 |

#### trashStore.ts

废键箱状态管理。

**类型定义**：

| 接口 | 字段 |
|------|------|
| `MoveToTrashRequest` | host, port, username?, password?, db, key |
| `BatchMoveToTrashRequest` | host, port, username?, password?, db, keys[] |
| `TrashItemResponse` | id, key, value, key_type, host, port, db, deleted_at, expires_at, is_expired |

**Actions**：

| Action | 后端命令 | 功能 |
|--------|---------|------|
| `moveToTrash()` | `move_to_trash` | 单个删除废键箱 |
| `batchMoveToTrash()` | `batch_move_to_trash` | 批量删除废键箱 |
| `getTrashItems()` | `get_trash_items` | 获取废键箱列表 |
| `restoreFromTrash()` | `restore_from_trash` | 恢复单个键 |
| `batchRestoreFromTrash()` | `batch_restore_from_trash` | 批量恢复键 |
| `permanentDelete()` | `permanent_delete_trash` | 永久删除 |
| `clearExpired()` | `clear_expired_trash` | 清理过期项 |

#### configStore.ts

应用配置状态管理，负责管理应用级别的配置项。

**功能**：持久化应用配置（如窗口布局、主题设置、最近连接、调试日志开关、操作审核开关等）、配置项的读写与同步。

**State**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `debugLogEnabled` | `boolean` | 调试日志开关 |
| `auditEnabled` | `boolean` | 操作审核日志开关（默认开启） |

**Actions**：

| Action | 后端命令 | 功能 |
|--------|---------|------|
| `setDebugLogEnabled(enabled)` | `set_debug_log_enabled` | 设置调试日志开关 |
| `loadDebugConfig()` | `get_debug_log_enabled` | 加载调试日志配置 |
| `setAuditEnabled(enabled)` | `set_audit_enabled` | 设置操作审核开关 |
| `loadAuditConfig()` | `get_audit_enabled` | 加载操作审核配置 |

#### logStore.ts

日志状态管理，负责管理和存储操作日志。

**功能**：记录操作日志、日志级别筛选、日志列表查询、慢日志数据管理。

#### auditStore.ts

审计日志状态管理，负责管理 Redis 命令审计日志的查询、统计和配置。

**State**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `logs` | `AuditEntry[]` | 审计日志列表 |
| `stats` | `CommandStats[]` | 命令统计列表 |
| `isLoading` | `boolean` | 加载状态 |
| `filters` | `object` | 筛选条件（server_id, command, 时间范围等） |
| `pagination` | `object` | 分页信息（page, pageSize, total） |
| `currentServer` | `object \| null` | 当前选中的服务器信息 |

**Actions**：

| Action | 后端命令 | 功能 |
|--------|---------|------|
| `fetchLogs()` | `audit_get_logs` | 分页查询审计日志，支持筛选 |
| `fetchStats()` | `audit_get_stats` | 获取命令执行统计 |
| `clearLogs()` | `audit_clear` | 清空审计日志 |
| `generateTestData()` | `audit_generate_test_data` | 生成测试数据 |
| `changePage()` / `changePageSize()` | — | 分页操作 |

### 4.3 会话管理层

#### Session.ts

单个 Redis 会话封装，管理一个 Redis 连接的状态和数据。

**功能**：封装单个 Redis 连接的完整生命周期（连接、切换 DB、加载数据、断开）、维护当前会话的键列表和选中状态、提供会话级别的操作方法。

#### SessionManager.ts

多会话管理器，统一管理多个 Redis 会话实例。

**功能**：创建/销毁会话、会话切换、会话列表维护、活跃会话状态同步、与 TabBar 组件联动。

### 4.4 适配层

#### browser-adapter.ts

浏览器环境适配层，用于在非 Tauri 环境（纯浏览器）下运行应用。

**功能**：检测当前运行环境（Tauri 桌面端 / Web 浏览器）、在浏览器模式下通过 HTTP 代理与后端通信、在 Tauri 模式下直接使用 `@tauri-apps/api` 的 `invoke` 调用、统一前后端通信接口，对上层业务代码透明。

---

## 5. 后端模块详解

### 5.1 入口 (main.rs)

注册 Tauri 插件和所有命令（共 50 个）。

| 分类 | 命令数量 | 命令列表 |
|------|---------|---------|
| 服务器管理 | 10 | add_server, edit_server, delete_server, get_servers, save_server_order, test_connection, get_debug_log_enabled, set_debug_log_enabled, get_audit_enabled, set_audit_enabled |
| Redis 操作 | 25 | connect, get_databases, get_keys, get_key_value, set_key_value, delete_key, search_keys, create_database, delete_database, flush_database, generate_test_data, slowlog_get, get_memory_info, get_type_distribution, get_server_info, get_key_stats, list_rpush, list_lset, list_lrem, set_sadd, set_srem, zset_zadd, zset_zrem, hash_hset, hash_hdel |
| 导入导出 | 2 | export_data, import_data |
| 废键箱 | 7 | move_to_trash, batch_move_to_trash, get_trash_items, restore_from_trash, batch_restore_from_trash, permanent_delete_trash, clear_expired_trash |
| 审计日志 | 6 | audit_get_logs, audit_get_stats, audit_clear, audit_get_config, audit_update_config, audit_generate_test_data |

### 5.2 命令层 (commands/)

#### commands/redis.rs

Redis 操作命令，25 个函数。

| 函数 | 功能 | 参数 |
|------|------|------|
| `connect` | 创建连接并 ping 测试 | host, port, username?, password?, db |
| `get_databases` | 获取所有数据库及 key 数量 | host, port, username?, password? |
| `get_keys` | 获取当前 DB 所有键 | host, port, username?, password?, db |
| `get_key_value` | 获取键值和类型 | host, port, username?, password?, db, key |
| `set_key_value` | 设置键值 | host, port, username?, password?, db, key, value, key_type |
| `delete_key` | 删除键 | host, port, username?, password?, db, key |
| `search_keys` | 按模式搜索键 | host, port, username?, password?, db, pattern |
| `create_database` | 创建/切换 DB | host, port, username?, password?, db |
| `delete_database` | 清空指定 DB | host, port, username?, password?, db |
| `flush_database` | 清空当前 DB | host, port, username?, password?, db |
| `generate_test_data` | 生成测试数据 | host, port, username?, password?, db, count |
| `slowlog_get` | 获取慢查询日志列表 | host, port, username?, password?, count? |
| `get_memory_info` | 获取 Redis 内存使用信息（支持游标分页） | host, port, username?, password?, cursor? |
| `get_type_distribution` | 获取键类型分布统计（异步全量扫描） | host, port, username?, password?, db |
| `get_server_info` | 获取 Redis 服务器信息（INFO） | host, port, username?, password? |
| `get_key_stats` | 获取键统计详情 | host, port, username?, password?, db |
| `list_rpush` | List 尾部添加元素（RPUSH） | host, port, username?, password?, db, key, value |
| `list_lset` | List 修改指定位置元素（LSET） | host, port, username?, password?, db, key, index, value |
| `list_lrem` | List 删除元素（LREM） | host, port, username?, password?, db, key, value, count |
| `set_sadd` | Set 添加成员（SADD） | host, port, username?, password?, db, key, member |
| `set_srem` | Set 删除成员（SREM） | host, port, username?, password?, db, key, member |
| `zset_zadd` | ZSet 添加成员（ZADD） | host, port, username?, password?, db, key, member, score |
| `zset_zrem` | ZSet 删除成员（ZREM） | host, port, username?, password?, db, key, member |
| `hash_hset` | Hash 设置字段（HSET） | host, port, username?, password?, db, key, field, value |
| `hash_hdel` | Hash 删除字段（HDEL） | host, port, username?, password?, db, key, field |

#### commands/server.rs

服务器配置管理命令，10 个函数。

| 函数 | 功能 |
|------|------|
| `add_server` | 添加服务器配置（检查 ID 唯一性） |
| `edit_server` | 编辑服务器配置（保留原创建时间） |
| `delete_server` | 按 ID 删除服务器 |
| `get_servers` | 获取所有服务器配置 |
| `save_server_order` | 保存服务器列表排序 |
| `test_connection` | 测试 Redis 连接 |
| `get_debug_log_enabled` | 获取调试日志开关状态 |
| `set_debug_log_enabled` | 设置调试日志开关 |
| `get_audit_enabled` | 获取操作审核开关状态 |
| `set_audit_enabled` | 设置操作审核开关 |

> 注：服务器配置导入导出功能（export_server_config / import_server_config）在前端通过文件对话框 + JSON 序列化实现，无独立 Tauri 命令。

#### commands/export.rs

导入导出命令，2 个函数。

| 函数 | 功能 |
|------|------|
| `export_data` | 导出当前 DB 所有键值为 JSON 文件（格式：`[{key, value, type}]`） |
| `import_data` | 从 JSON 文件导入键值到当前 DB |

#### commands/trash.rs

废键箱命令，7 个函数。

| 函数 | 功能 |
|------|------|
| `move_to_trash` | 保存键值到废键箱，从 Redis 删除（7 天过期） |
| `batch_move_to_trash` | 批量删除废键箱 |
| `get_trash_items` | 获取指定服务器的废键箱列表（自动清理过期项） |
| `restore_from_trash` | 恢复单个键到 Redis |
| `batch_restore_from_trash` | 批量恢复键 |
| `permanent_delete_trash` | 永久删除废键箱项 |
| `clear_expired_trash` | 清理所有过期废键箱项 |

#### commands/audit.rs

审计日志命令，6 个函数。

| 函数 | 功能 | 参数 |
|------|------|------|
| `audit_get_logs` | 分页查询审计日志，支持按服务器、命令、时间范围筛选 | AuditLogRequest（host, port, password?, page, page_size, server_id?, command?, start_time?, end_time?） |
| `audit_get_stats` | 获取命令执行统计（调用次数、平均耗时等） | AuditStatsRequest（host, port, password?, start_time?, end_time?） |
| `audit_clear` | 清空审计日志 | host, port, password? |
| `audit_get_config` | 获取审计配置 | 无参数 |
| `audit_update_config` | 更新审计配置（启用/禁用、过滤命令列表） | enabled, filter_commands |
| `audit_generate_test_data` | 生成测试审计数据 | host, port, password?, count |

### 5.3 Redis 连接模块 (redis/connection.rs)

底层 Redis 操作封装，结构体 `RedisConnection`。

| 方法 | 功能 |
|------|------|
| `new(host, port, password?)` | 创建连接（兼容包装，内部调用 new_with_auth） |
| `new_with_auth(host, port, username?, password?)` | 创建连接（支持 ACL 用户名认证，Redis >= 6.0） |
| `select(db)` | 切换数据库 |
| `ping()` | 测试连接 |
| `get_databases()` | 遍历 DB 0-15，返回有 key 的数据库 |
| `get_keys()` | 获取当前 DB 所有键（`KEYS *`） |
| `get_key_value(key)` | 获取键值（支持 5 种类型） |
| `set_key_value(key, value, key_type)` | 设置键值（list/set/zset/hash 使用 JSON 序列化） |
| `delete_key(key)` | 删除单个键 |
| `delete_keys(keys[])` | 批量删除键 |
| `search_keys(pattern)` | 按模式搜索键 |
| `flushdb()` | 清空当前数据库 |
| `get_memory_info(cursor?)` | 获取内存使用信息（支持游标分页扫描，Pipeline 合并 TYPE + MEMORY USAGE） |
| `slowlog_get(count?)` | 获取慢查询日志（SLOWLOG GET） |
| `get_server_info()` | 获取服务器信息（INFO） |
| `get_type_distribution()` | 获取键类型分布统计（基于 SCAN + TYPE，异步全量扫描） |
| `get_key_stats()` | 获取键统计详情 |
| `list_rpush(key, value)` | List 尾部添加元素 |
| `list_lset(key, index, value)` | List 修改指定位置元素 |
| `list_lrem(key, value, count)` | List 删除元素 |
| `set_sadd(key, member)` | Set 添加成员 |
| `set_srem(key, member)` | Set 删除成员 |
| `zset_zadd(key, member, score)` | ZSet 添加成员 |
| `zset_zrem(key, member)` | ZSet 删除成员 |
| `hash_hset(key, field, value)` | Hash 设置字段 |
| `hash_hdel(key, field)` | Hash 删除字段 |

**ACL 认证（Redis >= 6.0）**：

| 属性 | 值 |
|------|-----|
| 认证方式 | `AUTH <username> <password>` |
| URL 组合 | `redis://user:pass@host:port` / `redis://user@host:port` / `redis://:pass@host:port` / `redis://host:port` |
| 连接缓存 key | `host:port:username:password:db`（5 段，username 可为空字符串） |
| 默认行为 | username 为空时使用 Redis 默认用户 |

**数据类型处理**：

| 类型 | 存储方式 | 读取方式 |
|------|----------|----------|
| string | 原值 | GET |
| list | JSON 序列化 | LRANGE + JSON 反序列化 |
| set | JSON 序列化 | SMEMBERS + JSON 反序列化 |
| zset | JSON 序列化 | ZRANGE + JSON 反序列化 |
| hash | JSON 序列化 | HGETALL + JSON 反序列化 |

### 5.4 审计日志模块 (redis/audit.rs)

Redis 命令审计日志的底层模块，负责审计日志的写入和读取。

**AuditEntry 结构体**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String | 唯一标识（UUID） |
| `timestamp` | String | 记录时间（RFC3339） |
| `server_id` | String | 服务器 ID |
| `server_name` | String | 服务器名称 |
| `db` | u8 | 数据库编号 |
| `client_ip` | String | 客户端 IP |
| `command` | String | Redis 命令 |
| `args` | String | 命令参数 |
| `cost_ms` | u64 | 执行耗时（毫秒） |
| `success` | bool | 是否成功 |
| `error_message` | Option\<String\> | 错误信息 |

**CommandStats 结构体**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `command` | String | Redis 命令名 |
| `count` | u64 | 调用次数 |
| `total_cost_ms` | u64 | 总耗时（毫秒） |
| `avg_cost_ms` | f64 | 平均耗时（毫秒） |
| `success_count` | u64 | 成功次数 |
| `error_count` | u64 | 失败次数 |

**存储方式**：

| 属性 | 值 |
|------|-----|
| 存储结构 | Redis List（LPUSH + LTRIM） |
| Key | `redis:audit:logs` |
| 最大条数 | 1,000,000 条（超出自动裁剪最早的记录） |
| 写入方式 | `write_entry_to_list_sync` 函数，将 AuditEntry 序列化为 JSON 后 LPUSH 到列表头部，并执行 LTRIM 保持列表长度 |

### 5.5 存储模块 (storage/)

#### storage/config.rs

服务器配置持久化。

| 结构体 | 字段 |
|--------|------|
| `ServerConfig` | id, name, host, port, username?, password?, db, created, updated |
| `Config` | servers: Vec\<ServerConfig\>, debug_log_enabled: bool, audit_enabled: bool |
| `ConfigManager` | config, config_path |

| 属性 | 值 |
|------|-----|
| 存储路径 | `$HOME/.redis-helper/config.json` |
| 配置项 | servers（服务器列表）、debug_log_enabled（调试日志开关）、audit_enabled（操作审核开关，默认 true） |
| 管理器方法 | new, save, add_server, edit_server, delete_server, get_servers, get/set_debug_log_enabled, get/set_audit_enabled |

#### storage/trash.rs

废键箱数据持久化。

| 结构体 | 字段 |
|--------|------|
| `TrashConnection` | host, port, username?, password?, db |
| `TrashItem` | id, server_id, key, value, key_type, connection, deleted_at, expires_at |
| `TrashData` | items: Vec\<TrashItem\> |
| `TrashManager` | data, trash_path |

| 属性 | 值 |
|------|-----|
| 存储路径 | `$HOME/.redis-helper/trash.json` |
| 过期策略 | 7 天（通过 `expires_at` RFC3339 时间戳） |
| 管理器方法 | new, save, add_item, remove_item, remove_items, get_items_by_server, get_item, cleanup_expired |

---

## 6. Tauri 命令完整列表

### 6.1 服务器管理（10 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `add_server` | ServerRequest | Vec\<ServerConfig\> | 添加服务器配置 |
| `edit_server` | ServerRequest | Vec\<ServerConfig\> | 编辑服务器配置 |
| `delete_server` | { id: String } | Vec\<ServerConfig\> | 删除服务器 |
| `get_servers` | () | Vec\<ServerConfig\> | 获取所有服务器 |
| `save_server_order` | Vec\<ServerRequest\> | Vec\<ServerConfig\> | 保存服务器排序 |
| `test_connection` | TestConnectionRequest | TestConnectionResponse | 测试连接 |
| `get_debug_log_enabled` | () | bool | 获取调试日志开关 |
| `set_debug_log_enabled` | { enabled: bool } | bool | 设置调试日志开关 |
| `get_audit_enabled` | () | bool | 获取操作审核开关 |
| `set_audit_enabled` | { enabled: bool } | bool | 设置操作审核开关 |

### 6.2 Redis 操作（25 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `connect` | ConnectRequest | bool | 连接 Redis |
| `get_databases` | ConnectRequest | Vec\<(u8, usize)\> | 获取数据库列表 |
| `get_keys` | ConnectRequest | KeysResponse | 获取所有键 |
| `get_key_value` | KeyRequest | KeyValueResponse | 获取键值 |
| `set_key_value` | KeyValueRequest | bool | 设置键值 |
| `delete_key` | KeyRequest | bool | 删除键 |
| `search_keys` | SearchRequest | Vec\<String\> | 搜索键 |
| `create_database` | ConnectRequest | bool | 创建数据库 |
| `delete_database` | ConnectRequest | bool | 删除数据库 |
| `flush_database` | ConnectRequest | bool | 清空数据库 |
| `generate_test_data` | ConnectRequest, count | bool | 生成测试数据 |
| `slowlog_get` | SlowlogRequest | Vec\<SlowlogEntry\> | 获取慢查询日志 |
| `get_memory_info` | ConnectRequest, cursor? | MemoryInfoResponse | 获取内存使用信息（游标分页） |
| `get_type_distribution` | ConnectRequest | Vec\<KeyTypeStat\> | 获取键类型分布 |
| `get_server_info` | ConnectRequest | HashMap\<String, String\> | 获取服务器信息 |
| `get_key_stats` | ConnectRequest | Vec\<KeyStatItem\> | 获取键统计 |
| `list_rpush` | ListRpushRequest | i64 | List 添加元素 |
| `list_lset` | ListLsetRequest | bool | List 修改元素 |
| `list_lrem` | ListLremRequest | i64 | List 删除元素 |
| `set_sadd` | SetSaddRequest | i64 | Set 添加成员 |
| `set_srem` | SetSremRequest | i64 | Set 删除成员 |
| `zset_zadd` | ZsetZaddRequest | i64 | ZSet 添加成员 |
| `zset_zrem` | ZsetZremRequest | i64 | ZSet 删除成员 |
| `hash_hset` | HashHsetRequest | bool | Hash 设置字段 |
| `hash_hdel` | HashHdelRequest | i64 | Hash 删除字段 |

> 注：所有 Request 结构体均包含 `username: Option<String>` 字段，支持 Redis 6.0+ ACL 认证。

### 6.3 导入导出（2 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `export_data` | ExportRequest | () | 导出为 JSON |
| `import_data` | ImportRequest | () | 从 JSON 导入 |

### 6.4 废键箱（7 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `move_to_trash` | MoveToTrashRequest | () | 删除废键箱 |
| `batch_move_to_trash` | BatchMoveToTrashRequest | () | 批量删除 |
| `get_trash_items` | { host, port } | Vec\<TrashItemResponse\> | 获取列表 |
| `restore_from_trash` | { trash_id: String } | () | 恢复单个 |
| `batch_restore_from_trash` | { trash_ids: Vec\<String\> } | () | 批量恢复 |
| `permanent_delete_trash` | { trash_ids: Vec\<String\> } | () | 永久删除 |
| `clear_expired_trash` | () | () | 清理过期 |

### 6.5 审计日志（6 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `audit_get_logs` | AuditLogRequest | Vec\<AuditEntry\> | 分页查询审计日志，支持筛选 |
| `audit_get_stats` | AuditStatsRequest | Vec\<CommandStats\> | 获取命令统计 |
| `audit_clear` | { host, port, username?, password? } | bool | 清空审计日志 |
| `audit_get_config` | () | AuditConfig | 获取审计配置 |
| `audit_update_config` | { enabled, filter_commands } | bool | 更新审计配置 |
| `audit_generate_test_data` | { host, port, username?, password?, count } | usize | 生成测试数据 |

---

## 7. 数据存储

| 文件 | 路径 | 内容 |
|------|------|------|
| 服务器配置 | `~/.redis-helper/config.json` | 服务器列表（id, name, host, port, username, password, db）、debug_log_enabled、audit_enabled |
| 废键箱数据 | `~/.redis-helper/trash.json` | 废键项列表（key, value, type, deleted_at, expires_at） |

---

## 8. 依赖清单

### 8.1 前端依赖

| 包名 | 版本 | 用途 |
|------|------|------|
| vue | ^3.4.0 | 前端框架 |
| pinia | ^2.1.7 | 状态管理 |
| element-plus | ^2.4.4 | UI 组件库 |
| @tauri-apps/api | ^2.0.0 | Tauri 前端 API |
| @tauri-apps/plugin-dialog | ^2.7.0 | 文件对话框插件 |
| sortablejs | ^1.15.0 | 拖拽排序库（标签栏拖拽） |
| typescript | ^5.2.2 | TypeScript |
| vite | ^5.0.8 | 构建工具 |

### 8.2 后端依赖

| 包名 | 版本 | 用途 |
|------|------|------|
| tauri | ^2.0.0 | Tauri 框架 |
| tauri-plugin-dialog | 2 | 文件对话框插件 |
| tauri-plugin-fs | 2 | 文件系统访问插件 |
| redis | ^0.26.0 | Redis 客户端 |
| tokio | 1.0 (full) | 异步运行时 |
| serde | 1.0 (derive) | 序列化 |
| serde_json | 1.0 | JSON 处理 |
| chrono | 0.4 (serde) | 时间日期 |
| uuid | 1.0 (v4) | UUID 生成 |

---

## 9. 构建配置

### 9.1 NPM Scripts

| 脚本 | 命令 | 说明 |
|------|------|------|
| `dev` | `node server/ws-proxy.js & vite` | 启动前端开发服务器 + WebSocket 代理 |
| `start` | `node server/ws-proxy.js` | 启动 WebSocket 代理 |
| `build` | `vue-tsc && vite build` | 构建前端 |
| `tauri:dev` | `TAURI=true tauri dev` | 启动 Tauri 开发模式 |
| `tauri:build` | `tauri build` | 构建当前平台 |
| `tauri:build:mac` | `tauri build --target universal-apple-darwin` | 构建 macOS |
| `tauri:build:windows` | `tauri build --target x86_64-pc-windows-gnu` | 构建 Windows |
| `proxy` | `node server/ws-proxy.js` | 启动 WebSocket 代理服务器 |

### 9.2 Vite 分包策略

| Chunk | 包含模块 |
|-------|---------|
| `vue-vendor` | vue, pinia |
| `element-plus` | element-plus |
| `tauri-api` | @tauri-apps/api/core |

### 9.3 Tauri 窗口配置

| 配置项 | 值 |
|--------|-----|
| productName | Redis小助手 |
| identifier | com.redis.helper |
| 窗口尺寸 | 1000 x 800 |
| 分类 | DeveloperTool |
| 前端开发地址 | http://localhost:5173 |
| 前端构建目录 | ../dist |
