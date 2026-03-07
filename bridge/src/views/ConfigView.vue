<template>
  <div class="config-view">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>配置编辑</span>
          <el-button @click="goBack">返回</el-button>
        </div>
      </template>

      <!-- JSON 编辑器 -->
      <el-form v-loading="loading">
        <el-form-item label="配置文件 (JSON)">
          <el-input
            v-model="configJson"
            type="textarea"
            :rows="20"
            placeholder="配置 JSON..."
            @input="onInput"
          />
        </el-form-item>

        <!-- JSON 格式错误提示 -->
        <el-alert
          v-if="parseError"
          :title="parseError"
          type="error"
          show-icon
          :closable="false"
        />

        <el-divider />

        <el-form-item label="提交信息（可选）">
          <el-input
            v-model="commitMessage"
            placeholder="描述本次配置变更..."
            maxlength="100"
            show-word-limit
          />
        </el-form-item>

        <div class="actions">
          <el-button
            type="primary"
            size="large"
            @click="submitConfig"
            :loading="submitting"
            :disabled="!!parseError"
          >
            <el-icon><Check /></el-icon>
            应用配置
          </el-button>

          <el-button size="large" @click="resetConfig">重置</el-button>
          <el-button size="large" @click="formatJson">格式化</el-button>
        </div>
      </el-form>
    </el-card>

    <!-- 应用进度对话框 -->
    <el-dialog
      v-model="showProgress"
      title="正在应用配置"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      :show-close="false"
      width="400px"
    >
      <div class="progress-content">
        <el-steps :active="progressStep" finish-status="success">
          <el-step title="保存配置" />
          <el-step title="重启服务" />
          <el-step title="健康检查" />
        </el-steps>

        <div v-if="progressError" class="progress-error">
          <el-alert :title="progressError" type="error" show-icon />
        </div>
      </div>

      <template #footer>
        <el-button v-if="progressError" @click="showProgress = false"
          >关闭</el-button
        >
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Check } from '@element-plus/icons-vue'
import { getConfig, applyConfig } from '../api'

const router = useRouter()

// 状态
const configJson = ref('')
const originalJson = ref('')
const loading = ref(false)
const submitting = ref(false)
const commitMessage = ref('')
const parseError = ref('')
const showProgress = ref(false)
const progressStep = ref(0)
const progressError = ref('')

// 获取当前配置
const fetchConfig = async () => {
  loading.value = true
  try {
    const response = await getConfig()
    configJson.value = JSON.stringify(response.data, null, 2)
    originalJson.value = configJson.value
    parseError.value = ''
  } catch (error: any) {
    if (error.response?.status === 404) {
      // 配置文件不存在，使用空对象
      configJson.value = '{}'
      originalJson.value = '{}'
    } else {
      ElMessage.error('获取配置失败')
    }
  } finally {
    loading.value = false
  }
}

// 输入时验证 JSON
const onInput = () => {
  try {
    JSON.parse(configJson.value)
    parseError.value = ''
  } catch (e: any) {
    parseError.value = 'JSON 格式错误: ' + e.message
  }
}

// 格式化 JSON
const formatJson = () => {
  try {
    const parsed = JSON.parse(configJson.value)
    configJson.value = JSON.stringify(parsed, null, 2)
    parseError.value = ''
    ElMessage.success('格式化成功')
  } catch (e: any) {
    parseError.value = 'JSON 格式错误: ' + e.message
    ElMessage.error('格式化失败')
  }
}

// 提交配置
const submitConfig = async () => {
  // 验证 JSON
  let config: any
  try {
    config = JSON.parse(configJson.value)
  } catch (e: any) {
    parseError.value = 'JSON 格式错误: ' + e.message
    ElMessage.error(parseError.value)
    return
  }

  // 检查是否有变更
  if (configJson.value === originalJson.value) {
    ElMessage.warning('配置没有变更')
    return
  }

  showProgress.value = true
  progressStep.value = 0
  progressError.value = ''
  submitting.value = true

  try {
    // 步骤 1: 保存配置
    progressStep.value = 0
    await new Promise((resolve) => setTimeout(resolve, 500))

    // 步骤 2: 重启服务
    progressStep.value = 1
    await new Promise((resolve) => setTimeout(resolve, 500))

    // 调用 API
    const response = await applyConfig({
      config,
      message: commitMessage.value || undefined,
    })

    // 步骤 3: 健康检查
    progressStep.value = 2
    await new Promise((resolve) => setTimeout(resolve, 1000))

    if (response.data.success) {
      ElMessage.success('配置应用成功')
      originalJson.value = configJson.value
      setTimeout(() => {
        showProgress.value = false
        router.push('/status')
      }, 1000)
    } else {
      throw new Error(response.data.message)
    }
  } catch (error: any) {
    console.error('Failed to apply config:', error)
    progressError.value = error.response?.data?.error || error.message || '应用配置失败'
  } finally {
    submitting.value = false
  }
}

// 重置配置
const resetConfig = () => {
  configJson.value = originalJson.value
  commitMessage.value = ''
  parseError.value = ''
  ElMessage.info('配置已重置')
}

// 返回
const goBack = () => {
  router.push('/status')
}

// 生命周期
onMounted(() => {
  fetchConfig()
})
</script>

<style scoped>
.config-view {
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.actions {
  display: flex;
  justify-content: center;
  gap: 15px;
  margin-top: 20px;
}

.progress-content {
  padding: 20px;
}

.progress-error {
  margin-top: 20px;
}
</style>