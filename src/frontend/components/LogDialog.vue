<template>
  <el-dialog
    v-model="visible"
    width="92%"
    :destroy-on-close="false"
    :close-on-click-modal="false"
    draggable
    align-center
    class="log-dialog"
    @close="handleClose"
  >
    <!-- 顶部标签切换：慢日志 / 操作审计 -->
    <div class="log-main-tabs">
      <el-tabs v-model="mainTab" @tab-change="handleMainTabChange">
        <el-tab-pane label="慢日志" name="slowlog" />
        <el-tab-pane label="操作审计" name="audit" />
      </el-tabs>
    </div>

    <!-- 慢日志内容 -->
    <div v-if="mainTab === 'slowlog'" class="log-body">
      <!-- Tab 栏 -->
      <div class="log-tabs-bar">
        <div class="log-tabs">
        <div
          v-for="session in logStore.sessionList"
          :key="session.serverId"
          class="log-tab"
          :class="{ active: logStore.activeServerId === session.serverId }"
          @click="logStore.setActiveServer(session.serverId)"
        >
          {{ session.serverName }}
          </div>
        </div>

        <!-- 右侧操作 -->
        <div class="log-tab-actions">
          <span class="log-count">共 {{ filteredLogs.length }} 条</span>
          <el-button size="small" plain @click="handleClear">清空</el-button>
          <el-button size="small" :loading="loadingSlowLog" @click="handleLoadHistory">加载历史</el-button>
        </div>
      </div>

      <!-- 日志表格 -->
      <div ref="tableWrap" class="log-table-wrap" @scroll="onTableScroll">
        <table class="log-table">
          <thead>
            <tr>
              <th style="width:86px">Time</th>
              <th style="width:160px">Client</th>
              <th style="width:130px">CMD</th>
              <th>Args</th>
              <th style="width:84px;text-align:right;white-space:nowrap">Cost(ms)</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="log in filteredLogs"
              :key="log.id"
              :class="isSlowCmd(log) ? 'row-slow' : ''"
            >
              <td>{{ log.time }}</td>
              <td class="td-client" :title="log.client">{{ log.client }}</td>
              <td class="td-cmd"><b>{{ log.cmd }}</b></td>
              <td class="td-args">{{ log.args.join(' ') }}</td>
              <td class="td-cost" :class="isSlowCmd(log) ? 'cost-slow' : 'cost-normal'">
                {{ log.costMs !== null ? log.costMs.toFixed(2) : '-' }}
              </td>
            </tr>
            <tr v-if="filteredLogs.length === 0">
              <td colspan="5" class="empty-tip">
                <template v-if="logStore.loadingError && logStore.loadingError !== 'empty'">
                  <span style="color: var(--el-color-danger)">⚠️ 加载失败: {{ logStore.loadingError }}</span>
                  <br>请检查浏览器控制台获取详细信息
                </template>
                <template v-else-if="loadState === 'loading'">
                  正在加载历史命令...
                </template>
                <template v-else>
                  暂无日志，点击「加载历史」加载历史命令
                </template>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 底部过滤栏 -->
      <div class="log-footer">
        <el-input
          v-model="filterKeyword"
          placeholder="搜索 CMD / Args / Client"
          size="small"
          clearable
          style="width:200px"
        />
        <el-checkbox v-model="filterOnlyWrite" size="small">Only Write</el-checkbox>
        <el-checkbox v-model="filterSlowOnly" size="small">慢命令(&gt;10ms)</el-checkbox>
        <el-checkbox v-model="autoScroll" size="small">自动滚动</el-checkbox>
      </div>
    </div>

    <!-- 操作审计内容 -->
    <div v-else class="audit-body">
      <!-- 筛选栏 -->
      <div class="audit-filter-bar">
        <el-input
          v-model="auditFilterKeyword"
          placeholder="搜索命令/参数"
          size="small"
          clearable
          style="width: 200px"
        />
        
        <el-select
          v-model="auditFilterCommand"
          placeholder="选择命令"
          size="small"
          clearable
          filterable
          style="width: 120px"
        >
          <el-option
            v-for="cmd in availableCommands"
            :key="cmd"
            :label="cmd"
            :value="cmd"
          />
        </el-select>

        <el-date-picker
          v-model="auditDateRange"
          type="datetimerange"
          range-separator="至"
          start-placeholder="开始时间"
          end-placeholder="结束时间"
          size="small"
          style="width: 320px"
        />

        <el-button size="small" type="primary" @click="handleAuditSearch">
          <el-icon><Search /></el-icon> 搜索
        </el-button>
        
        <el-button size="small" @click="handleAuditClear" :loading="isClearing">
          <el-icon><Delete /></el-icon> 清空日志
        </el-button>

        <el-button size="small" @click="handleGenerateTestData" :loading="isGenerating">
          <el-icon><Plus /></el-icon> 生成测试数据
        </el-button>
      </div>

      <!-- 审计日志表格 -->
      <div class="audit-table-wrap">
        <el-table
          :data="auditStore.logs"
          v-loading="auditStore.isLoading"
          stripe
          border
          class="audit-table"
          :max-height="350"
        >
          <el-table-column prop="timestamp" label="时间" width="180">
            <template #default="{ row }">
              {{ formatTime(row.timestamp) }}
            </template>
          </el-table-column>

          <el-table-column prop="serverName" label="服务器" width="120" />
          
          <el-table-column prop="db" label="DB" width="60" />
          
          <el-table-column prop="command" label="命令" width="100">
            <template #default="{ row }">
              <el-tag :type="getCommandType(row.command)">{{ row.command }}</el-tag>
            </template>
          </el-table-column>

          <el-table-column prop="args" label="参数" min-width="200">
            <template #default="{ row }">
              <span class="audit-args-text">{{ row.args.join(' ') }}</span>
            </template>
          </el-table-column>

          <el-table-column prop="costMs" label="耗时(ms)" width="100">
            <template #default="{ row }">
              <span :class="getCostClass(row.costMs)">
                {{ row.costMs }}
              </span>
            </template>
          </el-table-column>

          <el-table-column prop="success" label="状态" width="80">
            <template #default="{ row }">
              <el-tag :type="row.success ? 'success' : 'danger'">
                {{ row.success ? '成功' : '失败' }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column prop="errorMessage" label="错误信息" min-width="150">
            <template #default="{ row }">
              <span v-if="row.errorMessage" class="audit-error-text">
                {{ row.errorMessage }}
              </span>
              <span v-else>-</span>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 分页 -->
      <div class="audit-pagination-bar">
        <span class="audit-record-count">共 {{ auditStore.pagination.total }} 条记录</span>
        <el-pagination
          v-model:current-page="auditCurrentPage"
          :page-size="auditStore.pagination.limit"
          :total="auditStore.pagination.total"
          layout="prev, pager, next"
          @current-change="handleAuditPageChange"
        />
      </div>

      <!-- 统计卡片 -->
      <div class="audit-stats-cards">
        <el-card class="audit-stat-card">
          <div class="audit-stat-icon">
            <el-icon :size="24"><DataAnalysis /></el-icon>
          </div>
          <div class="audit-stat-content">
            <div class="audit-stat-value">{{ auditStore.totalCommands.toLocaleString() }}</div>
            <div class="audit-stat-label">总命令数</div>
          </div>
        </el-card>

        <el-card class="audit-stat-card">
          <div class="audit-stat-icon audit-stat-icon-info">
            <el-icon :size="24"><Clock /></el-icon>
          </div>
          <div class="audit-stat-content">
            <div class="audit-stat-value">{{ auditStore.avgResponseTime }}ms</div>
            <div class="audit-stat-label">平均响应</div>
          </div>
        </el-card>

        <el-card class="audit-stat-card">
          <div class="audit-stat-icon audit-stat-icon-success">
            <el-icon :size="24"><Check /></el-icon>
          </div>
          <div class="audit-stat-content">
            <div class="audit-stat-value">{{ auditStore.successRate }}%</div>
            <div class="audit-stat-label">成功率</div>
          </div>
        </el-card>

        <el-card class="audit-stat-card">
          <div class="audit-stat-icon audit-stat-icon-warning">
            <el-icon :size="24"><InfoFilled /></el-icon>
          </div>
          <div class="audit-stat-content">
            <div class="audit-stat-value">{{ auditStore.slowestCommand }}</div>
            <div class="audit-stat-label">最慢命令</div>
          </div>
        </el-card>
      </div>

      <!-- 命令分布图表 -->
      <el-card title="命令分布" class="audit-chart-card">
        <div class="audit-command-chart">
          <div
            v-for="stat in sortedStats"
            :key="stat.command"
            class="audit-chart-item"
          >
            <div class="audit-chart-label">{{ stat.command }}</div>
            <div class="audit-chart-bar-container">
              <div
                class="audit-chart-bar"
                :style="{ width: getBarWidth(stat.count) + '%', backgroundColor: getBarColor(stat.command) }"
              />
            </div>
            <div class="audit-chart-count">{{ stat.count }}</div>
          </div>
          <div v-if="auditStore.stats.length === 0" class="audit-empty-chart">
            暂无统计数据
          </div>
        </div>
      </el-card>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useLogStore } from '../stores/logStore'
