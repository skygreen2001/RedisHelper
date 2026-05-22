# Redis 小助手 — 代码 Wiki

> 本文档为当前版本（Tauri 2.0 + Vue 3 + Rust）的代码结构说明。
> 旧版 PHP 版本文档见 [CODE_WIKI.old.md](CODE_WIKI.old.md)（如有）。

---

## 1. 项目概述

| 属性 | 值 |
|------|-----|
| 项目名称 | Redis小助手 (redis-helper) |
| 版本 | 1.2.0 |
| 技术栈 | Tauri 2.0 + Vue 3 + TypeScript + Pinia + Element Plus + Rust |
| 定位 | 跨平台 Redis 管理工具（支持桌面端和Web浏览器） |
| 前后端通信 | Tauri Commands（invoke），共 31 个命令 |

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
│   │   └── logStore.ts              # 日志状态管理
│   ├── server/                      # Web 服务器（浏览器模式）
│   │   └── proxy.ts                 # 代理服务器，转发请求到 Tauri 后端
│   └── help/                        # 帮助资源
│       └── ...                      # 帮助文档/图片等资源
├── src-tauri/                       # Rust 后端代码
│   ├── src/
│   │   ├── main.rs                  # Rust 入口，注册 31 个 Tauri 命令
│   │   ├── commands/                # 命令层（Tauri Command）
│   │   │   ├── mod.rs               # 模块声明
│   │   │   ├── server.rs            # 服务器配置命令（5 个）
│   │   │   ├── redis.rs             # Redis 操作命令（16 个）
│   │   │   ├── export.rs            # 导入导出命令（2 个）
│   │   │   └── trash.rs             # 废键箱命令（7 个）
│   │   ├── redis/                   # Redis 连接模块
│   │   │   ├── mod.rs               # 模块声明
│   │   │   └── connection.rs        # Redis 连接封装
│   │   └── storage/                 # 本地存储模块
│   │       ├── mod.rs               # 模块声明
│   │       ├── config.rs            # 服务器配置持久化
│   │       └── trash.rs             # 废键箱数据持久化
│   ├── Cargo.toml                   # Rust 依赖
│   └── tauri.conf.json              # Tauri 配置
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
        ├── Browser 模式：通过 server/proxy.ts 代理请求到后端
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

慢日志对话框组件，用于查看 Redis 服务器的慢查询日志。

**功能**：展示慢查询日志列表、查询耗时详情、慢日志筛选与分页。

#### TabBar.vue

标签栏组件，支持多会话标签的切换和管理。

**功能**：多标签展示、标签切换、标签关闭、拖拽排序（基于 sortablejs）、右键菜单操作。

### 4.2 状态管理层

#### redisStore.ts

Redis 操作状态管理。

**类型定义**：

| 接口 | 字段 |
|------|------|
| `ConnectRequest` | host, port, password?, db |
| `KeyRequest` | host, port, password?, db, key |
| `KeyValueRequest` | host, port, password?, db, key, value, key_type |
| `SearchRequest` | host, port, password?, db, pattern |
| `ExportRequest` | host, port, password?, db, file_path |
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

#### serverStore.ts

服务器配置状态管理。

**类型定义**：

| 接口 | 字段 |
|------|------|
| `Server` | id, name, host, port, password?, db, created, updated |
| `TestConnectionRequest` | host, port, password? |
| `TestConnectionResponse` | success, message |

**Actions**：

| Action | 后端命令 | 功能 |
|--------|---------|------|
| `loadServers()` | `get_servers` | 加载服务器列表 |
| `addServer()` | `add_server` | 添加服务器 |
| `editServer()` | `edit_server` | 编辑服务器 |
| `deleteServer()` | `delete_server` | 删除服务器 |
| `testConnection()` | `test_connection` | 测试连接 |

#### trashStore.ts

废键箱状态管理。

**类型定义**：

| 接口 | 字段 |
|------|------|
| `MoveToTrashRequest` | host, port, password?, db, key |
| `BatchMoveToTrashRequest` | host, port, password?, db, keys[] |
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

