<template>
  <div class="server-config-container">
    <div class="header">
      <h2>服务器配置</h2>
      <div class="header-actions">
        <el-button @click="exportConfig" :disabled="servers.length === 0">
          <el-icon><Download /></el-icon>
          导出配置
        </el-button>
        <el-button @click="importConfig">
          <el-icon><Upload /></el-icon>
          导入配置
        </el-button>
        <el-button type="primary" @click="showAddDialog = true">
          <el-icon><Plus /></el-icon>
          添加服务器
        </el-button>
      </div>
    </div>

    <!-- 隐藏的文件输入框 -->
    <input
      ref="fileInput"
      type="file"
      accept=".json"
      style="display: none"
      @change="handleFileSelect"
    />

    <div v-if="servers.length === 0" class="empty-state">
      <el-empty description="暂无服务器配置" />
    </div>
    <el-table
      v-else
      :data="servers"
      style="width: 100%"
      stripe
      row-key="id"
      :max-height="400"
    >
      <el-table-column label="排序" width="60" align="center">
        <template #default="{ row }">
          <div class="sort-buttons">
            <span
              class="sort-icon"
              :class="{ disabled: getServerIndex(row.id) === 0 }"
              @click="getServerIndex(row.id) > 0 && moveUp(row.id)"
            >
              ↑
            </span>
            <span
              class="sort-icon"
              :class="{ disabled: getServerIndex(row.id) === servers.length - 1 }"
              @click="getServerIndex(row.id) < servers.length - 1 && moveDown(row.id)"
            >
              ↓
            </span>
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="name" label="名称" width="180" />
      <el-table-column prop="host" label="服务器" width="180" />
      <el-table-column prop="port" label="端口" width="100" />
      <el-table-column prop="db" label="默认DB" width="100" />
      <el-table-column label="只读" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="row.readonly ? 'warning' : 'info'" size="small">
            {{ row.readonly ? '是' : '否' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="240" fixed="right">
        <template #default="{ row }">
          <div class="operation-buttons">
            <el-button size="small" @click="editServer(row)">
              <el-icon><Edit /></el-icon>
              编辑
            </el-button>
            <el-button size="small" type="danger" @click="confirmDelete(row)">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
            <el-button size="small" @click="testConnection(row)">
              <el-icon><Connection /></el-icon>
              测试
            </el-button>
          </div>
        </template>
      </el-table-column>
    </el-table>

    <!-- 配置区域 -->
    <div v-if="isDev" class="config-section">
      <div class="config-item">
        <div class="config-label">
          <span class="config-title">调试日志</span>
          <span class="config-description">开启后将在终端中显示详细的调试信息</span>
        </div>
        <el-switch
          v-model="debugEnabled"
          @change="handleDebugSwitchChange"
          active-text="开启"
          inactive-text="关闭"
        />
      </div>
    </div>

    <!-- 添加服务器对话框 -->
    <el-dialog
      v-model="showAddDialog"
      title="添加服务器"
      width="500px"
    >
      <el-form :model="serverForm" label-width="80px">
        <el-form-item label="名称" required>
          <el-input v-model="serverForm.name" placeholder="输入服务器名称" />
        </el-form-item>
        <el-form-item label="主机" required>
          <el-input v-model="serverForm.host" placeholder="输入主机地址" />
        </el-form-item>
        <el-form-item label="端口" required>
          <el-input-number v-model="serverForm.port" :min="1" :max="65535" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input v-model="serverForm.password" type="password" placeholder="输入密码" show-password />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="serverForm.username" placeholder="ACL in Redis >= 6.0" clearable />
          <span class="username-hint">Redis 6.0+ 访问控制列表（ACL）用户，可选</span>
        </el-form-item>
        <el-form-item label="默认DB">
          <el-input-number v-model="serverForm.db" :min="0" :max="15" />
        </el-form-item>
        <el-form-item label="只读">
          <el-switch v-model="serverForm.readonly" active-text="禁止删除" inactive-text="" />
          <span class="readonly-hint">开启后，该服务器的 DB 和 Key 将无法删除</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddDialog = false">取消</el-button>
          <el-button type="primary" @click="addServer">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 编辑服务器对话框 -->
    <el-dialog
      v-model="showEditDialog"
      title="编辑服务器"
      width="500px"
    >
      <el-form :model="serverForm" label-width="80px">
        <el-form-item label="名称" required>
          <el-input v-model="serverForm.name" placeholder="输入服务器名称" />
        </el-form-item>
        <el-form-item label="主机" required>
          <el-input v-model="serverForm.host" placeholder="输入主机地址" />
        </el-form-item>
        <el-form-item label="端口" required>
          <el-input-number v-model="serverForm.port" :min="1" :max="65535" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input v-model="serverForm.password" type="password" placeholder="输入密码" show-password />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="serverForm.username" placeholder="ACL in Redis >= 6.0" clearable />
          <span class="username-hint">Redis 6.0+ 访问控制列表（ACL）用户，可选</span>
        </el-form-item>
        <el-form-item label="默认DB">
          <el-input-number v-model="serverForm.db" :min="0" :max="15" />
        </el-form-item>
        <el-form-item label="只读">
          <el-switch v-model="serverForm.readonly" active-text="禁止删除" inactive-text="" />
          <span class="readonly-hint">开启后，该服务器的 DB 和 Key 将无法删除</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showEditDialog = false">取消</el-button>
          <el-button type="primary" @click="updateServer">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 测试连接对话框 -->
    <el-dialog
      v-model="showTestDialog"
      title="测试连接"
      width="400px"
    >
      <div class="test-result">
        <el-icon v-if="testResult.success" class="success-icon"><Check /></el-icon>
        <el-icon v-else class="error-icon"><Close /></el-icon>
        <p>{{ testResult.message }}</p>
      </div>
      <template #footer>
        <el-button @click="showTestDialog = false">关闭</el-button>
      </template>
    </el-dialog>

    <!-- 删除确认对话框 -->
    <el-dialog
      v-model="showDeleteDialog"
      title="删除确认"
      width="400px"
    >
      <div class="delete-confirm">
        <el-icon class="delete-confirm-icon"><Delete /></el-icon>
        <p>删除后将不能恢复，确认要删除{{ selectedServer?.name }}?</p>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showDeleteDialog = false">取消</el-button>
          <el-button type="danger" @click="deleteServer">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Plus, Edit, Delete, Connection, Check, Close, Download, Upload } from '@element-plus/icons-vue'
import { serverStore } from '../stores/serverStore'
import { configStore } from '../stores/configStore'
import { ElMessage, ElMessageBox } from 'element-plus'
import { isTauriEnv } from '../utils/tauri'
import { sessionManager } from '../sessions/SessionManager'

const server = serverStore()
const config = configStore()
const fileInput = ref<HTMLInputElement | null>(null)

// 开发环境判断
const isDev = import.meta.env.DEV

// 调试日志开关
const debugEnabled = ref<boolean>(false)

// 状态
const showAddDialog = ref<boolean>(false)
const showEditDialog = ref<boolean>(false)
const showTestDialog = ref<boolean>(false)
const showDeleteDialog = ref<boolean>(false)

// 表单数据
const serverForm = ref({
  id: '',
  name: '',
  host: '',
  port: 6379,
  password: '',
  username: '',
  db: 0,
  readonly: false
})

// 监听名称变化，自动同步标识（仅在添加对话框打开时）
watch(() => serverForm.value.name, (newName) => {
  if (showAddDialog.value) {
    serverForm.value.id = newName
  }
})

// 选中的服务器
const selectedServer = ref<any>(null)

// 测试结果
const testResult = ref({
  success: false,
  message: ''
})

// 计算属性
const servers = computed(() => server.servers)

// 方法
const addServer = async () => {
  if (!serverForm.value.id || !serverForm.value.name || !serverForm.value.host) {
    return
  }

  try {
    await server.addServer({
      id: serverForm.value.id,
      name: serverForm.value.name,
      host: serverForm.value.host,
      port: serverForm.value.port,
      password: serverForm.value.password || undefined,
      username: serverForm.value.username.trim() || undefined,
      db: serverForm.value.db,
      readonly: serverForm.value.readonly
    })
    showAddDialog.value = false
    resetForm()
  } catch (error) {
    console.error('添加服务器失败:', error)
  }
}

const editServer = (row: any) => {
  serverForm.value = {
    id: row.id,
    name: row.name,
    host: row.host,
    port: row.port,
    password: row.password || '',
    username: row.username || '',
    db: row.db,
    readonly: row.readonly || false
  }
  showEditDialog.value = true
}

const updateServer = async () => {
  try {
    await server.editServer({
      id: serverForm.value.id,
      name: serverForm.value.name,
      host: serverForm.value.host,
      port: serverForm.value.port,
      password: serverForm.value.password || undefined,
      username: serverForm.value.username.trim() || undefined,
      db: serverForm.value.db,
      readonly: serverForm.value.readonly
    })
    showEditDialog.value = false
  } catch (error) {
    console.error('更新服务器失败:', error)
  }
}

const confirmDelete = (row: any) => {
  // 不允许删除当前正在连接的服务器
  const activeServer = sessionManager.active.selectedServer
  if (activeServer && activeServer.id === row.id) {
    ElMessage.warning('不能删除当前正在连接的服务器，请先切换到其他连接')
    return
  }
  selectedServer.value = row
  showDeleteDialog.value = true
}

const deleteServer = async () => {
  if (!selectedServer.value) return
  
  const serverId = selectedServer.value.id
  
  try {
    // 关闭所有使用该连接的标签页
    const sessionsToClose = sessionManager.sessions.filter(
      s => s.selectedServer?.id === serverId
    )
    
    // 如果当前只有一个页面并且是当前连接，则重置该连接为无需要用户选择连接
    if (sessionManager.sessions.length === 1 && 
        sessionManager.sessions[0].selectedServer?.id === serverId) {
      // 删除后服务器列表为空，需要标记必须添加连接
      const remainingServers = server.servers.filter(s => s.id !== serverId)
      if (remainingServers.length === 0) {
        // 重置当前会话状态
        sessionManager.sessions[0].selectedServer = null
        sessionManager.sessions[0].selectedDb = null
        sessionManager.sessions[0].keys = []
        sessionManager.sessions[0].selectedKey = ''
        sessionManager.sessions[0].keyValue = ''
        sessionManager.sessions[0].message = ''
        // 标记必须添加连接才能关闭设置页面
        sessionManager.sessions[0].requireServerConnection = true
        // 跳转到服务器配置页面
        sessionManager.sessions[0].isSelectingServer = false
        sessionManager.sessions[0].showServerConfig = true
        sessionManager.sessions[0].title = '新标签'
      } else {
        // 还有其他服务器，重置为选择连接状态
        sessionManager.sessions[0].selectedServer = null
        sessionManager.sessions[0].isSelectingServer = true
        sessionManager.sessions[0].title = '新标签'
      }
    } else {
      // 关闭所有使用该连接的标签页
      for (const session of sessionsToClose) {
        sessionManager.closeSession(session.id)
      }
    }
    
    await server.deleteServer(serverId)
    showDeleteDialog.value = false
    selectedServer.value = null
    
    ElMessage.success('服务器已删除')
  } catch (error) {
    console.error('删除服务器失败:', error)
    ElMessage.error('删除服务器失败')
  }
}

const testConnection = async (row: any) => {
  try {
    const result = await server.testConnection({
      host: row.host,
      port: row.port,
      password: row.password,
      username: row.username
    })
    testResult.value = result
    showTestDialog.value = true
  } catch (error) {
    console.error('测试连接失败:', error)
  }
}

const resetForm = () => {
  serverForm.value = {
    id: '',
    name: '',
    host: '',
    port: 6379,
    password: '',
    username: '',
    db: 0,
    readonly: false
  }
}

// 导出配置
const exportConfig = async () => {
  if (servers.value.length === 0) {
    ElMessage.warning('没有可导出的服务器配置')
    return
  }

  const configData = {
    version: '1.0',
    exportedAt: new Date().toISOString(),
    servers: servers.value.map(s => ({
      id: s.id,
      name: s.name,
      host: s.host,
      port: s.port,
      username: s.username,
      password: s.password,
      db: s.db,
      readonly: s.readonly
    }))
  }

  const jsonStr = JSON.stringify(configData, null, 2)
  const fileName = `redis-servers-${new Date().toISOString().slice(0, 10)}.json`

  if (isTauriEnv()) {
    // Tauri 环境：使用文件对话框保存
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      const filePath = await save({
        title: '保存服务器配置',
        defaultPath: fileName,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      })
      if (!filePath) return // 用户取消

      const { writeTextFile } = await import('@tauri-apps/plugin-fs')
      await writeTextFile(filePath, jsonStr)
      ElMessage.success(`配置已导出到: ${filePath}`)
    } catch (err: any) {
      console.error('导出配置失败:', err)
      ElMessage.error(`导出失败: ${err.message || err}`)
    }
  } else {
    // 浏览器环境：下载文件
    const blob = new Blob([jsonStr], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = fileName
    a.click()
    URL.revokeObjectURL(url)
    ElMessage.success('配置已导出')
  }
}

// 导入配置
const importConfig = () => {
  fileInput.value?.click()
}

// 处理文件选择
const handleFileSelect = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  try {
    const text = await file.text()
    const data = JSON.parse(text)

    // 验证格式
    if (!data.servers || !Array.isArray(data.servers)) {
      throw new Error('无效的配置文件格式')
    }

    // 确认导入
    const confirm = await ElMessageBox.confirm(
      `即将导入 ${data.servers.length} 个服务器配置，是否继续？`,
      '导入确认',
      { confirmButtonText: '确定', cancelButtonText: '取消', type: 'info' }
    ).catch(() => false)

    if (!confirm) {
      input.value = ''
      return
    }

    // 导入配置
    let imported = 0
    for (const s of data.servers) {
      try {
        await server.addServer({
          id: s.id,
          name: s.name,
          host: s.host,
          port: s.port,
          password: s.password,
          username: s.username,
          db: s.db || 0,
          readonly: s.readonly || false
        })
        imported++
      } catch {
        // 跳过已存在的配置
      }
    }

    ElMessage.success(`成功导入 ${imported} 个服务器配置`)
  } catch (err: any) {
    ElMessage.error(`导入失败: ${err.message}`)
  } finally {
    input.value = ''
  }
}

