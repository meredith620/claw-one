<template>
  <div class="config-layout">
    <div class="config-header">
      <h2>Claw One 配置</h2>
      <div class="header-actions">
        <ConfigExportImport />
        <el-divider direction="vertical" />
        <el-tag v-if="hasUnsavedChanges" type="warning">有未保存的更改</el-tag>
        <el-button type="primary" @click="saveAll" :loading="saving">保存配置</el-button>
        <el-button @click="restartOpenClaw" :loading="restarting">重启服务</el-button>
      </div>
    </div>
    <div class="config-body">
      <div class="config-sidebar">
        <div v-for="item in menuItems" :key="item.key" class="menu-item" :class="{ active: currentModule === item.key }" @click="switchModule(item.key)">
          <span class="menu-icon">{{ item.icon }}</span>
          <span class="menu-label">{{ item.label }}</span>
        </div>
      </div>
      <div class="config-content">
        <ConfigReloadAlert />
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { restartService } from '../api'
import ConfigReloadAlert from '../components/ConfigReloadAlert.vue'
import ConfigExportImport from '../components/ConfigExportImport.vue'
import { markConfigSaved, requestConfigReload } from '../stores/config'

const router = useRouter()

const menuItems = [
  { key: 'provider', label: 'Provider', icon: '🧠' },
  { key: 'agent', label: 'Agent', icon: '🤖' },
  { key: 'memory', label: 'Memory', icon: '💾' },
  { key: 'channel', label: 'Channel', icon: '📱' },
]

const currentModule = ref('provider')
const saving = ref(false)
const restarting = ref(false)
const hasUnsavedChanges = ref(false)

const switchModule = (key: string) => {
  currentModule.value = key
  router.push(`/config/${key}`)
}

const saveAll = async () => {
  saving.value = true
  try {
    // 触发各模块保存事件
    window.dispatchEvent(new CustomEvent('claw:save-all'))
    markConfigSaved()
    
    // 显示热重载提示
    requestConfigReload()
    
    ElMessage.success('配置保存成功')
  } finally {
    saving.value = false
  }
}

const restartOpenClaw = async () => {
  try {
    await ElMessageBox.confirm('确定要重启 OpenClaw 服务吗？', '确认重启', {
      type: 'warning',
    })
    restarting.value = true
    const res = await restartService()
    ElMessage.success('重启指令已发送: ' + res.data?.message)
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('重启失败: ' + (error.response?.data?.error || error.message))
    }
  } finally {
    restarting.value = false
  }
}
</script>

<style scoped>
.config-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #f5f7fa;
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
}

.config-header h2 {
  margin: 0;
  font-size: 20px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.config-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.config-sidebar {
  width: 240px;
  background: #fff;
  border-right: 1px solid #e4e7ed;
  padding: 16px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  cursor: pointer;
  transition: all 0.3s;
  margin: 0 8px;
  border-radius: 8px;
}

.menu-item:hover {
  background: #f5f7fa;
}

.menu-item.active {
  background: #ecf5ff;
  color: #409eff;
}

.menu-icon {
  font-size: 20px;
}

.menu-label {
  flex: 1;
  font-weight: 500;
}

.config-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  padding-bottom: 60px;
}
</style>
