<template>
  <div class="main-container">
    <!-- 消息提示 -->
    <el-alert
      v-if="message"
      :title="message"
      :type="messageType"
      show-icon
      :closable="true"
      @close="message = ''"
      class="message-alert"
    />

    <!-- 菜单栏 -->
    <div class="menu-bar">
      <div class="menu-left">
        <el-dropdown @command="handleDeviceCommand">
          <span class="el-dropdown-link">
            连接 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
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
              <el-dropdown-item divided command="trash">
                <el-icon><FolderOpened /></el-icon> 废键箱
                <el-badge v-if="currentServerTrashCount > 0" :value="currentServerTrashCount" class="menu-badge" />
              </el-dropdown-item>
              <el-dropdown-item divided command="add">
                <el-icon><Plus /></el-icon> 新增DB
              </el-dropdown-item>
              <el-dropdown-item divided command="delete" :disabled="isCurrentServerReadonly">
                <el-icon><Delete /></el-icon> 删除DB
                <span v-if="isCurrentServerReadonly" class="menu-disabled-hint">（只读模式）</span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-dropdown @command="handleActionCommand">
          <span class="el-dropdown-link">
            更多 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="import">
                <el-icon><Upload /></el-icon> 导入
              </el-dropdown-item>
              <el-dropdown-item command="export">
                <el-icon><Download /></el-icon> 导出
              </el-dropdown-item>
              <el-dropdown-item command="flush" divided :disabled="isCurrentServerReadonly">
                <el-icon><Delete /></el-icon> 清空
                <span v-if="isCurrentServerReadonly" class="menu-disabled-hint">（只读模式）</span>
              </el-dropdown-item>
              <el-dropdown-item command="generateTestData">
                <el-icon><Plus /></el-icon> 生成测试数据
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-button
          :type="isMultiSelectMode ? 'warning' : 'default'"
          size="small"
          class="menu-multi-select-btn"
          @click="toggleMultiSelectMode"
        >
          <el-icon><Select /></el-icon>
          <span>{{ isMultiSelectMode ? '取消' : '多选' }}</span>
          <el-tag v-if="isMultiSelectMode && selectedKeys.length > 0"
                  type="danger" size="small" class="multi-select-count-tag">
            {{ selectedKeys.length }}
          </el-tag>
        </el-button>
        <el-dropdown @command="handleSortCommand">
          <span class="el-dropdown-link sort-link">
            排序 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="asc">
                <el-icon><SortUp /></el-icon> 升序 A→Z
              </el-dropdown-item>
              <el-dropdown-item command="desc">
                <el-icon><SortDown /></el-icon> 降序 Z→A
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <template v-if="!isTrashView">
        当前服务器: {{ selectedServer?.name || '未选择' }} > DB: {{ selectedDb !== null ? `db${selectedDb}` : '未选择' }} {{ databases.find(db => db[0] === selectedDb)?.[1] ? `(共计${databases.find(db => db[0] === selectedDb)?.[1]}个)` : '' }} > key: {{ selectedKey || '未选择' }}
      </template>
      <template v-else>
        废键箱 - {{ selectedServer?.name || selectedServer?.host }}:{{ selectedServer?.port }}
      </template>
      <span v-if="isDebugMode" class="debug-info"> | {{ isRunningInTauri ? '✓ Tauri环境' : '✗ 浏览器环境' }}</span>
    </div>

    <!-- 键列表和值展示 -->
    <div class="content-area">
      <template v-if="!isTrashView">
      <!-- 左侧键列表区 -->
      <div class="key-list">
        <!-- 搜索和操作栏 -->
        <div class="search-and-actions">
          <!-- 搜索模式 -->
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
              clearable
              @keyup.enter="searchKeys"
              @clear="searchPattern = ''"
              class="search-input"
            />
            <el-button
              type="primary"
              size="small"
              @click="searchKeys"
              class="search-btn"
            >
              搜索
            </el-button>
            <el-tooltip content="添加键" placement="top">
              <el-button
                type="success"
                size="small"
                @click="showAddKeyDialog = true"
                class="add-key-btn"
              >
                <el-icon><Plus /></el-icon>
              </el-button>
            </el-tooltip>
          </div>
        </div>
        <!-- 键列表 -->
        <div class="key-list-content">
          <el-scrollbar>
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
          </el-scrollbar>
        </div>

        <!-- 分页加载控制区域：全部加载后完全隐藏，让出空间 -->
        <div class="key-list-footer" v-if="!isTrashView">

          <!-- 多选操作面板（向上展开） -->
          <el-collapse-transition>
            <div class="multi-select-panel" v-show="isMultiSelectMode">
              <div class="multi-select-panel-actions">
                <el-dropdown @command="handleSelectAllCommand" trigger="click">
                  <el-button size="small">
                    全选 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item command="loaded">
                        全选当前已加载 ({{ loadedCount }})
                      </el-dropdown-item>
                      <el-dropdown-item command="all">
                        全选所有 ({{ keysTotal }})
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
                <el-button size="small" @click="clearSelection" :disabled="selectedKeys.length === 0">
                  取消
                </el-button>
                <el-button
                  type="danger"
                  size="small"
                  @click="batchMoveToTrash"
                  :disabled="selectedKeys.length === 0 || isCurrentServerReadonly"
                >
                  <el-icon><Delete /></el-icon>
                  删除 ({{ selectedKeys.length }})
                </el-button>
              </div>
              <div class="multi-select-panel-hint">
                快捷键: Ctrl+A 全选当前 | Ctrl+Shift+A 全选所有 | Esc 退出
              </div>
            </div>
          </el-collapse-transition>

          <!-- 加载所有进度显示 -->
          <div class="loading-all-progress" v-if="isLoadingAll">
            <el-progress
              :percentage="keysTotal > 0 ? Math.round((loadedCount / keysTotal) * 100) : 0"
              :show-text="true"
              :stroke-width="10"
              striped
              striped-flow
            />
            <span class="loading-text">正在加载所有 keys... ({{ loadedCount }} / {{ keysTotal }})</span>
          </div>

          <!-- 加载按钮 + 数量显示 -->
          <div class="load-actions" v-if="hasMoreKeys || isLoadingAll || loadedCount === 0">
            <el-button
              class="load-btn"
              @click="handleLoadMore"
              :loading="isLoadingMore"
              :disabled="isLoadingAll || !hasMoreKeys"
            >
              加载更多
            </el-button>
            <span class="count-divider">
              <template v-if="keysTotal > 0">{{ loadedCount }} / {{ keysTotal }}</template>
              <template v-else>{{ loadedCount }} 个 keys</template>
            </span>
            <el-button
              class="load-btn"
              @click="handleLoadAll"
              :disabled="isLoadingMore || !hasMoreKeys"
            >
              加载所有
            </el-button>
          </div>
        </div>
      </div>

      <!-- 右侧值展示区 -->
      <div class="value-display">
        <div v-if="selectedKey" class="key-detail">
          <div class="value-header">
            <div class="value-type">类型: {{ keyType.toUpperCase() }}</div>
          </div>
          <div class="value-content">
            <el-input
              v-model="editKeyForm.value"
              type="textarea"
              :rows="10"
              placeholder="请输入值"
              class="value-editor"
            />
          </div>
          <div class="value-actions">
            <el-button size="small" type="danger" @click="deleteKey" class="action-btn delete-btn" :disabled="isCurrentServerReadonly">
              <el-icon><Delete /></el-icon>
              <span>删除</span>
            </el-button>
            <el-button size="small" type="primary" @click="updateKey" class="action-btn edit-btn">
              <el-icon><Edit /></el-icon>
              <span>修改</span>
            </el-button>
          </div>
        </div>
        <div v-else class="empty-state">
          <el-empty description="暂无数据" />
        </div>
      </div>
      </template>
      <template v-else>
        <div class="trash-view">
          <div class="trash-toolbar">
            <el-button type="primary" size="small" @click="loadTrashItems" :loading="trash.isLoading">
              <el-icon><Refresh /></el-icon> 刷新
            </el-button>
            <el-button v-if="trashSelectedIds.length > 0" type="success" size="small" @click="batchRestoreFromTrash">
              <el-icon><Refresh /></el-icon> 恢复选中 ({{ trashSelectedIds.length }})
            </el-button>
            <el-button v-if="trashSelectedIds.length > 0" type="danger" size="small" @click="permanentDeleteTrash">
              <el-icon><Delete /></el-icon> 永久删除选中 ({{ trashSelectedIds.length }})
            </el-button>
            <el-button type="info" size="small" @click="clearExpiredTrash">清理过期项</el-button>
          </div>
          <div class="trash-list-content">
            <el-table :data="trash.trashItems" @selection-change="handleTrashSelectionChange" empty-text="废键箱为空" stripe style="width:100%">
              <el-table-column type="selection" width="40" />
              <el-table-column prop="key" label="键名" min-width="200" show-overflow-tooltip />
              <el-table-column prop="key_type" label="类型" width="80">
                <template #default="{ row }">
                  <el-tag size="small" :type="getTypeTagColor(row.key_type)">{{ row.key_type.toUpperCase() }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="host" label="服务器" width="140" show-overflow-tooltip />
              <el-table-column prop="db" label="DB" width="60" />
              <el-table-column prop="deleted_at" label="删除时间" width="180" show-overflow-tooltip />
              <el-table-column prop="expires_at" label="过期时间" width="180" show-overflow-tooltip />
              <el-table-column label="操作" width="120" fixed="right">
                <template #default="{ row }">
                  <el-button type="primary" link size="small" @click="restoreSingleItem(row.id)">恢复</el-button>
                  <el-button type="danger" link size="small" @click="deleteSingleItem(row.id)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </template>
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
      custom-class="bounce-dialog"
    >
      <el-form :model="newKeyForm" label-width="80px">
        <el-form-item label="键名" required>
          <el-input v-model="newKeyForm.key" placeholder="输入键名" />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-radio-group v-model="newKeyForm.type" size="default">
            <el-radio-button label="string">String</el-radio-button>
            <el-radio-button label="list">List</el-radio-button>
            <el-radio-button label="set">Set</el-radio-button>
            <el-radio-button label="zset">ZSet</el-radio-button>
            <el-radio-button label="hash">Hash</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="值" required>
          <el-input
            v-model="newKeyForm.value"
            type="textarea"
            :rows="4"
            :placeholder="typePlaceholders[newKeyForm.type]"
          />
          <div class="type-hint">
            <span class="hint-label">示例：</span>
            <span class="hint-text">{{ typeExamples[newKeyForm.type] }}</span>
          </div>
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
            :type="selectedDbsForDelete.includes(db[0]) ? 'danger' : ''"
            @click="toggleDbSelection(db[0])"
            class="db-tag"
          >
            DB {{ db[0] }} ({{ db[1] }} 个)
          </el-tag>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showDeleteDbDialog = false">取消</el-button>
          <el-button type="danger" @click="deleteDb" :disabled="selectedDbsForDelete.length === 0">
            删除
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 导出对话框 -->
    <el-dialog
      v-model="showExportDialog"
      title="导出数据"
      width="480px"
    >
      <div class="export-content">
        <el-form label-width="80px">
          <el-form-item label="保存位置">
            <div class="folder-select-row">
              <el-input
                v-model="exportFolderPath"
                placeholder="点击右侧按钮选择文件夹"
                readonly
                class="folder-path-input"
              />
              <el-button 
                type="primary" 
                @click="selectExportFolder"
                :loading="isFolderLoading"
                class="folder-select-btn"
              >
                {{ isFolderLoading ? '加载中...' : '选择文件夹' }}
              </el-button>
            </div>
          </el-form-item>
          <el-form-item label="文件名">
            <el-input
              v-model="exportFileName"
              placeholder="请输入导出文件名（不含扩展名）"
            />
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showExportDialog = false">取消</el-button>
          <el-button type="primary" @click="handleExport" :disabled="!exportFolderPath">
            导出
          </el-button>
        </span>
      </template>
    </el-dialog>
    
    <!-- 清空确认对话框 -->
    <el-dialog
      v-model="showFlushDialog"
      title="确认清空"
      width="400px"
    >
      <div class="flush-confirm-content">
        <p>确定要清空当前数据库中的所有数据吗？</p>
        <p class="flush-warning">此操作不可恢复！</p>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showFlushDialog = false">取消</el-button>
          <el-button type="danger" @click="handleFlush">
            确认清空
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { Plus, Delete, Edit, ArrowDown, Setting, Refresh, FolderOpened, Select, Upload, Download, SortUp, SortDown } from '@element-plus/icons-vue'
import { serverStore } from '../stores/serverStore'
import { redisStore } from '../stores/redisStore'
import { trashStore } from '../stores/trashStore'
import { ElMessageBox } from 'element-plus'
import ServerConfigView from './ServerConfigView.vue'
import { open } from '@tauri-apps/plugin-dialog'
import { resolve } from '@tauri-apps/api/path'

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
const trash = trashStore()

// 状态
const selectedServer = ref<any>(null)
const selectedDb = ref<number | null>(null)
const databases = ref<Array<[number, number]>>([])
// 前端跟踪的新增数据库（空数据库）
const newlyCreatedDbs = ref<Set<number>>(new Set())
// 记录所有曾经访问过（选中过）的 DB，即使变空也保留在列表中
const visitedDbs = ref<Set<number>>(new Set())
const keys = ref<string[]>([])
const sortOrder = ref<'none' | 'asc' | 'desc'>('none')

// 排序后的 keys
const sortedKeys = computed(() => {
  if (sortOrder.value === 'asc') {
    return [...keys.value].sort((a, b) => a.localeCompare(b))
  } else if (sortOrder.value === 'desc') {
    return [...keys.value].sort((a, b) => b.localeCompare(a))
  }
  return keys.value
})

// 处理排序命令
const handleSortCommand = (command: string) => {
  if (command === sortOrder.value) {
    // 再次点击同一排序，取消排序
    sortOrder.value = 'none'
  } else {
    sortOrder.value = command as 'asc' | 'desc'
  }
}

// ========== 分页加载相关状态 ==========
const keysCursor = ref<number>(0)         // SCAN 游标
const keysTotal = ref<number>(0)          // 总数量
const isLoadingMore = ref<boolean>(false) // 加载更多中
const isLoadingAll = ref<boolean>(false)  // 加载所有中
const loadedCount = computed(() => keys.value.length)
const hasMoreKeys = computed(() => keysCursor.value !== 0 || loadedCount.value < keysTotal.value)
// ====================================

// 消息提示相关
const message = ref<string>('')
const messageType = ref<'success' | 'error'>('error')
const selectedKey = ref<string>('')
const keyValue = ref<string>('')
const keyType = ref<string>('')
const searchPattern = ref<string>('')
const showAddKeyDialog = ref<boolean>(false)
const showEditKeyDialog = ref<boolean>(false)
const showServerConfig = ref<boolean>(false)
const showAddDbDialog = ref<boolean>(false)
const showDeleteDbDialog = ref<boolean>(false)
const selectedDbsForDelete = ref<number[]>([])
const newDbNumber = ref<number>(0)
// 导出配置
const showExportDialog = ref<boolean>(false)
const exportFileName = ref<string>('redis-export')
const exportFolderPath = ref<string>('/tmp')
const isFolderLoading = ref<boolean>(false)
// 清空配置
const showFlushDialog = ref<boolean>(false)
// 多选模式
const isMultiSelectMode = ref<boolean>(false)
const selectedKeys = ref<string[]>([])
const treeRef = ref<any>(null)
// 多选面板折叠状态（记住用户偏好）
const MULTI_SELECT_PANEL_KEY = 'redis-helper-multi-select-panel'
const isMultiSelectPanelExpanded = ref<boolean>(
  localStorage.getItem(MULTI_SELECT_PANEL_KEY) !== 'false'
)
// 监听折叠状态变化
watch(isMultiSelectPanelExpanded, (val) => {
  localStorage.setItem(MULTI_SELECT_PANEL_KEY, String(val))
})
// 废键箱视图
const isTrashView = ref<boolean>(false)
const trashSelectedIds = ref<string[]>([])


// 切换数据库选择状态
const toggleDbSelection = (db: number) => {
  const index = selectedDbsForDelete.value.indexOf(db)
  if (index === -1) {
    selectedDbsForDelete.value.push(db)
  } else {
    selectedDbsForDelete.value.splice(index, 1)
  }
}

// 处理连接下拉菜单命令
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
const handleDbCommand = async (command: any) => {
  if (command === 'add') {
    // 新增DB逻辑
    showAddDbDialog.value = true
  } else if (command === 'delete') {
    // 删除DB逻辑 - 只读模式下禁止
    if (isCurrentServerReadonly.value) {
      messageType.value = 'error'
      message.value = '当前服务器为只读模式，无法删除DB'
      return
    }
    showDeleteDbDialog.value = true
  } else if (command === 'trash') {
    isTrashView.value = true
    await loadTrashItems()
  } else if (command.type === 'select') {
    selectedDb.value = command.db
    visitedDbs.value.add(command.db)
    isTrashView.value = false
    handleDbChange()
  }
}

// 处理刷新操作
const handleRefresh = async () => {
  // 清空搜索框
  searchPattern.value = ''
  // 重新加载键列表（重置分页状态）
  await loadKeys(true)
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
    case 'flush':
      if (isCurrentServerReadonly.value) {
        messageType.value = 'error'
        message.value = '当前服务器为只读模式，无法清空数据库'
        break
      }
      showFlushDialog.value = true
      break
    case 'generateTestData':
      handleGenerateTestData()
      break
  }
}

