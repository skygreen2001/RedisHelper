# 计划：生成设计决策记录文档

## 摘要

将本次会话中关于 Redis 管理工具的完整设计决策过程整理为一份 Markdown 设计决策记录文档（ADR），记录从旧版 PHP 项目到新版 Tauri 桌面应用的架构演进历程、关键技术决策及其理由。

## 当前状态分析

### 项目现状
- **项目名称**：Redis小助手 (redis-helper)
- **当前技术栈**：Tauri 2.0 + Vue 3 + TypeScript + Element Plus + Pinia + Rust + redis-rs
- **已有文档**：
  - `docs/design.md` — 旧版 PHP 架构设计文档（v1.0）
  - `docs/CODE_WIKI.md` — 旧版 PHP 代码 Wiki
  - `docs/UI.md` — UI 设计分析文档
  - `help/HELP.md` — 用户使用帮助文档
  - `.trae/documents/` — 5 份开发计划文档

### 会话决策历程（需整理的关键决策）

1. **初始架构设计**：Go + React + PostgreSQL 后端架构（v1.0 设计文档）
2. **转向 Electron**：用户要求使用 Electron 框架，不要后端
3. **技术选型确认**：Vue 3 + Element Plus + ioredis + electron-store + 内嵌 HTTP 服务
4. **开发边界控制**：严格对齐旧版已有功能，裁剪超范围功能（命令终端、Cluster/Sentinel、TLS 等）
5. **双端部署需求**：一套代码同时支持桌面客户端和 Web 服务端
6. **最终实现**：实际采用 Tauri 2.0 + Rust + Vue 3 方案（已开发完成）

## 拟定变更

### 创建文件
- **文件路径**：`docs/DESIGN_DECISIONS.md`
- **内容结构**：

```
# Redis 管理工具 — 设计决策记录 (ADR)

## 概述
- 项目演进时间线
- 文档目的

## 决策记录

### ADR-001: 桌面框架选型
- 状态：已决定
- 背景：旧版 PHP 需 Web 服务器，用户希望桌面客户端
- 候选方案：Electron vs Tauri 2.0
- 决策：采用 Tauri 2.0
- 理由：性能更好、包体积更小、安全性更高（Rust 后端）
- 后果：需要 Rust 开发能力，但换来更好的用户体验

### ADR-002: 前端框架选型
- 状态：已决定
- 背景：旧版使用 Vue 2 + iView
- 候选方案：Vue 3 + Element Plus vs React + Ant Design vs Vue 2 + iView
- 决策：Vue 3 + Element Plus
- 理由：与旧版风格接近，迁移成本最低

### ADR-003: 后端架构决策
- 状态：已决定
- 背景：从 PHP 后端到无独立后端
- 候选方案：Go 后端 + PostgreSQL vs Electron 主进程 + electron-store vs Tauri Rust 后端
- 决策：Tauri Rust 后端 + JSON 文件存储
- 理由：无需外部依赖，桌面应用内嵌后端

### ADR-004: 开发边界控制
- 状态：已决定
- 背景：初始设计包含过多新功能
- 决策：v2.0 严格对齐旧版已有功能
- 范围内：服务器管理、DB 管理、键 CRUD、搜索、导入导出
- 范围外：命令终端、Cluster/Sentinel、TLS、监控面板、深色主题

### ADR-005: 双端部署架构
- 状态：已搁置（Tauri 方案下重新评估）
- 背景：用户希望一套代码同时支持桌面和 Web
- 候选方案：环境适配层（IPC/HTTP 自动切换）
- 当前状态：Tauri 方案下暂未实现 Web 端

### ADR-006: Redis 客户端选型
- 状态：已决定
- 候选方案：ioredis vs node-redis vs redis-cli（Node.js 方案下）
- 最终实现：redis-rs（Rust Redis 客户端）
- 理由：Tauri 方案下使用 Rust 原生 Redis 客户端

### ADR-007: 配置存储方案
- 状态：已决定
- 候选方案：electron-store vs SQLite vs JSON 文件
- 最终实现：JSON 文件（~/.redis-helper/config.json）
- 理由：简单轻量，无需额外依赖

### ADR-008: UI 设计延续
- 状态：已决定
- 决策：保持旧版 UI 风格（双栏布局、深色导航栏、蓝色主色调）
- 参考：help/ui/ 目录下 57 张截图

## 架构演进时间线
- v1.0（旧版）：PHP + Vue 2 + iView + PhpRedis
- 设计阶段 v1.0：Go + React + PostgreSQL（未实施）
- 设计阶段 v2.0：Electron + Vue 3 + Element Plus + ioredis（未实施）
- 设计阶段 v3.0：Electron 双端部署 + 环境适配层（未实施）
- 当前实现：Tauri 2.0 + Vue 3 + Element Plus + Rust + redis-rs

## 参考文档
- docs/design.md — 旧版架构设计
- docs/CODE_WIKI.md — 旧版代码 Wiki
- docs/UI.md — UI 设计分析
- help/HELP.md — 用户帮助文档
```

## 假设与决策

- 文档语言：中文（与用户输入一致）
- 文档格式：Markdown
- 文档位置：`docs/DESIGN_DECISIONS.md`（与现有文档放在一起）
- 内容来源：本次会话的完整对话历史 + 现有项目文件

## 验证步骤

1. 确认 `docs/DESIGN_DECISIONS.md` 文件已创建
2. 检查文档包含所有 8 个 ADR 决策记录
3. 检查架构演进时间线完整覆盖从旧版到当前实现的全部阶段
4. 检查 Markdown 格式正确（标题层级、表格语法、代码块闭合）
5. 确认文档与现有项目实际状态一致（Tauri 而非 Electron）
