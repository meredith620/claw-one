<template>
  <div class="config-export-import">
    <el-dropdown split-button type="primary" @click="exportConfig" @command="handleCommand">
      <el-icon><Download /></el-icon>
      导出配置
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item command="export">导出为 JSON</el-dropdown-item>
          <el-dropdown-item command="exportEncrypted" divided>加密导出</el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>

    <el-button @click="showImportDialog = true">
      <el-icon><Upload /></el-icon>
      导入配置
    </el-button>

    <!-- 导入对话框 -->
    <el-dialog v-model="showImportDialog" title="导入配置" width="500px">
      <el-upload
        ref="uploadRef"
        class="config-uploader"
        drag
        action="#"
        :auto-upload="false"
        :on-change="handleFileChange"
        :limit="1"
        accept=".json"
      >
        <el-icon class="el-icon--upload"><upload-filled /></el-icon>
        <div class="el-upload__text">
          拖拽文件到此处或 <em>点击上传</em>
        </div>
        <template #tip>
          <div class="el-upload__tip">
            请上传 JSON 格式的配置文件
          </div>
        </template>
      </el-upload>

      <el-alert
        v-if="importPreview"
        :title="`配置预览：${importPreview.providers} 个 Provider, ${importPreview.agents} 个 Agent`"
        type="info"
        :closable="false"
        class="import-preview"
      />

      <template #footer>
        <el-button @click="showImportDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmImport" :loading="importing" :disabled="!importData">确认导入</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Download, Upload, UploadFilled } from '@element-plus/icons-vue'
import { getConfig, applyConfig } from '../api'
import { requestConfigReload } from '../stores/config'

const showImportDialog = ref(false)
const importing = ref(false)
const importData = ref<any>(null)
const importPreview = ref<{ providers: number; agents: number; channels: number } | null>(null)
const uploadRef = ref()

// 导出配置
const exportConfig = async () => {
  try {
    const res = await getConfig()
    const config = res.data
    
    // 创建下载链接
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `openclaw-config-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
    
    ElMessage.success('配置已导出')
  } catch (error: any) {
    ElMessage.error('导出失败: ' + (error.response?.data?.error || error.message))
  }
}

// 处理下拉菜单命令
const handleCommand = (command: string) => {
  if (command === 'exportEncrypted') {
    ElMessage.info('加密导出功能开发中...')
  }
}

// 处理文件选择
const handleFileChange = (file: any) => {
  const reader = new FileReader()
  reader.onload = (e) => {
    try {
      const content = e.target?.result as string
      const config = JSON.parse(content)
      
      // 验证配置格式
      if (!config || typeof config !== 'object') {
        ElMessage.error('无效的配置文件格式')
        return
      }
      
      importData.value = config
      
      // 预览配置内容
      const providers = Object.keys(config.models?.providers || {}).length
      const agents = config.agents?.list?.length || 0
      const channels = Object.keys(config.channels?.mattermost?.accounts || {}).length
      
      importPreview.value = { providers, agents, channels }
    } catch (err) {
      ElMessage.error('解析配置文件失败')
      importData.value = null
      importPreview.value = null
    }
  }
  reader.readAsText(file.raw)
}

// 确认导入
const confirmImport = async () => {
  if (!importData.value) return
  
  try {
    await ElMessageBox.confirm(
      '导入配置将覆盖当前所有配置，确定继续吗？',
      '确认导入',
      { type: 'warning' }
    )
    
    importing.value = true
    
    await applyConfig({
      config: importData.value,
      message: 'Import configuration from file'
    })
    
    ElMessage.success('配置导入成功')
    showImportDialog.value = false
    importData.value = null
    importPreview.value = null
    
    // 请求重启
    requestConfigReload()
    
    // 刷新页面
    setTimeout(() => {
      window.location.reload()
    }, 1500)
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('导入失败: ' + (error.response?.data?.error || error.message))
    }
  } finally {
    importing.value = false
  }
}
</script>

<style scoped>
.config-export-import {
  display: flex;
  gap: 12px;
}

.config-uploader {
  width: 100%;
}

.import-preview {
  margin-top: 16px;
}
</style>