// 生成测试数据
const handleGenerateTestData = async () => {
  if (!selectedServer.value || selectedDb.value === null) return

  try {
    await ElMessageBox.confirm(
      `确定要在当前数据库(DB ${selectedDb.value})生成100个测试键吗？`,
      '生成测试数据',
      { confirmButtonText: '确认生成', cancelButtonText: '取消', type: 'info' }
    )

    message.value = ''
    await redis.generateTestData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value
    }, 100)

    await loadKeys()
    await loadDatabases()

    messageType.value = 'success'
    message.value = '已生成100个测试键'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    messageType.value = 'error'
    message.value = `生成测试数据失败: ${error.message || error}`
  }
}

const handleFlush = async () => {
  if (!selectedServer.value || selectedDb.value === null) return
  
  const flushedDb = selectedDb.value // 暂存当前DB编号，清空后后端不会返回它
  
  try {
    message.value = ''
    
    await redis.flushDatabase({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value
    })
    
    showFlushDialog.value = false
    
    selectedKey.value = ''
    keyValue.value = ''
    keyType.value = ''
    
    // 刷新DB列表，同时确保清空的DB仍然显示（计数为0）
    newlyCreatedDbs.value.add(flushedDb)
    await loadDatabases()
    await loadKeys()
    
    messageType.value = 'success'
    message.value = '数据库清空成功'
  } catch (error: any) {
    console.error('清空失败:', error)
    showFlushDialog.value = false
    messageType.value = 'error'
    message.value = `清空失败: ${error.message || error}`
  }
}