import { useAuditStore } from '../stores/auditStore'
import { isTauriEnv } from '../utils/tauri'
import { Search, Delete, Plus, DataAnalysis, Clock, Check, InfoFilled } from '@element-plus/icons-vue'

const props = defineProps<{ modelValue: boolean; host?: string; port?: number; username?: string; password?: string }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const logStore = useLogStore()
const auditStore = useAuditStore()

const mainTab = ref<'slowlog' | 'audit'>('slowlog')

// 慢日志相关
const filterKeyword = ref('')
const filterOnlyWrite = ref(false)
const filterSlowOnly = ref(false)
const autoScroll = ref(true)
const loadingSlowLog = ref(false)
const tableWrap = ref<HTMLElement | null>(null)
const loadState = ref<'idle' | 'loading' | 'done' | 'error'>('idle')
let userScrolled = false

// 审计相关
const auditFilterKeyword = ref('')
const auditFilterCommand = ref('')
const auditDateRange = ref<[Date, Date] | null>(null)
const auditCurrentPage = ref(1)
const isClearing = ref(false)
const isGenerating = ref(false)

const availableCommands = ['GET', 'SET', 'DEL', 'HSET', 'HGET', 'HGETALL', 'LPUSH', 'LRANGE', 'RPUSH', 'SADD', 'SMEMBERS', 'ZADD', 'ZRANGE']

