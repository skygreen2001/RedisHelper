<template>
  <el-dialog
    v-model="visible"
    title="内存分析"
    width="92%"
    :destroy-on-close="true"
    :close-on-click-modal="false"
    draggable
    align-center
    class="memory-dialog"
    @close="handleClose"
  >
    <div class="memory-body">
      <div v-if="loading" class="loading-state">
        <el-icon class="is-loading"><Loading /></el-icon>
        <span>正在分析内存...</span>
      </div>

      <div v-else class="memory-content">
        <div class="dashboard-section">
          <div class="section-header" @click="dashboardCollapsed = !dashboardCollapsed">
            <span class="collapsible-icon" :class="{ collapsed: dashboardCollapsed }">
              <el-icon><ArrowDown /></el-icon>
            </span>
            <span>内存概览</span>
          </div>
          <div v-show="!dashboardCollapsed" class="dashboard-body">
            <div class="memory-dashboard">
              <div class="memory-card">
                <div class="memory-card-title">已用内存</div>
                <div class="memory-card-value">{{ memoryInfo.used_memory_human }}</div>
                <div class="memory-card-detail">
                  总内存 {{ formatBytes(memoryInfo.maxmemory) }} · 
                  使用率 {{ calculateUsageRate() }}%
                </div>
              </div>

              <div class="memory-card">
                <div class="memory-card-title">峰值内存</div>
                <div class="memory-card-value">{{ memoryInfo.used_memory_peak_human }}</div>
                <div class="memory-card-detail">历史最高使用量</div>
              </div>

              <div class="memory-card" :class="getFragmentationClass()">
                <div class="memory-card-title">内存碎片率</div>
                <div class="memory-card-value">{{ memoryInfo.mem_fragmentation_ratio.toFixed(2) }}</div>
                <div class="memory-card-detail">{{ getFragmentationSuggestion() }}</div>
              </div>

              <div class="memory-card">
                <div class="memory-card-title">键总数</div>
                <div class="memory-card-value">{{ memoryInfo.keys_count.toLocaleString() }}</div>
                <div class="memory-card-detail">当前数据库</div>
              </div>

              <div class="memory-card">
                <div class="memory-card-title">过期键占比</div>
                <div class="memory-card-value">{{ memoryInfo.expired_keys_ratio.toFixed(1) }}%</div>
                <div class="memory-card-detail">{{ calculateExpiredCount() }} 个键设置过期</div>
              </div>

              <div class="memory-card">
                <div class="memory-card-title">大键数量</div>
                <div class="memory-card-value">{{ memoryInfo.large_keys_count }}</div>
                <div class="memory-card-detail">占用显著内存</div>
              </div>
            </div>
          </div>
        </div>

        <div class="charts-section">
          <div class="section-header" @click="chartsCollapsed = !chartsCollapsed">
            <span class="collapsible-icon" :class="{ collapsed: chartsCollapsed }">
              <el-icon><ArrowDown /></el-icon>
            </span>
            <span>键类型分析</span>
            <span v-if="loadingTypes" class="chart-loading-tag">
              <el-icon class="is-loading" style="font-size:12px"><Loading /></el-icon>
              扫描中...
            </span>
            <span v-else-if="typeStatsComplete" class="chart-complete-tag">全量统计</span>
          </div>
          <div v-show="!chartsCollapsed" class="charts-body">
            <div class="chart-box">
              <div class="chart-title">
                键类型分布
              </div>
              <div v-if="loadingTypes && memoryInfo.key_type_stats.length === 0" class="chart-placeholder">
                <el-icon class="is-loading"><Loading /></el-icon>
                <span>正在全量扫描键类型...</span>
              </div>
              <template v-else>
                <div class="pie-chart" :style="getPieChartStyle()">
                  <div class="pie-center">
                    <div class="pie-center-text">{{ memoryInfo.keys_count }}</div>
                    <div class="pie-center-label">Total</div>
                  </div>
                </div>
                <div class="chart-legend">
                  <div 
                    v-for="stat in memoryInfo.key_type_stats" 
                    :key="stat.key_type"
                    class="legend-item"
                  >
                    <div class="legend-dot" :style="{ background: getTypeColor(stat.key_type) }"></div>
                    <span>{{ stat.key_type.toUpperCase() }}</span>
                    <span class="legend-percent">{{ stat.memory_percent.toFixed(1) }}%</span>
                  </div>
                </div>
              </template>
            </div>

            <div class="chart-box">
              <div class="chart-title">
                键类型数量占比
              </div>
              <div v-if="loadingTypes && memoryInfo.key_type_stats.length === 0" class="chart-placeholder">
                <el-icon class="is-loading"><Loading /></el-icon>
                <span>正在统计...</span>
              </div>
              <div v-else class="bar-chart">
                <div 
                  v-for="stat in memoryInfo.key_type_stats" 
                  :key="stat.key_type"
                  class="bar-item"
                >
                  <div class="bar-label">{{ stat.key_type.toUpperCase() }}</div>
                  <div class="bar-track">
                    <div 
                      class="bar-fill" 
                      :style="{ 
                        width: Math.max(stat.memory_percent, 2) + '%',
                        background: getTypeColor(stat.key_type)
                      }"
                    >
                      {{ stat.count.toLocaleString() }} 个
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="key-list-section">
          <div class="key-list-header">
            <div class="header-key">
              <span>🔑</span>
              <span>Key</span>
            </div>
            <div class="header-size" @click="toggleSort">
              <span>Size</span>
              <span class="sort-icon">{{ sortOrder === 'asc' ? '↑' : '↓' }}</span>
            </div>
          </div>

          <div class="key-list-content" ref="listContentRef">
            <div 
              v-for="(item, index) in paginatedKeyList" 
              :key="index"
              class="key-list-item"
            >
              <div class="item-key">
                <span class="item-index">{{ Math.min((currentPage - 1) * pageSize + index + 1, memoryInfo.keys_count) }}.</span>
                <span class="item-name">{{ item.key }}</span>
              </div>
              <div class="item-size">
                <span class="size-badge">{{ item.size_human }}</span>
              </div>
            </div>
          </div>

          <div class="key-list-pagination" @click="handlePaginationClick" @wheel.prevent>
            <el-pagination
              :current-page="currentPage"
              :page-size="pageSize"
              :total="memoryInfo.keys_count"
              :page-sizes="[50, 100, 200, 500]"
              :pager-count="5"
              layout="total, sizes, prev, pager, next, jumper"
              @size-change="handleSizeChange"
              @current-change="handleCurrentChange"
              :prev-text="'上一页'"
              :next-text="'下一页'"
              ref="paginationRef"
            />
          </div>

          <div class="key-list-footer">
            <span class="list-summary">
              已扫描 {{ memoryInfo.key_memory_list.length }} / 总计 {{ memoryInfo.keys_count }} 个键 · 
              当前页大小：{{ formatBytes(calculatePageTotalSize()) }}
              <span v-if="loadingMore" class="loading-more">（加载中...）</span>
            </span>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="memory-dialog-footer">
        <el-button @click="handleRefresh" :loading="loading">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button @click="visible = false">关闭</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { Loading, Refresh, ArrowDown } from '@element-plus/icons-vue'