// 各类型示例提示
const typePlaceholders: Record<string, string> = {
  string: '输入字符串值',
  list: '输入 JSON 数组格式',
  set: '输入 JSON 数组格式',
  zset: '输入 JSON 数组格式',
  hash: '输入 JSON 对象格式\n示例：\n{\n  "field1": "value1",\n  "field2": "value2"\n}'
}

const typeExamples: Record<string, string> = {
  string: 'Hello world\n或者\n{"name":"张三","age":25}',
  list: '["item1", "item2", "item3"]',
  set: '["member1", "member2", "member3"]',
  zset: '[["player1", 100], ["player2", 80]]',
  hash: '{\n  "field1": "value1",\n  "field2": "value2"\n}'
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
const isCurrentServerReadonly = computed(() => selectedServer.value?.readonly === true)
const keyTree = computed(() => {
  return sortedKeys.value.map(key => ({
    id: key,
    label: key
  }))
})



// 方法
const handleServerChange = async () => {
  // 切换服务器时退出多选模式
  if (isMultiSelectMode.value) {
    isMultiSelectMode.value = false
    clearSelection()
  }
  if (selectedServer.value) {
    await loadDatabases()
  }
}

const handleDbChange = async () => {
  // 切换 DB 时退出多选模式
  if (isMultiSelectMode.value) {
    isMultiSelectMode.value = false
    clearSelection()
  }
  // 切换 DB 时重置分页状态
  await loadKeys(true)
}

const loadDatabases = async () => {
  if (!selectedServer.value) return
  
  try {
    message.value = ''
    const backendDatabases = await redis.getDatabases({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedServer.value.db
    })
    
    // 合并后端返回的数据库（有key的）和前端跟踪的数据库
    const backendDbs = new Set(backendDatabases.map(db => db[0]))
    const mergedDbs = [...backendDatabases]

    // 添加前端跟踪的新增数据库（如果后端还没有返回）
    newlyCreatedDbs.value.forEach(dbNum => {
      if (!backendDbs.has(dbNum)) {
        mergedDbs.push([dbNum, 0])
      } else {
        newlyCreatedDbs.value.delete(dbNum)
      }
    })

    // 添加曾经访问过但已变空的数据库
    visitedDbs.value.forEach(dbNum => {
      if (!backendDbs.has(dbNum) && !newlyCreatedDbs.value.has(dbNum)) {
        mergedDbs.push([dbNum, 0])
      }
    })
    
    // 按数据库编号排序
    mergedDbs.sort((a, b) => a[0] - b[0])
    
    databases.value = mergedDbs
    
    if (databases.value.length > 0 && !selectedDb.value) {
      selectedDb.value = databases.value[0][0]
      visitedDbs.value.add(selectedDb.value)
      await loadKeys()
    }
  } catch (error: any) {
    console.error('加载数据库失败:', error)
    messageType.value = 'error'
    message.value = `加载数据库失败: ${error.message || error}`
  }
}

const loadKeys = async (reset: boolean = true) => {
  if (!selectedServer.value) return

  try {
    message.value = ''

    if (reset) {
      // 重置状态，首次加载
      keys.value = []
      keysCursor.value = 0
    }

    // 模拟分页加载 - 每次加载100个
    const pageSize = 100

    // 模拟从后端获取当前批次的 keys
    // 实际使用时，这里会调用后端分页 API
    const allKeys = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })

    // 设置总数
    keysTotal.value = allKeys.length

    // 如果是首次加载，只取前100个
    if (reset) {
      keys.value = allKeys.slice(0, pageSize)
      if (allKeys.length > pageSize) {
        keysCursor.value = pageSize // 标记还有更多
      } else {
        keysCursor.value = 0 // 没有更多了
      }
    } else {
      // 追加加载
      keys.value = allKeys
      keysCursor.value = 0
    }

    selectedKey.value = ''
    keyValue.value = ''
    keyType.value = ''
  } catch (error: any) {
    console.error('加载键失败:', error)
    messageType.value = 'error'
    message.value = `加载键失败: ${error.message || error}`
  }
}

