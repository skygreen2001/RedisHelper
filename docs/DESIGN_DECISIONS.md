# Redis 管理工具 — 设计决策记录 (ADR)

| 项目名称 | Redis小助手 (redis-helper) |
|---------|--------------------------|
| 文档类型 | 设计决策记录 (Architecture Decision Records) |
| 编写日期 | 2026-04-28 |
| 基于版本 | Online Redis Manager → Redis小助手 |

---

## 目录

1. [概述](#1-概述)
2. [架构演进时间线](#2-架构演进时间线)
3. [ADR-001: 桌面框架选型](#adr-001-桌面框架选型)
4. [ADR-002: 前端框架选型](#adr-002-前端框架选型)
5. [ADR-003: 后端架构决策](#adr-003-后端架构决策)
6. [ADR-004: 开发边界控制](#adr-004-开发边界控制)
7. [ADR-005: 双端部署架构](#adr-005-双端部署架构)
8. [ADR-006: Redis 客户端选型](#adr-006-redis-客户端选型)
9. [ADR-007: 配置存储方案](#adr-007-配置存储方案)
10. [ADR-008: UI 设计延续](#adr-008-ui-设计延续)
11. [参考文档](#参考文档)

---

## 1. 概述

本文档记录了 Redis 管理工具从旧版 PHP Web 应用到新版 Tauri 桌面应用的完整设计决策过程。每个决策记录包含背景、候选方案、最终决策、决策理由和后果分析。

### 1.1 项目背景

Online Redis Manager 是一个基于 PHP + Vue 2 + iView 技术栈的在线 Redis 管理工具，需要部署在 Web 服务器上运行，依赖 PHP 环境和 PhpRedis 扩展。为了降低部署门槛、提升用户体验，项目进行了技术栈迁移。

### 1.2 设计目标

- **桌面客户端**：双击即用，无需安装 PHP、Nginx 等外部依赖
- **跨平台**：支持 Windows 和 macOS
- **功能对齐**：严格对齐旧版已有功能，不引入新功能
- **UI 延续**：保持与旧版相似的视觉风格和交互模式

---

## 2. 架构演进时间线

```
v1.0（旧版，已完成）
├── 技术栈：PHP + Vue 2 + iView + PhpRedis + PhpSpreadsheet
├── 运行方式：Web 服务器部署
├── 功能：服务器管理、DB 管理、键 CRUD、搜索、Excel 导入导出
└── 文档：docs/design.md、docs/CODE_WIKI.md、docs/UI.md
        │
        ▼ 设计阶段（未实施）
v1.0 设计稿（Go 后端方案）
├── 技术栈：Go + Gin + React + Ant Design + PostgreSQL
├── 运行方式：独立后端服务 + React SPA
└── 结论：架构过重，与项目定位不符
        │
        ▼ 设计阶段（未实施）
v2.0 设计稿（Electron 方案）
├── 技术栈：Electron + Vue 3 + Element Plus + ioredis + electron-store
├── 运行方式：Electron 桌面客户端
├── 新增：内嵌 HTTP 服务（局域网 Web 访问）
└── 结论：用户希望一套代码双端部署
        │
        ▼ 设计阶段（未实施）
v3.0 设计稿（Electron 双端方案）
├── 架构：shared/ + electron/ + server/ 三层共享
├── 核心设计：环境适配层（IApiAdapter 接口）
├── Electron 环境 → IPC 通信
├── Web 环境 → HTTP 请求（axios）
└── 结论：架构复杂，后续转向 Tauri 方案
        │
        ▼ 当前实现
Redis小助手（Tauri 2.0）
├── 技术栈：Tauri 2.0 + Vue 3 + Element Plus + Pinia + Rust + redis-rs
├── 运行方式：原生桌面应用
├── 配置存储：JSON 文件（~/.redis-helper/config.json）
├── 平台：macOS + Windows
└── 状态：核心功能已实现
```

---

## 3. ADR-001: 桌面框架选型

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | Tauri 2.0 |

### 背景

旧版 PHP 方案需要 Web 服务器环境，部署门槛高。用户希望将产品重构为桌面客户端，双击即用。

### 候选方案

| 方案 | 优点 | 缺点 |
|------|------|------|
| **Electron** | 生态成熟，社区庞大，Web 技术栈复用 | 包体积大（>100MB），内存占用高 |
| **Tauri 2.0** | 包体积小（~10MB），内存占用低，安全性高（Rust） | 需要 Rust 开发能力，生态相对较新 |
| **Qt / C#** | 原生性能 | 开发效率低，跨平台复杂 |

### 决策

采用 **Tauri 2.0**。

### 理由

1. **包体积**：Tauri 打包后约 10MB，Electron 通常 >100MB
2. **内存占用**：Tauri 使用系统 WebView，内存占用远低于 Electron 的 Chromium
3. **安全性**：Rust 后端天然内存安全，攻击面更小
4. **性能**：Rust 后端性能优异，适合 Redis 连接管理场景

### 后果

- 需要 Rust 开发能力（学习曲线）
- 前端仍使用 Vue 3 + TypeScript，Web 开发经验可复用
- Tauri 2.0 生态持续完善，长期风险可控

---

## 4. ADR-002: 前端框架选型

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | Vue 3 + Element Plus |

### 背景

旧版使用 Vue 2 + iView，需要选择新版前端框架。

### 候选方案

| 方案 | 与旧版相似度 | 生态成熟度 | TypeScript 支持 |
|------|------------|-----------|----------------|
| **Vue 3 + Element Plus** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| React + Ant Design | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Vue 2 + iView (保持不变) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |

### 决策

采用 **Vue 3 + Element Plus + Pinia + TypeScript**。

### 理由

1. **迁移成本最低**：Vue 2 → Vue 3 的 API 变化可控，iView → Element Plus 的组件 API 风格接近
2. **UI 风格延续**：Element Plus 的视觉风格与旧版 iView 最为接近
3. **中文文档完善**：Element Plus 中文文档质量高，开发效率有保障
4. **Composition API**：Vue 3 的 Composition API 更适合复杂状态管理

### 后果

- 开发者需要学习 Vue 3 Composition API
- iView 组件需要逐一映射到 Element Plus 对应组件
- Pinia 替代 Vuex，状态管理 API 更简洁

---

## 5. ADR-003: 后端架构决策

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | Tauri Rust 后端 |

### 背景

旧版使用 PHP 后端处理 Redis 操作和 Excel 导入导出。桌面化后需要重新设计后端架构。

### 候选方案演进

| 阶段 | 方案 | 结论 |
|------|------|------|
| 设计阶段 1 | Go + Gin + PostgreSQL | ❌ 架构过重，桌面应用不需要独立数据库 |
| 设计阶段 2 | Electron 主进程 + ioredis + electron-store | ❌ Electron 方案被否决 |
| 设计阶段 3 | Electron + Express 双端共享服务层 | ❌ 架构复杂度过高 |
| **最终** | **Tauri Rust 后端 + JSON 文件存储** | ✅ 简洁高效 |

### 决策

采用 **Tauri Rust 后端**，通过 Tauri Commands 暴露 API 给前端。

### 理由

1. **无外部依赖**：不需要 PHP、Node.js、PostgreSQL 等外部运行时
2. **内嵌后端**：Rust 代码编译为原生二进制，与前端打包为一个应用
3. **JSON 文件存储**：配置存储在 `~/.redis-helper/config.json`，简单轻量
4. **Tauri Commands**：前端通过 `invoke()` 调用 Rust 后端命令，类型安全

### 后果

- 后端逻辑使用 Rust 编写，开发效率低于 Node.js/Go
- 但运行时性能和安全性更优
- 17 个 Tauri Commands 覆盖全部业务功能

---

## 6. ADR-004: 开发边界控制

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | v2.0 严格对齐旧版已有功能 |

### 背景

初始架构设计中包含大量新功能（命令终端、Cluster/Sentinel 支持、TLS 连接、监控面板、深色主题、自动更新等），但这些功能在旧版中并不存在。

### 决策

**v2.0 严格对齐旧版已有功能，不引入任何新功能。**

### 范围内（v2.0 必须完成）

| 功能 | 说明 |
|------|------|
| 服务器配置 CRUD | 添加、编辑、删除、列表展示 |
| 多服务器切换 | 顶部菜单下拉选择 |
| DB 列表查看 | 通过 Redis 命令获取 |
| 新增 DB | 找空位 SELECT + 写入标记 |
| 删除 DB | FLUSHDB（二次确认） |
| 键列表浏览 | KEYS 命令 + 分页 |
| 键模糊搜索 | KEYS *keyword* 模式匹配 |
| 查看键值 | 自动识别 5 种类型 |
| 修改键值 | DEL + SET |
| 新增键 | Strings / Sets / Lists / Sorted Sets / Hashes |
| 删除键 | DEL（二次确认） |
| 数据导入导出 | JSON 格式（旧版为 Excel） |

### 范围外（后续版本）

| 功能 | 规划版本 | 理由 |
|------|---------|------|
| 命令终端 | v2.1 | 旧版无此功能 |
| Cluster/Sentinel 支持 | v2.1 | 旧版仅 Standalone |
| TLS 连接 | v2.1 | 旧版无此功能 |
| 深色主题 | v2.2 | 旧版无此功能 |
| 自动更新 | v2.2 | 旧版无此功能 |
| 监控面板 | v3.0 | 旧版无此功能 |

### 理由

1. **降低风险**：功能范围可控，避免过度设计
2. **快速交付**：聚焦核心功能，缩短开发周期
3. **质量保证**：每个功能都有旧版参考，便于验证

### 后果

- v2.0 功能范围明确，开发优先级清晰
- 部分用户可能期望新功能，需要通过版本规划管理预期

---

## 7. ADR-005: 双端部署架构

| 属性 | 值 |
|------|-----|
| **状态** | ⏸️ 已搁置（Tauri 方案下重新评估） |
| **决策日期** | 2026-04 |

### 背景

用户希望一套代码同时支持桌面客户端和 Web 服务端部署，方便在服务器上通过浏览器访问。

### 设计阶段方案（Electron 时期）

在 Electron 方案下，设计了完整的环境适配层架构：

```
前端代码（100% 共享）
    │
    ├── ElectronAdapter（IPC 通信）
    └── WebAdapter（HTTP/axios 请求）
    │
共享服务层（shared/services）
    │
    ├── Electron 主进程入口
    └── Express 服务端入口
```

核心设计：
- `IApiAdapter` 接口定义统一的 API 调用规范
- `ElectronAdapter` 通过 `ipcRenderer.invoke()` 调用主进程
- `WebAdapter` 通过 `axios` 调用 REST API
- 运行时自动判断环境，对上层业务代码透明

### 当前状态

在 Tauri 方案下，双端部署需求暂时搁置。Tauri 的 Web 服务器能力有限，如需 Web 访问，可考虑：
- 独立部署一个轻量 Node.js/Express 服务端
- 或使用 Tauri 的自定义协议能力

### 后果

- 当前版本仅支持桌面客户端
- 如后续需要 Web 访问，可参考之前设计的适配层架构

---

## 8. ADR-006: Redis 客户端选型

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | redis-rs (Rust) |

### 背景

需要选择 Redis 客户端库连接和管理 Redis 实例。

### 候选方案演进

| 阶段 | 方案 | 语言 | 结论 |
|------|------|------|------|
| 旧版 | PhpRedis | PHP | ✅ 已使用 |
| 设计阶段 | ioredis | Node.js | ❌ Electron 方案未实施 |
| **最终** | **redis-rs** | **Rust** | ✅ 当前使用 |

### 决策

采用 **redis-rs**（Rust Redis 客户端库）。

### 理由

1. **Tauri 生态**：Tauri 后端使用 Rust，redis-rs 是 Rust 生态最成熟的 Redis 客户端
2. **异步支持**：基于 tokio 异步运行时，性能优异
3. **类型安全**：Rust 类型系统保证编译时安全
4. **功能完整**：支持 String/List/Set/ZSet/Hash 五种核心数据类型

### 后果

- 当前仅支持 Standalone 模式
- Cluster/Sentinel 支持规划在 v2.1

---

## 9. ADR-007: 配置存储方案

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | JSON 文件 |

### 背景

需要持久化存储服务器配置信息（地址、端口、密码等）。

### 候选方案演进

| 阶段 | 方案 | 结论 |
|------|------|------|
| 旧版 | localStorage / JSON 文件 | ✅ 两种模式并存 |
| 设计阶段 | electron-store | ❌ Electron 方案未实施 |
| 设计阶段 | IConfigService 接口 + 多实现 | ❌ 架构过重 |
| **最终** | **JSON 文件** | ✅ `~/.redis-helper/config.json` |

### 决策

采用 **JSON 文件**存储配置。

### 理由

1. **简单轻量**：无需数据库或额外依赖
2. **跨平台**：文件系统 API 在各平台表现一致
3. **可读性好**：JSON 格式便于调试和手动修改
4. **安全性**：配合系统权限，配置文件仅当前用户可读

### 后果

- 配置文件路径：`~/.redis-helper/config.json`
- 多实例并发写入需要文件锁机制
- 大量配置时性能不如数据库

---

## 10. ADR-008: UI 设计延续

| 属性 | 值 |
|------|-----|
| **状态** | ✅ 已决定 |
| **决策日期** | 2026-04 |
| **最终方案** | 延续旧版 UI 风格 |

### 背景

旧版已有成熟的 UI 设计，包含 57 张截图记录的完整交互流程。

### 设计规范

| 特征 | 旧版 | 新版 |
|------|------|------|
| **整体布局** | 左右双栏（键列表 + 值详情） | 保持一致 |
| **导航栏** | 深色背景 (#3c4e5a) | Element Plus 深色主题 |
| **主色调** | 蓝色 (#1890ff 风格) | Element Plus 蓝色 (#409EFF) |
| **服务器选择** | 顶部下拉菜单 | Element Plus SubMenu |
| **DB 选择** | 顶部下拉菜单 | Element Plus SubMenu |
| **键列表** | iView List 组件 | Element Plus 虚拟列表 |
| **对话框** | iView Modal | Element Plus Dialog |
| **表格** | iView Table | Element Plus Table |
| **二次确认** | iView 确认弹窗 | Element Plus Popconfirm |

### 交互流程（与旧版一致）

```
选择服务器 → 选择 DB → 浏览键列表 → 点击键查看值 → 编辑/删除
     ↓           ↓          ↓              ↓
   设置界面    新增DB     搜索/筛选     新增键/导入/导出
```

### 理由

1. **用户习惯**：延续旧版交互模式，降低迁移成本
2. **参考素材**：57 张 UI 截图提供完整的视觉参考
3. **组件映射**：iView → Element Plus 的组件映射关系明确

### 后果

- 新版 UI 与旧版高度相似，用户无需重新学习
- Element Plus 的默认样式与旧版略有差异，需要自定义主题覆盖

---

## 参考文档

| 文档 | 路径 | 说明 |
|------|------|------|
| 旧版架构设计 | `docs/design.md` | PHP 版本的完整架构设计 |
| 旧版代码 Wiki | `docs/CODE_WIKI.md` | PHP 版本的代码结构和 API 文档 |
| UI 设计分析 | `docs/UI.md` | UI 设计分析和页面流逻辑 |
| 用户帮助文档 | `help/HELP.md` | 图文并茂的使用帮助（含 57 张截图） |
| UI 截图目录 | `help/ui/` | 旧版所有 UI 界面截图 |
| 开发计划 | `.trae/documents/` | 各阶段的开发计划文档 |