import { redisStore, type MemoryInfoResponse } from '../stores/redisStore'
import { ElMessage } from 'element-plus'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const redis = redisStore()
const loading = ref(false)
const loadingMore = ref(false)
const loadingTypes = ref(false)        // 类型分布异步加载中
const typeStatsComplete = ref(false)   // 是否已完成全量类型统计
const sortOrder = ref<'asc' | 'desc'>('desc')
const currentPage = ref(1)
const pageSize = ref(100)
const nextCursor = ref('0')
const paginationRef = ref()
const listContentRef = ref<HTMLElement>()
const chartsCollapsed = ref(false)
const dashboardCollapsed = ref(false)
const memoryInfo = ref<MemoryInfoResponse>({
  used_memory: 0,
  used_memory_human: '0B',
  used_memory_peak: 0,
  used_memory_peak_human: '0B',
  mem_fragmentation_ratio: 0,
  maxmemory: 0,
  keys_count: 0,
  expired_keys_ratio: 0,
  large_keys_count: 0,
  key_memory_list: [],
  key_type_stats: [],
  next_cursor: '0',
})

const TYPE_COLORS: Record<string, string> = {
  string: '#409EFF',
  hash: '#67c23a',
  list: '#e6a23c',
  set: '#f56c6c',
  zset: '#909399',
}

function getTypeColor(type: string): string {
  return TYPE_COLORS[type] || '#909399'
}