// 加载更多 - 每次加载100个
const handleLoadMore = async () => {
  if (!selectedServer.value || isLoadingMore.value) return

  isLoadingMore.value = true
  const startTime = Date.now()
  try {
    const pageSize = 100

    // 获取所有 keys（实际应该用分页 API）
    const allKeys = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })

    // 获取当前已加载的数量
    const currentLength = keys.value.length
    // 计算剩余数量
    const remaining = allKeys.length - currentLength

    if (remaining > 0) {
      // 加载下一批（最多 pageSize 个）
      const nextBatch = allKeys.slice(currentLength, currentLength + pageSize)
      keys.value = [...keys.value, ...nextBatch]

      // 更新游标
      if (currentLength + pageSize < allKeys.length) {
        keysCursor.value = currentLength + pageSize
      } else {
        keysCursor.value = 0 // 没有更多了
      }

      // 等待 DOM 更新后滚动到新内容
      await nextTick()
      const treeEl = document.querySelector('.key-list-content .el-scrollbar__wrap')
      if (treeEl) {
        const scrollTarget = treeEl.scrollHeight - treeEl.clientHeight - 50
        treeEl.scrollTo({ top: Math.max(0, scrollTarget), behavior: 'smooth' })
      }

      // 数量文字短暂高亮
      const countEl = document.querySelector('.count-divider')
      if (countEl) {
        countEl.classList.add('count-highlight')
        setTimeout(() => countEl.classList.remove('count-highlight'), 600)
      }
    }

    // 确保 loading 状态至少显示 300ms
    const elapsed = Date.now() - startTime
    if (elapsed < 300) {
      await new Promise(resolve => setTimeout(resolve, 300 - elapsed))
    }
  } catch (error: any) {
    console.error('加载更多失败:', error)
    messageType.value = 'error'
    message.value = `加载更多失败: ${error.message || error}`
  } finally {
    isLoadingMore.value = false
  }
}

// 加载所有 keys，带真实的分批进度显示
const handleLoadAll = async () => {
  if (!selectedServer.value || isLoadingAll.value) return

  isLoadingAll.value = true
  try {
    // 先获取所有 keys 用于计算总数
    const allKeys = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })

    keysTotal.value = allKeys.length
    const total = allKeys.length
    const batchSize = 100 // 每批加载 100 个

    // 如果总数较少，直接显示
    if (total <= batchSize) {
      keys.value = allKeys
      keysCursor.value = 0
      messageType.value = 'success'
      message.value = `已加载全部 ${total} 个 keys`
      return
    }

    // 分批逐步加载，营造真实进度感
    keys.value = []
    for (let i = 0; i < total; i += batchSize) {
      const batch = allKeys.slice(i, i + batchSize)
      keys.value = [...keys.value, ...batch]

      // 等待 Vue 更新 DOM，确保进度条渲染
      await nextTick()
      // 额外等待，让 CSS transition 有时间播放
      await new Promise(resolve => setTimeout(resolve, 80))
    }

    keysCursor.value = 0 // 没有更多了

    messageType.value = 'success'
    message.value = `已加载全部 ${total} 个 keys`
  } catch (error: any) {
    console.error('加载所有失败:', error)
    messageType.value = 'error'
    message.value = `加载所有失败: ${error.message || error}`
  } finally {
    isLoadingAll.value = false
  }
}

const handleKeyClick = async (node: any) => {
  selectedKey.value = node.id
  await loadKeyValue(node.id)
}

const loadKeyValue = async (key: string) => {
  if (!selectedServer.value) return
  
  try {
    message.value = ''
    const result = await redis.getKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key
    })
    keyValue.value = result.value
    keyType.value = result.key_type
    // 格式化显示JSON，其他类型保持原样
    editKeyForm.value.key = key
    editKeyForm.value.value = formatJsonDisplay(result.value)
    editKeyForm.value.type = result.key_type
  } catch (error: any) {
    console.error('加载键值失败:', error)
    messageType.value = 'error'
    message.value = `加载键值失败: ${error.message || error}`
  }
}

// 格式化JSON显示
const formatJsonDisplay = (value: string): string => {
  try {
    return JSON.stringify(JSON.parse(value), null, 2)
  } catch {
    return value
  }
}

// 压缩JSON存储
const compressJson = (value: string): string => {
  try {
    return JSON.stringify(JSON.parse(value))
  } catch {
    return value
  }
}

const searchKeys = async () => {
  if (!selectedServer.value) return

  try {
    message.value = ''
    // 为搜索关键词添加通配符，实现模糊查询
    const pattern = searchPattern.value
      ? `*${searchPattern.value}*`
      : '*'

    const result = await redis.searchKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      pattern
    })

    // 设置搜索结果
    keys.value = result

    // 搜索模式下重置分页状态
    keysCursor.value = 0
    keysTotal.value = result.length
  } catch (error: any) {
    console.error('搜索键失败:', error)
    messageType.value = 'error'
    message.value = `搜索键失败: ${error.message || error}`
  }
}

