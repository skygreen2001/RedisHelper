<template>
  <div class="main-container">
    <!-- 错误提示 -->
    <el-alert
      v-if="errorMessage"
      :title="errorMessage"
      type="error"
      show-icon
      :closable="true"
      @close="errorMessage = ''"
      class="error-alert"
    />

    <!-- 菜单栏 -->
    <div class="menu-bar">
      <div class="menu-left">
        <el-dropdown @command="handleDeviceCommand">
          <span class="el-dropdown-link">
            设备 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="server in servers"
                :key="server.id"
                :command="{ type: 'select', server }"
              >
                {{ server.name }}
              </el-dropdown-item>
              <el-dropdown-item divided command="settings">
                <el-icon><Setting /></el-icon> 设置
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-dropdown @command="handleDbCommand">
          <span class="el-dropdown-link">
            DB <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="db in databases"
                :key="db[0]"
                :command="{ type: 'select', db: db[0] }"
              >
                DB {{ db[0] }} ({{ db[1] }} 个)
              </el-dropdown-item>
              <el-dropdown-item divided command="add">
                <el-icon><Plus /></el-icon> 新增DB
              </el-dropdown-item>
              <el-dropdown-item command="delete">
                <el-icon><Delete /></el-icon> 删除DB
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      当前服务器: {{ selectedServer?.name || '未选择' }} > DB: {{ selectedDb !== null ? `db${selectedDb}` : '未选择' }} {{ databases.find(db => db[0] === selectedDb)?.[1] ? `(共计${databases.find(db => db[0] === selectedDb)?.[1]}个)` : '' }} > key: {{ selectedKey || '未选择' }}
      <span v-if="isDebugMode" class="debug-info"> | {{ isRunningInTauri ? '✓ Tauri环境' : '✗ 浏览器环境' }}</span>
    </div>

    <!-- 键列表和值展示 -->
    <div class="content-area">
      <!-- 左侧键列表区 -->
      <div class="key-list">
        <!-- 搜索和操作栏 -->
        <div class="search-and-actions">
          <div class="search-section">
            <el-button
              type="primary"
              plain
              size="small"
              @click="handleRefresh"
              class="refresh-btn"
            >
              <el-icon><Refresh /></el-icon>
            </el-button>
            <el-input
              v-model="searchPattern"
              placeholder="请输入关键词"
              @keyup.enter="searchKeys"
              class="search-input"
            />
            <el-button
              v-if="searchPattern"
              @click="searchPattern = ''"
              class="clear-btn"
            >
              <el-icon><Close /></el-icon>
            </el-button>
            <el-button
              type="primary"
              size="small"
              @click="searchKeys"
              class="search-btn"
            >
              搜索
            </el-button>
            <el-dropdown @command="handleActionCommand" class="more-actions">
              <el-button
                type="primary"
                plain
                size="small"
              >
                <el-icon><More /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="add">
                    添加
                  </el-dropdown-item>
                  <el-dropdown-item command="import">
                    导入
                  </el-dropdown-item>
                  <el-dropdown-item command="export">
                    导出
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
        <!-- 键列表 -->
        <div class="key-list-content">
          <el-scrollbar>
            <el-tree
              :data="keyTree"
              node-key="id"
              default-expand-all
              @node-click="handleKeyClick"
              :highlight-current="true"
            >
              <template #default="{ node }">
                <span class="key-item">{{ node.label }}</span>
              </template>
            </el-tree>
          </el-scrollbar>
        </div>
      </div>

      <!-- 右侧值展示区 -->
      <div class="value-display">
        <div v-if="selectedKey" class="key-detail">
          <div class="value-header">
            <div class="value-type">类型: {{ keyType.toUpperCase() }}</div>
          </div>
          <div class="value-content">
            <pre>{{ formattedValue }}</pre>
          </div>
          <div class="value-actions">
            <el-button size="small" type="danger" @click="deleteKey">[删除]</el-button>
            <el-button size="small" @click="showEditKeyDialog = true">[修改]</el-button>
          </div>
        </div>
        <div v-else class="empty-state">
          <el-empty description="请选择一个键" />
        </div>
      </div>
    </div>

    <!-- 底部标题栏 -->
    <div class="footer">
      Redis 系统数据监控  BB只为更好
    </div>

    <!-- 添加键对话框 -->
    <el-dialog
      v-model="showAddKeyDialog"
      title="添加键"
      width="500px"
    >
      <el-form :model="newKeyForm" label-width="80px">
        <el-form-item label="键名" required>
          <el-input v-model="newKeyForm.key" placeholder="输入键名" />
        </el-form-item>
        <el-form-item label="值" required>
          <el-input
            v-model="newKeyForm.value"
            type="textarea"
            :rows="4"
            placeholder="输入值"
          />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="newKeyForm.type" placeholder="选择类型">
            <el-option label="String" value="string" />
            <el-option label="List" value="list" />
            <el-option label="Set" value="set" />
            <el-option label="ZSet" value="zset" />
            <el-option label="Hash" value="hash" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddKeyDialog = false">取消</el-button>
          <el-button type="primary" @click="addKey">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 编辑键对话框 -->
    <el-dialog
      v-model="showEditKeyDialog"
      title="修改键"
      width="500px"
    >
      <el-form :model="editKeyForm" label-width="80px">
        <el-form-item label="键名" required>
          <el-input v-model="editKeyForm.key" disabled />
        </el-form-item>
        <el-form-item label="值" required>
          <el-input
            v-model="editKeyForm.value"
            type="textarea"
            :rows="4"
            placeholder="输入值"
          />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="editKeyForm.type" placeholder="选择类型">
            <el-option label="String" value="string" />
            <el-option label="List" value="list" />
            <el-option label="Set" value="set" />
            <el-option label="ZSet" value="zset" />
            <el-option label="Hash" value="hash" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showEditKeyDialog = false">取消</el-button>
          <el-button type="primary" @click="updateKey">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 服务器配置页面 -->
    <el-dialog
      v-model="showServerConfig"
      title="服务器配置"
      width="800px"
      height="80vh"
      destroy-on-close
    >
      <ServerConfigView />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="closeServerConfig">关闭</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 新增DB对话框 -->
    <el-dialog
      v-model="showAddDbDialog"
      title="新增DB"
      width="400px"
    >
      <el-form :model="{ db: newDbNumber }" label-width="80px">
        <el-form-item label="DB编号" required>
          <el-input-number v-model="newDbNumber" :min="0" :max="15" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddDbDialog = false">取消</el-button>
          <el-button type="primary" @click="addDb">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 删除DB对话框 -->
    <el-dialog
      v-model="showDeleteDbDialog"
      title="删除DB"
      width="400px"
    >
      <div class="delete-db-content">
        <p>请选择要删除的DB：</p>
        <div class="db-tags">
          <el-tag
            v-for="db in databases"
            :key="db[0]"
            :type="selectedDbForDelete === db[0] ? 'danger' : ''"
            effect="plain"
            @click="selectedDbForDelete = db[0]"
            style="margin: 5px; cursor: pointer"
          >
            DB {{ db[0] }} ({{ db[1] }} 个)
          </el-tag>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showDeleteDbDialog = false">取消</el-button>
          <el-button type="danger" @click="deleteDb" :disabled="selectedDbForDelete === null">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Plus, Delete, ArrowDown, Setting, Refresh, Close, More } from '@element-plus/icons-vue'