function formatBytes(bytes: number): string {
  const KB = 1024
  const MB = KB * 1024
  const GB = MB * 1024
  
  if (bytes >= GB) {
    return (bytes / GB).toFixed(2) + 'GB'
  } else if (bytes >= MB) {
    return (bytes / MB).toFixed(2) + 'MB'
  } else if (bytes >= KB) {
    return (bytes / KB).toFixed(2) + 'KB'
  } else {
    return bytes + 'B'
  }
}

function calculateUsageRate(): string {
  if (memoryInfo.value.maxmemory === 0) return '0'
  const rate = (memoryInfo.value.used_memory / memoryInfo.value.maxmemory) * 100
  return rate.toFixed(1)
}

function calculateExpiredCount(): string {
  const count = Math.round(memoryInfo.value.keys_count * memoryInfo.value.expired_keys_ratio / 100)
  return count.toLocaleString()
}

function getFragmentationClass(): string {
  const ratio = memoryInfo.value.mem_fragmentation_ratio
  if (ratio > 1.5) return 'danger'
  if (ratio > 1.2) return 'warning'
  return 'success'
}

function getFragmentationSuggestion(): string {
  const ratio = memoryInfo.value.mem_fragmentation_ratio
  if (ratio > 1.5) return '建议：碎片过多'
  if (ratio > 1.2) return '建议：可优化'
  return '状态：正常'
}

function getPieChartStyle(): Record<string, string> {
  const stats = memoryInfo.value.key_type_stats
  if (stats.length === 0) return {}
  
  let gradient = ''
  let currentPercent = 0
  
  stats.forEach((stat) => {
    const color = getTypeColor(stat.key_type)
    const nextPercent = currentPercent + stat.memory_percent
    gradient += `${color} ${currentPercent}% ${nextPercent}%, `
    currentPercent = nextPercent
  })
  
  gradient = gradient.slice(0, -2)
  
  return {
    background: `conic-gradient(${gradient})`
  }
}

// 按 SCAN 顺序分页（稳定分页，不因排序而打乱）
const paginatedRawList = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return memoryInfo.value.key_memory_list.slice(start, end)
})

// 当前页内按排序规则排序
const paginatedKeyList = computed(() => {
  const list = [...paginatedRawList.value]
  if (sortOrder.value === 'asc') {
    return list.sort((a, b) => a.size - b.size)
  } else {
    return list.sort((a, b) => b.size - a.size)
  }
})

function toggleSort() {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  currentPage.value = 1
}

// 点击分页区域阻止默认行为，防止 Tauri 下触发 scrollIntoView
function handlePaginationClick(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  blurActiveElement()
}

// 点击分页按钮后立即 blur，防止 focus 引起浏览器 scrollIntoView（Tauri 下尤其明显）
function blurActiveElement() {
  const el = document.activeElement as HTMLElement | null
  if (el && el.tagName !== 'BODY') el.blur()
}

async function handleSizeChange(size: number) {
  blurActiveElement()
  pageSize.value = size
  currentPage.value = 1
  scrollToTop()

  const neededCount = pageSize.value
  const loadedCount = memoryInfo.value.key_memory_list.length
  if (neededCount > loadedCount && nextCursor.value !== '0') {
    await loadMoreUntil(neededCount)
  }
}

async function handleCurrentChange(page: number) {
  blurActiveElement()
  currentPage.value = page
  // 点击分页时折叠顶部区块，减少视觉干扰
  dashboardCollapsed.value = true
  chartsCollapsed.value = true
  scrollToTop()

  const neededCount = page * pageSize.value
  const loadedCount = memoryInfo.value.key_memory_list.length
  if (neededCount > loadedCount && nextCursor.value !== '0') {
    await loadMoreUntil(neededCount)
  }
}

// 列表滚动到顶部（只滚动列表区域，不影响页面）
function scrollToTop() {
  nextTick(() => {
    const content = document.querySelector('.key-list-content') as HTMLElement
    if (content) {
      content.scrollTop = 0
    }
  })
}

// 增量加载更多数据，直到满足 neededCount 或无更多数据
async function loadMoreUntil(neededCount: number) {
  if (loadingMore.value) return
  loadingMore.value = true
  try {
    const serverInfo = currentServerInfo.value
    if (!serverInfo) return
    // 循环加载直到满足需要的数据量，或无更多数据
    while (memoryInfo.value.key_memory_list.length < neededCount && nextCursor.value !== '0') {
      const result = await redis.getMemoryInfo(serverInfo, nextCursor.value)
      if (result.key_memory_list.length === 0) break
      memoryInfo.value.key_memory_list.push(...result.key_memory_list)
      nextCursor.value = result.next_cursor
    }
  } catch (error: any) {
    const errorMsg = error instanceof Error 
      ? error.message 
      : (typeof error === 'string' ? error : String(error))
    ElMessage.error('加载更多数据失败：' + errorMsg)
  } finally {
    loadingMore.value = false
  }
}