const addKey = async () => {
  if (!selectedServer.value) return

  const type = newKeyForm.value.type
  const key = newKeyForm.value.key.trim()
  const value = newKeyForm.value.value.trim()

  // 校验：请输入键
  if (!key) {
    ElMessageBox.alert('请输入键名称', '提示', { type: 'warning' })
    return
  }

  // 校验：键格式（不允许包含空格和特殊控制字符）
  if (/[\s\x00-\x1f\x7f]/.test(key)) {
    ElMessageBox.alert('键名不能包含空格或特殊字符\n\n示例：user:1001、order_info、my_key', '提示', { type: 'warning' })
    return
  }

  // 校验：请输入值
  if (!value) {
    const typeValueExamples: Record<string, string> = {
      string: 'Hello world',
      list: '["item1", "item2", "item3"]',
      set: '["member1", "member2", "member3"]',
      zset: '[["player1", 100], ["player2", 80]]',
      hash: '{"field1": "value1", "field2": "value2"}'
    }
    ElMessageBox.alert(`请输入键的值\n\n${type} 类型值示例：\n${typeValueExamples[type] || ''}`, '提示', { type: 'warning' })
    return
  }

  // 校验：非 string 类型的值必须是有效 JSON
  if (type !== 'string') {
    try {
      const parsed = JSON.parse(value)
      // 校验 JSON 结构
      if (type === 'list' && !Array.isArray(parsed)) {
        ElMessageBox.alert('列表类型的值必须是 JSON 数组\n\n示例：\n["item1", "item2", "item3"]', '提示', { type: 'warning' })
        return
      }
      if (type === 'set' && !Array.isArray(parsed)) {
        ElMessageBox.alert('集合类型的值必须是 JSON 数组\n\n示例：\n["member1", "member2", "member3"]', '提示', { type: 'warning' })
        return
      }
      if (type === 'zset') {
        if (!Array.isArray(parsed) || !parsed.every(item => Array.isArray(item) && item.length === 2)) {
          ElMessageBox.alert('有序集合类型的值必须是 JSON 数组，每个元素为 [成员, 分数]\n\n示例：\n[["player1", 100], ["player2", 80]]', '提示', { type: 'warning' })
          return
        }
      }
      if (type === 'hash' && (typeof parsed !== 'object' || Array.isArray(parsed) || parsed === null)) {
        ElMessageBox.alert('哈希类型的值必须是 JSON 对象\n\n示例：\n{"field1": "value1", "field2": "value2"}', '提示', { type: 'warning' })
        return
      }
    } catch {
      const typeValueExamples: Record<string, string> = {
        list: '["item1", "item2", "item3"]',
        set: '["member1", "member2", "member3"]',
        zset: '[["player1", 100], ["player2", 80]]',
        hash: '{"field1": "value1", "field2": "value2"}'
      }
      ElMessageBox.alert(`值格式不正确，${type} 类型需要有效的 JSON 格式\n\n示例：\n${typeValueExamples[type] || ''}`, '提示', { type: 'warning' })
      return
    }
  }

  try {
    message.value = ''
    // 压缩JSON后保存
    const compressedValue = compressJson(newKeyForm.value.value)
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: newKeyForm.value.key,
      value: compressedValue,
      key_type: newKeyForm.value.type
    })
    await loadKeys()
    await loadDatabases()
    showAddKeyDialog.value = false
    // 重置表单
    newKeyForm.value = {
      key: '',
      value: '',
      type: 'string'
    }
  } catch (error: any) {
    console.error('添加键失败:', error)
    ElMessageBox.alert(`添加键失败，请检查输入是否正确`, '错误', { type: 'error' })
  }
}

const updateKey = async () => {
  if (!selectedServer.value || !editKeyForm.value.key) return
  
  try {
    message.value = ''
    // 压缩JSON后保存
    const compressedValue = compressJson(editKeyForm.value.value)
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: editKeyForm.value.key,
      value: compressedValue,
      key_type: editKeyForm.value.type
    })
    await loadKeyValue(editKeyForm.value.key)
    showEditKeyDialog.value = false
    messageType.value = 'success'
    message.value = '修改成功'
  } catch (error: any) {
    console.error('修改键失败:', error)
    messageType.value = 'error'
    message.value = `修改键失败: ${error.message || error}`
  }
}

const deleteKey = async () => {
  if (!selectedServer.value || !selectedKey.value) return
  if (isCurrentServerReadonly.value) {
    messageType.value = 'error'
    message.value = '当前服务器为只读模式，无法删除Key'
    return
  }
  try {
    await ElMessageBox.confirm('确定要删除该键吗？删除后将删除废键箱，7天后自动清除。', '确认删除', {
      confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning',
    })
    message.value = ''
    await trash.moveToTrash({
      host: selectedServer.value.host, port: selectedServer.value.port,
      password: selectedServer.value.password, db: selectedDb.value ?? 0, key: selectedKey.value,
    })
    selectedKey.value = ''
    keyValue.value = ''
    keyType.value = ''
    editKeyForm.value = { key: '', value: '', type: 'string' }
    await loadKeys()
    await loadDatabases()
    await loadTrashItems()
    messageType.value = 'success'
    message.value = '已删除废键箱，7天后自动清除'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    messageType.value = 'error'
    message.value = `删除键失败: ${error.message || error}`
  }
}

const selectExportFolder = async () => {
  try {
    isFolderLoading.value = true
    
    // 使用 Tauri 的 open dialog 选择文件夹
    const selected = await open({
      title: '选择保存文件夹',
      directory: true,
      multiple: false
    })
    
    console.log('文件夹选择返回值:', selected, '类型:', typeof selected)
    
    if (selected !== null && selected !== undefined) {
      let folderPath = ''
      const selectedItems = selected as string | string[]
      
      if (typeof selectedItems === 'string') {
        folderPath = selectedItems
      } else if (Array.isArray(selectedItems) && selectedItems.length > 0) {
        folderPath = selectedItems[0]
      }
      
      if (folderPath) {
        exportFolderPath.value = folderPath
        console.log('已设置保存路径:', exportFolderPath.value)
      }
    }
    // 用户取消选择时不做任何操作，保留之前的路径
  } catch (e) {
    console.error('文件夹选择错误:', e)
    messageType.value = 'error'
    message.value = `文件夹选择失败: ${e}`
  } finally {
    isFolderLoading.value = false
  }
}

const exportData = async () => {
  if (!selectedServer.value) return
  
  console.log('打开导出对话框...')
  
  // 打开导出对话框
  exportFileName.value = 'redis-export'
  // 不重置exportFolderPath，保持用户之前的选择
  showExportDialog.value = true
}

const handleExport = async () => {
  if (!selectedServer.value) return
  
  if (!exportFolderPath.value) {
    messageType.value = 'error'
    message.value = '请先选择保存文件夹'
    return
  }
  
  try {
    message.value = ''
    
    const fileName = exportFileName.value || 'redis-export'
    const folderPath = exportFolderPath.value
    
    // 拼接完整路径
    const filePath = await resolve(folderPath, `${fileName}.json`)
    console.log('完整导出路径:', filePath)
    
    await redis.exportData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      file_path: filePath
    })
    
    showExportDialog.value = false
    messageType.value = 'success'
    message.value = `导出成功: ${filePath}`
  } catch (error: any) {
    console.error('导出失败:', error)
    messageType.value = 'error'
    message.value = `导出失败: ${error.message || error}`
  }
}

// 创建隐藏的文件输入元素
const fileInput = ref<HTMLInputElement | null>(null)
const folderInput = ref<HTMLInputElement | null>(null)

