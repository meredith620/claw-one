<template>
  <div class="provider-module" v-loading="loading">
    <!-- 模型优先级设置 - 移到最上方 -->
    <div class="priority-section">
      <div class="section-header">
        <span class="section-title">🎯 模型优先级设置</span>
        <el-button type="primary" size="small" @click="savePriority" :loading="savingPriority">保存</el-button>
      </div>
      <p class="priority-desc">从已启用的 Provider 实例中选择模型，按优先级排序</p>

      <div class="priority-list">
        <div class="priority-item primary">
          <label>Primary (主模型):</label>
          <el-select v-model="modelPriority.primary" placeholder="选择主模型" class="priority-select">
            <el-option v-for="model in availableModels" :key="model.value" :label="model.label" :value="model.value" />
          </el-select>
        </div>

        <div v-for="(fallback, index) in modelPriority.fallbacks" :key="index" class="priority-item">
          <label>Fallback {{ index + 1 }}:</label>
          <el-select v-model="modelPriority.fallbacks[index]" placeholder="选择备用模型" class="priority-select" clearable>
            <el-option v-for="model in availableModels" :key="model.value" :label="model.label" :value="model.value" />
          </el-select>
          <el-button link type="danger" @click="removeFallback(index)">删除</el-button>
        </div>
        
        <el-button v-if="modelPriority.fallbacks.length < 3" link type="primary" @click="addFallback" class="add-fallback-btn">
          <el-icon><Plus /></el-icon>添加 Fallback
        </el-button>
      </div>
    </div>

    <!-- Provider 分类列表 -->
    <div v-for="type in providerTypes" :key="type.id" class="provider-section">
      <div class="section-header" :class="{ collapsed: isCollapsed(type.id) }" @click="toggleCollapse(type.id)">
        <div class="section-title">
          <el-icon class="collapse-icon">
            <ArrowDown v-if="!isCollapsed(type.id)" />
            <ArrowRight v-else />
          </el-icon>
          <span class="provider-icon">{{ type.icon }}</span>
          <span>{{ type.name }}</span>
          <el-tag :type="getStats(type.id).enabled > 0 ? 'success' : 'info'" size="small" class="stats-tag">
            {{ getStats(type.id).enabled }}/{{ getStats(type.id).total }}
          </el-tag>
        </div>
        <el-button type="primary" size="small" @click.stop="openAddDialog(type.id)">+ 添加实例</el-button>
      </div>

      <div v-show="!isCollapsed(type.id)">
        <div v-if="getInstances(type.id).length === 0" class="empty-state">
          <el-empty description="暂无实例" :image-size="60" />
        </div>

        <div class="instances-grid">
          <div v-for="instance in getInstances(type.id)" :key="instance.id" class="instance-card" :class="{ disabled: !instance.enabled }">
            <div class="instance-header">
              <span class="instance-id">{{ instance.id }}</span>
              <div class="instance-actions">
                <el-tag :type="instance.enabled ? 'success' : 'info'" size="small">{{ instance.enabled ? '已启用' : '未启用' }}</el-tag>
                <el-button link type="primary" @click="editInstance(instance)">配置</el-button>
                <el-button link type="danger" @click="deleteInstance(instance)">删除</el-button>
              </div>
            </div>
            <div class="instance-meta">
              <span v-if="instance.version">版本: {{ instance.version }}</span>
              <span v-if="instance.defaultModel">模型: {{ instance.defaultModel }}</span>
              <span v-if="instance.baseUrl" class="baseurl">URL: {{ instance.baseUrl }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="showAddDialog" :title="dialogTitle" width="550px">
      <el-form :model="formData" label-width="120px">
        <el-form-item v-if="currentType === 'custom'" label="Provider ID" required>
          <el-input v-model="formData.id" placeholder="如 redfrog、my-provider" :disabled="isEditing" />
          <div class="form-hint">自定义 Provider 的唯一标识</div>
        </el-form-item>
        <el-form-item v-else label="实例名称" required>
          <el-input v-model="formData.name" placeholder="如 work、personal" :disabled="isEditing" />
          <div v-if="!isEditing" class="form-hint">Provider ID: {{ currentType }}-{{ formData.name || 'xxx' }}</div>
        </el-form-item>
        <el-form-item v-if="currentType === 'moonshot'" label="地区">
          <el-radio-group v-model="formData.region">
            <el-radio-button label="global">国际版</el-radio-button>
            <el-radio-button label="cn">中国版</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="currentType === 'moonshot'" label="模型">
          <el-select v-model="formData.defaultModel" filterable allow-create style="width: 100%" placeholder="选择或输入模型名称">
            <el-option-group
              v-for="group in getModelOptionGroups('moonshot')"
              :key="group.label"
              :label="group.label"
            >
              <el-option
                v-for="model in group.options"
                :key="model.value"
                :label="model.label"
                :value="model.value"
              >
                <div class="model-option">
                  <span class="model-name">{{ model.label }}</span>
                  <span v-if="model.desc" class="model-desc">{{ model.desc }}</span>
                </div>
              </el-option>
            </el-option-group>
          </el-select>
          <div class="form-hint">可选择推荐模型或输入自定义模型名称</div>
        </el-form-item>
        <el-form-item v-if="currentType === 'minimax'" label="地区">
          <el-radio-group v-model="formData.region">
            <el-radio-button label="global">国际版</el-radio-button>
            <el-radio-button label="cn">中国版</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="currentType === 'minimax'" label="模型">
          <el-select v-model="formData.defaultModel" filterable allow-create style="width: 100%" placeholder="选择或输入模型名称">
            <el-option-group
              v-for="group in getModelOptionGroups('minimax')"
              :key="group.label"
              :label="group.label"
            >
              <el-option
                v-for="model in group.options"
                :key="model.value"
                :label="model.label"
                :value="model.value"
              >
                <div class="model-option">
                  <span class="model-name">{{ model.label }}</span>
                  <span v-if="model.desc" class="model-desc">{{ model.desc }}</span>
                </div>
              </el-option>
            </el-option-group>
          </el-select>
          <div class="form-hint">可选择推荐模型或输入自定义模型名称</div>
        </el-form-item>
        <el-form-item v-if="currentType === 'custom'" label="Base URL" required>
          <el-input v-model="formData.baseUrl" placeholder="如 https://api.example.com/v1" />
          <div class="form-hint">自定义 Provider 的 API 基础 URL</div>
        </el-form-item>
        
        <el-form-item label="API 协议" required>
          <el-select v-model="formData.api" placeholder="选择 API 接口协议" style="width: 100%">
            <el-option 
              v-for="api in apiTypeOptions" 
              :key="api.value" 
              :label="api.label" 
              :value="api.value" 
            />
          </el-select>
          <div class="form-hint">必须符合 OpenClaw 支持的 API 类型</div>
        </el-form-item>
        
        <!-- GitHub Copilot 使用 OAuth 登录 -->
        <el-form-item v-if="currentType === 'github-copilot'" label="GitHub 登录">
          <div v-if="!githubOAuthComplete">
            <el-button type="primary" @click="initGithubOAuth" :loading="githubOAuthIniting">
              {{ githubOAuthIniting ? '初始化中...' : '授权 GitHub Copilot' }}
            </el-button>
            <div v-if="githubDeviceCode" class="github-oauth-info">
              <p>1. 访问以下网址：<a :href="githubDeviceCode.verification_uri" target="_blank">{{ githubDeviceCode.verification_uri }}</a></p>
              <p>2. 输入验证码：<strong>{{ githubDeviceCode.user_code }}</strong></p>
              <p>3. 在 GitHub 页面上点击授权</p>
              <p>4. 授权完成后点击下方按钮完成验证</p>
              <el-button type="success" @click="checkGithubOAuthStatus" :loading="githubPolling" :disabled="githubPolling">
                {{ githubPolling ? '检查中...' : '完成授权' }}
              </el-button>
            </div>
            <div v-if="githubOAuthError" class="verify-result error">{{ githubOAuthError }}</div>
          </div>
          <div v-else class="verify-result success">
            ✅ GitHub Copilot 授权成功
          </div>
        </el-form-item>
        
        <!-- 其他 Provider 使用 API Key -->
        <el-form-item v-else label="API Key" required>
          <el-input v-model="formData.apiKey" type="password" placeholder="输入 API Key" show-password>
            <template #append>
              <el-button @click="verifyCredentials" :loading="verifying" :disabled="verifying">
                {{ verifying ? '验证中...' : '验证' }}
              </el-button>
            </template>
          </el-input>
          <div v-if="verifyStatus" class="verify-result" :class="verifyStatus.valid ? 'success' : 'error'">
            {{ verifyStatus.message }}
          </div>
        </el-form-item>
        <el-form-item label="默认模型" required>
          <el-select v-model="formData.defaultModel" filterable allow-create style="width: 100%" placeholder="选择或输入模型名称">
            <el-option-group 
              v-for="group in getModelOptionGroups(currentType)" 
              :key="group.label" 
              :label="group.label"
            >
              <el-option 
                v-for="model in group.options" 
                :key="model.value" 
                :label="model.label" 
                :value="model.value"
              >
                <div class="model-option">
                  <span class="model-name">{{ model.label }}</span>
                  <span v-if="model.desc" class="model-desc">{{ model.desc }}</span>
                </div>
              </el-option>
            </el-option-group>
          </el-select>
          <div class="form-hint">可选择推荐模型或输入自定义模型名称</div>
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="formData.enabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="saveInstance" :loading="saving">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, ArrowDown, ArrowRight } from '@element-plus/icons-vue'
import { getProviders, saveProvider, deleteProvider, getModelPriority, saveModelPriority, verifyProvider, githubCopilotInit, githubCopilotStatus } from '../api'
import { useConfigValidation } from '../composables/useConfigValidation'

const providerTypes = [
  { id: 'moonshot', name: 'Moonshot', icon: '🌙' },
  { id: 'openai', name: 'OpenAI', icon: '🤖' },
  { id: 'anthropic', name: 'Anthropic', icon: '🧠' },
  { id: 'minimax', name: 'MiniMax', icon: '🔵' },
  { id: 'github-copilot', name: 'GitHub Copilot', icon: '💻' },
  { id: 'custom', name: '其他 Provider', icon: '🔧' },
]

const loading = ref(false)
const saving = ref(false)
const savingPriority = ref(false)

// 验证状态
const verifying = ref(false)
const verifyStatus = ref<{ valid: boolean; message: string } | null>(null)
const verifyTimeout = ref<ReturnType<typeof setTimeout> | null>(null)

// GitHub Copilot OAuth 状态
const githubOAuthIniting = ref(false)
const githubOAuthComplete = ref(false)
const githubDeviceCode = ref<{ device_code: string; user_code: string; verification_uri: string; expires_in: number; interval: number } | null>(null)
const githubPolling = ref(false)
const githubOAuthError = ref<string | null>(null)

// 配置验证
const { validating, validateAndShow } = useConfigValidation()
const currentType = ref('moonshot')
const isEditing = ref(false)

const instances = reactive<Record<string, any[]>>({
  moonshot: [], openai: [], anthropic: [], minimax: [], 'github-copilot': [], custom: [],
})

const modelPriority = reactive({
  primary: '', fallbacks: [] as string[]
})

// 折叠状态管理
const collapsedSections = ref<Set<string>>(new Set())

const showAddDialog = ref(false)
const formData = reactive({
  id: '',
  name: '',
  region: 'global',
  apiKey: '',
  baseUrl: '',
  api: 'openai-chat',
  defaultModel: '',
  enabled: true,
})

const dialogTitle = computed(() => isEditing.value ? `配置 ${formData.id}` : `添加 ${providerTypes.find(t => t.id === currentType.value)?.name} 实例`)

const availableModels = computed(() => {
  const models: { value: string; label: string }[] = []
  Object.values(instances).flat().forEach((inst: any) => {
    if (inst.enabled && inst.defaultModel) {
      models.push({ value: `${inst.id}/${inst.defaultModel}`, label: `${inst.id} - ${inst.defaultModel}` })
    }
  })
  return models
})

// 获取实例统计
const getStats = (typeId: string) => {
  const insts = instances[typeId] || []
  const total = insts.length
  const enabled = insts.filter((i: any) => i.enabled).length
  return { total, enabled }
}

// 检查是否折叠
const isCollapsed = (typeId: string) => collapsedSections.value.has(typeId)

// 切换折叠状态
const toggleCollapse = (typeId: string) => {
  if (collapsedSections.value.has(typeId)) {
    collapsedSections.value.delete(typeId)
  } else {
    collapsedSections.value.add(typeId)
  }
}

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const [providersRes, priorityRes] = await Promise.all([getProviders(), getModelPriority()])
    
    const providers = providersRes.data || []
    
    // 清空现有实例
    instances.moonshot = []
    instances.openai = []
    instances.anthropic = []
    instances.minimax = []
    instances.custom = []
    
    // 分类 provider
    providers.forEach((p: any) => {
      const id = p.id || ''
      const baseUrl = p.baseUrl || ''
      
      // 判断 provider 类型
      // Moonshot: 包括 moonshot- 前缀、各种 kimi 变体
      const isMoonshot = id.startsWith('moonshot-') || 
                         id === 'kimi-coding' ||
                         baseUrl.includes('kimi.com') || 
                         baseUrl.includes('moonshot.cn') || 
                         baseUrl.includes('moonshot.ai')
      
      const isOpenAI = id.startsWith('openai-') || baseUrl.includes('openai.com')
      const isAnthropic = id.startsWith('anthropic-') || id.includes('claude') || baseUrl.includes('anthropic.com')
      const isMinimax = baseUrl.includes('minimax.io') || baseUrl.includes('minimaxi.com')
      
      if (isMoonshot) {
        // 为 kimi-coding 设置正确的 version
        if (id === 'kimi-coding' && !p.version) {
          p.version = 'coding'
        }
        instances.moonshot.push(p)
      } else if (isMinimax) {
        instances.minimax.push(p)
      } else if (isOpenAI) {
        instances.openai.push(p)
      } else if (isAnthropic) {
        instances.anthropic.push(p)
      } else {
        // 其他所有 provider 归为 custom
        instances.custom.push(p)
      }
    })
    
    // 自动折叠无启用实例的 provider
    providerTypes.forEach(type => {
      const stats = getStats(type.id)
      if (stats.enabled === 0) {
        collapsedSections.value.add(type.id)
      }
    })
    
    modelPriority.primary = priorityRes.data?.primary || ''
    modelPriority.fallbacks = priorityRes.data?.fallbacks?.length > 0 ? priorityRes.data.fallbacks : []
  } catch (error: any) {
    ElMessage.error('加载失败: ' + (error.response?.data?.error || error.message))
  } finally {
    loading.value = false
  }
}