// 后台异步全量扫描键类型分布（不阻塞主界面）
async function loadTypeDistribution() {
  const serverInfo = currentServerInfo.value
  if (!serverInfo) return
  loadingTypes.value = true
  typeStatsComplete.value = false
  try {
    const stats = await redis.getTypeDistribution(serverInfo)
    // 更新类型分布（替换采样数据）
    memoryInfo.value.key_type_stats = stats
    typeStatsComplete.value = true
  } catch (error: any) {
    // 类型分布加载失败不影响主界面，静默处理
    console.error('[MemoryDialog] 全量类型扫描失败:', error)
  } finally {
    loadingTypes.value = false
  }
}

function calculatePageTotalSize(): number {
  return paginatedKeyList.value.reduce((sum, item) => sum + item.size, 0)
}

async function handleRefresh() {
  loading.value = true
  typeStatsComplete.value = false
  try {
    const serverInfo = currentServerInfo.value
    if (serverInfo) {
      const result = await redis.getMemoryInfo(serverInfo)
      memoryInfo.value = result
      nextCursor.value = result.next_cursor
      currentPage.value = 1
      // 主数据加载完成后，后台异步加载全量类型分布
      loadTypeDistribution()
    }
  } catch (error: any) {
    const errorMsg = error instanceof Error 
      ? error.message 
      : (typeof error === 'string' ? error : String(error))
    ElMessage.error('刷新内存信息失败：' + errorMsg)
  } finally {
    loading.value = false
  }
}

function handleClose() {
  // 重置状态
}

const currentServerInfo = ref<{ host: string; port: number; username?: string; password?: string; db: number } | null>(null)

async function load(serverInfo: { host: string; port: number; username?: string; password?: string; db: number }) {
  currentServerInfo.value = serverInfo
  loading.value = true
  typeStatsComplete.value = false
  try {
    const result = await redis.getMemoryInfo(serverInfo)
    memoryInfo.value = result
    nextCursor.value = result.next_cursor
    currentPage.value = 1
    // 主数据加载完成后，后台异步加载全量类型分布
    loadTypeDistribution()
  } catch (error: any) {
    const errorMsg = error instanceof Error 
      ? error.message 
      : (typeof error === 'string' ? error : String(error))
    ElMessage.error('获取内存信息失败：' + errorMsg)
  } finally {
    loading.value = false
  }
}

defineExpose({ load })
</script>

<style scoped>
.memory-body {
  display: flex;
  flex-direction: column;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--el-text-color-secondary);
}

.memory-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.dashboard-section {
  background: #fafafa;
  border-radius: 12px;
  border: 1px solid #ebeef5;
  overflow: hidden;
}

.dashboard-body {
  padding: 20px;
}

.dashboard-body .memory-dashboard {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 16px;
}

.memory-card {
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e8eb 100%);
  border-radius: 12px;
  padding: 20px;
  border-left: 4px solid #409EFF;
  transition: all 0.3s ease;
}

.memory-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.08);
}

.memory-card-title {
  font-size: 13px;
  color: #909399;
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.memory-card-value {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  margin-bottom: 6px;
}

.memory-card-detail {
  font-size: 12px;
  color: #606266;
}

.memory-card.warning {
  border-left-color: #e6a23c;
}

.memory-card.danger {
  border-left-color: #f56c6c;
}

.memory-card.success {
  border-left-color: #67c23a;
}

.charts-section {
  background: #fafafa;
  border-radius: 12px;
  border: 1px solid #ebeef5;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  cursor: pointer;
  user-select: none;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
  border-bottom: 1px solid #ebeef5;
  transition: background 0.2s;
}

.section-header:hover {
  background: #f0f2f5;
}

.charts-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  padding: 20px;
}

.collapsible-icon {
  display: inline-flex;
  align-items: center;
  transition: transform 0.3s ease;
  color: #909399;
}

.collapsible-icon.collapsed {
  transform: rotate(-90deg);
}

.chart-box {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  border: 1px solid #ebeef5;
}