// 处理调试日志开关变化
const handleDebugSwitchChange = async (value: boolean) => {
  await config.setDebugLogEnabled(value)
  ElMessage.success(value ? '调试日志已开启' : '调试日志已关闭')
}

// 获取服务器索引
const getServerIndex = (serverId: string): number => {
  return server.servers.findIndex(s => s.id === serverId)
}

// 向上移动
const moveUp = async (serverId: string) => {
  const index = getServerIndex(serverId)
  if (index <= 0) return

  const newServers = [...server.servers]
  const temp = newServers[index]
  newServers[index] = newServers[index - 1]
  newServers[index - 1] = temp

  server.servers = newServers
  await saveServerOrder()
}

// 向下移动
const moveDown = async (serverId: string) => {
  const index = getServerIndex(serverId)
  if (index >= server.servers.length - 1) return

  const newServers = [...server.servers]
  const temp = newServers[index]
  newServers[index] = newServers[index + 1]
  newServers[index + 1] = temp

  server.servers = newServers
  await saveServerOrder()
}

// 保存服务器顺序
const saveServerOrder = async () => {
  try {
    await server.saveServerOrder([...server.servers])
    ElMessage.success('服务器顺序已保存')
  } catch (error) {
    console.error('保存服务器顺序失败:', error)
    ElMessage.error('保存顺序失败')
  }
}

