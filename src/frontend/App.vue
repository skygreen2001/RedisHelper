<template>
  <div class="app-container">
    <MainView />
  </div>
</template>

<script setup lang="ts">
import MainView from './views/MainView.vue'
</script>

<style scoped>
.app-container {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}
</style>