<template>
  <div class="server-config-container">
    <div class="header">
      <h2>服务器配置</h2>
      <el-button type="primary" @click="showAddDialog = true">
        <el-icon><Plus /></el-icon>
        添加服务器
      </el-button>
    </div>

    <el-table :data="servers" style="width: 100%">
      <el-table-column prop="name" label="服务器名称" width="180" />
      <el-table-column prop="host" label="主机地址" />
      <el-table-column prop="port" label="端口" width="100" />
      <el-table-column prop="db" label="默认DB" width="100" />
      <el-table-column prop="created" label="创建时间" width="200" />
      <el-table-column prop="updated" label="更新时间" width="200" />
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="editServer(row)">
            <el-icon><Edit /></el-icon>
            编辑
          </el-button>
          <el-button size="small" type="danger" @click="deleteServer(row)">
            <el-icon><Delete /></el-icon>
            删除
          </el-button>
          <el-button size="small" @click="testConnection(row)">
            <el-icon><Connection /></el-icon>
            测试
          </el-button>
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

// 表单数据
const serverForm = ref({
  id: '',
  name: '',
  host: '',
  port: 6379,
  password: '',
  db: 0
})

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

const deleteServer = async (row: any) => {
  try {
    await server.deleteServer(row.id)
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
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header h2 {
  margin: 0;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px;
  text-align: center;
}

.success-icon {
  font-size: 48px;
  color: #67c23a;
}

.error-icon {
  font-size: 48px;
  color: #f56c6c;
}

.test-result p {
  margin: 0;
  font-size: 16px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>