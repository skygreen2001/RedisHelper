# 多选删除功能实现计划

## 摘要

为 Redis 小助手实现多选删除功能，采用**可折叠的底部操作栏**方案，支持复选框+快捷键交互，提供全选当前/全选所有两种选择模式。

---

## 当前状态分析

### 已有实现

| 组件 | 状态 | 说明 |
|------|------|------|
| 后端 `trash.rs` | ✅ 已完成 | 废键箱管理器，支持批量操作 |
| 后端 `commands/trash.rs` | ✅ 已完成 | 7个 Tauri Command |
| 前端 `trashStore.ts` | ✅ 已完成 | Pinia store，包含 `batchMoveToTrash` |
| 前端废键箱视图 | ✅ 已完成 | el-table 展示、恢复、删除 |
| 前端多选功能 | ⚠️ 已注释 | 第115-135行、167-192行被注释 |

### 需要解决的问题

1. **双 el-tree 实例问题**：当前使用 v-if/v-else 切换两个 el-tree，存在性能开销和状态同步问题
2. **缺少可折叠操作栏**：多选功能入口不明确
3. **缺少全选逻辑**：没有全选当前/全选所有的区分
4. **缺少键盘快捷键**：不支持 Ctrl+A、Esc 等快捷操作

---

## 用户确认的设计决策

| 决策点 | 选择 |
|--------|------|
| 多选位置 | 键列表底部操作栏（可折叠） |
| 折叠默认状态 | 记住用户偏好（localStorage） |
| 全选范围 | 提供两种选择（全选当前已加载 / 全选所有） |
| 多选交互 | 复选框 + 快捷键（Ctrl/Shift） |
| 操作按钮 | 仅显示"移入废键箱" |

---

## UI 布局设计

```
┌─────────────────────────────────────────┐
│ [刷新] [搜索框] [搜索] [更多操作▼]       │  ← 搜索栏（保持现状）
├─────────────────────────────────────────┤
│ ▼ 多选模式                    [展开/收起] │  ← 可折叠操作栏（新增）
│ ┌─────────────────────────────────────┐ │
│ │ [进入多选] [全选 ▼] [取消选择]       │ │  ← 展开内容
│ │   ├─ 全选当前已加载 (100)            │ │
│ │   └─ 全选所有 (5000)                │ │
│ │ [移入废键箱 (N)]                     │ │
│ │ 快捷键: Ctrl+A | Ctrl+Shift+A | Esc │ │
│ └─────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│ [键列表 el-tree]                         │  ← 单实例，动态 show-checkbox
│ ☐ key1                                  │
│ ☐ key2                                  │
├─────────────────────────────────────────┤
│ [加载更多] 100/5000 [加载所有]           │  ← 分页区域（保持现状）
└─────────────────────────────────────────┘
```

---

## 实现方案

### 1. 单实例 el-tree 动态绑定

**修改前**（第166-192行）：
```vue
<el-tree v-if="!isMultiSelectMode" ... />
<el-tree v-else show-checkbox ... />
```

**修改后**：
```vue
<el-tree
  :data="keyTree"
  node-key="id"
  default-expand-all
  :show-checkbox="isMultiSelectMode"
  :highlight-current="!isMultiSelectMode"
  @node-click="handleKeyClick"
  @check-change="handleCheckChange"
  ref="treeRef"
  empty-text="暂无数据"
>
  <template #default="{ node }">
    <span class="key-item">{{ node.label }}</span>
  </template>
</el-tree>
```

**优势**：
- 避免组件销毁/重建的性能开销
- 保持滚动位置和展开状态
- 简化状态管理

### 2. 可折叠多选操作栏

**位置**：搜索栏（`.search-and-actions`）和键列表（`.key-list-content`）之间

**折叠状态存储**：
```typescript
const MULTI_SELECT_PANEL_KEY = 'redis-helper-multi-select-panel'
const isMultiSelectPanelExpanded = ref(
  localStorage.getItem(MULTI_SELECT_PANEL_KEY) !== 'false'
)

watch(isMultiSelectPanelExpanded, (val) => {
  localStorage.setItem(MULTI_SELECT_PANEL_KEY, String(val))
})
```

### 3. 全选功能实现

| 模式 | 说明 | 实现方式 |
|------|------|----------|
| 全选当前已加载 | 选中 `keys.value` 数组中的所有键 | `treeRef.value.setCheckedKeys(keys.value)` |
| 全选所有 | 选中数据库中所有键 | 先调用 `handleLoadAll()`，再全选 |

### 4. 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+A` | 全选当前已加载 |
| `Ctrl+Shift+A` | 全选所有 |
| `Escape` | 退出多选模式 |

---

## 文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `src/frontend/views/MainView.vue` | 1. 添加多选状态变量<br>2. 修改 el-tree 为单实例<br>3. 添加多选操作栏组件<br>4. 添加全选/取消选择方法<br>5. 添加键盘快捷键<br>6. 添加样式 |

---

## 详细实现步骤

### 步骤 1：添加状态变量

在 `<script setup>` 中添加：
```typescript
// 多选模式状态
const isMultiSelectMode = ref(false)
const selectedKeys = ref<string[]>([])
const treeRef = ref<any>(null)

// 多选面板折叠状态（记住用户偏好）
const MULTI_SELECT_PANEL_KEY = 'redis-helper-multi-select-panel'
const isMultiSelectPanelExpanded = ref(
  localStorage.getItem(MULTI_SELECT_PANEL_KEY) !== 'false'
)

// 监听折叠状态变化
watch(isMultiSelectPanelExpanded, (val) => {
  localStorage.setItem(MULTI_SELECT_PANEL_KEY, String(val))
})
```

### 步骤 2：修改 el-tree 为单实例

替换第166-192行的双实例代码为单实例动态绑定。

### 步骤 3：添加多选操作栏组件

在搜索栏和键列表之间插入可折叠的多选操作栏。

### 步骤 4：添加方法实现

- `toggleMultiSelectMode()` - 切换多选模式
- `handleSelectAllCommand(command)` - 处理全选下拉命令
- `selectAllLoaded()` - 全选当前已加载
- `selectAllKeys()` - 全选所有（异步）
- `clearSelection()` - 取消选择
- `handleKeyDown(event)` - 键盘快捷键处理
- `batchMoveToTrash()` - 批量移入废键箱

### 步骤 5：注册键盘事件

在 `onMounted` 中添加：
```typescript
document.addEventListener('keydown', handleKeyDown)
```

添加 `onUnmounted` 清理：
```typescript
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})
```

### 步骤 6：添加样式

添加多选操作栏的 CSS 样式。

---

## 注意事项

1. **性能考虑**：全选所有时，如果 keys 数量很大（>10000），应显示加载进度
2. **状态同步**：切换 DB 或服务器时，自动退出多选模式并清空选择
3. **用户体验**：
   - 进入多选模式时，自动展开操作栏
   - 选中项数量实时显示
   - 快捷键提示信息
4. **边界情况**：
   - 搜索后全选应只选中搜索结果
   - 加载更多后保持已选中状态

---

## 验证步骤

1. [ ] 单实例 el-tree 在多选/非多选模式切换正常
2. [ ] 折叠状态在刷新页面后保持
3. [ ] 全选当前已加载功能正常
4. [ ] 全选所有功能正常（先加载后全选）
5. [ ] Ctrl+A / Ctrl+Shift+A / Esc 快捷键正常
6. [ ] 批量移入废键箱功能正常
7. [ ] 切换 DB/服务器时状态正确重置
8. [ ] 搜索后全选只选中搜索结果
