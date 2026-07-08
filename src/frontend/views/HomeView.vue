<template>
  <div class="home-view">
    <!-- 顶部统计卡片 -->
    <div class="stats-cards">
      <!-- 服务器信息卡片 -->
      <div class="stat-card">
        <div class="card-header">
          <el-icon class="card-icon"><Setting /></el-icon>
          <span class="card-title">服务器</span>
        </div>
        <div class="card-content">
          <template v-if="serverInfoLoaded">
            <div class="info-item">
              <span class="info-label">Redis版本:</span>
              <span class="info-value">{{ serverInfo.redis_version }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">OS:</span>
              <span class="info-value">{{ serverInfo.os }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">进程ID:</span>
              <span class="info-value">{{ serverInfo.process_id }}</span>
            </div>
          </template>
          <template v-else>
            <div class="skeleton-item">
              <el-skeleton :rows="3" animated />
            </div>
          </template>
        </div>
      </div>

      <!-- 内存信息卡片 -->
      <div class="stat-card">
        <div class="card-header">
          <el-icon class="card-icon"><FolderOpened /></el-icon>
          <span class="card-title">内存</span>
        </div>
        <div class="card-content">
          <template v-if="serverInfoLoaded">
            <div class="info-item">
              <span class="info-label">已用内存:</span>
              <span class="info-value highlight">{{ memoryInfo.used_memory_human }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">内存占用峰值:</span>
              <span class="info-value">{{ memoryInfo.used_memory_peak_human }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">Lua占用内存:</span>
              <span class="info-value">{{ memoryInfo.lua_memory_human }}</span>
            </div>
          </template>
          <template v-else>
            <div class="skeleton-item">
              <el-skeleton :rows="3" animated />
            </div>
          </template>
        </div>
      </div>

      <!-- 状态信息卡片 -->
      <div class="stat-card">
        <div class="card-header">
          <el-icon class="card-icon"><Refresh /></el-icon>
          <span class="card-title">状态</span>
        </div>
        <div class="card-content">
          <template v-if="serverInfoLoaded">
            <div class="info-item">
              <span class="info-label">客户端连接数:</span>
              <span class="info-value highlight">{{ statusInfo.connected_clients }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">历史连接数:</span>
              <span class="info-value">{{ statusInfo.total_connections_received }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">历史命令数:</span>
              <span class="info-value">{{ statusInfo.total_commands_processed }}</span>
            </div>
          </template>
          <template v-else>
            <div class="skeleton-item">
              <el-skeleton :rows="3" animated />
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 键值统计表格 -->
    <div class="table-section">
      <div class="section-header">
        <el-icon class="section-icon"><DataAnalysis /></el-icon>
        <span class="section-title">键值统计</span>
      </div>
      <el-table 
        :data="keyStats" 
        border 
        style="width: 100%" 
        class="stats-table"
        v-loading="keyStatsLoading"
        element-loading-text="加载中..."
        element-loading-background="rgba(255, 255, 255, 0.9)"
        empty-text="暂无数据"
      >
        <el-table-column prop="db" label="DB" sortable />
        <el-table-column prop="keys" label="Keys" sortable />
        <el-table-column prop="expires" label="Expires" sortable />
        <el-table-column prop="avg_ttl" label="Avg TTL" sortable />
      </el-table>
    </div>

    <!-- Redis信息全集 -->
    <div class="table-section info-section">
      <div class="section-header">
        <el-icon class="section-icon"><InfoFilled /></el-icon>
        <span class="section-title">Redis信息全集</span>
        <el-button type="primary" size="small" @click="refreshServerInfo" :loading="refreshingInfo">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-input
          v-model="searchInfo"
          placeholder="搜索"
          class="search-input"
          clearable
        />
      </div>
      
      <!-- 分组标签页 -->
      <el-tabs v-model="activeInfoTab" class="info-tabs">
        <el-tab-pane label="全部" name="all" />
        <el-tab-pane 
          v-for="section in infoSections" 
          :key="section.key" 
          :label="section.label" 
          :name="section.key"
        />
      </el-tabs>
      
      <template v-if="infoListLoaded">
        <el-table 
          :data="currentInfoList" 
          border 
          style="width: 100%" 
          class="info-table"
          empty-text="暂无数据"
        >
          <el-table-column type="index" label="#" width="50" />
          <el-table-column prop="key" label="Key" min-width="220">
            <template #default="{ row }">
              <span class="info-key">{{ row.key }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="value" label="Value" show-overflow-tooltip min-width="250">
            <template #default="{ row }">
              <div class="value-cell">
                <span class="value-text">{{ row.value }}</span>
                <el-button 
                  link
                  size="small" 
                  class="copy-btn"
                  @click="copyValue(row.value)"
                >
                  复制
                </el-button>
              </div>
            </template>
          </el-table-column>
        </el-table>
      </template>
      <template v-else>
        <div class="skeleton-wrapper">
          <el-skeleton :rows="8" animated />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { DataAnalysis, Refresh, FolderOpened, InfoFilled, Setting } from '@element-plus/icons-vue'
import { redisStore, type ServerInfoResponse } from '../stores/redisStore';

interface ServerInfo {
  redis_version: string
  os: string
  process_id: string
}

interface MemoryInfo {
  used_memory_human: string
  used_memory_peak_human: string
  lua_memory_human: string
}

interface StatusInfo {
  connected_clients: string
  total_connections_received: string
  total_commands_processed: string
}

interface KeyStat {
  db: string
  keys: number
  expires: number
  avg_ttl: number
}

interface InfoItem {
  key: string
  value: string
  section: string
}

interface InfoSection {
  key: string
  label: string
}

const props = defineProps<{
  server: any
  db: number | null
}>()

const redis = redisStore()

const serverInfo = ref<ServerInfo>({
  redis_version: '',
  os: '',
  process_id: ''
})

const memoryInfo = ref<MemoryInfo>({
  used_memory_human: '',
  used_memory_peak_human: '',
  lua_memory_human: ''
})

const statusInfo = ref<StatusInfo>({
  connected_clients: '',
  total_connections_received: '',
  total_commands_processed: ''
})

const keyStats = ref<KeyStat[]>([])
const infoList = ref<InfoItem[]>([])
const searchInfo = ref('')
const activeInfoTab = ref('all')
const serverInfoLoaded = ref(false)
const keyStatsLoading = ref(false)
const infoListLoaded = ref(false)
const refreshingInfo = ref(false)

const infoSections: InfoSection[] = [
  { key: 'server', label: '服务器' },
  { key: 'clients', label: '客户端' },
  { key: 'memory', label: '内存' },
  { key: 'persistence', label: '持久化' },
  { key: 'stats', label: '统计' },
  { key: 'replication', label: '复制' },
  { key: 'cpu', label: 'CPU' },
  { key: 'cluster', label: '集群' },
  { key: 'keyspace', label: '键空间' }
]

const filteredInfo = computed(() => {
  let list = infoList.value
  if (!searchInfo.value) return list
  const keyword = searchInfo.value.toLowerCase()
  return list.filter(item => 
    item.key.toLowerCase().includes(keyword) || 
    item.value.toLowerCase().includes(keyword)
  )
})

const currentInfoList = computed(() => {
  if (activeInfoTab.value === 'all') return filteredInfo.value
  return filteredInfo.value.filter(item => item.section === activeInfoTab.value)
})

const copyValue = async (value: string) => {
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success('复制成功')
  } catch (err) {
    ElMessage.error('复制失败')
  }
}

const loadServerInfo = async () => {
  if (!props.server) return
  
  // 重置加载状态
  serverInfoLoaded.value = false
  infoListLoaded.value = false
  
  try {
    // 先加载服务器信息（这个比较快）
    const serverInfoRaw = await redis.getServerInfo({
      host: props.server.host,
      port: props.server.port,
      username: props.server.username,
      password: props.server.password,
      db: props.db || 0
    })
    
    // 解析 serverInfo 并组织成分组的 InfoItem
    const parsedInfo = parseServerInfo(serverInfoRaw)
    infoList.value = parsedInfo
    
    // 更新服务器信息卡片
    serverInfo.value = {
      redis_version: serverInfoRaw.redis_version || '',
      os: serverInfoRaw.os || '',
      process_id: serverInfoRaw.process_id || ''
    }
    
    // 更新内存信息卡片
    memoryInfo.value = {
      used_memory_human: serverInfoRaw.used_memory_human || '',
      used_memory_peak_human: serverInfoRaw.used_memory_peak_human || '',
      lua_memory_human: serverInfoRaw.used_memory_lua_human || ''
    }
    
    // 更新状态信息卡片
    statusInfo.value = {
      connected_clients: serverInfoRaw.connected_clients || '',
      total_connections_received: serverInfoRaw.total_connections_received || '',
      total_commands_processed: serverInfoRaw.total_commands_processed || ''
    }
    
    // 标记服务器信息加载完成
    serverInfoLoaded.value = true
    infoListLoaded.value = true
    
    // 在后台加载键值统计（不阻塞主界面）
    loadKeyStatsInBackground()
  } catch (error) {
    console.error('加载服务器信息失败:', error)
    ElMessage.error('加载服务器信息失败')
    // 即使失败也显示加载完成状态，避免骨架屏一直显示
    serverInfoLoaded.value = true
    infoListLoaded.value = true
  }
}

const loadKeyStatsInBackground = async () => {
  if (!props.server) return
  
  keyStatsLoading.value = true
  try {
    const keyStatsData = await redis.getKeyStats({
      host: props.server.host,
      port: props.server.port,
      username: props.server.username,
      password: props.server.password,
      db: props.db || 0
    })
    keyStats.value = keyStatsData
  } catch (err) {
    console.warn('获取键值统计失败:', err)
  } finally {
    keyStatsLoading.value = false
  }
}

const refreshServerInfo = async () => {
  refreshingInfo.value = true
  try {
    const serverInfoRaw = await redis.getServerInfo({
      host: props.server.host,
      port: props.server.port,
      username: props.server.username,
      password: props.server.password,
      db: props.db || 0
    })
    
    const parsedInfo = parseServerInfo(serverInfoRaw)
    infoList.value = parsedInfo
    
    serverInfo.value = {
      redis_version: serverInfoRaw.redis_version || '',
      os: serverInfoRaw.os || '',
      process_id: serverInfoRaw.process_id || ''
    }
    
    memoryInfo.value = {
      used_memory_human: serverInfoRaw.used_memory_human || '',
      used_memory_peak_human: serverInfoRaw.used_memory_peak_human || '',
      lua_memory_human: serverInfoRaw.used_memory_lua_human || ''
    }
    
    statusInfo.value = {
      connected_clients: serverInfoRaw.connected_clients || '',
      total_connections_received: serverInfoRaw.total_connections_received || '',
      total_commands_processed: serverInfoRaw.total_commands_processed || ''
    }
    
    ElMessage.success('刷新成功')
  } catch (error) {
    console.error('刷新服务器信息失败:', error)
    ElMessage.error('刷新失败')
  } finally {
    refreshingInfo.value = false
  }
}

const parseServerInfo = (serverInfo: ServerInfoResponse): InfoItem[] => {
  const result: InfoItem[] = []
  
  for (const [key, value] of Object.entries(serverInfo)) {
    let section = 'server'
    
    if (key.startsWith('redis_') || key === 'os' || key === 'arch_bits' || key === 'multiplexing_api' || 
        key === 'atomicvar_api' || key === 'gcc_version' || key === 'process_id' || key === 'process_supervised' || 
        key === 'run_id' || key === 'tcp_port' || key === 'server_time_usec' || key === 'uptime_in_seconds' || 
        key === 'uptime_in_days' || key === 'hz' || key === 'configured_hz' || key === 'lru_clock' || 
        key === 'executable' || key === 'config_file' || key === 'redis_mode') {
      section = 'server'
    } else if (key.startsWith('connected_clients') || key.startsWith('client_') || key === 'blocked_clients' || 
               key === 'tracking_clients' || key === 'clients_in_timeout_table' || key === 'maxclients') {
      section = 'clients'
    } else if (key.startsWith('used_memory') || key.startsWith('mem_') || key === 'maxmemory' || 
               key === 'maxmemory_policy' || key === 'mem_allocator') {
      section = 'memory'
    } else if (key.startsWith('loading') || key.startsWith('rdb_') || key.startsWith('aof_')) {
      section = 'persistence'
    } else if (key.startsWith('total_') || key.startsWith('instantaneous_') || key.startsWith('rejected_') || 
               key.startsWith('sync_') || key === 'expired_keys' || key === 'evicted_keys' || 
               key.startsWith('keyspace_') || key.startsWith('pubsub_')) {
      section = 'stats'
    } else if (key === 'role' || key.startsWith('connected_slaves') || key.startsWith('master_') || 
               key.startsWith('slave_') || key.startsWith('repl_')) {
      section = 'replication'
    } else if (key.startsWith('used_cpu')) {
      section = 'cpu'
    } else if (key.startsWith('cluster_')) {
      section = 'cluster'
    } else if (key.startsWith('db')) {
      section = 'keyspace'
    }
    
    result.push({ key, value, section })
  }
  
  return result
}

watch(() => [props.server, props.db], () => {
  loadServerInfo()
})

onMounted(() => {
  loadServerInfo()
})
</script>

<style scoped>
.home-view {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
  background-color: #f5f7fa;
  box-sizing: border-box;
}

.stats-cards {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.stat-card {
  flex: 1;
  min-width: 280px;
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.card-header {
  display: flex;
  align-items: center;
  padding: 14px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #ffffff;
}

.card-icon {
  margin-right: 8px;
  font-size: 18px;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
}

.card-content {
  padding: 16px;
}

.skeleton-item {
  padding: 4px 0;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 13px;
  color: #909399;
}

.info-value {
  font-size: 13px;
  color: #303133;
  font-weight: 500;
}

.info-value.highlight {
  color: #67c23a;
  font-weight: 600;
}

.table-section {
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  padding: 16px;
  margin-bottom: 20px;
}

.table-section.info-section {
  padding-bottom: 16px;
}

.skeleton-wrapper {
  padding: 8px 0;
}

.section-header {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 10px;
}

.section-header .el-button {
  margin-left: auto;
}

.section-icon {
  margin-right: 8px;
  color: #667eea;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.search-input {
  width: 200px;
}

.info-tabs {
  margin-bottom: 12px;
}

.stats-table,
.info-table {
  font-size: 13px;
}

.stats-table .el-table__header th,
.info-table .el-table__header th {
  background-color: #fafafa;
  color: #606266;
  font-weight: 500;
}

.stats-table .el-table__body tr:hover,
.info-table .el-table__body tr:hover {
  background-color: #f5f7fa;
}

.info-key {
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  color: #303133;
  font-weight: 500;
}

.value-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.value-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #606266;
}

.copy-btn {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.2s;
  color: #409eff;
}

.value-cell:hover .copy-btn {
  opacity: 1;
}
</style>
