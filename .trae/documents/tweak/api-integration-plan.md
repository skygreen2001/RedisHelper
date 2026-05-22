# 🔧 前后端 API 对接计划

## 📊 现状分析

### 当前实现情况

1. **前端**：
   - 使用 Vue 3 + TypeScript + Pinia
   - 已实现完整的 API 调用结构
   - 目前在非 Tauri 环境下使用模拟数据作为 fallback
   - 关键文件：
     - [redisStore.ts](file:///Library/WebServer/Documents/redis/src/frontend/stores/redisStore.ts)
     - [serverStore.ts](file:///Library/WebServer/Documents/redis/src/frontend/stores/serverStore.ts)
     - [MainView.vue](file:///Library/WebServer/Documents/redis/src/frontend/views/MainView.vue)

2. **后端**：
   - 使用 Rust + Tauri
   - 已实现所有必要的 Redis 命令
   - 关键文件：
     - [redis.rs](file:///Library/WebServer/Documents/redis/src-tauri/src/commands/redis.rs)
     - [server.rs](file:///Library/WebServer/Documents/redis/src-tauri/src/commands/server.rs)
     - [connection.rs](file:///Library/WebServer/Documents/redis/src-tauri/src/redis/connection.rs)

3. **权限配置**：
   - 已在 [tauri.capabilities.json](file:///Library/WebServer/Documents/redis/src-tauri/tauri.capabilities.json) 中配置了所有命令的权限

### 测试环境
- 本地 Redis 服务器：
  - Host: 127.0.0.1
  - Port: 8368
  - Password: 7htVhQrFP2big7NiBde5fpEEdUaa

## 🛠️ 实施计划

### 第一步：优化前端 API 调用

**目标**：去除静态模拟数据，确保在 Tauri 环境下直接调用后端 API，在浏览器环境下提供更好的错误处理。

**修改文件**：
- [redisStore.ts](file:///Library/WebServer/Documents/redis/src/frontend/stores/redisStore.ts)
- [serverStore.ts](file:///Library/WebServer/Documents/redis/src/frontend/stores/serverStore.ts)

**修改内容**：
1. 移除所有模拟数据 fallback
2. 增强错误处理，提供更友好的错误提示
3. 确保 API 调用参数与后端一致

### 第二步：添加本地 Redis 服务器配置

**目标**：默认添加本地 Redis 服务器配置，方便测试。

**修改文件**：
- [serverStore.ts](file:///Library/WebServer/Documents/redis/src/frontend/stores/serverStore.ts)

**修改内容**：
1. 在初始化时自动添加本地 Redis 服务器配置
2. 确保配置信息正确（host: 127.0.0.1, port: 8368, password: 7htVhQrFP2big7NiBde5fpEEdUaa）

### 第三步：增强错误处理和用户反馈

**目标**：提供更好的错误处理和用户反馈，确保操作失败时用户能看到明确的错误信息。

**修改文件**：
- [MainView.vue](file:///Library/WebServer/Documents/redis/src/frontend/views/MainView.vue)
- [ServerConfigView.vue](file:///Library/WebServer/Documents/redis/src/frontend/views/ServerConfigView.vue)

**修改内容**：
1. 添加错误提示组件
2. 在操作失败时显示错误信息
3. 优化 UI 反馈，确保用户知道操作状态

### 第四步：测试所有功能

**目标**：确保所有功能都能正常工作，包括：

1. **服务器管理**：
   - 添加服务器
   - 编辑服务器
   - 删除服务器
   - 测试连接

2. **Redis 操作**：
   - 连接到 Redis
   - 查看数据库列表
   - 查看键列表
   - 查看键值
   - 设置键值
   - 删除键
   - 搜索键
   - 创建数据库
   - 删除数据库

3. **数据导入导出**：
   - 导出数据
   - 导入数据

### 第五步：构建和验证

**目标**：确保生产构建能正常工作。

**操作步骤**：
1. 清理旧的构建文件
2. 运行 `npm run tauri build`
3. 测试构建后的应用
4. 验证所有功能在生产环境中正常工作

## 📝 详细修改内容

### 1. redisStore.ts 修改

- 移除所有 `else` 分支中的模拟数据
- 增强错误处理，确保错误能正确传递给前端
- 确保 API 调用参数与后端一致

### 2. serverStore.ts 修改

- 移除模拟数据
- 在初始化时添加本地 Redis 服务器配置
- 增强错误处理

### 3. MainView.vue 修改

- 添加错误提示组件
- 在操作失败时显示错误信息
- 优化 UI 反馈

## ⚠️ 风险和注意事项

1. **Redis 连接失败**：
   - 确保本地 Redis 服务器已启动
   - 检查端口和密码是否正确

2. **API 调用错误**：
   - 确保前后端参数格式一致
   - 检查权限配置是否正确

3. **构建失败**：
   - 确保所有依赖已正确安装
   - 检查 TypeScript 类型错误

## ✅ 验证标准

修复成功的标志：
- 能成功连接到本地 Redis 服务器
- 能正常执行所有 Redis 操作
- 操作失败时显示明确的错误信息
- 生产构建能正常工作
- 所有功能都能在 Tauri 环境中正常运行