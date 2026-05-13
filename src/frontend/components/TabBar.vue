<template>
  <div v-if="sessionManager.showTabBar" class="tab-bar">
    <div class="tab-list" ref="tabListRef">
      <div
        v-for="session in sessionManager.sessions"
        :key="session.id"
        class="tab-item"
        :class="{ active: session.id === sessionManager.activeSessionId }"
        @click="sessionManager.switchSession(session.id)"
      >
        <span class="tab-title">{{ session.title }}</span>
        <span
          v-if="sessionManager.sessions.length > 1"
          class="tab-close"
          @click.stop="sessionManager.closeSession(session.id)"
        >×</span>
      </div>
    </div>
    <div class="tab-new" @click="sessionManager.createSession()" title="新建标签">
      +
    </div>
  </div>
</template>

<script setup lang="ts">
import { sessionManager } from '../sessions/SessionManager'
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  background: #f5f5f5;
  border-bottom: 1px solid #e4e7ed;
  padding: 0 8px;
  height: 36px;
  user-select: none;
  -webkit-app-region: no-drag;
}

.tab-list {
  display: flex;
  align-items: center;
  overflow-x: auto;
  flex: 1;
  gap: 2px;
  scrollbar-width: none;
}

.tab-list::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 6px 6px 0 0;
  cursor: pointer;
  font-size: 13px;
  color: #606266;
  background: transparent;
  border: 1px solid transparent;
  border-bottom: none;
  white-space: nowrap;
  max-width: 180px;
  min-width: 60px;
  transition: all 0.15s ease;
}

.tab-item:hover {
  background: #e8e8e8;
}

.tab-item.active {
  background: #ffffff;
  color: #409eff;
  border-color: #e4e7ed;
  font-weight: 500;
}

.tab-title {
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  font-size: 14px;
  line-height: 1;
  color: #909399;
  flex-shrink: 0;
}

.tab-close:hover {
  background: #c0c4cc;
  color: #ffffff;
}

.tab-new {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 18px;
  color: #909399;
  flex-shrink: 0;
  margin-left: 4px;
}

.tab-new:hover {
  background: #e8e8e8;
  color: #409eff;
}
</style>