**功能**：持久化应用配置（如窗口布局、主题设置、最近连接等）、配置项的读写与同步。

#### logStore.ts

日志状态管理，负责管理和存储操作日志。

**功能**：记录操作日志、日志级别筛选、日志列表查询、慢日志数据管理。

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

注册 Tauri 插件和所有命令（共 31 个）。

| 分类 | 命令数量 | 命令列表 |
|------|---------|---------|
| 服务器管理 | 5 | add_server, edit_server, delete_server, get_servers, test_connection |
| Redis 操作 | 16 | connect, get_databases, get_keys, get_key_value, set_key_value, delete_key, search_keys, create_database, delete_database, flush_database, scan_keys, get_memory_info, get_slowlog, get_server_info, get_memory_stats, get_key_type_distribution |
| 导入导出 | 2 | export_data, import_data |
| 服务器配置导入导出 | 2 | export_server_config, import_server_config |
| 废键箱 | 7 | move_to_trash, batch_move_to_trash, get_trash_items, restore_from_trash, batch_restore_from_trash, permanent_delete_trash, clear_expired_trash |

### 5.2 命令层 (commands/)

#### commands/redis.rs

Redis 操作命令，16 个函数。

| 函数 | 功能 | 参数 |
|------|------|------|
| `connect` | 创建连接并 ping 测试 | host, port, password?, db |
| `get_databases` | 获取所有数据库及 key 数量 | host, port, password? |
| `get_keys` | 获取当前 DB 所有键 | host, port, password?, db |
| `get_key_value` | 获取键值和类型 | host, port, password?, db, key |
| `set_key_value` | 设置键值 | host, port, password?, db, key, value, key_type |
| `delete_key` | 删除键 | host, port, password?, db, key |
| `search_keys` | 按模式搜索键 | host, port, password?, db, pattern |
| `create_database` | 创建/切换 DB | host, port, password?, db |
| `delete_database` | 清空指定 DB | host, port, password?, db |
| `flush_database` | 清空当前 DB | host, port, password?, db |
| `scan_keys` | SCAN 分页获取键（替代 KEYS，适合大数据量） | host, port, password?, db, cursor, pattern, count |
| `get_memory_info` | 获取 Redis 内存使用信息 | host, port, password? |
| `get_slowlog` | 获取慢查询日志列表 | host, port, password?, count? |
| `get_server_info` | 获取 Redis 服务器信息（INFO） | host, port, password? |
| `get_memory_stats` | 获取内存统计详情（MEMORY STATS） | host, port, password? |
| `get_key_type_distribution` | 获取键类型分布统计 | host, port, password?, db |

#### commands/server.rs

服务器配置管理命令，7 个函数。

| 函数 | 功能 |
|------|------|
| `add_server` | 添加服务器配置（检查 ID 唯一性） |
| `edit_server` | 编辑服务器配置（保留原创建时间） |
| `delete_server` | 按 ID 删除服务器 |
| `get_servers` | 获取所有服务器配置 |
| `test_connection` | 测试 Redis 连接 |
| `export_server_config` | 导出服务器配置为 JSON 文件 |
| `import_server_config` | 从 JSON 文件导入服务器配置 |

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

### 5.3 Redis 连接模块 (redis/connection.rs)

底层 Redis 操作封装，结构体 `RedisConnection`。

| 方法 | 功能 |
|------|------|
| `new(host, port, password?)` | 创建连接（支持密码认证） |
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
| `scan_keys(cursor, pattern, count)` | SCAN 分页获取键（适合大数据量场景） |
| `get_memory_info()` | 获取内存使用信息（MEMORY DOCTOR / INFO memory） |
| `get_slowlog(count?)` | 获取慢查询日志（SLOWLOG GET） |
| `get_server_info()` | 获取服务器信息（INFO） |
| `get_memory_stats()` | 获取内存统计详情（MEMORY STATS） |
| `get_key_type_distribution()` | 获取键类型分布统计（基于 SCAN + TYPE） |

**数据类型处理**：

