<template>
  <div class="app-container">
    <el-container style="height: 100vh; width: 100vw;">
      <el-header height="60px" class="header">
        <div class="logo">Redis小助手</div>
        <el-menu
          :default-active="activeMenu"
          class="el-menu-demo"
          mode="horizontal"
          @select="handleMenuSelect"
        >
          <el-menu-item index="main">
            <el-icon><HomeFilled /></el-icon>
            <span>主界面</span>
          </el-menu-item>
          <el-menu-item index="server">
            <el-icon><Setting /></el-icon>
            <span>服务器设置</span>
          </el-menu-item>
        </el-menu>
      </el-header>
      <el-main>
        <MainView v-if="activeMenu === 'main'" />
        <ServerConfigView v-else-if="activeMenu === 'server'" />
      </el-main>
      <el-footer height="40px" class="footer">
        <span>Redis小助手 © 2026</span>
      </el-footer>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { HomeFilled, Setting } from '@element-plus/icons-vue'
import MainView from './views/MainView.vue'
import ServerConfigView from './views/ServerConfigView.vue'

const activeMenu = ref('main')

const handleMenuSelect = (key: string) => {
  activeMenu.value = key
}
</script>

<style scoped>
.app-container {
  width: 100%;
  height: 100%;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: #1890ff;
  color: white;
  padding: 0 20px;
}

.logo {
  font-size: 20px;
  font-weight: bold;
}

.el-menu-demo {
  background-color: transparent;
  border-bottom: none;
}

.el-menu-item {
  color: white;
}

.el-menu-item.is-active {
  color: white;
  background-color: rgba(255, 255, 255, 0.2);
}

.footer {
  text-align: center;
  line-height: 40px;
  background-color: #f5f7fa;
  color: #909399;
}
</style>