<template>
  <div class="config-view">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>配置向导</span>
          <el-button @click="goBack">返回</el-button>
        </div>
      </template>

      <!-- 配置表单 -->
      <el-form
        ref="formRef"
        :model="config"
        label-position="top"
        v-loading="loading"
      >
        <!-- Gateway 配置 -->
        <el-divider>Gateway 配置</el-divider>

        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="端口" prop="gateway.port">
              <el-input-number
                v-model="config.gateway.port"
                :min="1"
                :max="65535"
                style="width: 100%"
              />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="绑定地址" prop="gateway.bind">
              <el-input v-model="config.gateway.bind" placeholder="127.0.0.1" />
            </el-form-item>
          </el-col>
        </el-row>

        <!-- 模型配置 -->
        <el-divider>
          模型配置
          <el-button type="primary" size="small" @click="addModel">
            添加模型
          </el-button>
        </el-divider>

        <div v-for="(model, index) in config.models" :key="index" class="model-item">
          <el-card shadow="hover">
            <template #header>
              <div class="item-header">
                <span>模型 #{{ index + 1 }}</span>
                <el-button type="danger" size="small" @click="removeModel(index)">删除</el-button>
              </div>
            </template>

            <el-row :gutter="20">
              <el-col :span="8">
                <el-form-item :label="`ID`" :prop="`models.${index}.id`">
                  <el-input v-model="model.id" placeholder="如: gpt-4" />
                </el-form-item>
              </el-col>
              <el-col :span="8">
                <el-form-item :label="`提供商`" :prop="`models.${index}.provider`">
                  <el-input v-model="model.provider" placeholder="如: openai" />
                </el-form-item>
              </el-col>
              <el-col :span="8">
                <el-form-item :label="`API Key`" :prop="`models.${index}.apiKey`">
                  <el-input
                    v-model="model.apiKey"
                    type="password"
                    placeholder="输入 API Key"
                    show-password
                  />
                </el-form-item>
              </el-col>
            </el-row>
          </el-card>
        </div>

        <el-empty v-if="config.models.length === 0" description="暂无模型配置" />

        <!-- 渠道配置 -->
        <el-divider>
          渠道配置
          <el-button type="primary" size="small" @click="addChannel">
            添加渠道
          </el-button>
        </el-divider>

        <div v-for="(channel, index) in config.channels" :key="index" class="channel-item">
          <el-card shadow="hover">
            <template #header>
              <div class="item-header">
                <span>渠道 #{{ index + 1 }}</span>
                <el-button type="danger" size="small" @click="removeChannel(index)">删除</el-button>
              </div>
            </template>

            <el-row :gutter="20">
              <el-col :span="8">
                <el-form-item :label="`ID`" :prop="`channels.${index}.id`">
                  <el-input v-model="channel.id" placeholder="如: telegram-bot" />
                </el-form-item>
              </el-col>
              <el-col :span="8">
                <el-form-item :label="`类型`" :prop="`channels.${index}.type`">
                  <el-input v-model="channel.type" placeholder="如: telegram" />
                </el-form-item>
              </el-col>
              <el-col :span="8">
                <el-form-item :label="`启用`" :prop="`channels.${index}.enabled`">
                  <el-switch v-model="channel.enabled" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-card>
        </div>

        <el-empty v-if="config.channels.length === 0" description="暂无渠道配置" />

        <!-- 提交区域 -->
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
          <el-button type="primary" size="large" @click="submitConfig" :loading="submitting">
            <el-icon><Check /></el-icon>
            应用配置
          </el-button>

          <el-button size="large" @click="resetConfig">重置</el-button>
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
        <el-button v-if="progressError" @click="showProgress = false">关闭</el-button>
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
import type { Config } from '../types'

const router = useRouter()
const formRef = ref()

// 默认配置
const defaultConfig: Config = {
  version: '1.0',
  gateway: {
    port: 18790,
    bind: '127.0.0.1',
  },
  models: [],
  channels: [],
}

// 状态
const config = ref<Config>({ ...defaultConfig })
const originalConfig = ref<Config>({ ...defaultConfig })
const loading = ref(false)
const submitting = ref(false)
const commitMessage = ref('')
const showProgress = ref(false)
const progressStep = ref(0)
const progressError = ref('')

// 获取当前配置
const fetchConfig = async () => {
  loading.value = true
  try {
    const response = await getConfig()
    config.value = response.data
    originalConfig.value = JSON.parse(JSON.stringify(response.data))
  } catch (error) {
    // 如果配置文件不存在，使用默认配置
    console.log('Using default config')
  } finally {
    loading.value = false
  }
}

// 添加模型
const addModel = () => {
  config.value.models.push({
    id: '',
    provider: '',
    apiKey: '',
  })
}

// 删除模型
const removeModel = (index: number) => {
  config.value.models.splice(index, 1)
}

// 添加渠道
const addChannel = () => {
  config.value.channels.push({
    id: '',
    type: '',
    enabled: true,
  })
}

// 删除渠道
const removeChannel = (index: number) => {
  config.value.channels.splice(index, 1)
}

// 提交配置
const submitConfig = async () => {
  // 基本验证
  if (config.value.gateway.port < 1 || config.value.gateway.port > 65535) {
    ElMessage.error('端口必须在 1-65535 之间')
    return
  }

  // 检查是否有变更
  if (JSON.stringify(config.value) === JSON.stringify(originalConfig.value)) {
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
    await new Promise(resolve => setTimeout(resolve, 500)) // 视觉延迟

    // 步骤 2: 重启服务
    progressStep.value = 1
    await new Promise(resolve => setTimeout(resolve, 500))

    // 调用 API
    const response = await applyConfig({
      config: config.value,
      message: commitMessage.value || undefined,
    })

    // 步骤 3: 健康检查
    progressStep.value = 2
    await new Promise(resolve => setTimeout(resolve, 1000))

    if (response.data.success) {
      ElMessage.success('配置应用成功')
      originalConfig.value = JSON.parse(JSON.stringify(config.value))
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
  config.value = JSON.parse(JSON.stringify(originalConfig.value))
  commitMessage.value = ''
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

.model-item,
.channel-item {
  margin-bottom: 15px;
}

.item-header {
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