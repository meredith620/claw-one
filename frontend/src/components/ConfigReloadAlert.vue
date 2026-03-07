<template>
  <!-- 配置变更提示条 -->
  <el-alert
    v-if="showReloadAlert"
    title="配置已更新"
    type="success"
    :closable="false"
    show-icon
    class="reload-alert"
  >
    <template #default>
      <div class="reload-content">
        <span>配置已保存，需要重启 OpenClaw 生效</span>
        <el-button type="primary" size="small" @click="reloadConfig" :loading="reloading">
          <el-icon><Refresh /></el-icon>
          立即重启
        </el-button>
        <el-button size="small" @click="dismiss">稍后</el-button>
      </div>
    </template>
  </el-alert>

  <!-- 保存成功提示 -->
  <el-alert
    v-else-if="showSavedAlert"
    title="配置已保存"
    type="success"
    :closable="true"
    @close="dismissSaved"
    show-icon
    class="saved-alert"
  >
    <template #default>
      <span>{{ savedMessage }}</span>
    </template>
  </el-alert>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { configState, completeConfigReload } from '../stores/config'
import { restartService } from '../api'

const reloading = ref(false)
const showSavedAlert = ref(false)
const savedMessage = ref('')

const showReloadAlert = computed(() => configState.pendingReload)

// 监听保存事件
watch(() => configState.lastSaved, () => {
  if (configState.lastSaved) {
    savedMessage.value = `配置已保存于 ${configState.lastSaved.toLocaleTimeString()}`
    showSavedAlert.value = true
    setTimeout(() => { showSavedAlert.value = false }, 5000)
  }
})

const reloadConfig = async () => {
  reloading.value = true
  try {
    await restartService()
    ElMessage.success('OpenClaw 重启中，请稍候...')
    completeConfigReload()
    
    // 轮询检查服务状态
    setTimeout(checkServiceStatus, 3000)
  } catch (error: any) {
    ElMessage.error('重启失败: ' + (error.response?.data?.error || error.message))
  } finally {
    reloading.value = false
  }
}

const checkServiceStatus = async () => {
  // 简单刷新页面来检查服务是否恢复
  window.location.reload()
}

const dismiss = () => {
  completeConfigReload()
}

const dismissSaved = () => {
  showSavedAlert.value = false
}
</script>

<style scoped>
.reload-alert, .saved-alert {
  margin-bottom: 16px;
}

.reload-content {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.reload-content span {
  flex: 1;
}
</style>