import { serverStore } from '../stores/serverStore'
import { redisStore } from '../stores/redisStore'
import ServerConfigView from './ServerConfigView.vue'

// 简化的 Tauri 环境检测
function checkIsTauri(): boolean {
  try {
    if (typeof window === 'undefined') return false
    const win = window as any
    // 只要有 Tauri 相关对象就认为是 Tauri 环境
    return !!(win.__TAURI__ || win.__TAURI_IPC__)
  } catch (e) {
    console.warn('Tauri检测失败:', e)
    return false
  }
}

// 调试模式 - 设为 true 可以看到环境信息
const isDebugMode = true
const isRunningInTauri = checkIsTauri()

const server = serverStore()
const redis = redisStore()

// 状态
const selectedServer = ref<any>(null)
const selectedDb = ref<number | null>(null)
const databases = ref<Array<[number, number]>>([])
const keys = ref<string[]>([])
const selectedKey = ref<string>('')
const keyValue = ref<string>('')
const keyType = ref<string>('')
const searchPattern = ref<string>('')
const showAddKeyDialog = ref<boolean>(false)
const showEditKeyDialog = ref<boolean>(false)
const showServerConfig = ref<boolean>(false)
const showAddDbDialog = ref<boolean>(false)
const showDeleteDbDialog = ref<boolean>(false)
const selectedDbForDelete = ref<number | null>(null)
const newDbNumber = ref<number>(0)
const errorMessage = ref<string>('')