const WRITE_CMDS = new Set([
  'SET', 'SETEX', 'SETNX', 'MSET', 'MSETNX', 'GETSET', 'GETDEL', 'GETEX',
  'DEL', 'UNLINK', 'EXPIRE', 'EXPIREAT', 'PEXPIRE', 'PEXPIREAT', 'PERSIST',
  'RENAME', 'RENAMENX', 'INCR', 'INCRBY', 'INCRBYFLOAT', 'DECR', 'DECRBY',
  'APPEND', 'SETRANGE',
  'HSET', 'HMSET', 'HSETNX', 'HDEL', 'HINCRBY', 'HINCRBYFLOAT',
  'LPUSH', 'LPUSHX', 'RPUSH', 'RPUSHX', 'LPOP', 'RPOP', 'LINSERT',
  'LSET', 'LREM', 'LTRIM', 'LMOVE', 'RPOPLPUSH',
  'SADD', 'SREM', 'SMOVE', 'SPOP', 'SINTERSTORE', 'SUNIONSTORE', 'SDIFFSTORE',
  'ZADD', 'ZINCRBY', 'ZREM', 'ZREMRANGEBYRANK', 'ZREMRANGEBYSCORE', 'ZREMRANGEBYLEX',
  'ZPOPMIN', 'ZPOPMAX', 'ZUNIONSTORE', 'ZINTERSTORE', 'ZDIFFSTORE',
  'FLUSHDB', 'FLUSHALL', 'SELECT',
])