// 生命周期
onMounted(async () => {
  await server.loadServers()
  await config.loadDebugConfig()
  debugEnabled.value = config.debugLogEnabled
})
</script>

<style scoped>
.server-config-container {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  overflow: hidden;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e4e7ed;
}

.header h2 {
  margin: 0;
  color: #303133;
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.header-actions .el-button {
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 14px;
}

/* 配置区域 */
.config-section {
  background-color: #fafafa;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 16px 20px;
  margin-top: 20px;
}

.config-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.config-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.config-title {
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.config-description {
  font-size: 13px;
  color: #909399;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #fafafa;
  border: 1px dashed #d9d9d9;
  border-radius: 4px;
  margin: 20px 0;
  padding: 40px;
}

/* 表格 */
.el-table {
  flex: 1;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

/* 排序按钮 */
.sort-buttons {
  display: flex;
  flex-direction: column;
  gap: 2px;
  align-items: center;
}

.sort-icon {
  cursor: pointer;
  color: #606266;
  font-size: 20px;
  font-weight: bold;
  transition: all 0.2s;
  padding: 2px 8px;
  border-radius: 4px;
}

.sort-icon:hover:not(.disabled) {
  color: #409eff;
  background-color: #ecf5ff;
}

.sort-icon.disabled {
  cursor: not-allowed;
  color: #d9d9d9;
}

.el-table :deep(.el-table__header-wrapper) {
  background-color: #fafafa;
}

.el-table :deep(.el-table__header th) {
  font-weight: 600;
  color: #303133;
  background-color: #fafafa;
  border-bottom: 1px solid #e4e7ed;
  height: 48px;
  line-height: 48px;
}

.el-table :deep(.el-table__row) {
  height: 60px;
}

.el-table :deep(.el-table__row:hover) {
  background-color: #f5f7fa;
}

.el-table :deep(.el-table__row.el-table__row--striped) {
  background-color: #fafafa;
}

.el-table :deep(.el-table__cell) {
  vertical-align: middle;
  padding: 12px 16px;
}

.el-table :deep(.el-table__column--fixed-right) {
  background-color: #ffffff;
}

.operation-buttons {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
}

.operation-buttons .el-button {
  flex: 1;
  min-width: 60px;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
  margin: 0;
  transition: all 0.3s ease;
}

.operation-buttons .el-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.operation-buttons .el-button--danger {
  background-color: #f56c6c;
  border-color: #f56c6c;
  color: white;
}

.operation-buttons .el-button--danger:hover {
  background-color: #f78989;
  border-color: #f78989;
}

.operation-buttons .el-button:last-child {
  margin-right: 0;
}

/* 测试连接结果 */
.test-result {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 40px;
  text-align: center;
  flex-direction: column;
  min-height: 200px;
  justify-content: center;
}

.success-icon {
  font-size: 72px;
  color: #67c23a;
  margin-bottom: 20px;
  animation: pulse 1s ease-in-out;
}

.error-icon {
  font-size: 72px;
  color: #f56c6c;
  margin-bottom: 20px;
  animation: shake 0.5s ease-in-out;
}

.test-result p {
  margin: 0;
  font-size: 16px;
  color: #303133;
  font-weight: 500;
  line-height: 1.6;
  max-width: 300px;
}

/* 动画效果 */
@keyframes pulse {
  0% {
    transform: scale(0.8);
    opacity: 0.5;
  }
  50% {
    transform: scale(1.1);
    opacity: 1;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

@keyframes shake {
  0%, 100% {
    transform: translateX(0);
  }
  25% {
    transform: translateX(-5px);
  }
  75% {
    transform: translateX(5px);
  }
}

/* 删除确认 */
.delete-confirm {
  padding: 30px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.delete-confirm p {
  margin: 0;
  font-size: 15px;
  color: #303133;
  line-height: 1.6;
  font-weight: 500;
  max-width: 300px;
}

.delete-confirm-icon {
  font-size: 48px;
  color: #f56c6c;
  margin-bottom: 20px;
  animation: shake 0.5s ease-in-out;
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
  font-size: 14px;
}

/* 表单样式优化 */
:deep(.el-form-item) {
  margin-bottom: 16px;
}

:deep(.el-form-item__label) {
  font-weight: 500;
  color: #303133;
  font-size: 14px;
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

:deep(.el-input-number) {
  width: 100%;
}

:deep(.el-input-number__wrapper) {
  border-radius: 4px;
}

:deep(.el-input-number__wrapper:focus-within) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

.readonly-hint {
  margin-left: 12px;
  font-size: 12px;
  color: #909399;
}

.username-hint {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: #909399;
  line-height: 1.4;
}
</style>