// 处理设备下拉菜单命令
const handleDeviceCommand = (command: any) => {
  if (command === 'settings') {
    // 显示服务器配置页面
    showServerConfig.value = true
  } else if (command.type === 'select') {
    selectedServer.value = command.server
    handleServerChange()
  }
}

// 处理DB下拉菜单命令
const handleDbCommand = (command: any) => {
  if (command === 'add') {
    // 新增DB逻辑
    showAddDbDialog.value = true
  } else if (command === 'delete') {
    // 删除DB逻辑
    showDeleteDbDialog.value = true
  } else if (command.type === 'select') {
    selectedDb.value = command.db
    handleDbChange()
  }
}

// 处理刷新操作
const handleRefresh = async () => {
  // 清空搜索框
  searchPattern.value = ''
  // 重新加载键列表
  await loadKeys()
}

// 处理更多操作命令
const handleActionCommand = (command: string) => {
  switch (command) {
    case 'add':
      showAddKeyDialog.value = true
      break
    case 'import':
      importData()
      break
    case 'export':
      exportData()
      break
  }
}

// 表单数据
const newKeyForm = ref({
  key: '',
  value: '',
  type: 'string'
})

const editKeyForm = ref({
  key: '',
  value: '',
  type: 'string'
})

// 计算属性
const servers = computed(() => server.servers)
const keyTree = computed(() => {
  return keys.value.map(key => ({
    id: key,
    label: key
  }))
})

const formattedValue = computed(() => {
  try {
    return JSON.stringify(JSON.parse(keyValue.value), null, 2)
  } catch {
    return keyValue.value
  }
})

// 方法
const handleServerChange = async () => {
  if (selectedServer.value) {
    await loadDatabases()
  }
}

const handleDbChange = async () => {
  await loadKeys()
}

const loadDatabases = async () => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    databases.value = await redis.getDatabases({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedServer.value.db
    })
    if (databases.value.length > 0) {
      selectedDb.value = databases.value[0][0]
      await loadKeys()
    }
  } catch (error: any) {
    console.error('加载数据库失败:', error)
    errorMessage.value = `加载数据库失败: ${error.message || error}`
  }
}

const loadKeys = async () => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    keys.value = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })
    selectedKey.value = ''
    keyValue.value = ''
    keyType.value = ''
  } catch (error: any) {
    console.error('加载键失败:', error)
    errorMessage.value = `加载键失败: ${error.message || error}`
  }
}

const handleKeyClick = async (node: any) => {
  selectedKey.value = node.id
  await loadKeyValue(node.id)
}

const loadKeyValue = async (key: string) => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    const result = await redis.getKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key
    })
    keyValue.value = result.value
    keyType.value = result.key_type
  } catch (error: any) {
    console.error('加载键值失败:', error)
    errorMessage.value = `加载键值失败: ${error.message || error}`
  }
}

const searchKeys = async () => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    // 为搜索关键词添加通配符，实现模糊查询
    const pattern = searchPattern.value 
      ? `*${searchPattern.value}*` 
      : '*'
    
    keys.value = await redis.searchKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      pattern
    })
  } catch (error: any) {
    console.error('搜索键失败:', error)
    errorMessage.value = `搜索键失败: ${error.message || error}`
  }
}

const addKey = async () => {
  if (!selectedServer.value || !newKeyForm.value.key) return
  
  try {
    errorMessage.value = ''
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: newKeyForm.value.key,
      value: newKeyForm.value.value,
      key_type: newKeyForm.value.type
    })
    await loadKeys()
    showAddKeyDialog.value = false
    // 重置表单
    newKeyForm.value = {
      key: '',
      value: '',
      type: 'string'
    }
  } catch (error: any) {
    console.error('添加键失败:', error)
    errorMessage.value = `添加键失败: ${error.message || error}`
  }
}

const updateKey = async () => {
  if (!selectedServer.value || !editKeyForm.value.key) return
  
  try {
    errorMessage.value = ''
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: editKeyForm.value.key,
      value: editKeyForm.value.value,
      key_type: editKeyForm.value.type
    })
    await loadKeyValue(editKeyForm.value.key)
    showEditKeyDialog.value = false
  } catch (error: any) {
    console.error('修改键失败:', error)
    errorMessage.value = `修改键失败: ${error.message || error}`
  }
}