const SLOW_THRESHOLD_MS = 10

// 慢日志过滤
function isSlowCmd(log: { costMs: number | null }) {
  return log.costMs !== null && log.costMs > SLOW_THRESHOLD_MS
}

const filteredLogs = computed(() => {
  const logs = logStore.activeLogs
  return logs.filter(log => {
    if (filterOnlyWrite.value && !WRITE_CMDS.has(log.cmd)) return false
    if (filterSlowOnly.value && !isSlowCmd(log)) return false
    if (filterKeyword.value) {
      const kw = filterKeyword.value.toLowerCase()
      const inCmd = log.cmd.toLowerCase().includes(kw)
      const inArgs = log.args.join(' ').toLowerCase().includes(kw)
      const inClient = log.client.toLowerCase().includes(kw)
      if (!inCmd && !inArgs && !inClient) return false
    }
    return true
  })
})

// 审计统计排序
const sortedStats = computed(() => {
  return [...auditStore.stats].sort((a, b) => b.count - a.count)
})

// 格式化时间
function formatTime(timestamp: number) {
  const date = new Date(timestamp)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  const ms = String(date.getMilliseconds()).padStart(3, '0')
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}.${ms}`
}

function getCommandType(command: string) {
  const readCommands = ['GET', 'HGET', 'HGETALL', 'LRANGE', 'SMEMBERS', 'ZRANGE', 'MGET']
  const writeCommands = ['SET', 'HSET', 'LPUSH', 'RPUSH', 'SADD', 'ZADD', 'DEL']
  
  if (readCommands.includes(command)) return 'info'
  if (writeCommands.includes(command)) return 'warning'
  return 'primary'
}

function getCostClass(costMs: number) {
  if (costMs > 100) return 'audit-cost-high'
  if (costMs > 10) return 'audit-cost-medium'
  return 'audit-cost-low'
}

function getBarWidth(count: number) {
  const max = Math.max(...auditStore.stats.map(s => s.count), 1)
  return (count / max * 100)
}

function getBarColor(command: string) {
  const colors: Record<string, string> = {
    'GET': '#409EFF',
    'SET': '#67C23A',
    'DEL': '#F56C6C',
    'HSET': '#E6A23C',
    'HGET': '#909399',
    'HGETALL': '#B37FEB',
    'LPUSH': '#13C2C2',
    'LRANGE': '#5B8FF9',
  }
  return colors[command] || '#909399'
}

// 慢日志自动滚动
watch(filteredLogs, () => {
  if (!autoScroll.value || userScrolled) return
  nextTick(() => {
    if (tableWrap.value) {
      tableWrap.value.scrollTop = tableWrap.value.scrollHeight
    }
  })
})

function onTableScroll() {
  if (!tableWrap.value) return
  const { scrollTop, scrollHeight, clientHeight } = tableWrap.value
  userScrolled = scrollTop + clientHeight < scrollHeight - 40
}

watch(autoScroll, (val) => {
  if (val) {
    userScrolled = false
    nextTick(() => {
      if (tableWrap.value) {
        tableWrap.value.scrollTop = tableWrap.value.scrollHeight
      }
    })
  }
})

function handleClear() {
  logStore.clearLogs(logStore.activeServerId)
}

async function handleLoadHistory() {
  const sid = logStore.activeServerId
  if (!sid) return
  loadingSlowLog.value = true
  loadState.value = 'loading'
  try {
    await logStore.loadSlowLog(sid)
    loadState.value = logStore.loadingError ? 'error' : 'done'
  } catch (err: any) {
    console.error('加载历史失败:', err.message)
    loadState.value = 'error'
  } finally {
    loadingSlowLog.value = false
  }
}

// 审计搜索
function handleAuditSearch() {
  if (auditDateRange.value) {
    auditStore.setFilter('startTime', auditDateRange.value[0].getTime())
    auditStore.setFilter('endTime', auditDateRange.value[1].getTime())
  } else {
    auditStore.setFilter('startTime', null)
    auditStore.setFilter('endTime', null)
  }
  
  auditStore.setFilter('command', auditFilterCommand.value || null)
  
  auditCurrentPage.value = 1
  auditStore.fetchLogs()
}

// 清空审计日志
async function handleAuditClear() {
  isClearing.value = true
  try {
    console.log('[LogDialog] handleAuditClear: calling clearLogs')
    await auditStore.clearLogs()
    console.log('[LogDialog] handleAuditClear: clearLogs completed')
    console.log('[LogDialog] stats after clear:', auditStore.stats.length)
    
    // 测试：清空后重新获取统计数据，验证后端是否真的清空了
    await auditStore.fetchStats()
    console.log('[LogDialog] stats after fetchStats:', auditStore.stats.length)
    
    if (auditStore.stats.length > 0) {
      console.warn('[LogDialog] WARNING: Stats still not empty after clear!')
      alert('警告：统计数据仍然存在，可能是后端没有正确清空')
    }
  } catch (error) {
    console.error('[LogDialog] handleAuditClear error:', error)
    alert('清空失败: ' + (error instanceof Error ? error.message : String(error)))
  } finally {
    isClearing.value = false
  }
}

// 生成测试数据
async function handleGenerateTestData() {
  isGenerating.value = true
  try {
    await auditStore.generateTestData(100)
    await auditStore.fetchLogs()
    await auditStore.fetchStats()
  } finally {
    isGenerating.value = false
  }
}

// 审计分页
function handleAuditPageChange(page: number) {
  auditCurrentPage.value = page
  auditStore.pagination.offset = (page - 1) * auditStore.pagination.limit
  auditStore.fetchLogs()
}

// 打开对话框时自动加载历史
watch(visible, (isOpen) => {
  if (isOpen) {
    console.log('[LogDialog] 对话框打开, 环境:', isTauriEnv() ? 'TAURI' : 'BROWSER',
      'activeServerId:', logStore.activeServerId,
      'sessions:', Object.keys(logStore.sessions))
    
    nextTick(() => {
      console.log('[LogDialog] nextTick, activeServerId:', logStore.activeServerId)
      if (logStore.activeServerId) {
        handleLoadHistory()
      } else {
        console.warn('[LogDialog] 打开时无 activeServerId，跳过自动加载')
      }
      
      // 同时初始化审计存储
      if (props.host && props.port) {
        auditStore.setCurrentServer(props.host, props.port, props.password, props.username)
      }
    })
  }
})

// 标签切换
function handleMainTabChange(tabName: string) {
  if (tabName === 'audit') {
    // 切换到审计标签时，确保有服务器信息并加载数据
    if (props.host && props.port) {
      auditStore.setCurrentServer(props.host, props.port, props.password)
    }
    auditStore.fetchLogs()
    auditStore.fetchStats()
  }
}

async function handleClose() {
  logStore.reset()
}
</script>

<style scoped>
.log-body {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 200px);
  max-height: 600px;
  min-height: 360px;
}

.log-tabs-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 0.5px solid var(--el-border-color-lighter);
  padding: 0 4px;
  flex-shrink: 0;
}

.log-tabs {
  display: flex;
}

.log-tab {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 8px 14px;
  font-size: 13px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
}

.log-tab.active {
  color: var(--el-color-primary);
  border-bottom-color: var(--el-color-primary);
  font-weight: 500;
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot-green {
  background: #3b6d11;
}

.dot-gray {
  background: #c0bdb4;
}

.monitor-tag {
  font-size: 11px;
  padding: 0 5px;
  height: 18px;
  line-height: 18px;
}

.log-tab-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 4px;
  flex-shrink: 0;
}

.log-count {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

.log-table-wrap {
  flex: 1;
  overflow: auto;
  border: 0.5px solid var(--el-border-color-lighter);
  border-radius: 6px;
  margin: 8px 0;
}

.log-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  table-layout: fixed;
}

.log-table th {
  background: var(--el-fill-color-lighter);
  padding: 7px 10px;
  text-align: left;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  border-bottom: 0.5px solid var(--el-border-color-lighter);
  position: sticky;
  top: 0;
  z-index: 1;
}

.log-table td {
  padding: 6px 10px;
  border-bottom: 0.5px solid var(--el-border-color-extra-light);
  font-family: var(--el-font-family-mono, ui-monospace, monospace);
  vertical-align: top;
  word-break: break-all;
}

.log-table tr:hover td {
  background: var(--el-fill-color-extra-light);
}

.log-table .row-slow td {
  color: var(--el-color-danger);
}

.log-table .row-slow td b {
  color: var(--el-color-danger);
}

.td-cmd {
  white-space: nowrap;
}

.td-cost {
  text-align: right;
  white-space: nowrap;
}

.td-client {
  color: var(--el-text-color-placeholder);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.td-args {
  color: var(--el-text-color-regular);
  word-break: break-all;
}

.cost-normal {
  color: var(--el-text-color-placeholder);
}

.cost-slow {
  color: var(--el-color-danger);
  font-weight: 500;
}

.empty-tip {
  text-align: center;
  color: var(--el-text-color-placeholder);
  padding: 32px 0 !important;
  font-family: var(--el-font-family) !important;
}

.log-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 2px 2px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

/* 审计样式 */
.audit-body {
  padding-top: 8px;
}

.audit-filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
  align-items: center;
}

.audit-table-wrap {
  margin-bottom: 16px;
}

.audit-args-text {
  font-family: monospace;
  font-size: 12px;
  color: #666;
}

.audit-error-text {
  color: #f56c6c;
  font-size: 12px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.audit-cost-high { color: #f56c6c; font-weight: bold; }
.audit-cost-medium { color: #e6a23c; }
.audit-cost-low { color: #67c23a; }

.audit-pagination-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-top: 16px;
  border-top: 1px solid #eee;
}

.audit-record-count {
  color: #999;
  font-size: 13px;
}

.audit-stats-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.audit-stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
}

.audit-stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.audit-stat-icon-info {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.audit-stat-icon-success {
  background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
}

.audit-stat-icon-warning {
  background: linear-gradient(135deg, #fc4a1a 0%, #f7b733 100%);
}

.audit-stat-content {
  flex: 1;
}

.audit-stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.audit-stat-label {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.audit-chart-card {
  margin-bottom: 16px;
}

.audit-command-chart {
  padding: 16px;
}

.audit-chart-item {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.audit-chart-label {
  width: 80px;
  font-weight: bold;
  font-size: 13px;
}

.audit-chart-bar-container {
  flex: 1;
  height: 24px;
  background: #f5f7fa;
  border-radius: 12px;
  overflow: hidden;
}

.audit-chart-bar {
  height: 100%;
  border-radius: 12px;
  transition: width 0.3s ease;
}

.audit-chart-count {
  width: 60px;
  text-align: right;
  font-size: 13px;
  color: #909399;
}

.audit-empty-chart {
  text-align: center;
  color: #909399;
  padding: 40px;
}
</style>

<!-- 全局样式：弹窗拖拽手柄 + 尺寸约束 -->
<style>
.log-dialog .el-dialog__header {
  cursor: move;
  user-select: none;
  margin-right: 0;
  padding: 12px 16px;
}

/* 弹窗最大宽度不超过窗口 92%，最小保证可用 */
.log-dialog .el-dialog {
  max-width: 92vw !important;
  min-width: 520px;
}
</style>