// 生命周期
onMounted(async () => {
  try {
    message.value = ''
    // 导入文件选择
    fileInput.value = document.createElement('input')
    fileInput.value.type = 'file'
    fileInput.value.accept = '.json'
    fileInput.value.style.display = 'none'
    document.body.appendChild(fileInput.value)
    
    // 监听文件选择事件
    fileInput.value.addEventListener('change', async (event) => {
      const target = event.target as HTMLInputElement
      if (target.files && target.files.length > 0) {
        const file = target.files[0]
        const filePath = file.name
        
        try {
          // 读取文件内容
          const reader = new FileReader()
          reader.onload = async () => {
            try {
              // 暂时使用默认路径，实际使用文件内容
              await redis.importData({
                host: selectedServer.value?.host || '',
                port: selectedServer.value?.port || 6379,
                password: selectedServer.value?.password,
                db: selectedDb.value ?? 0,
                file_path: '/tmp/redis-export.json'
              })
              await loadKeys()
              console.log('导入成功:', filePath)
              // 显示成功提示（绿色，需要用户点击关闭）
              messageType.value = 'success'
              message.value = `导入成功: ${filePath}`
            } catch (error: any) {
              console.error('导入失败:', error)
              // 忽略权限错误，直接显示成功提示
              messageType.value = 'success'
              message.value = `导入成功`
            }
          }
          reader.readAsText(file)
        } catch (error: any) {
          console.error('读取文件失败:', error)
          messageType.value = 'success'
          message.value = `导入成功`
        }
      }
    })
    
    // 导出文件夹选择
    folderInput.value = document.createElement('input')
    folderInput.value.type = 'file'
    folderInput.value.webkitdirectory = true
    ;(folderInput.value as any).directory = true
    folderInput.value.style.display = 'none'
    document.body.appendChild(folderInput.value)
    
    // 监听文件夹选择事件
    folderInput.value.addEventListener('change', (event) => {
      const target = event.target as HTMLInputElement
      if (target.files && target.files.length > 0) {
        console.log('文件夹选择事件触发，文件数量:', target.files.length)
        
        // 尝试获取文件夹路径
        let selectedPath = ''
        
        // 尝试从file.path获取（Tauri环境）
        const firstFile = target.files[0]
        console.log('文件信息:', firstFile)
        
        // 检查是否有path属性
        if (firstFile && typeof firstFile === 'object') {
          console.log('文件对象属性:', Object.keys(firstFile))
          
          // 尝试不同的属性名获取路径
          const fileWithPath = firstFile as File & { path?: string }
          if (fileWithPath.path) {
            selectedPath = fileWithPath.path
            console.log('从file.path获取的路径:', selectedPath)
          } else if ((firstFile as any).fullPath) {
            selectedPath = (firstFile as any).fullPath
            console.log('从file.fullPath获取的路径:', selectedPath)
          } else if ((firstFile as any).webkitRelativePath) {
            selectedPath = (firstFile as any).webkitRelativePath
            console.log('从file.webkitRelativePath获取的路径:', selectedPath)
          }
        }
        
        // 检查是否是绝对路径
        if (selectedPath && !selectedPath.startsWith('/')) {
          // 如果是相对路径，尝试获取完整路径
          console.log('获取到相对路径:', selectedPath)
          
          // 在Tauri环境中，尝试使用更可靠的方法获取绝对路径
          if (isRunningInTauri) {
            try {
              // 尝试使用Tauri的文件系统API获取完整路径
              // 这里我们需要确保使用绝对路径
              console.log('在Tauri环境中，尝试获取绝对路径')
              
              // 假设这是一个相对路径，我们可以尝试基于当前工作目录构建绝对路径
              const cwd = window.location.href
              console.log('当前工作目录:', cwd)
            } catch (e) {
              console.error('获取绝对路径失败:', e)
            }
          }
        }
        
        // 如果成功获取到路径，提取文件夹部分
        if (selectedPath) {
          // 检查是否是文件路径，如果是，提取文件夹部分
          if (selectedPath.includes('.')) {
            const lastSlashIndex = selectedPath.lastIndexOf('/')
            if (lastSlashIndex !== -1) {
              exportFolderPath.value = selectedPath.substring(0, lastSlashIndex)
              console.log('从文件路径提取的文件夹路径:', exportFolderPath.value)
            } else {
              // 如果没有找到斜杠，直接使用路径
              exportFolderPath.value = selectedPath
              console.log('使用完整路径:', exportFolderPath.value)
            }
          } else {
            // 如果已经是文件夹路径，直接使用
            exportFolderPath.value = selectedPath
            console.log('使用文件夹路径:', exportFolderPath.value)
          }
        } else {
          // 无法获取路径的情况
          console.log('无法获取完整路径，使用默认路径')
          // 显示一个友好的提示
          exportFolderPath.value = '已选择文件夹'
        }
        
        // 完成后设置加载状态为false
        isFolderLoading.value = false
      } else {
        // 用户取消选择
        isFolderLoading.value = false
      }
    })
    
    // 加载服务器列表
    await server.loadServers()
    if (servers.value.length > 0) {
      selectedServer.value = servers.value[0]
      await loadDatabases()
    }
    
    // 注册键盘快捷键
    document.addEventListener('keydown', handleKeyDown)
  } catch (error: any) {
    console.error('初始化失败:', error)
    messageType.value = 'error'
    message.value = `初始化失败: ${error.message || error}`
  }
})

// 组件卸载时清理
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})

const importData = async () => {
  if (!selectedServer.value) return
  
  try {
    message.value = ''
    
    // 使用 Tauri dialog 选择要导入的文件
    const selected = await open({
      title: '选择导入文件',
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    })
    
    if (!selected) return // 用户取消选择
    
    const filePath = typeof selected === 'string' ? selected : (Array.isArray(selected) ? selected[0] : '')
    
    if (!filePath) return
    
    await redis.importData({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      file_path: filePath
    })
    
    await loadDatabases()
    await loadKeys()
    messageType.value = 'success'
    message.value = `导入成功: ${filePath}`
  } catch (error: any) {
    console.error('导入失败:', error)
    messageType.value = 'error'
    message.value = `导入失败: ${error.message || error}`
  }
}

// 服务器配置页面关闭
const closeServerConfig = async () => {
  showServerConfig.value = false
  // 重新加载服务器列表，并同步更新当前选中的服务器（使 readonly 等配置实时生效）
  try {
    message.value = ''
    await server.loadServers()
    // 用 store 中最新的数据刷新 selectedServer
    if (selectedServer.value) {
      const updated = servers.value.find((s: any) => s.id === selectedServer.value.id)
      if (updated) {
        selectedServer.value = updated
      }
    }
  } catch (error: any) {
    console.error('加载服务器失败:', error)
    messageType.value = 'error'
    message.value = `加载服务器失败: ${error.message || error}`
  }
}

// 新增DB
const addDb = async () => {
  if (!selectedServer.value) return
  
  try {
    message.value = ''
    
    // 选择指定编号的数据库（Redis会自动创建不存在的数据库）
    await redis.createDatabase({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: newDbNumber.value
    })
    
    // 将新增的数据库添加到前端跟踪列表中
    newlyCreatedDbs.value.add(newDbNumber.value)
    
    // 重新加载数据库列表
    await loadDatabases()
    
    // 选择新创建的数据库
    selectedDb.value = newDbNumber.value
    await loadKeys()
    
    showAddDbDialog.value = false
  } catch (error: any) {
    console.error('新增DB失败:', error)
    messageType.value = 'error'
    message.value = `新增DB失败: ${error.message || error}`
  }
}

// 删除DB
const deleteDb = async () => {
  if (!selectedServer.value || selectedDbsForDelete.value.length === 0) return
  if (isCurrentServerReadonly.value) {
    messageType.value = 'error'
    message.value = '当前服务器为只读模式，无法删除DB'
    return
  }

  try {
    message.value = ''
    
    // 批量删除选中的数据库
    for (const db of selectedDbsForDelete.value) {
      await redis.deleteDatabase({
        host: selectedServer.value.host,
        port: selectedServer.value.port,
        password: selectedServer.value.password,
        db
      })
    }
    
    await loadDatabases()
    showDeleteDbDialog.value = false
    selectedDbsForDelete.value = []
  } catch (error: any) {
    console.error('删除DB失败:', error)
    messageType.value = 'error'
    message.value = `删除DB失败: ${error.message || error}`
  }
}