.chart-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.chart-loading-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #909399;
  font-weight: 400;
}

.chart-complete-tag {
  font-size: 11px;
  color: #67c23a;
  background: #f0f9eb;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 400;
}

.chart-sample-tag {
  font-size: 11px;
  color: #e6a23c;
  background: #fdf6ec;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 400;
}

.chart-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 140px;
  color: #909399;
  font-size: 13px;
}

.pie-chart {
  width: 180px;
  height: 180px;
  border-radius: 50%;
  margin: 0 auto 16px;
  position: relative;
}

.pie-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 80px;
  height: 80px;
  background: white;
  border-radius: 50%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.pie-center-text {
  font-size: 20px;
  font-weight: 700;
  color: #303133;
}

.pie-center-label {
  font-size: 11px;
  color: #909399;
}

.chart-legend {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #606266;
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.legend-percent {
  margin-left: auto;
  color: #909399;
}

.bar-chart {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.bar-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.bar-label {
  width: 60px;
  font-size: 13px;
  color: #606266;
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 24px;
  background: #ebeef5;
  border-radius: 12px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 12px;
  display: flex;
  align-items: center;
  padding-left: 10px;
  color: white;
  font-size: 12px;
  font-weight: 600;
  min-width: fit-content;
}

.key-list-section {
  border: 1px solid #ebeef5;
  border-radius: 8px;
  overflow: hidden;
  background: #fafafa;
}

.key-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background: #fff;
  border-bottom: 1px solid #ebeef5;
  font-weight: 600;
  color: #303133;
}

.header-key {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-size {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: #409EFF;
}

.header-size:hover {
  color: #66b1ff;
}

.sort-icon {
  font-size: 12px;
}

.key-list-content {
  max-height: 400px;
  overflow-y: auto;
}

.key-list-pagination {
  display: flex;
  justify-content: center;
  padding: 16px;
  background: #fff;
  border-top: 1px solid #f0f0f0;
  border-bottom: 1px solid #f0f0f0;
}

.key-list-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 20px;
  background: #fff;
  border-bottom: 1px solid #f0f0f0;
  transition: background 0.2s;
}

.key-list-item:hover {
  background: #f5f7fa;
}

.key-list-item:last-child {
  border-bottom: none;
}

.item-key {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  overflow: hidden;
}

.item-index {
  font-size: 12px;
  color: #909399;
  min-width: 32px;
}

.item-name {
  font-size: 14px;
  color: #303133;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-size {
  display: flex;
  align-items: center;
}

.size-badge {
  background: linear-gradient(135deg, #66b1ff 0%, #409EFF 100%);
  color: white;
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.key-list-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background: #fff;
  border-top: 1px solid #ebeef5;
}

.list-summary {
  font-size: 13px;
  color: #909399;
}

.loading-more {
  color: #409EFF;
  font-size: 12px;
}

.memory-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

@media (max-width: 768px) {
  .charts-section {
    grid-template-columns: 1fr;
  }
}
</style>

<style>
.memory-dialog .el-dialog__header {
  cursor: move;
  user-select: none;
  margin-right: 0;
  padding: 12px 16px;
}

.memory-dialog .el-dialog {
  max-width: 92vw !important;
  min-width: 600px;
}

/* 分页器按钮 focus 不触发浏览器滚动（Tauri 下 focus 会 scrollIntoView 导致页面跳动） */
.memory-dialog .el-pager li:focus {
  outline: none;
  box-shadow: none !important;
  scroll-margin-top: 0;
  scroll-margin-bottom: 0;
}
.memory-dialog .el-pagination button:focus {
  outline: none;
  box-shadow: none !important;
  scroll-margin-top: 0;
  scroll-margin-bottom: 0;
}
.memory-dialog .el-pagination button:focus-visible {
  outline: none;
}
.memory-dialog .el-pager li:focus-visible {
  outline: none;
}

/* 防止分页器 select 元素触发滚动 */
.memory-dialog .el-pagination .el-select {
  scroll-margin-top: 0;
}

/* 防止分页切换时对话框滚动 */
.memory-dialog .el-dialog__body {
  overflow: visible;
}

/* 分页区域禁止页面级滚动 */
.memory-dialog .key-list-pagination {
  scroll-margin-top: 0;
  scroll-margin-bottom: 0;
  scroll-padding-top: 0;
  scroll-padding-bottom: 0;
}
</style>