const deleteKey = async () => {
  if (!selectedServer.value || !selectedKey.value) return
  
  try {
    errorMessage.value = ''
    await redis.deleteKey({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: selectedKey.value
    })
    await loadKeys()
  } catch (error: any) {
    console.error('删除键失败:', error)
    errorMessage.value = `删除键失败: ${error.message || error}`
  }
}

const exportData = async () => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    // 这里应该打开文件选择对话框，获取文件路径
    // 暂时使用默认路径
    const filePath = '/tmp/redis-export.json'
    await redis.exportData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      file_path: filePath
    })
    console.log('导出成功:', filePath)
  } catch (error: any) {
    console.error('导出失败:', error)
    errorMessage.value = `导出失败: ${error.message || error}`
  }
}

const importData = async () => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    // 这里应该打开文件选择对话框，获取文件路径
    // 暂时使用默认路径
    const filePath = '/tmp/redis-export.json'
    await redis.importData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      file_path: filePath
    })
    await loadKeys()
    console.log('导入成功:', filePath)
  } catch (error: any) {
    console.error('导入失败:', error)
    errorMessage.value = `导入失败: ${error.message || error}`
  }
}

// 服务器配置页面关闭
const closeServerConfig = async () => {
  showServerConfig.value = false
  // 重新加载服务器列表
  try {
    errorMessage.value = ''
    await server.loadServers()
  } catch (error: any) {
    console.error('加载服务器失败:', error)
    errorMessage.value = `加载服务器失败: ${error.message || error}`
  }
}

// 新增DB
const addDb = async () => {
  if (!selectedServer.value) return
  
  try {
    errorMessage.value = ''
    await redis.createDatabase({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: newDbNumber.value
    })
    await loadDatabases()
    showAddDbDialog.value = false
  } catch (error: any) {
    console.error('新增DB失败:', error)
    errorMessage.value = `新增DB失败: ${error.message || error}`
  }
}

// 删除DB
const deleteDb = async () => {
  if (!selectedServer.value || selectedDbForDelete.value === null) return
  
  try {
    errorMessage.value = ''
    await redis.deleteDatabase({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDbForDelete.value
    })
    await loadDatabases()
    showDeleteDbDialog.value = false
    selectedDbForDelete.value = null
  } catch (error: any) {
    console.error('删除DB失败:', error)
    errorMessage.value = `删除DB失败: ${error.message || error}`
  }
}

// 生命周期
onMounted(async () => {
  try {
    errorMessage.value = ''
    await server.loadServers()
    if (servers.value.length > 0) {
      selectedServer.value = servers.value[0]
      await loadDatabases()
    }
  } catch (error: any) {
    console.error('初始化失败:', error)
    errorMessage.value = `初始化失败: ${error.message || error}`
  }
})
</script>

<style scoped>
.main-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
  border: 1px solid #e4e7ed;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* 错误提示 */
.error-alert {
  margin: 10px 20px;
  border-radius: 4px;
}

