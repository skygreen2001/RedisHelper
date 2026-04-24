# Redis小助手

Redis小助手是一个基于Tauri 2.0框架开发的跨平台Redis管理工具，提供简单、直观的Redis服务器管理和数据操作功能。

## 功能特性

- **多服务器管理**：添加、编辑、删除Redis服务器配置
- **数据库管理**：切换和管理不同的Redis数据库
- **键值操作**：查看、添加、编辑、删除Redis键值
- **数据类型支持**：支持String、List、Set、ZSet、Hash五种核心数据类型
- **搜索功能**：按模式搜索键
- **数据导入导出**：支持批量操作数据
- **配置存储**：本地存储服务器配置

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
或者 npm run tauri:build 
```
该命令会根据你当前的操作系统生成对应的安装包。如果你在 Windows 上运行这个命令，它会生成一个 Windows 的安装包；在 macOS 上运行，则会生成一个 macOS 的安装包；在 Linux 上运行，则会生成一个适用于 Linux 的安装包。


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

> **注意**：交叉编译需要额外安装 `llvm` 和 `mingw-w64` 工具链。构建产物为 `.exe` 文件，位于 `src-tauri/target/x86_64-pc-windows-gnu/release/`

#### 构建所有平台

```bash
npm run tauri:build:all
```

### 构建产物位置

- macOS: `src-tauri/target/release/bundle/macos/`
- Windows: `src-tauri/target/release/bundle/msi/`

## 使用方法

1. **添加服务器**：在"服务器设置"页面添加Redis服务器配置
2. **连接服务器**：在主界面选择服务器和数据库
3. **管理键值**：查看、添加、编辑、删除键值
4. **搜索键**：使用搜索框按模式搜索键
5. **导入导出**：导出数据为JSON格式或从JSON文件导入数据

## 首次安装体验

应用首次安装时不会包含任何预配置的服务器，包括本地服务器。用户需要：

1. 点击"设备"下拉菜单
2. 选择"设置"
3. 手动添加 Redis 服务器配置

## 项目结构

```
redis/
├── src/
│   └── frontend/       # 前端代码
│       ├── views/         # 页面
│       ├── stores/        # 状态管理
│       ├── App.vue        # 根组件
│       └── main.ts        # 入口文件
├── src-tauri/           # Rust后端代码
│   ├── src/           # Rust源码
│   │   ├── commands/  # Tauri命令
│   │   ├── redis/     # Redis操作
│   │   ├── storage/   # 存储管理
│   │   └── main.rs    # 主入口
│   ├── icons/           # 应用图标
│   ├── Cargo.toml     # Rust依赖
│   └── tauri.conf.json    # Tauri配置
├── index.html             # HTML模板
├── package.json           # 前端依赖
├── tsconfig.json          # TypeScript配置
├── tsconfig.node.json    # TypeScript Node配置
├── vite.config.ts         # Vite配置
└── README.md              # 项目说明
```

## 注意事项

- 本应用使用本地JSON文件存储服务器配置，配置文件位于 `~/.redis-helper/config.json`
- 导入导出功能暂时使用默认路径 `/tmp/redis-export.json`，后续版本将支持文件选择对话框
- 应用需要网络权限以连接Redis服务器

## 开发注意事项

- 确保本地有 Redis 服务器用于测试
- 测试服务器连接功能正常
- 验证数据导入导出功能

## 版本历史

- v0.1.0：初始版本，实现核心功能

## 许可证

MIT License

## 贡献

欢迎贡献代码！请提交Pull Request或打开Issue。

## 贡献指南

1. **提交Pull Request**：在项目仓库打开Pull Request，描述你的贡献内容。
2. **打开Issue**：如果有问题或建议，打开Issue并详细描述。
3. **代码规范**：请遵循Vue 3、TypeScript、Element Plus、Pinia的代码规范。
4. **测试**：在本地测试你的贡献，确保不会引入新的问题。
5. **文档**：如果有必要，更新项目文档。

## 贡献者

- [TAURI](https://tauri.app/): https://tauri.app/