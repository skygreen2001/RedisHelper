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

## 安装步骤

### 环境要求

- Node.js v16+ 
- Rust 最新稳定版
- Tauri CLI

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
npm run tauri dev
```

### 构建应用

```bash
# 构建前端
npm run build

# 打包应用
npm run tauri build
```

## 使用方法

1. **添加服务器**：在"服务器设置"页面添加Redis服务器配置
2. **连接服务器**：在主界面选择服务器和数据库
3. **管理键值**：查看、添加、编辑、删除键值
4. **搜索键**：使用搜索框按模式搜索键
5. **导入导出**：导出数据为JSON格式或从JSON文件导入数据

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

## 版本历史

- v0.1.0：初始版本，实现核心功能

## 许可证

MIT License