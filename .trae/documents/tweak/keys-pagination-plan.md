# Keys 分页加载功能实施计划

## 任务概述
在 Redis 客户端左侧 keys 区域添加分页加载功能：
1. 默认加载 100 个 key
2. 添加「加载更多」按钮（每次再加载 100 个）
3. 添加「加载所有」按钮（加载全部 keys，带进度提示）
4. 显示已加载数量 / 总数（如 100 / 2560）

## 当前状态分析

### 前端 (Vue3 + Pinia + Element Plus)
- **MainView.vue**: 主界面，包含 keys 列表展示区域
- **redisStore.ts**: Redis 相关 API 调用封装

### 后端 (Rust + Tauri)
- **redis.rs**: Tauri 命令定义
- **connection.rs**: Redis 连接和操作方法

### 当前 keys 获取逻辑
- `get_keys()` 方法使用 `KEYS *` 命令获取所有 keys
- 没有分页机制，一次性返回所有 keys

## 实施计划

### Phase 1: 后端 API 改造

#### 1.1 修改 `src-tauri/src/commands/redis.rs`
- 修改 `get_keys` 命令，添加分页参数
- 新增 `get_keys_count` 命令获取总数量

```rust
// 新增分页请求结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct GetKeysRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub cursor: u64,      // SCAN 游标
    pub count: usize,     // 每次加载数量
}

// 新增分页响应结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct GetKeysResponse {
    pub keys: Vec<String>,
    pub cursor: u64,      // 返回的游标，0 表示结束
    pub total: usize,     // 总数量
}
```

#### 1.2 修改 `src-tauri/src/redis/connection.rs`
- 新增 `scan_keys` 方法使用 SCAN 命令替代 KEYS 命令
- 新增 `get_keys_count` 方法获取总数量

```rust
// 使用 SCAN 命令分页获取 keys
pub fn scan_keys(&mut self, cursor: u64, count: usize) -> Result<(Vec<String>, u64), Box<dyn Error>> {
    // 使用 SCAN 命令
    let result: redis::Iter<String> = redis::cmd("SCAN")
        .arg(cursor)
        .arg("COUNT")
        .arg(count)
        .query(&mut self.conn)?;
    
    let keys: Vec<String> = result.collect();
    let next_cursor = 0; // 实际应该从 SCAN 结果获取
    Ok((keys, next_cursor))
}

// 获取总数量
pub fn get_keys_count(&mut self) -> Result<usize, Box<dyn Error>> {
    let info: String = redis::cmd("DBSIZE").query(&mut self.conn)?;
    Ok(info.parse().unwrap_or(0))
}
```

### Phase 2: 前端 Store 改造

#### 2.1 修改 `src/frontend/stores/redisStore.ts`
- 添加分页相关状态和方法

```typescript
interface GetKeysRequest {
  host: string
  port: number
  password?: string
  db: number
  cursor: number
  count: number
}

interface GetKeysResponse {
  keys: string[]
  cursor: number
  total: number
}

// Store 中新增
async getKeysPaginated(params: GetKeysRequest): Promise<GetKeysResponse> {
  return await invoke<GetKeysResponse>('get_keys_paginated', { req: params })
}

async getKeysCount(params: ConnectRequest): Promise<number> {
  return await invoke<number>('get_keys_count', { req: params })
}
```

### Phase 3: 前端 UI 改造

#### 3.1 修改 `src/frontend/views/MainView.vue`

**数据结构变更：**
```typescript
// 分页相关状态
const keys = ref<string[]>([])
const keysCursor = ref<number>(0)        // 当前 SCAN 游标
const keysTotal = ref<number>(0)         // 总数量
const isLoadingMore = ref<boolean>(false) // 加载更多中
const isLoadingAll = ref<boolean>(false)  // 加载所有中
const loadedCount = computed(() => keys.value.length)
const hasMoreKeys = computed(() => keysCursor.value !== 0 || loadedCount.value < keysTotal.value)
```

