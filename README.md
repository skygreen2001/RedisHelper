# Redis小助手

Redis小助手是一个基于Tauri 2.0框架开发的跨平台Redis管理工具，提供简单、直观的Redis服务器管理和数据操作功能。

## 支持平台

- **MacOS 客户端** (Tauri 构建)
- **Windows 客户端** (Tauri 构建)
- **Web 浏览器** (Web服务器模式)

## 功能特性

- **多服务器管理**：添加、编辑、删除Redis服务器配置，支持Redis 6.0+ ACL用户名认证
- **数据库管理**：切换和管理不同的Redis数据库，支持新增/删除DB
- **键值操作**：查看、添加、编辑、删除Redis键值
- **数据类型支持**：支持String、List、Set、ZSet、Hash五种核心数据类型
- **元素级编辑**：List/Set/ZSet/Hash支持覆盖模式和元素编辑模式，表格化增删改查单个元素
- **搜索功能**：按模式搜索键（支持*通配符）
- **排序功能**：按键名升序或降序排列，ZSet支持按Score/Member排序
- **多选删除**：支持批量选择键并删除废键箱
- **废键箱**：已删除键的临时存储，支持恢复或永久删除（7天自动清理）
- **数据导入导出**：支持JSON格式批量导入导出
- **多窗口多标签**：支持多标签页浏览和操作系统级多窗口，每个标签拥有独立会话
- **Redis 内存分析**：内存使用仪表盘、键类型分布统计、大键排行榜（Pipeline优化，游标分页）
- **Redis 监控面板**：服务器信息、内存状态、键统计等实时数据展示
- **Redis 慢日志**：慢查询日志查看，支持搜索过滤，帮助定位性能瓶颈
- **操作审计日志**：记录所有键值读写操作（GET/SET/DEL），支持筛选、分页、统计分析，可全局开关
- **服务器配置导入导出**：支持JSON格式批量导入导出服务器连接配置
- **浏览器兼容模式**：支持在Web浏览器中运行（通过WebSocket代理）
- **中文原生菜单**：macOS平台中文菜单栏支持
- **可拖拽分隔条**：键列表与值展示区宽度可自由调整
- **快捷键支持**：Ctrl+A全选、Ctrl+Shift+A全选所有、Esc退出多选、Ctrl+T新建标签、Ctrl+W关闭标签

## 技术栈

- **前端**：Vue 3 + TypeScript + Element Plus + Pinia
- **后端**：Rust + Tauri 2.0 + redis-rs
- **构建工具**：Vite

## 环境要求

- Node.js 18+
- npm 或 yarn
- Rust 1.70+ (Tauri 需要)
- 对于 Windows 构建，需要在 Windows 环境或在 MacOS 上使用 Homebrew 安装 Rust 并配置交叉编译

## 安装步骤

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装Tauri CLI（如果未安装）
npm install -g @tauri-apps/cli
```

### 开发模式运行

#### Tauri 桌面端开发

```bash
# 启动 Tauri 开发服务器
npm run tauri:dev
```

#### Web 浏览器开发

浏览器无法直接连接 Redis（TCP 协议），需要通过 WebSocket 代理转发请求。

**1. 启动开发服务器**

```bash
npm run dev
```

**2. 访问应用**

打开浏览器访问 `http://localhost:5173`

> **说明**：`npm run dev` 会自动启动 WebSocket 代理服务，Vite 关闭时代理也会自动终止。

**浏览器环境与 Tauri 环境差异**

| 功能 | Tauri 环境 | 浏览器环境 |
|------|-----------|-----------|
| Redis 连接 | Rust 后端直接连接 | WebSocket 代理转发 |
| 服务器配置存储 | 本地 JSON 文件 | localStorage |
| 废键箱存储 | 本地 JSON 文件 | localStorage |
| 文件导入导出 | 系统文件对话框 | 浏览器下载/文件选择 |
| 配置持久化 | 跨浏览器共享 | 仅当前浏览器 |

### 构建桌面应用

#### 操作系统自适应

