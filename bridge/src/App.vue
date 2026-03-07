<template>
  <div id="app">
    <el-container style="height: 100vh;">
      <el-header v-if="showHeader" style="background: #fff; border-bottom: 1px solid #e4e7ed; padding: 0 20px;">
        <div style="display: flex; align-items: center; justify-content: space-between; height: 60px;">
          <div style="display: flex; align-items: center; gap: 12px;">
            <span style="font-size: 20px;">🐾</span>
            <span style="font-size: 18px; font-weight: 600;">Claw One</span>
          </div>
          <el-menu :router="true" mode="horizontal" :ellipsis="false" style="border-bottom: none;">
            <el-menu-item index="/status">状态</el-menu-item>
            <el-menu-item index="/config">配置</el-menu-item>
          </el-menu>
        </div>
      </el-header>
      <el-main style="padding: 0; overflow: hidden;">
        <router-view />
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const showHeader = computed(() => {
  // 在 setup 向导和 safe mode 页面不显示导航
  return !['/setup', '/safe-mode'].includes(route.path)
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: #f5f7fa;
}

#app {
  height: 100vh;
}
</style>
