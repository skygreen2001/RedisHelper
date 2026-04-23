<template>
  <div class="server-config-container">
    <div class="header">
      <h2>服务器配置</h2>
      <el-button type="primary" @click="showAddDialog = true">
        <el-icon><Plus /></el-icon>
        添加服务器
      </el-button>
    </div>

    <div v-if="servers.length === 0" class="empty-state">
      <el-empty description="暂无服务器配置" />
    </div>
    <el-table v-else :data="servers" style="width: 100%" stripe>
      <el-table-column prop="id" label="标识" width="120" />
      <el-table-column prop="name" label="名称" width="180" />
      <el-table-column prop="host" label="服务器" width="180" />
      <el-table-column prop="port" label="端口" width="100" />
      <el-table-column prop="db" label="默认DB" width="100" />
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

    <!-- 添加服务器对话框 -->
    <el-dialog
      v-model="showAddDialog"
      title="添加服务器"
      width="500px"
    >
      <el-form :model="serverForm" label-width="80px">
        <el-form-item label="标识" required>
          <el-input v-model="serverForm.id" placeholder="输入服务器标识" />
        </el-form-item>
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
          <el-input v-model="serverForm.password" type="password" placeholder="输入密码" />
        </el-form-item>
        <el-form-item label="默认DB">
          <el-input-number v-model="serverForm.db" :min="0" :max="15" />
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
        <el-form-item label="标识" required>
          <el-input v-model="serverForm.id" disabled />
        </el-form-item>
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
          <el-input v-model="serverForm.password" type="password" placeholder="输入密码" />
        </el-form-item>
        <el-form-item label="默认DB">
          <el-input-number v-model="serverForm.db" :min="0" :max="15" />
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
        <p>删除后将不能恢复，确认要删除{{ selectedServer?.name }}[{{ selectedServer?.id }}]?</p>
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
import { ref, computed, onMounted } from 'vue'
import { Plus, Edit, Delete, Connection, Check, Close } from '@element-plus/icons-vue'
import { serverStore } from '../stores/serverStore'

const server = serverStore()

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
  db: 0
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
      db: serverForm.value.db
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
    db: row.db
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
      db: serverForm.value.db
    })
    showEditDialog.value = false
  } catch (error) {
    console.error('更新服务器失败:', error)
  }
}

const confirmDelete = (row: any) => {
  selectedServer.value = row
  showDeleteDialog.value = true
}

const deleteServer = async () => {
  if (!selectedServer.value) return
  
  try {
    await server.deleteServer(selectedServer.value.id)
    showDeleteDialog.value = false
    selectedServer.value = null
  } catch (error) {
    console.error('删除服务器失败:', error)
  }
}

const testConnection = async (row: any) => {
  try {
    const result = await server.testConnection({
      host: row.host,
      port: row.port,
      password: row.password
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
    db: 0
  }
}

// 生命周期
onMounted(async () => {
  await server.loadServers()
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

.header .el-button {
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 14px;
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
</style>