```bash
npm run tauri build
# 或者 npm run tauri:build 
```

该命令会根据你当前的操作系统生成对应的安装包。

#### MacOS 客户端

```bash
npm run tauri:build:mac
```

#### Windows 客户端

在 Windows 环境下直接运行：

```bash
npm run tauri:build:windows
```

**在 MacOS 上交叉编译（需要提前安装依赖）：**

```bash
# 安装必要的工具
brew install llvm mingw-w64

# 设置环境变量
export PATH="/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/mingw-w64/bin:$PATH"

# 构建 Windows 版本
npm run tauri:build:windows
```

> **注意**：交叉编译需要额外安装 `llvm` 和 `mingw-w64` 工具链。

#### 构建所有桌面平台

```bash
npm run tauri:build:all
```

### 构建Web 浏览器应用

Web 浏览器应用不支持独立部署（需要 WebSocket 代理连接 Redis）。

```bash
npm run build
```

构建产物位于 `dist/` 目录，为纯静态文件（HTML/CSS/JS）。如需部署，需要：

1. 将 `dist/` 部署到 Web 服务器
2. 在服务器端运行 WebSocket 代理：`node run proxy`
3. 代理服务器需与 Redis 网络互通

> **注意**：Web 版本的功能与桌面端完全一致，但服务器配置和废键箱数据存储在浏览器 localStorage 中，不同用户/浏览器间不共享。

### 构建产物位置

- macOS 桌面端: `src-tauri/target/release/bundle/macos/`
- Windows 桌面端: `src-tauri/target/release/bundle/msi/`
- Web 浏览器版: `dist/`

## 使用方法

### 首次使用

1. 点击顶部菜单栏"连接" → "设置"
2. 添加 Redis 服务器配置
3. 返回主界面选择服务器和数据库

### 基本操作

- **查看键值**：点击左侧键列表中的键
- **添加键**：点击搜索栏右侧的"+"按钮
- **搜索键**：在搜索框输入关键词，支持*通配符
- **排序键**：点击顶部菜单栏"排序"，选择升序或降序
- **编辑键值**：在右侧值展示区修改后点击"修改"
- **删除键**：在右侧值展示区点击"删除"

### 多选删除

1. 点击顶部菜单栏"多选"按钮
2. 键列表显示复选框，勾选需要删除的键
3. 使用"全选"按钮可快速选择所有键
4. 点击"删除(N)"按钮将选中的键删除废键箱
5. 按 Esc 或再次点击"多选"退出多选模式

**快捷键**：
- `Ctrl+A`：全选当前已加载的键
- `Ctrl+Shift+A`：全选所有键（先加载全部）
- `Esc`：退出多选模式

### 废键箱

1. 点击顶部菜单栏"DB" → "废键箱"
2. 查看已删除的键列表
3. 勾选需要恢复的键，点击"恢复"按钮
4. 或点击"永久删除"彻底删除键
5. 废键箱中的键7天后自动清理

### 数据导入导出

1. 点击顶部菜单栏"更多"
2. 选择"导入"或"导出"
3. 按提示选择文件或确认导出

## 项目结构

```
redis/
├── src/
│   └── frontend/           # 前端代码
│       ├── views/          # 页面（MainView, HomeView, ServerConfigView）
│       ├── components/     # 组件（TabBar, MemoryDialog, LogDialog）
│       ├── sessions/       # 会话管理（Session, SessionManager）
│       ├── stores/         # 状态管理 (Pinia)
│       ├── adapters/       # 浏览器适配层
│       ├── utils/          # 工具函数
│       ├── App.vue         # 根组件
│       └── main.ts         # 入口文件
├── src-tauri/              # Rust后端代码
│   ├── src/
│   │   ├── commands/       # Tauri命令
│   │   ├── redis/          # Redis操作
│   │   ├── storage/        # 存储管理
│   │   └── main.rs         # 主入口
│   ├── Cargo.toml          # Rust依赖
│   └── tauri.conf.json     # Tauri配置
├── server/                 # WebSocket代理服务
│   └── ws-proxy.js         # WebSocket Redis代理 + 静态文件服务
├── docs/                   # 设计文档
│   ├── UI.md               # UI设计文档
│   ├── DESIGN_DECISIONS.md # 设计决策记录
│   ├── CODE_WIKI.md        # 代码Wiki
│   └── design.md           # 架构设计文档
├── help/                   # 用户帮助文档
│   └── HELP.md             # 使用帮助
├── index.html              # HTML模板
├── package.json            # 前端依赖
├── vite.config.ts          # Vite配置
└── README.md               # 项目说明
```

