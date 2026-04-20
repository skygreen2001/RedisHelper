<template>
  <div class="main-container">
    <!-- 服务器和数据库选择 -->
    <div class="top-bar">
      <el-form :inline="true" class="form-inline">
        <el-form-item label="服务器">
          <el-select v-model="selectedServer" placeholder="选择服务器" @change="handleServerChange">
            <el-option
              v-for="server in servers"
              :key="server.id"
              :label="server.name"
              :value="server"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="数据库">
          <el-select v-model="selectedDb" placeholder="选择数据库" @change="handleDbChange">
            <el-option
              v-for="db in databases"
              :key="db[0]"
              :label="`DB ${db[0]} (${db[1]} keys)`"
              :value="db[0]"
            />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="refreshKeys">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <!-- 搜索和操作栏 -->
    <div class="search-bar">
      <el-input
        v-model="searchPattern"
        placeholder="搜索键..."
        prefix-icon="Search"
        @keyup.enter="searchKeys"
      />
      <div class="action-buttons">
        <el-button @click="showAddKeyDialog = true">
          <el-icon><Plus /></el-icon>
          添加
        </el-button>
        <el-button @click="exportData">
          <el-icon><Download /></el-icon>
          导出
        </el-button>
        <el-button @click="importData">
          <el-icon><Upload /></el-icon>
          导入
        </el-button>
      </div>
    </div>

    <!-- 键列表和值展示 -->
    <div class="content-area">
      <!-- 键列表 -->
      <div class="key-list">
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

      <!-- 值展示 -->
      <div class="value-display">
        <div v-if="selectedKey" class="key-detail">
          <div class="key-header">
            <h3>{{ selectedKey }}</h3>
            <div class="key-type">{{ keyType }}</div>
          </div>
          <div class="key-actions">
            <el-button type="primary" @click="showEditKeyDialog = true">
              <el-icon><Edit /></el-icon>
              修改
            </el-button>
            <el-button type="danger" @click="deleteKey">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
          </div>
          <div class="value-content">
            <pre>{{ formattedValue }}</pre>
          </div>
        </div>
        <div v-else class="empty-state">
          <el-empty description="请选择一个键" />
        </div>
      </div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Plus, Edit, Delete, Refresh, Download, Upload } from '@element-plus/icons-vue'
import { serverStore } from '../stores/serverStore'
import { redisStore } from '../stores/redisStore'

const server = serverStore()
const redis = redisStore()

// 状态
const selectedServer = ref<any>(null)
const selectedDb = ref<number>(0)
const databases = ref<Array<[number, number]>>([])
const keys = ref<string[]>([])
const selectedKey = ref<string>('')
const keyValue = ref<string>('')
const keyType = ref<string>('')
const searchPattern = ref<string>('')
const showAddKeyDialog = ref<boolean>(false)
const showEditKeyDialog = ref<boolean>(false)

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
  } catch (error) {
    console.error('加载数据库失败:', error)
  }
}

const loadKeys = async () => {
  if (!selectedServer.value) return
  
  try {
    keys.value = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value
    })
    selectedKey.value = ''
    keyValue.value = ''
    keyType.value = ''
  } catch (error) {
    console.error('加载键失败:', error)
  }
}

const handleKeyClick = async (node: any) => {
  selectedKey.value = node.id
  await loadKeyValue(node.id)
}

const loadKeyValue = async (key: string) => {
  if (!selectedServer.value) return
  
  try {
    const result = await redis.getKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
      key
    })
    keyValue.value = result.value
    keyType.value = result.key_type
  } catch (error) {
    console.error('加载键值失败:', error)
  }
}

const searchKeys = async () => {
  if (!selectedServer.value) return
  
  try {
    keys.value = await redis.searchKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
      pattern: searchPattern.value || '*'
    })
  } catch (error) {
    console.error('搜索键失败:', error)
  }
}

const refreshKeys = async () => {
  await loadKeys()
}

const addKey = async () => {
  if (!selectedServer.value || !newKeyForm.value.key) return
  
  try {
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
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
  } catch (error) {
    console.error('添加键失败:', error)
  }
}

const updateKey = async () => {
  if (!selectedServer.value || !editKeyForm.value.key) return
  
  try {
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
      key: editKeyForm.value.key,
      value: editKeyForm.value.value,
      key_type: editKeyForm.value.type
    })
    await loadKeyValue(editKeyForm.value.key)
    showEditKeyDialog.value = false
  } catch (error) {
    console.error('修改键失败:', error)
  }
}

const deleteKey = async () => {
  if (!selectedServer.value || !selectedKey.value) return
  
  try {
    await redis.deleteKey({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
      key: selectedKey.value
    })
    await loadKeys()
  } catch (error) {
    console.error('删除键失败:', error)
  }
}

const exportData = async () => {
  if (!selectedServer.value) return
  
  try {
    // 这里应该打开文件选择对话框，获取文件路径
    // 暂时使用默认路径
    const filePath = '/tmp/redis-export.json'
    await redis.exportData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
      file_path: filePath
    })
    console.log('导出成功:', filePath)
  } catch (error) {
    console.error('导出失败:', error)
  }
}

const importData = async () => {
  if (!selectedServer.value) return
  
  try {
    // 这里应该打开文件选择对话框，获取文件路径
    // 暂时使用默认路径
    const filePath = '/tmp/redis-export.json'
    await redis.importData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value,
      file_path: filePath
    })
    await loadKeys()
    console.log('导入成功:', filePath)
  } catch (error) {
    console.error('导入失败:', error)
  }
}

// 生命周期
onMounted(async () => {
  await server.loadServers()
  if (servers.value.length > 0) {
    selectedServer.value = servers.value[0]
    await loadDatabases()
  }
})
</script>

<style scoped>
.main-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.top-bar {
  margin-bottom: 20px;
}

.search-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.action-buttons {
  display: flex;
  gap: 10px;
}

.content-area {
  flex: 1;
  display: flex;
  gap: 20px;
  overflow: hidden;
}

.key-list {
  width: 30%;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
}

.key-list :deep(.el-tree-node.is-current > .el-tree-node__content) {
  background-color: #ecf5ff;
}

.value-display {
  flex: 1;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 20px;
  overflow: auto;
}

.key-detail {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.key-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.key-header h3 {
  margin: 0;
}

.key-type {
  background-color: #ecf5ff;
  color: #409eff;
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 12px;
}

.key-actions {
  margin-bottom: 20px;
  display: flex;
  gap: 10px;
}

.value-content {
  flex: 1;
  background-color: #f5f7fa;
  padding: 15px;
  border-radius: 4px;
  overflow: auto;
}

.value-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.empty-state {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>