const getInstances = (typeId: string) => instances[typeId] || []

const getModelOptions = (typeId: string) => {
  const opts: Record<string, { value: string; label: string }[]> = {
    moonshot: [
      { value: 'kimi-k2.5', label: 'Kimi K2.5' },
      { value: 'kimi-k2-thinking', label: 'Kimi K2 Thinking' },
      { value: 'kimi-k2-thinking-turbo', label: 'Kimi K2 Thinking Turbo' },
      { value: 'kimi-k2-turbo', label: 'Kimi K2 Turbo' },
    ],
    openai: [
      { value: 'gpt-4o', label: 'GPT-4o' },
      { value: 'gpt-4o-mini', label: 'GPT-4o Mini' },
    ],
    anthropic: [
      { value: 'claude-3-opus', label: 'Claude 3 Opus' },
      { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet' },
    ],
    minimax: [
      { value: 'MiniMax-M2.7', label: 'MiniMax M2.7' },
      { value: 'MiniMax-M2.7-highspeed', label: 'MiniMax M2.7 Highspeed' },
    ],
    'github-copilot': [
      { value: 'gpt-4o', label: 'GPT-4o' },
      { value: 'gpt-4o-mini', label: 'GPT-4o Mini' },
      { value: 'o1-mini', label: 'o1 Mini' },
      { value: 'o1-preview', label: 'o1 Preview' },
    ],
  }
  return opts[typeId] || []
}

// API 类型选项（必须符合 OpenClaw 支持的类型）
const apiTypeOptions = [
  { value: 'openai-responses', label: 'OpenAI Responses' },
  { value: 'openai-completions', label: 'OpenAI Completions' },
  { value: 'openai-codex-responses', label: 'OpenAI Codex' },
  { value: 'anthropic-messages', label: 'Anthropic Messages' },
  { value: 'google-generative-ai', label: 'Google Gemini' },
  { value: 'ollama', label: 'Ollama' },
]

// 分组模型选项，带描述信息
const getModelOptionGroups = (typeId: string) => {
  const groups: Record<string, { label: string; options: { value: string; label: string; desc?: string }[] }[]> = {
    moonshot: [
      {
        label: 'K2 系列（推荐）',
        options: [
          { value: 'kimi-k2.5', label: 'Kimi K2.5', desc: '通用对话，支持256K上下文' },
          { value: 'kimi-k2-thinking', label: 'Kimi K2 Thinking', desc: '深度思考，复杂推理' },
          { value: 'kimi-k2-thinking-turbo', label: 'Kimi K2 Thinking Turbo', desc: '快速深度思考' },
          { value: 'kimi-k2-turbo', label: 'Kimi K2 Turbo', desc: '快速响应，256K上下文' },
        ]
      }
    ],
    openai: [
      {
        label: '推荐模型',
        options: [
          { value: 'gpt-4o', label: 'GPT-4o', desc: '最新多模态模型，性价比高' },
          { value: 'gpt-4o-mini', label: 'GPT-4o Mini', desc: '轻量快速，成本低' },
        ]
      },
      {
        label: '其他模型',
        options: [
          { value: 'gpt-4-turbo', label: 'GPT-4 Turbo', desc: '高能力模型' },
          { value: 'gpt-3.5-turbo', label: 'GPT-3.5 Turbo', desc: '经济实用' },
        ]
      }
    ],
    anthropic: [
      {
        label: '推荐模型',
        options: [
          { value: 'claude-3-opus', label: 'Claude 3 Opus', desc: '最强能力，复杂任务' },
          { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet', desc: '平衡性能与速度' },
        ]
      },
      {
        label: '其他模型',
        options: [
          { value: 'claude-3-haiku', label: 'Claude 3 Haiku', desc: '最快响应' },
          { value: 'claude-2.1', label: 'Claude 2.1', desc: '200K上下文' },
        ]
      }
    ],
    minimax: [
      {
        label: 'M2.7 系列（推荐）',
        options: [
          { value: 'MiniMax-M2.7', label: 'MiniMax M2.7', desc: '通用推理，支持图像理解，204.8K上下文' },
          { value: 'MiniMax-M2.7-highspeed', label: 'MiniMax M2.7 Highspeed', desc: '快速推理，文本专用' },
        ]
      }
    ],
    'github-copilot': [
      {
        label: '推荐模型',
        options: [
          { value: 'gpt-4o', label: 'GPT-4o', desc: '最新多模态模型，性价比高' },
          { value: 'gpt-4o-mini', label: 'GPT-4o Mini', desc: '轻量快速，成本低' },
        ]
      },
      {
        label: 'o 系列',
        options: [
          { value: 'o1-mini', label: 'o1 Mini', desc: '快速推理模型' },
          { value: 'o1-preview', label: 'o1 Preview', desc: '深度推理预览版' },
        ]
      }
    ],
  }
  return groups[typeId] || [{ label: '自定义', options: [] }]
}

const openAddDialog = (typeId: string) => {
  currentType.value = typeId
  isEditing.value = false
  formData.id = ''
  formData.name = ''
  formData.apiKey = ''
  formData.defaultModel = ''
  formData.enabled = true
  formData.region = 'global'
  // 修复: 使用 OpenClaw 支持的 API 类型
  formData.api = typeId === 'anthropic' ? 'anthropic-messages' : typeId === 'moonshot' ? 'openai-completions' : typeId === 'minimax' ? 'anthropic-messages' : 'openai-responses'
  formData.baseUrl = ''
  showAddDialog.value = true
}

const editInstance = (instance: any) => {
  // 根据 ID 前缀、名称或 baseUrl 推断类型
  const id = instance.id || ''
  const baseUrl = instance.baseUrl || ''
  
  // Moonshot: 包括 moonshot- 前缀、kimi-coding、各种 kimi 变体
  const isMoonshot = id.startsWith('moonshot-') || 
                     id === 'kimi-coding' ||
                     baseUrl.includes('kimi.com') || 
                     baseUrl.includes('moonshot.cn') || 
                     baseUrl.includes('moonshot.ai')
  
  const isOpenAI = id.startsWith('openai-') || baseUrl.includes('openai.com')
  const isAnthropic = id.startsWith('anthropic-') || id.includes('claude') || baseUrl.includes('anthropic.com')
  const isMinimax = baseUrl.includes('minimax.io') || baseUrl.includes('minimaxi.com')
  
  if (isMoonshot) {
    currentType.value = 'moonshot'
  } else if (isMinimax) {
    currentType.value = 'minimax'
  } else if (isOpenAI) {
    currentType.value = 'openai'
  } else if (isAnthropic) {
    currentType.value = 'anthropic'
  } else {
    currentType.value = 'custom'
  }
  
  isEditing.value = true
  formData.id = instance.id
  formData.name = instance.name || instance.id.split('-')[1] || instance.id
  formData.apiKey = instance.apiKey || ''
  formData.defaultModel = instance.defaultModel || ''
  formData.enabled = instance.enabled !== false
  formData.baseUrl = instance.baseUrl || ''
  formData.api = instance.api || (currentType.value === 'anthropic' ? 'anthropic-messages' : currentType.value === 'moonshot' ? 'openai-completions' : currentType.value === 'minimax' ? 'anthropic-messages' : 'openai-responses')
  // region 检测
  if (currentType.value === 'moonshot') {
    if (baseUrl.includes('moonshot.cn')) {
      formData.region = 'cn'
    } else {
      formData.region = 'global'
    }
  } else if (currentType.value === 'minimax') {
    if (baseUrl.includes('minimaxi.com')) {
      formData.region = 'cn'
    } else {
      formData.region = 'global'
    }
  } else {
    formData.region = 'global'
  }
  showAddDialog.value = true
}

const saveInstance = async () => {
  // 对于非 custom 类型，需要验证 name；对于 custom 类型，直接验证 id
  if (currentType.value !== 'custom' && !formData.name && !isEditing.value) { 
    ElMessage.error('请输入实例名称'); return 
  }
  if (currentType.value === 'custom' && !formData.id && !isEditing.value) {
    ElMessage.error('请输入 Provider ID'); return
  }
  if (currentType.value === 'custom' && !formData.api && !isEditing.value) {
    ElMessage.error('请选择 API 协议'); return
  }
  // MiniMax API Key 格式校验已移除（MiniMax 实际不使用 JWT 格式）
  if (!formData.apiKey) { ElMessage.error('请输入 API Key'); return }
  if (!formData.defaultModel) { ElMessage.error('请选择默认模型'); return }

  // 构建 ID
  let id: string
  if (isEditing.value) {
    id = formData.id
  } else if (currentType.value === 'custom') {
    id = formData.id
  } else {
    id = `${currentType.value}-${formData.name}`
  }
  
  saving.value = true
  
  try {
    // 对于 custom 类型，使用表单中输入的 baseUrl；对于标准类型，使用预定义 baseUrl
    let baseUrl: string
    if (currentType.value === 'custom') {
      baseUrl = formData.baseUrl || ''
    } else {
      const baseUrls: Record<string, Record<string, string>> = {
        moonshot: { global: 'https://api.moonshot.ai/v1', cn: 'https://api.moonshot.cn/v1' },
        openai: { default: 'https://api.openai.com/v1' },
        anthropic: { default: 'https://api.anthropic.com/v1' },
        minimax: { global: 'https://api.minimax.io/anthropic', cn: 'https://api.minimaxi.com/anthropic' },
      }
      baseUrl = baseUrls[currentType.value]?.[formData.region] || baseUrls[currentType.value]?.default || ''
    }

    const data = {
      id, 
      name: currentType.value === 'custom' ? formData.name || id : formData.name,
      region: (currentType.value === 'moonshot' || currentType.value === 'minimax') ? formData.region : undefined,
      enabled: formData.enabled,
      apiKey: formData.apiKey,
      baseUrl: baseUrl,
      defaultModel: formData.defaultModel,
      api: currentType.value === 'moonshot' ? 'openai-completions' : (currentType.value === 'minimax' ? 'anthropic-messages' : (formData.api || 'openai-chat')),
    }

    // 构建完整配置进行验证
    const allProviders: Record<string, any> = {}
    Object.values(instances).flat().forEach((inst: any) => {
      allProviders[inst.id] = inst
    })
    allProviders[id] = data

    const configToValidate = {
      models: { providers: allProviders },
      agents: { defaults: { workspace: '~/.openclaw/workspace', agentDir: '~/.openclaw/agent' } },
      channels: {}
    }

    // 验证配置
    const isValid = await validateAndShow(configToValidate)
    if (!isValid) {
      saving.value = false
      return
    }

    await saveProvider(id, data)
    ElMessage.success(isEditing.value ? '配置已保存' : `实例 "${id}" 添加成功`)
    showAddDialog.value = false
    
    // 添加实例后自动展开
    collapsedSections.value.delete(currentType.value)
    
    await loadData()
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
  } finally {
    saving.value = false
  }
}

const deleteInstance = async (instance: any) => {
  try {
    await ElMessageBox.confirm(`确定删除实例 "${instance.id}" 吗？`, '确认删除', { type: 'warning' })
    await deleteProvider(instance.id)
    ElMessage.success('实例已删除')
    await loadData()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败: ' + (error.response?.data?.error || error.message))
    }
  }
}

const savePriority = async () => {
  savingPriority.value = true
  try {
    await saveModelPriority({
      primary: modelPriority.primary,
      fallbacks: modelPriority.fallbacks.filter(f => f && f !== '')
    })
    ElMessage.success('模型优先级已保存')
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
  } finally {
    savingPriority.value = false
  }
}

const addFallback = () => { 
  if (modelPriority.fallbacks.length < 3) modelPriority.fallbacks.push('') 
}
const removeFallback = (index: number) => { 
  modelPriority.fallbacks.splice(index, 1) 
}

// API Key 格式校验
const validateApiKeyFormat = (apiKey: string, apiType: string): { valid: boolean; message: string } => {
  if (!apiKey || apiKey.trim().length === 0) {
    return { valid: false, message: '请输入 API Key' }
  }

  // MiniMax API Key 格式校验已移除（MiniMax 实际不使用 JWT 格式，eyJ 前缀要求不正确）

  // OpenAI: 应以 sk- 开头
  if ((apiType === 'openai-chat' || apiType === 'openai-completions' || apiType === 'openai-responses' || apiType === 'openai-codex-responses') && !apiKey.startsWith('sk-')) {
    return { valid: false, message: 'OpenAI API Key 应以 sk- 开头' }
  }

  return { valid: true, message: '' }
}

// 验证凭证
const verifyCredentials = async () => {
  // 1. 检查 API Key 是否存在
  if (!formData.apiKey) {
    verifyStatus.value = { valid: false, message: '请输入 API Key' }
    return
  }

  // 2. 格式校验
  const formatCheck = validateApiKeyFormat(formData.apiKey, formData.api)
  if (!formatCheck.valid) {
    verifyStatus.value = { valid: false, message: formatCheck.message }
    return
  }

  // 3. 设置 verifying 状态
  verifying.value = true
  verifyStatus.value = null

  // 4. 设置超时
  const timeoutId = setTimeout(() => {
    verifying.value = false
    verifyStatus.value = { valid: false, message: '验证超时，请检查网络连接' }
    ElMessage.error('验证请求超时')
  }, 15000)
  verifyTimeout.value = timeoutId as any

  try {
    // 构建 baseUrl
    let baseUrl: string
    if (formData.baseUrl) {
      baseUrl = formData.baseUrl
    } else {
      const baseUrls: Record<string, Record<string, string>> = {
        moonshot: { global: 'https://api.moonshot.ai/v1', cn: 'https://api.moonshot.cn/v1' },
        openai: { default: 'https://api.openai.com/v1' },
        anthropic: { default: 'https://api.anthropic.com/v1' },
        minimax: { global: 'https://api.minimax.io/anthropic', cn: 'https://api.minimaxi.com/anthropic' },
        'github-copilot': { default: 'https://api.individual.githubcopilot.com' },
        custom: { default: '' },
      }
      baseUrl = baseUrls[currentType.value]?.[formData.region] || baseUrls[currentType.value]?.default || ''
    }

    const response = await verifyProvider({
      apiKey: formData.apiKey,
      baseUrl: baseUrl,
      api: formData.api,
    })

    // 清除超时
    if (verifyTimeout.value) {
      clearTimeout(verifyTimeout.value)
      verifyTimeout.value = null
    }

    if (response.data.success) {
      verifyStatus.value = {
        valid: response.data.valid,
        message: response.data.message || (response.data.valid ? '验证通过' : '验证失败')
      }
      if (!response.data.valid) {
        ElMessage.error('API Key 无效，请检查')
      }
    } else {
      verifyStatus.value = {
        valid: false,
        message: response.data.error || '验证失败'
      }
      ElMessage.error(response.data.error || '验证请求失败')
    }
  } catch (e: any) {
    if (verifyTimeout.value) {
      clearTimeout(verifyTimeout.value)
      verifyTimeout.value = null
    }
    verifyStatus.value = { valid: false, message: '网络错误，请检查连接' }
    ElMessage.error('验证请求失败')
  } finally {
    verifying.value = false
  }
}

// GitHub Copilot OAuth 初始化
const initGithubOAuth = async () => {
  githubOAuthIniting.value = true
  githubOAuthError.value = null
  githubDeviceCode.value = null
  
  try {
    const response = await githubCopilotInit()
    if (response.data.success) {
      githubDeviceCode.value = {
        device_code: response.data.device_code,
        user_code: response.data.user_code,
        verification_uri: response.data.verification_uri,
        expires_in: response.data.expires_in,
        interval: response.data.interval,
      }
      ElMessage.info('请在浏览器中完成 GitHub 授权')
    } else {
      githubOAuthError.value = response.data.error || '初始化失败'
      ElMessage.error(githubOAuthError.value)
    }
  } catch (e: any) {
    githubOAuthError.value = e.message || '初始化失败'
    ElMessage.error(githubOAuthError.value)
  } finally {
    githubOAuthIniting.value = false
  }
}

// 检查 GitHub OAuth 状态
const checkGithubOAuthStatus = async () => {
  githubPolling.value = true
  githubOAuthError.value = null
  
  try {
    const response = await githubCopilotStatus()
    if (response.data.pending) {
      // 仍在等待授权
      githubOAuthError.value = '尚未完成授权，请确保已在 GitHub 页面点击授权'
      ElMessage.warning(githubOAuthError.value)
    } else if (!response.data.pending && response.data.access_token) {
      // 授权成功
      githubOAuthComplete.value = true
      formData.apiKey = response.data.access_token
      formData.baseUrl = 'https://api.individual.githubcopilot.com'
      formData.api = 'openai-chat'
      verifyStatus.value = { valid: true, message: 'GitHub Copilot 授权成功' }
      ElMessage.success('GitHub Copilot 授权成功！')
    } else if (response.data.error) {
      githubOAuthError.value = response.data.error_description || response.data.error
      ElMessage.error(githubOAuthError.value)
    }
  } catch (e: any) {
    githubOAuthError.value = e.message || '检查状态失败'
    ElMessage.error(githubOAuthError.value)
  } finally {
    githubPolling.value = false
  }
}

// region 切换时重置 verifyStatus
watch(() => formData.region, () => {
  verifyStatus.value = null
})

// GitHub Copilot 类型切换时重置 OAuth 状态
watch(() => currentType.value, (newType) => {
  if (newType !== 'github-copilot') {
    githubOAuthComplete.value = false
    githubDeviceCode.value = null
    githubOAuthError.value = null
  }
})

onMounted(loadData)
</script>

<style scoped>
.provider-module {
  max-width: 900px;
}

.provider-section, .priority-section {
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  cursor: pointer;
  user-select: none;
}

.section-header.collapsed {
  margin-bottom: 0;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
}

.collapse-icon {
  font-size: 14px;
  color: #909399;
  transition: transform 0.2s;
}

.stats-tag {
  margin-left: 4px;
}

.provider-icon {
  font-size: 24px;
}

.empty-state {
  padding: 0;
  min-height: auto;
}

.empty-state :deep(.el-empty) {
  padding: 4px 0;
  margin: 0;
}

.empty-state :deep(.el-empty__image) {
  width: 16px;
  height: 16px;
  margin-bottom: 2px;
}

.empty-state :deep(.el-empty__description) {
  margin-top: 0;
  font-size: 11px;
}

.instances-grid {
  display: grid;
  gap: 12px;
}

.instance-card {
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 16px;
}

.instance-card.disabled {
  opacity: 0.6;
  background: #f5f7fa;
}

.instance-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.instance-id {
  font-weight: 600;
  font-size: 15px;
}

.instance-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.instance-meta {
  display: flex;
  gap: 16px;
  color: #606266;
  font-size: 13px;
}

.instance-meta .baseurl {
  color: #909399;
  font-size: 12px;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.priority-desc {
  color: #606266;
  font-size: 13px;
  margin: -10px 0 16px;
}

.priority-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.priority-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.priority-item label {
  width: 120px;
  flex-shrink: 0;
}

.priority-select {
  width: 300px;
}

.add-fallback-btn {
  margin-top: 8px;
}

.form-hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

/* 模型选项样式 */
.model-option {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 4px 0;
}

.model-name {
  font-weight: 500;
  font-size: 14px;
}

.model-desc {
  font-size: 12px;
  color: #909399;
}

.verify-result {
  margin-top: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
}

.verify-result.success {
  background: #f0f9ff;
  color: #067647;
  border: 1px solid #b7eb8f;
}

.verify-result.error {
  background: #fff2f0;
  color: #cf1322;
  border: 1px solid #ffccc7;
}

.github-oauth-info {
  margin-top: 12px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 13px;
  line-height: 1.8;
}

.github-oauth-info p {
  margin: 4px 0;
}

.github-oauth-info a {
  color: #409eff;
  word-break: break-all;
}
</style>