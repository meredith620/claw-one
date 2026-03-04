<template>
  <div class="setup-wizard">
    <!-- 步骤条 -->
    <el-steps :active="currentStep" finish-status="success" class="steps-header">
      <el-step title="欢迎" />
      <el-step title="模型配置" />
      <el-step title="渠道配置" />
      <el-step title="完成" />
    </el-steps>

    <!-- 步骤 1: 欢迎 -->
    <div v-if="currentStep === 0" class="step-content">
      <el-result icon="success" title="欢迎使用 Claw One">
        <template #sub-title>
          <div class="welcome-text">
            <p>Claw One 让 OpenClaw 开箱即用</p>
            <p>只需几个简单步骤，即可完成初始化配置</p>
            <ul class="feature-list">
              <li>✅ 可视化配置模型 API</li>
              <li>✅ 配置消息渠道（Telegram/微信等）</li>
              <li>✅ 自动备份，配置错误可恢复</li>
            </ul>
          </div>
        </template>
        <template #extra>
          <el-button type="primary" size="large" @click="nextStep">开始配置</el-button>
        </template>
      </el-result>
    </div>

    <!-- 步骤 2: 模型配置 -->
    <div v-if="currentStep === 1" class="step-content">
      <h3>配置 AI 模型</h3>
      <p class="step-desc">添加至少一个 AI 模型提供商</p>

      <div v-for="(model, index) in setupData.models" :key="index" class="model-card">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>模型 #{{ index + 1 }}</span>
              <el-button type="danger" size="small" @click="removeModel(index)">删除</el-button>
            </div>
          </template>

          <el-form label-position="top">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="模型 ID">
                  <el-input v-model="model.id" placeholder="如: gpt-4" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="提供商">
                  <el-select v-model="model.provider" placeholder="选择提供商" style="width: 100%">
                    <el-option label="OpenAI" value="openai" />
                    <el-option label="Anthropic" value="anthropic" />
                    <el-option label="Kimi" value="kimi" />
                    <el-option label="DeepSeek" value="deepseek" />
                    <el-option label="其他" value="custom" />
                  </el-select>
                </el-form-item>
              </el-col>
            </el-row>

            <el-form-item label="API Key">
              <el-input
                v-model="model.apiKey"
                type="password"
                placeholder="输入 API Key"
                show-password
              />
            </el-form-item>

            <el-form-item label="Base URL (可选)">
              <el-input v-model="model.baseUrl" placeholder="如: https://api.openai.com" />
            </el-form-item>
          </el-form>
        </el-card>
      </div>

      <el-button type="primary" @click="addModel" class="add-btn">+ 添加模型</el-button>

      <div class="step-actions">
        <el-button @click="prevStep">上一步</el-button>
        <el-button type="primary" @click="nextStep" :disabled="!hasValidModels"
          >下一步</el-button
        >
      </div>
    </div>

    <!-- 步骤 3: 渠道配置 -->
    <div v-if="currentStep === 2" class="step-content">
      <h3>配置消息渠道</h3>
      <p class="step-desc">添加消息渠道，让 AI 可以通过聊天应用与你交互</p>

      <div v-for="(channel, index) in setupData.channels" :key="index" class="channel-card">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>渠道 #{{ index + 1 }}</span>
              <el-button type="danger" size="small" @click="removeChannel(index)">删除</el-button>
            </div>
          </template>

          <el-form label-position="top">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="渠道 ID">
                  <el-input v-model="channel.id" placeholder="如: my-telegram-bot" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="渠道类型">
                  <el-select v-model="channel.type" placeholder="选择类型" style="width: 100%">
                    <el-option label="Telegram" value="telegram" />
                    <el-option label="Mattermost" value="mattermost" />
                    <el-option label="Discord" value="discord" />
                    <el-option label="其他" value="custom" />
                  </el-select>
                </el-form-item>
              </el-col>
            </el-row>

            <el-form-item label="Token / Webhook">
              <el-input
                v-model="channel.token"
                type="password"
                placeholder="输入 Token 或 Webhook URL"
                show-password
              />
            </el-form-item>

            <el-form-item>
              <el-checkbox v-model="channel.enabled">启用此渠道</el-checkbox>
            </el-form-item>
          </el-form>
        </el-card>
      </div>

      <el-button type="primary" @click="addChannel" class="add-btn">+ 添加渠道</el-button>

      <!-- 跳过选项 -->
      <el-alert
        title="提示"
        type="info"
        :closable="false"
        class="skip-tip"
      >
        <p>暂时不需要？可以跳过，稍后可以在配置页面添加。</p>
        <el-button link @click="skipChannels">跳过此步骤</el-button>
      </el-alert>

      <div class="step-actions">
        <el-button @click="prevStep">上一步</el-button>
        <el-button type="primary" @click="nextStep">下一步</el-button>
      </div>
    </div>

    <!-- 步骤 4: 完成 -->
    <div v-if="currentStep === 3" class="step-content">
      <el-result icon="success" title="配置完成！">
        <template #sub-title>
          <div class="summary">
            <h4>配置摘要</h4>
            <p>已配置模型: {{ setupData.models.length }} 个</p>
            <p>已配置渠道: {{ setupData.channels.length }} 个</p>
          </div>
        </template>
        <template #extra>
          <el-button
            type="primary"
            size="large"
            @click="completeSetup"
            :loading="completing"
          >
            开始使用
          </el-button>
        </template>
      </el-result>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { checkFirstSetup, completeSetup as apiCompleteSetup } from '../api'

