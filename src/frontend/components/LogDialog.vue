<template>
  <el-dialog
    v-model="visible"
    title="日志"
    width="92%"
    :destroy-on-close="false"
    :close-on-click-modal="false"
    draggable
    align-center
    class="log-dialog"
    @close="handleClose"
  >
    <div class="log-body">
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
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useLogStore } from '../stores/logStore'
import { isTauriEnv } from '../utils/tauri'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const logStore = useLogStore()

const filterKeyword = ref('')
const filterOnlyWrite = ref(false)
const filterSlowOnly = ref(false)
const autoScroll = ref(true)
const loadingSlowLog = ref(false)
const tableWrap = ref<HTMLElement | null>(null)
const loadState = ref<'idle' | 'loading' | 'done' | 'error'>('idle')
let userScrolled = false

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

// 自动滚动到底部
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

// 打开对话框时自动加载历史
watch(visible, (isOpen) => {
  if (isOpen) {
    // 环境诊断日志
    console.log('[LogDialog] 对话框打开, 环境:', isTauriEnv() ? 'TAURI' : 'BROWSER',
      'activeServerId:', logStore.activeServerId,
      'sessions:', Object.keys(logStore.sessions))
    // 等下一个 tick，确保 MainView 已设置 activeServerId
    nextTick(() => {
      console.log('[LogDialog] nextTick, activeServerId:', logStore.activeServerId)
      if (logStore.activeServerId) {
        handleLoadHistory()
      } else {
        console.warn('[LogDialog] 打开时无 activeServerId，跳过自动加载')
      }
    })
  }
})

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
  /* width 由 el-dialog 的 width="92%" 控制 */
  max-width: 92vw !important;
  min-width: 520px;
}
</style>