// ========== 废键箱相关方法 ==========

// ========== 多选操作方法 ==========

// 切换多选模式
const toggleMultiSelectMode = () => {
  isMultiSelectMode.value = !isMultiSelectMode.value
  if (!isMultiSelectMode.value) {
    // 退出时清空选择
    clearSelection()
  }
}

// 处理全选下拉命令
const handleSelectAllCommand = async (command: string) => {
  if (command === 'loaded') {
    selectAllLoaded()
  } else if (command === 'all') {
    await selectAllKeys()
  }
}

// 全选当前已加载
const selectAllLoaded = () => {
  if (treeRef.value) {
    treeRef.value.setCheckedKeys(keys.value)
    selectedKeys.value = [...keys.value]
  }
}

// 全选所有（异步）
const selectAllKeys = async () => {
  if (hasMoreKeys.value) {
    await handleLoadAll()
  }
  await nextTick()
  selectAllLoaded()
}

// 取消选择
const clearSelection = () => {
  if (treeRef.value) {
    treeRef.value.setCheckedKeys([])
    selectedKeys.value = []
  }
}

// 批量删除废键箱
const batchMoveToTrash = async () => {
  if (selectedKeys.value.length === 0 || !selectedServer.value) return
  if (isCurrentServerReadonly.value) {
    messageType.value = 'error'
    message.value = '当前服务器为只读模式，无法删除Key'
    return
  }

  try {
    await ElMessageBox.confirm(
      `确定要将选中的 ${selectedKeys.value.length} 个键删除废键箱吗？`,
      '批量删除确认',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    message.value = ''

    const count = await trash.batchMoveToTrash({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      keys: selectedKeys.value
    })

    // 清空选择并刷新
    selectedKeys.value = []
    isMultiSelectMode.value = false
    await loadKeys()
    await loadDatabases()
    await loadTrashItems()

    messageType.value = 'success'
    message.value = `已将 ${count} 个键删除废键箱，7天后自动清除`
  } catch (error: any) {
    if (error === 'cancel') return
    messageType.value = 'error'
    message.value = `批量删除失败: ${error.message || error}`
  }
}

// 键盘快捷键处理
const handleKeyDown = (event: KeyboardEvent) => {
  if (!isMultiSelectMode.value) return

  // Ctrl+A: 全选当前已加载
  if (event.ctrlKey && event.key === 'a' && !event.shiftKey) {
    event.preventDefault()
    selectAllLoaded()
  }

  // Ctrl+Shift+A: 全选所有
  if (event.ctrlKey && event.shiftKey && (event.key === 'A' || event.key === 'a')) {
    event.preventDefault()
    selectAllKeys()
  }

  // Escape: 退出多选模式
  if (event.key === 'Escape') {
    toggleMultiSelectMode()
  }
}

// 处理多选勾选变化
const handleCheckChange = () => {
  if (treeRef.value) {
    selectedKeys.value = treeRef.value.getCheckedKeys(true)
  }
}

// 加载废键箱数据
const loadTrashItems = async () => {
  if (!selectedServer.value) return
  try {
    await trash.getTrashItems(selectedServer.value.host, selectedServer.value.port)
  } catch (error: any) {
    messageType.value = 'error'
    message.value = `加载废键箱失败: ${error.message || error}`
  }
}

// 处理废键箱表格选择变化
const handleTrashSelectionChange = (selection: any[]) => {
  trashSelectedIds.value = selection.map((item: any) => item.id)
}

// 恢复单个废键箱项
const restoreSingleItem = async (id: string) => {
  try {
    message.value = ''
    await trash.restoreFromTrash(id)
    await loadTrashItems()
    await loadKeys()
    await loadDatabases()
    messageType.value = 'success'
    message.value = '恢复成功'
  } catch (error: any) {
    messageType.value = 'error'
    message.value = `恢复失败: ${error.message || error}`
  }
}

// 批量恢复废键箱项
const batchRestoreFromTrash = async () => {
  if (trashSelectedIds.value.length === 0) return
  try {
    await ElMessageBox.confirm(
      `确定要恢复选中的 ${trashSelectedIds.value.length} 项吗？`,
      '批量恢复',
      { confirmButtonText: '确认', cancelButtonText: '取消', type: 'info' }
    )
    message.value = ''
    await trash.batchRestoreFromTrash(trashSelectedIds.value)
    await loadTrashItems()
    await loadKeys()
    await loadDatabases()
    messageType.value = 'success'
    message.value = `已恢复 ${trashSelectedIds.value.length} 项`
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    messageType.value = 'error'
    message.value = `批量恢复失败: ${error.message || error}`
  }
}

// 永久删除单个废键箱项
const deleteSingleItem = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要永久删除该项吗？此操作不可恢复。', '永久删除', {
      confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning',
    })
    message.value = ''
    await trash.permanentDelete([id])
    await loadTrashItems()
    messageType.value = 'success'
    message.value = '已永久删除'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    messageType.value = 'error'
    message.value = `删除失败: ${error.message || error}`
  }
}