**方法变更：**
```typescript
// 修改 loadKeys 为分页加载
const loadKeys = async (reset: boolean = true) => {
  if (!selectedServer.value) return
  
  try {
    if (reset) {
      // 重置状态，首次加载
      keys.value = []
      keysCursor.value = 0
    }
    
    // 先获取总数
    keysTotal.value = await redis.getKeysCount({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })
    
    // 分页加载第一批（100个）
    const response = await redis.getKeysPaginated({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      cursor: keysCursor.value,
      count: 100
    })
    
    keys.value = [...keys.value, ...response.keys]
    keysCursor.value = response.cursor
  } catch (error) {
    console.error('加载键失败:', error)
  }
}

// 加载更多
const loadMoreKeys = async () => {
  if (!selectedServer.value || isLoadingMore.value) return
  
  isLoadingMore.value = true
  try {
    const response = await redis.getKeysPaginated({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      cursor: keysCursor.value,
      count: 100
    })
    
    keys.value = [...keys.value, ...response.keys]
    keysCursor.value = response.cursor
  } finally {
    isLoadingMore.value = false
  }
}

// 加载所有
const loadAllKeys = async () => {
  if (!selectedServer.value || isLoadingAll.value) return
  
  isLoadingAll.value = true
  try {
    while (keysCursor.value !== 0 || keys.value.length < keysTotal.value) {
      const response = await redis.getKeysPaginated({
        host: selectedServer.value.host,
        port: selectedServer.value.port,
        password: selectedServer.value.password,
        db: selectedDb.value ?? 0,
        cursor: keysCursor.value,
        count: 500  // 批量加载更多
      })
      
      keys.value = [...keys.value, ...response.keys]
      keysCursor.value = response.cursor
      
      // 给 UI 刷新机会
      await new Promise(resolve => setTimeout(resolve, 10))
    }
  } finally {
    isLoadingAll.value = false
  }
}
```

**UI 模板变更：**
在 key-list 区域底部添加加载控制区域：

```vue
<!-- 在 key-list-content 下方添加 -->
<div class="key-list-footer">
  <div class="keys-count-info">
    <span class="count-text">{{ loadedCount }} / {{ keysTotal }}</span>
  </div>
  <div class="load-actions" v-if="hasMoreKeys && !isLoadingAll">
    <el-button 
      size="small" 
      @click="loadMoreKeys" 
      :loading="isLoadingMore"
      :disabled="isLoadingAll"
    >
      加载更多
    </el-button>
    <el-button 
      size="small" 
      type="primary" 
      @click="loadAllKeys"
      :loading="isLoadingAll"
      :disabled="isLoadingMore"
    >
      加载所有
    </el-button>
  </div>
  <div class="loading-all-progress" v-if="isLoadingAll">
    <el-progress :percentage="Math.round((loadedCount / keysTotal) * 100)" :show-text="true" />
    <span class="loading-text">正在加载所有 keys...</span>
  </div>
</div>
```

**样式添加：**
```css
.key-list-footer {
  padding: 12px 15px;
  border-top: 1px solid #e4e7ed;
  background-color: #f5f7fa;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.keys-count-info {
  text-align: center;
  font-size: 13px;
  color: #606266;
}

.count-text {
  font-weight: 500;
  color: #1890ff;
}

.load-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
}

.loading-all-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
}

.loading-text {
  font-size: 12px;
  color: #909399;
}
```

### Phase 4: 搜索功能适配

搜索功能也需要支持分页，或者保持当前一次性返回（因为搜索结果通常较少）。

修改 `searchKeys` 方法：
```typescript
const searchKeys = async () => {
  if (!selectedServer.value) return
  
  try {
    message.value = ''
    const pattern = searchPattern.value ? `*${searchPattern.value}*` : '*'
    
    // 搜索时一次性返回所有结果（通常搜索结果较少）
    keys.value = await redis.searchKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      pattern
    })
    
    // 搜索模式下重置分页状态
    keysCursor.value = 0
    keysTotal.value = keys.value.length
  } catch (error) {
    console.error('搜索键失败:', error)
  }
}
```

## 关键决策

### 1. 使用 SCAN 替代 KEYS
- **原因**: KEYS 命令在大数据量时会阻塞 Redis，SCAN 是增量迭代器，不会阻塞
- **影响**: 需要修改后端使用 SCAN 命令

### 2. 分页数量
- **默认加载**: 100 个
- **加载更多**: 100 个
- **加载所有**: 每批 500 个（加快加载速度）

### 3. 进度显示
- 使用 Element Plus 的 Progress 组件
- 显示百分比和当前加载数量

### 4. 按钮显示逻辑
- 当还有更多 keys 时显示「加载更多」和「加载所有」按钮
- 加载过程中禁用按钮
- 加载所有时显示进度条，隐藏按钮

## 文件修改清单

1. `src-tauri/src/commands/redis.rs` - 添加分页命令
2. `src-tauri/src/redis/connection.rs` - 添加 SCAN 方法
3. `src/frontend/stores/redisStore.ts` - 添加分页 store 方法
4. `src/frontend/views/MainView.vue` - 添加 UI 和逻辑

## 验证步骤

1. 连接一个包含大量 keys（>1000）的 Redis 服务器
2. 验证默认只加载 100 个 keys
3. 验证显示 "100 / 总数"
4. 点击「加载更多」，验证再加载 100 个
5. 点击「加载所有」，验证显示进度条并最终加载全部
6. 验证搜索功能正常工作
7. 验证切换 DB 后分页状态重置
