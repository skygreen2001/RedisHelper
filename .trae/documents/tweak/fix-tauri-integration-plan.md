# 🔧 修复 Tauri 前后端集成问题

## 📊 问题分析

### 当前问题
运行 `npm run tauri build` 后，前端和后端没有正常对接。

### 🔍 问题根因

1. **Tauri 2.0 权限配置问题**
   - [tauri.capabilities.json](file:///Library/WebServer/Documents/redis/src-tauri/tauri.capabilities.json) 中缺少对自定义命令的权限配置
   - Tauri 2.0 要求明确配置哪些命令可以被前端调用

2. **其他可能的问题**
   - 前端的环境检测可能有问题
   - 命令注册可能有问题

---

## 🛠️ 修复计划

### 第一步：修复权限配置文件

**文件：** `src-tauri/tauri.capabilities.json`

**修改内容：**
- 添加我们所有自定义命令的权限配置
- 确保所有后端命令都能被前端调用

**需要添加的命令：**
```
server:add_server
server:edit_server
server:delete_server
server:get_servers
server:test_connection
redis:connect
redis:get_databases
redis:get_keys
redis:get_key_value
redis:set_key_value
redis:delete_key
redis:search_keys
redis:create_database
redis:delete_database
export:export_data
export:import_data
```

---

### 第二步：改进 Tauri 检测机制

**文件：** 
- `src/frontend/stores/serverStore.ts`
- `src/frontend/stores/redisStore.ts`
- `src/frontend/views/MainView.vue`

**修改内容：**
- 简化检测逻辑，直接尝试调用后端命令而不检测
- 或者添加更详细的调试输出
- 确保在打包环境中正常工作

---

### 第三步：检查并确保命令正确注册

**文件：** `src-tauri/src/main.rs`

**验证内容：**
- 确保所有命令都已正确注册
- 检查命令格式是否符合 Tauri 2.0 要求

---

### 第四步：测试修复

**操作步骤：**
1. 清理旧的构建文件
2. 重新运行 `npm run tauri build`
3. 测试新打包的应用
4. 查看状态栏的调试信息
5. 检查功能是否正常工作

---

## 📝 详细修改内容

### 1. tauri.capabilities.json 修改

```json
{
  "$schema": "../node_modules/@tauri-apps/cli/schemas/desktop-capabilities.schema.json",
  "identifier": "main",
  "description": "Redis Helper main capabilities",
  "windows": ["main"],
  "permissions": [
    // ... 现有权限 ...
    {
      "name": "core:allow-command",
      "allow": ["add_server", "edit_server", "delete_server", "get_servers", "test_connection"]
    },
    {
      "name": "core:allow-command",
      "allow": ["connect", "get_databases", "get_keys", "get_key_value", "set_key_value", "delete_key", "search_keys", "create_database", "delete_database"]
    },
    {
      "name": "core:allow-command",
      "allow": ["export_data", "import_data"]
    }
  ]
}
```

### 2. 简化 Tauri 检测

在 stores 中，先尝试直接调用后端命令，如果失败再用模拟数据。

---

## ⚠️ 风险和注意事项

1. **权限配置格式**：Tauri 2.0 的权限配置可能有特定的格式要求
2. **命令命名**：需要确保权限配置中的命令名与实际注册的一致
3. **向后兼容**：确保修改不破坏 dev 模式

---

## ✅ 验证标准

修复成功的标志：
- 状态栏显示 "✓ Tauri环境"
- 服务器配置能正确保存和加载
- Redis 连接测试能正常工作
- 键值对操作能正常执行