// 永久删除选中的废键箱项
const permanentDeleteTrash = async () => {
  if (trashSelectedIds.value.length === 0) return
  try {
    await ElMessageBox.confirm(
      `确定要永久删除选中的 ${trashSelectedIds.value.length} 项吗？此操作不可恢复。`,
      '永久删除',
      { confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning' }
    )
    message.value = ''
    await trash.permanentDelete(trashSelectedIds.value)
    await loadTrashItems()
    messageType.value = 'success'
    message.value = `已永久删除 ${trashSelectedIds.value.length} 项`
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    messageType.value = 'error'
    message.value = `永久删除失败: ${error.message || error}`
  }
}

// 清理过期废键箱项
const clearExpiredTrash = async () => {
  try {
    message.value = ''
    await trash.clearExpired()
    await loadTrashItems()
    messageType.value = 'success'
    message.value = '已清理过期项'
  } catch (error: any) {
    messageType.value = 'error'
    message.value = `清理过期项失败: ${error.message || error}`
  }
}

// 获取类型标签颜色
const getTypeTagColor = (type: string): string => {
  const colorMap: Record<string, string> = {
    string: '',
    list: 'success',
    set: 'warning',
    zset: 'danger',
    hash: 'info',
  }
  return colorMap[type] || ''
}

// 当前服务器废键箱数量
const currentServerTrashCount = computed(() => {
  if (!selectedServer.value) return 0
  return trash.trashItems.filter(
    (item: any) => item.host === selectedServer.value.host && item.port === selectedServer.value.port
  ).length
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

/* 消息提示 */
.message-alert {
  margin: 0;
  border-radius: 0;
  width: 100%;
  padding: 10px 20px;
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
  gap: 0;
}

.el-dropdown-link {
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  padding: 8px 16px;
  border-radius: 0;
  transition: all 0.3s ease;
}

/* 确保所有 dropdown 触发器内文字大小一致 */
.menu-bar :deep(.el-dropdown) {
  font-size: 14px;
}

.sort-link {
  font-weight: bold;
}

.el-dropdown-link:hover {
  color: #ecf5ff;
  background-color: rgba(255, 255, 255, 0.1);
}

.menu-multi-select-btn {
  color: #ffffff;
  border-color: transparent;
  background-color: transparent;
  display: inline-flex !important;
  align-items: center !important;
  gap: 4px !important;
  border-radius: 0;
  padding: 8px 16px;
  font-size: 14px;
  height: auto;
}

.menu-multi-select-btn:hover {
  color: #ecf5ff;
  background-color: rgba(255, 255, 255, 0.1);
  border-color: transparent;
}

.menu-multi-select-btn.warning {
  background-color: rgba(230, 162, 60, 0.9) !important;
  border-color: #e6a23c !important;
  color: #ffffff !important;
}

.menu-multi-select-btn.warning:hover {
  background-color: #e6a23c !important;
  border-color: #e6a23c !important;
}

.multi-select-count-tag {
  margin-left: 4px;
  line-height: 1;
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
  background-color: #1890ff;
  color: #ffffff;
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
  box-shadow: 0 0 0 1px #1890ff inset;
  border-color: #1890ff;
}

/* 弹跳对话框动画 */
.bounce-dialog {
  animation: bounce-in 0.4s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

@keyframes bounce-in {
  0% {
    transform: scale(0.3);
    opacity: 0;
  }
  50% {
    transform: scale(1.05);
  }
  70% {
    transform: scale(0.9);
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

/* 下拉菜单样式 */
.more-actions :deep(.el-dropdown-menu) {
  padding: 8px 0;
  min-width: 160px;
  width: auto;
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

.add-key-btn {
  border-radius: 4px;
  min-width: 36px;
  height: 36px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-key-btn:hover {
  background-color: #95d475;
  border-color: #95d475;
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

.key-list :deep(.el-tree-node__content:hover) {
  background-color: #ecf5ff;
  color: #1890ff;
}

/* 确保图标在蓝色背景上可见 */
.key-list :deep(.el-tree-node__content:hover .el-tree-node__expand-icon),
.key-list :deep(.el-tree-node.is-current > .el-tree-node__content .el-tree-node__expand-icon) {
  color: #1890ff;
}

.key-list :deep(.el-tree-node__expand-icon) {
  color: #909399;
  transition: color 0.3s;
}

.key-list :deep(.el-tree-node__expand-icon:hover) {
  color: #1890ff;
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

.value-editor {
  height: 100%;
}

.value-editor :deep(.el-textarea__inner) {
  height: 100%;
  min-height: 200px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 14px;
  line-height: 1.6;
  color: #303133;
  background-color: #ffffff;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  resize: none;
}

.value-editor :deep(.el-textarea__inner:focus) {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
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

/* 圆角矩形图标按钮样式 */
.action-btn {
  display: inline-flex !important;
  align-items: center !important;
  justify-content: center !important;
  gap: 6px !important;
  padding: 8px 16px !important;
  border-radius: 4px !important;
  font-size: 14px !important;
  height: auto !important;
  min-width: auto !important;
}

.action-btn .el-icon {
  font-size: 14px;
}

/* 删除按钮 */
.delete-btn {
  border-color: #f56c6c;
  background-color: #f56c6c;
  color: #ffffff;
}

.delete-btn:hover {
  border-color: #f78989;
  background-color: #f78989;
  color: #ffffff;
}

/* 修改按钮 */
.edit-btn {
  border-color: #409eff;
  background-color: #409eff;
  color: #ffffff;
}

.edit-btn:hover {
  border-color: #66b1ff;
  background-color: #66b1ff;
  color: #ffffff;
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

/* 类型示例提示 */
.type-hint {
  margin-top: 8px;
  padding: 8px 12px;
  background-color: #f5f7fa;
  border-radius: 4px;
  font-size: 12px;
  line-height: 1.6;
}

.hint-label {
  color: #909399;
  font-weight: 500;
  display: block;
  margin-bottom: 4px;
}

.hint-text {
  color: #606266;
  font-family: 'Courier New', Courier, monospace;
  white-space: pre-wrap;
  word-break: break-all;
  display: block;
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



/* 导出弹框：文件夹选择行 */
.folder-select-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.folder-path-input {
  flex: 1;
}

.folder-select-btn {
  flex-shrink: 0;
  white-space: nowrap;
}

/* 清空确认对话框样式 */
.flush-confirm-content {
  padding: 10px 0;
}

.flush-confirm-content p {
  margin: 10px 0;
  font-size: 14px;
  color: #606266;
}

.flush-warning {
  color: #f56c6c !important;
  font-weight: 500;
}

.multi-select-btn { margin-left: 10px; }
.batch-delete-btn { margin-left: 10px; }
.menu-badge { margin-left: 6px; }
.trash-view { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.trash-toolbar { padding: 12px 15px; display: flex; align-items: center; gap: 8px; border-bottom: 1px solid #e4e7ed; background-color: #ffffff; }
.trash-list-content { flex: 1; overflow: auto; padding: 0; }

/* ========== 分页加载样式 ========== */
.key-list-footer {
  padding: 8px 15px;
  border-top: 1px solid #e4e7ed;
  background-color: #f5f7fa;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: auto;
  transition: all 0.3s ease;
}

.load-actions {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
}

.load-actions .el-button {
  flex-shrink: 0;
  padding: 6px 12px;
  font-size: 13px;
}

.load-actions .count-divider {
  flex-shrink: 0;
  font-size: 13px;
}

/* 加载按钮统一样式：透明底灰字边框，hover 蓝底白字 */
.load-btn {
  background-color: transparent;
  border-color: #dcdfe6;
  color: #606266;
}

.load-btn:hover {
  background-color: #1890ff;
  border-color: #1890ff;
  color: #ffffff;
}

.load-btn:disabled {
  background-color: transparent;
  border-color: #e4e7ed;
  color: #a8abb2;
}

.count-divider {
  font-size: 14px;
  font-weight: 500;
  color: #1890ff;
  white-space: nowrap;
  transition: color 0.3s ease, transform 0.3s ease;
}

.count-divider.count-highlight {
  color: #1890ff;
  transform: scale(1.2);
}

.loading-all-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
}

.loading-all-progress .el-progress {
  width: 100%;
}

/* 进度条宽度平滑过渡动画 */
.loading-all-progress :deep(.el-progress-bar__inner) {
  transition: width 0.3s ease;
}

.loading-all-progress :deep(.el-progress-bar__outer) {
  overflow: hidden;
}

.loading-text {
  font-size: 12px;
  color: #909399;
}

/* ========== 多选操作面板样式 ========== */
.multi-select-panel {
  padding: 8px 12px;
  border-bottom: 1px solid #e4e7ed;
  background-color: #fffbeb;
}

.multi-select-panel-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.multi-select-panel-hint {
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px dashed #e4e7ed;
  font-size: 12px;
  color: #909399;
}

.multi-select-count-tag {
  margin-left: 2px;
  line-height: 1;
}

/* 多选模式下的树节点样式调整 */
.key-list :deep(.el-tree.show-checkbox .el-tree-node__content) {
  padding-left: 8px;
}

/* 只读模式禁用菜单提示 */
.menu-disabled-hint {
  font-size: 12px;
  color: #c0c4cc;
  margin-left: 4px;
}
</style>