| 类型 | 存储方式 | 读取方式 |
|------|----------|----------|
| string | 原值 | GET |
| list | JSON 序列化 | LRANGE + JSON 反序列化 |
| set | JSON 序列化 | SMEMBERS + JSON 反序列化 |
| zset | JSON 序列化 | ZRANGE + JSON 反序列化 |
| hash | JSON 序列化 | HGETALL + JSON 反序列化 |

### 5.4 存储模块 (storage/)

#### storage/config.rs

服务器配置持久化。

| 结构体 | 字段 |
|--------|------|
| `ServerConfig` | id, name, host, port, password?, db, created, updated |
| `Config` | servers: Vec\<ServerConfig\> |
| `ConfigManager` | config, config_path |

| 属性 | 值 |
|------|-----|
| 存储路径 | `$HOME/.redis-helper/config.json` |
| 管理器方法 | new, save, add_server, edit_server, delete_server, get_servers |

#### storage/trash.rs

废键箱数据持久化。

| 结构体 | 字段 |
|--------|------|
| `TrashConnection` | host, port, password?, db |
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

### 6.1 服务器管理（5 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `add_server` | ServerRequest | () | 添加服务器配置 |
| `edit_server` | ServerRequest | () | 编辑服务器配置 |
| `delete_server` | { id: String } | () | 删除服务器 |
| `get_servers` | () | Vec\<ServerConfig\> | 获取所有服务器 |
| `test_connection` | TestConnectionRequest | TestConnectionResponse | 测试连接 |

### 6.2 Redis 操作（16 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `connect` | ConnectRequest | () | 连接 Redis |
| `get_databases` | { host, port, password? } | Vec\<(u8, u64)\> | 获取数据库列表 |
| `get_keys` | { host, port, password?, db } | Vec\<String\> | 获取所有键 |
| `get_key_value` | KeyRequest | KeyValueResponse | 获取键值 |
| `set_key_value` | KeyValueRequest | () | 设置键值 |
| `delete_key` | KeyRequest | () | 删除键 |
| `search_keys` | SearchRequest | Vec\<String\> | 搜索键 |
| `create_database` | { host, port, password?, db } | () | 创建数据库 |
| `delete_database` | { host, port, password?, db } | () | 删除数据库 |
| `flush_database` | { host, port, password?, db } | () | 清空数据库 |
| `scan_keys` | { host, port, password?, db, cursor, pattern?, count? } | { cursor, keys } | SCAN 分页获取键 |
| `get_memory_info` | { host, port, password? } | MemoryInfo | 获取内存使用信息 |
| `get_slowlog` | { host, port, password?, count? } | Vec\<SlowlogEntry\> | 获取慢查询日志 |
| `get_server_info` | { host, port, password? } | ServerInfo | 获取服务器信息 |
| `get_memory_stats` | { host, port, password? } | MemoryStats | 获取内存统计详情 |
| `get_key_type_distribution` | { host, port, password?, db } | Vec\<(String, u64)\> | 获取键类型分布 |

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

### 6.5 服务器配置导入导出（2 个）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `export_server_config` | { file_path: String } | () | 导出服务器配置为 JSON |
| `import_server_config` | { file_path: String } | () | 从 JSON 导入服务器配置 |

---

## 7. 数据存储

| 文件 | 路径 | 内容 |
|------|------|------|
| 服务器配置 | `~/.redis-helper/config.json` | 服务器列表（id, name, host, port, password, db） |
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
| `dev` | `vite` | 启动前端开发服务器 |
| `build` | `vue-tsc && vite build` | 构建前端 |
| `tauri:dev` | `tauri dev` | 启动 Tauri 开发模式 |
| `tauri:build` | `tauri build` | 构建当前平台 |
| `tauri:build:mac` | `tauri build --target universal-apple-darwin` | 构建 macOS |
| `tauri:build:windows` | `tauri build --target x86_64-pc-windows-gnu` | 构建 Windows |
| `proxy` | `node src/frontend/server/proxy.ts` | 启动代理服务器（浏览器模式） |

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