const router = useRouter()

// 当前步骤
const currentStep = ref(0)
const completing = ref(false)

// 初始化数据
const setupData = ref<{
  models: Array<{
    id: string
    provider: string
    apiKey: string
    baseUrl?: string
  }>
  channels: Array<{
    id: string
    type: string
    token: string
    enabled: boolean
  }>
}>({
  models: [],
  channels: [],
})

// 检查是否首次配置
onMounted(async () => {
  try {
    const response = await checkFirstSetup()
    if (!response.data.is_first_setup) {
      // 不是首次配置，跳转到状态页面
      router.push('/status')
    }
  } catch (error) {
    console.error('Failed to check setup status:', error)
  }
  
  // 添加默认空模型
  if (setupData.value.models.length === 0) {
    addModel()
  }
})

// 计算属性
const hasValidModels = computed(() => {
  return setupData.value.models.some(
    (m) => m.id.trim() && m.provider.trim() && m.apiKey.trim()
  )
})

// 添加模型
const addModel = () => {
  setupData.value.models.push({
    id: '',
    provider: '',
    apiKey: '',
    baseUrl: '',
  })
}

// 删除模型
const removeModel = (index: number) => {
  setupData.value.models.splice(index, 1)
  if (setupData.value.models.length === 0) {
    addModel()
  }
}

// 添加渠道
const addChannel = () => {
  setupData.value.channels.push({
    id: '',
    type: '',
    token: '',
    enabled: true,
  })
}

// 删除渠道
const removeChannel = (index: number) => {
  setupData.value.channels.splice(index, 1)
}

// 跳过渠道配置
const skipChannels = () => {
  currentStep.value = 3
}

// 上一步
const prevStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

// 下一步
const nextStep = () => {
  if (currentStep.value < 3) {
    currentStep.value++
  }
}

// 完成配置
const completeSetup = async () => {
  completing.value = true
  try {
    // 构建配置对象
    const config = buildConfig()
    
    // 保存配置
    // TODO: 调用 API 保存配置
    
    // 标记初始化完成
    await apiCompleteSetup()
    
    ElMessage.success('初始化完成！')
    router.push('/status')
  } catch (error) {
    ElMessage.error('保存配置失败')
  } finally {
    completing.value = false
  }
}

// 构建配置对象
const buildConfig = () => {
  // 转换为 openclaw.json 格式
  const models: Record<string, any> = {}
  setupData.value.models.forEach((model) => {
    models[model.id] = {
      provider: model.provider,
      apiKey: model.apiKey,
      baseUrl: model.baseUrl,
    }
  })

  return {
    version: '1.0',
    models,
    channels: setupData.value.channels,
  }
}
</script>

<style scoped>
.setup-wizard {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 20px;
}

.steps-header {
  margin-bottom: 40px;
}

.step-content {
  padding: 20px 0;
}

.welcome-text {
  text-align: left;
  max-width: 500px;
  margin: 0 auto;
}

.feature-list {
  margin-top: 20px;
  padding-left: 20px;
}

.feature-list li {
  margin: 10px 0;
}

.step-desc {
  color: #666;
  margin-bottom: 20px;
}

.model-card,
.channel-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.add-btn {
  margin-bottom: 20px;
}

.skip-tip {
  margin: 20px 0;
}

.step-actions {
  display: flex;
  justify-content: center;
  gap: 20px;
  margin-top: 30px;
}

.summary {
  text-align: center;
  padding: 20px;
}

.summary h4 {
  margin-bottom: 15px;
}

.summary p {
  margin: 5px 0;
  color: #666;
}
</style>