/* 菜单栏 */
.menu-bar {
  background-color: #1890ff;
  color: white;
  padding: 0 20px;
  height: 48px;
  display: flex;
  align-items: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.menu-left {
  display: flex;
  gap: 30px;
}

.el-dropdown-link {
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  padding: 8px 12px;
  border-radius: 4px;
  transition: all 0.3s ease;
}

.el-dropdown-link:hover {
  color: #ecf5ff;
  background-color: rgba(255, 255, 255, 0.1);
}

/* 状态栏 */
.status-bar {
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  padding: 10px 20px;
  font-size: 14px;
  color: #606266;
  font-weight: 500;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.debug-info {
  font-size: 12px;
  color: #909399;
}

/* 内容区域 */
.content-area {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* 左侧键列表区 */
.key-list {
  width: 35%;
  border-right: 1px solid #e4e7ed;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
}

/* 搜索和操作栏 */
.search-and-actions {
  padding: 12px 15px;
  background-color: #ffffff;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  margin: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.search-section {
  display: flex;
  align-items: center;
  gap: 0;
  width: 100%;
}

.refresh-btn {
  border-radius: 4px 0 0 4px;
  min-width: 36px;
  height: 36px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #dcdfe6;
  border-right: none;
  color: #606266;
}

.refresh-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
  border-right: none;
}

.search-input {
  flex: 1;
  height: 36px;
  position: relative;
}

.search-input :deep(.el-input__wrapper) {
  border-radius: 0;
  border-left: none;
  border-right: none;
  padding: 0 12px;
  width: 100%;
  height: 100%;
}

.search-input :deep(.el-input__input) {
  height: 100%;
  line-height: 36px;
}

.search-input :deep(.el-input__wrapper.is-focus) {
  box-shadow: none;
  border-color: #1890ff;
}

.clear-btn {
  position: absolute;
  right: 60px;
  top: 50%;
  transform: translateY(-50%);
  padding: 0;
  min-width: 24px;
  height: 24px;
  background: transparent;
  z-index: 1;
}

.clear-btn:hover {
  color: #f56c6c;
  background: transparent;
}

/* 下拉菜单样式 */
.more-actions :deep(.el-dropdown-menu) {
  padding: 8px 0;
  min-width: 300px;
  width: 300px;
}

.more-actions :deep(.el-dropdown-item) {
  padding: 8px 20px;
  margin: 0;
  height: auto;
  line-height: 1.4;
}

.search-btn {
  min-width: 60px;
  height: 36px;
  border-radius: 0 4px 4px 0;
  background-color: #1890ff;
  border-color: #1890ff;
  border-left: none;
}

.search-btn:hover {
  background-color: #409eff;
  border-color: #409eff;
}

.more-actions {
  margin-left: 10px;
}

.more-actions .el-button {
  border-radius: 4px;
  min-width: 36px;
  height: 36px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #dcdfe6;
  color: #606266;
}

.more-actions .el-button:hover {
  border-color: #1890ff;
  color: #1890ff;
}

.key-list-content {
  flex: 1;
  overflow: auto;
}

.key-list :deep(.el-tree) {
  background-color: #ffffff;
}

.key-list :deep(.el-tree-node.is-current > .el-tree-node__content) {
  background-color: #ecf5ff;
  color: #1890ff;
  font-weight: 500;
}

.key-list :deep(.el-tree-node__content) {
  height: 36px;
  line-height: 36px;
  padding: 0 16px;
}

.key-item {
  font-size: 14px;
}

/* 右侧值展示区 */
.value-display {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  overflow: auto;
  background-color: #ffffff;
}

.key-detail {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.value-header {
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e4e7ed;
}

.value-type {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.value-content {
  flex: 1;
  background-color: #f5f7fa;
  padding: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: auto;
  margin-bottom: 16px;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.05);
}

.value-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: 'Courier New', Courier, monospace;
  font-size: 14px;
  line-height: 1.6;
  color: #303133;
}

.value-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-start;
  padding-top: 8px;
  border-top: 1px solid #e4e7ed;
}

.value-actions .el-button {
  font-size: 13px;
  padding: 6px 12px;
  border-radius: 4px;
  min-width: 60px;
}

/* 底部标题栏 */
.footer {
  background-color: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  padding: 10px 20px;
  font-size: 12px;
  color: #909399;
  text-align: center;
  font-weight: 400;
}

/* 空状态 */
.empty-state {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #fafafa;
  border: 1px dashed #d9d9d9;
  border-radius: 4px;
  margin: 20px;
}

/* 对话框 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
}

.dialog-footer .el-button {
  min-width: 80px;
  padding: 8px 16px;
  border-radius: 4px;
}

/* 删除DB对话框 */
.delete-db-content {
  padding: 20px;
}

.delete-db-content p {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.db-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.db-tags .el-tag {
  margin: 0;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.db-tags .el-tag:hover {
  background-color: #ecf5ff;
  border-color: #d9ecff;
  color: #1890ff;
}

.db-tags .el-tag.is-danger {
  background-color: #fef0f0;
  border-color: #fbc4c4;
  color: #f56c6c;
}

/* 表单样式优化 */
:deep(.el-form-item) {
  margin-bottom: 16px;
}

:deep(.el-form-item__label) {
  font-weight: 500;
  color: #303133;
}

:deep(.el-form-item__label.is-required:not(.el-form-item__label--left)::before) {
  content: "*";
  color: #f56c6c;
  margin-right: 4px;
}

:deep(.el-input__wrapper) {
  border-radius: 4px;
}

:deep(.el-input__wrapper:focus-within) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

:deep(.el-select__wrapper) {
  border-radius: 4px;
}

:deep(.el-select__wrapper:focus-within) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

:deep(.el-textarea__inner) {
  border-radius: 4px;
  font-family: 'Courier New', Courier, monospace;
}

:deep(.el-textarea__inner:focus) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}


</style>