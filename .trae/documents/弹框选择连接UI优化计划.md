# 弹框选择连接UI优化计划

## 任务概述

将新建标签页时的服务器选择从内嵌卡片形式改为弹框（对话框）形式，提升用户体验。

## 当前状态分析

### 现有实现
- **状态控制**：`Session.isSelectingServer` 控制显示/隐藏
- **UI形式**：全屏覆盖的内嵌卡片（`select-server-view` + `select-server-card`）
- **位置**：`MainView.vue` 第 131-157 行

### 存在的问题
1. 占用整个内容区域，视觉突兀
2. 无法在不关闭的情况下与其他标签页交互
3. 不符合现代 UI 设计模式

## 方案设计

使用 Element Plus 的 `el-dialog` 组件替换内嵌卡片，保持相同的业务逻辑。

### 弹框行为设计

| 场景 | 行为 |
|------|------|
| 点击新建标签页 | 弹框显示（`isSelectingServer = true`） |
| 选择服务器 | 弹框关闭，加载数据库 |
| 点击取消按钮 | 如果有多标签则关闭当前标签，否则仅关闭弹框 |
| 点击遮罩层 | 不关闭（`close-on-click-modal: false`） |
| 按 ESC 键 | 仅当有多标签时可关闭 |
| 点击右上角关闭 | 仅当有多标签时显示关闭按钮 |

## 具体改动

### 文件：MainView.vue

#### 1. 删除内嵌卡片代码（第 131-157 行）
删除以下内容：
```vue
<!-- 新建标签页 - 选择连接 -->
<div v-if="sessionManager.active.isSelectingServer" class="select-server-view">
  ...
</div>
```

#### 2. 添加 el-dialog 弹框
在其他对话框附近（如服务器配置对话框之后）添加：
```vue
<!-- 选择连接对话框 -->
<el-dialog
  v-model="sessionManager.active.isSelectingServer"
  title="选择连接"
  width="450px"
  :close-on-click-modal="false"
  :show-close="sessionManager.sessions.length > 1"
  :close-on-press-escape="sessionManager.sessions.length > 1"
  align-center
  destroy-on-close
>
  <p class="select-server-hint">请选择一个服务器连接以创建标签页</p>
  <div class="select-server-list">
    <div
      v-for="server in servers"
      :key="server.id"
      class="select-server-item"
      @click="handleNewTabSelectServer(server)"
    >
      <div class="server-item-info">
        <span class="server-item-name">{{ server.name }}</span>
        <span class="server-item-addr">{{ server.host }}:{{ server.port }}</span>
      </div>
    </div>
    <div v-if="servers.length === 0" class="select-server-empty">
      <p>暂无可用连接</p>
      <el-button type="primary" size="small" @click="openServerConfigFromDialog">前往设置</el-button>
    </div>
  </div>
  <template #footer>
    <span class="dialog-footer">
      <el-button 
        v-if="sessionManager.sessions.length > 1" 
        @click="handleCancelNewTab"
      >
        取消
      </el-button>
      <span v-else></span>
    </span>
  </template>
</el-dialog>
```

#### 3. 修改脚本函数

**修改 `handleNewTabSelectServer`**：
```typescript
const handleNewTabSelectServer = async (server: any) => {
  const session = sessionManager.active
  session.isSelectingServer = false  // 关闭弹框
  session.selectedServer = server
  session.updateTitle()
  await loadDatabases()
}
```

**修改 `handleCancelNewTab`**：
```typescript
const handleCancelNewTab = () => {
  const session = sessionManager.active
  if (sessionManager.sessions.length > 1) {
    sessionManager.closeSession(session.id)
  } else {
    session.isSelectingServer = false
  }
}
```

**添加新函数**：
```typescript
const openServerConfigFromDialog = () => {
  sessionManager.active.isSelectingServer = false
  sessionManager.active.showServerConfig = true
}
```

#### 4. 修改样式

**保留以下样式**（用于对话框内的列表样式）：
- `.select-server-hint`
- `.select-server-list`
- `.select-server-item`
- `.server-item-info`
- `.server-item-name`
- `.server-item-addr`
- `.select-server-empty`

**删除以下样式**（不再需要）：
- `.select-server-view`
- `.select-server-card`
- `.select-server-card h3`
- `.select-server-actions`

## 验证步骤

1. 点击新建标签页（Ctrl+T 或菜单）→ 弹框显示
2. 选择服务器 → 弹框关闭，标签页标题更新，加载数据库
3. 点击取消 → 如果有多标签则关闭当前标签
4. 无可用连接时 → 显示"前往设置"按钮，点击打开服务器配置
5. 应用启动时 → 自动连接第一个服务器，不显示弹框