## 配置存储

### Tauri 桌面端

- 服务器配置：`~/.redis-helper/config.json`
- 废键箱数据：`~/.redis-helper/trash.json`

### Web 浏览器

- 服务器配置：localStorage (`redis-helper-servers`)
- 废键箱数据：localStorage (`redis-helper-trash`)
- 数据仅存储在当前浏览器，清除浏览器数据会丢失配置

## 注意事项

### 通用注意事项

- 本应用使用本地JSON文件存储配置和废键箱数据
- 导入导出功能使用系统文件选择对话框
- 应用需要网络权限以连接Redis服务器
- 废键箱中的键7天后自动清理，请及时恢复需要的键

### Web 开发注意事项

- WebSocket 代理默认端口为 `8765`，确保端口未被占用
- 代理服务仅在开发模式下自动启动，生产构建需要 Tauri 环境
- 浏览器环境下的配置存储在 localStorage，不同浏览器间不共享
- 如需远程 Redis 服务器，确保网络可达且防火墙允许连接

## 开发注意事项

- 确保本地有 Redis 服务器用于测试
- 测试服务器连接功能正常
- 验证数据导入导出功能
- 测试多选删除和废键箱功能

## 版本历史

- **v1.3.0** (2026-07)：新增元素级编辑功能（List/Set/ZSet/Hash表格化增删改查）、Redis ACL用户名认证支持（Redis 6.0+）、操作审核全局开关配置、内存分析性能优化（Pipeline合并+游标分页+异步类型分布）、慢日志功能修复与优化（IPC参数格式/噪音过滤/耗时单位）、macOS编辑菜单优化、代码重构与修复
- **v1.2.0** (2026-05-22)：新增多窗口多标签会话管理、浏览器兼容模式（Web平台支持）、Redis内存分析功能（内存仪表盘/键类型分布/大键排行）、Redis监控面板（服务器信息/内存状态/键统计）、Redis慢日志查看、操作审计日志（GET/SET/DEL操作记录与统计分析）、服务器配置导入导出、中文原生菜单支持、键类型分布统计、可拖拽分隔条、键值详情刷新按钮、调试日志全局控制；重构标签栏状态管理、服务器选择UI、后端接口参数格式统一；修复Redis SCAN游标未更新问题
- **v1.1.0** (2025-05-10)：新增多选排序删除和废键箱功能、服务器只读模式、删除DB，清空等操作二次确认安全保护、优化搜索功能，优化UI布局
- **v1.0.0** (2025-04)：初始版本，实现核心功能

## 文档

- [代码Wiki](docs/CODE_WIKI.md) - 代码结构与模块详解
- [UI设计文档](docs/UI.md) - 界面设计规范与交互流程
- [设计决策记录](docs/DESIGN_DECISIONS.md) - 架构决策记录（ADR）
- [架构设计文档](docs/design.md) - 旧版架构设计文档
- [用户帮助文档](help/HELP.md) - 图文使用帮助

## 许可证

MIT License

## 贡献

欢迎贡献代码！请提交Pull Request或打开Issue。

## 技术致谢

- [Vue 3](https://vuejs.org/)
- [Element Plus](https://element-plus.org/)
- [Pinia](https://pinia.vuejs.org/)
- [TypeScript](https://www.typescriptlang.org/)
- [Rust](https://www.rust-lang.org/)
- [Tauri](https://tauri.app/)
- [Trae](https://trae.cn/)
