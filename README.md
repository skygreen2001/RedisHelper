# Redis小助手

Redis小助手是一个基于Tauri 2.0框架开发的跨平台Redis管理工具，提供简单、直观的Redis服务器管理和数据操作功能。

## 功能特性

- **多服务器管理**：添加、编辑、删除Redis服务器配置
- **数据库管理**：切换和管理不同的Redis数据库，支持新增/删除DB
- **键值操作**：查看、添加、编辑、删除Redis键值
- **数据类型支持**：支持String、List、Set、ZSet、Hash五种核心数据类型
- **搜索功能**：按模式搜索键（支持*通配符）
- **排序功能**：按键名升序或降序排列
- **多选删除**：支持批量选择键并移入废键箱
- **废键箱**：已删除键的临时存储，支持恢复或永久删除（7天自动清理）
- **数据导入导出**：支持JSON格式批量导入导出
- **快捷键支持**：Ctrl+A全选、Ctrl+Shift+A全选所有、Esc退出多选

## 技术栈

- **前端**：Vue 3 + TypeScript + Element Plus + Pinia
- **后端**：Rust + Tauri 2.0 + redis-rs
- **构建工具**：Vite

## 支持平台

- **macOS 客户端** (Tauri 构建)
- **Windows 客户端** (Tauri 构建)

## 环境要求

- Node.js 18+
- npm 或 yarn
- Rust 1.70+ (Tauri 需要)
- 对于 Windows 构建，需要在 Windows 环境或配置交叉编译

## 安装步骤

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装Tauri CLI（如果未安装）
npm install -g @tauri-apps/cli
```

### 开发模式运行

```bash
# 启动开发服务器
npm run tauri:dev
```

### 构建应用

#### 操作系统自适应

```bash
npm run tauri build
# 或者 npm run tauri:build 
```

该命令会根据你当前的操作系统生成对应的安装包。

#### macOS 客户端

```bash
npm run tauri:build:mac
```

#### Windows 客户端

在 Windows 环境下直接运行：

```bash
npm run tauri:build:windows
```

**在 macOS 上交叉编译（需要提前安装依赖）：**

```bash
# 安装必要的工具
brew install llvm mingw-w64

# 设置环境变量
export PATH="/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/mingw-w64/bin:$PATH"

# 构建 Windows 版本
npm run tauri:build:windows
```

> **注意**：交叉编译需要额外安装 `llvm` 和 `mingw-w64` 工具链。

#### 构建所有平台

```bash
npm run tauri:build:all
```

### 构建产物位置

- macOS: `src-tauri/target/release/bundle/macos/`
- Windows: `src-tauri/target/release/bundle/msi/`

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
4. 点击"移入(N)"按钮将选中的键移入废键箱
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
│       ├── views/          # 页面
│       ├── stores/         # 状态管理 (Pinia)
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
├── docs/                   # 设计文档
│   ├── UI.md               # UI设计文档
│   ├── DESIGN_DECISIONS.md # 设计决策记录
│   └── design.md           # 架构设计文档
├── index.html              # HTML模板
├── package.json            # 前端依赖
├── vite.config.ts          # Vite配置
└── README.md               # 项目说明
```

## 配置存储

- 服务器配置：`~/.redis-helper/config.json`
- 废键箱数据：`~/.redis-helper/trash.json`

## 注意事项

- 本应用使用本地JSON文件存储配置和废键箱数据
- 导入导出功能使用系统文件选择对话框
- 应用需要网络权限以连接Redis服务器
- 废键箱中的键7天后自动清理，请及时恢复需要的键

## 开发注意事项

- 确保本地有 Redis 服务器用于测试
- 测试服务器连接功能正常
- 验证数据导入导出功能
- 测试多选删除和废键箱功能

## 版本历史

- **v1.1.0** (2025-05-10)：新增多选删除和废键箱功能，优化UI布局
- **v1.0.0** (2025-04)：初始版本，实现核心功能

## 文档

- [UI设计文档](docs/UI.md)
- [设计决策记录](docs/DESIGN_DECISIONS.md)
- [架构设计文档](docs/design.md)

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
