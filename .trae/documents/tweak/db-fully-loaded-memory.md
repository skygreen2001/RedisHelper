# DB 切换时记忆"已全部加载"状态

## 任务理解
当用户在 DB A 中点击了"加载所有"，之后切换到 DB B，再切回 DB A 时，应该直接显示全部 keys，而不是再次只显示 100 个并要求用户手动操作。

## 当前状态分析
- `handleDbChange` 每次切换 DB 都调用 `loadKeys(true)`，`reset=true` 会清空所有分页状态
- 分页状态（`keysCursor`、`keysTotal`、`keys`）是全局单份，没有按 DB 维度隔离
- 切换 DB 后之前的加载状态完全丢失

## 方案对比

### 方案 A：只缓存标记（推荐）
- **思路**：用 Set 记录哪些 DB 已全部加载过，切回时直接调用 `loadKeys(false)` 全量请求后端
- **优点**：实现简单，数据始终是最新的
- **缺点**：切回时仍需等待后端请求（大数据量时有延迟）

### 方案 B：缓存 keys 数据
- **思路**：用 Map 按 DB 维度缓存 keys 数组，切回时直接从缓存读取，零延迟显示
- **优点**：切回瞬间显示，体验最好
- **缺点**：占用内存；数据可能过时（需后台刷新或手动刷新时清除缓存）

**推荐方案 B**，因为用户已经"加载所有"过，说明该 DB 的 keys 数据量是可接受的，缓存内存开销可控，且切回体验最好。

## 实施方案（方案 B）

### 修改文件：`src/frontend/views/MainView.vue`

#### 1. 新增缓存状态
```typescript
// 按 DB 维度缓存已加载的 keys（key 格式: "host:port:db"）
const keysCache = ref<Map<string, string[]>>(new Map())

// 生成 DB 标识
const getDbKey = (host: string, port: number, db: number) => `${host}:${port}:${db}`
```

#### 2. 修改 `handleDbChange` — 优先从缓存读取
```typescript
const handleDbChange = async () => {
  if (!selectedServer.value || selectedDb.value === null) return

  const dbKey = getDbKey(selectedServer.value.host, selectedServer.value.port, selectedDb.value)
  const cached = keysCache.value.get(dbKey)

  if (cached && cached.length > 0) {
    // 有缓存，直接使用（零延迟）
    keys.value = cached
    keysTotal.value = cached.length
    keysCursor.value = 0
    selectedKey.value = ''
    keyValue.value = ''
    keyType.value = ''
  } else {
    // 无缓存，走分页模式
    await loadKeys(true)
  }
}
```

#### 3. 修改 `handleLoadAll` — 加载成功后写入缓存
```typescript
// 在 keys.value = allKeys 之后添加：
const dbKey = getDbKey(selectedServer.value.host, selectedServer.value.port, selectedDb.value ?? 0)
keysCache.value.set(dbKey, [...allKeys])
```

#### 4. 修改 `handleRefresh` — 刷新时清除当前 DB 缓存
```typescript
const handleRefresh = async () => {
  searchPattern.value = ''
  // 清除当前 DB 缓存，强制从后端重新加载
  if (selectedServer.value && selectedDb.value !== null) {
    keysCache.value.delete(getDbKey(selectedServer.value.host, selectedServer.value.port, selectedDb.value))
  }
  await loadKeys(true)
}
```

#### 5. 修改 `searchKeys` — 搜索时不受缓存影响（保持现有逻辑）
无需修改，搜索结果不写入缓存。

#### 6. 修改 `addKey` / `deleteKey` — 数据变更时清除当前 DB 缓存
在增删 key 成功后，清除当前 DB 的缓存：
```typescript
keysCache.value.delete(getDbKey(selectedServer.value.host, selectedServer.value.port, selectedDb.value ?? 0))
```

## 验证步骤
1. 打开 DB A（>100 keys），默认加载 100 个
2. 点击"加载所有"，确认全部加载，footer 消失
3. 切换到 DB B
4. 切回 DB A → 应瞬间显示全部 keys，无加载延迟
5. 点击刷新按钮 → 应回到分页模式（只显示 100 个）
6. 在 DB A 添加/删除 key → 缓存清除，下次切回走分页
