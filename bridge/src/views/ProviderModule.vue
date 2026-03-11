<template>
  <div class="provider-module" v-loading="loading">
    <div v-for="type in providerTypes" :key="type.id" class="provider-section">
      <div class="section-header">
        <div class="section-title">
          <span class="provider-icon">{{ type.icon }}</span>
          <span>{{ type.name }}</span>
        </div>
        <el-button type="primary" size="small" @click="openAddDialog(type.id)">+ 添加实例</el-button>
      </div>

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
          </div>
        </div>
      </div>
    </div>

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
        <el-form-item v-if="currentType === 'moonshot'" label="版本">
          <el-radio-group v-model="formData.version">
            <el-radio-button label="coding">Coding</el-radio-button>
            <el-radio-button label="ai">.ai</el-radio-button>
            <el-radio-button label="cn">.cn</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="currentType === 'custom'" label="Base URL" required>
          <el-input v-model="formData.baseUrl" placeholder="如 https://api.example.com/v1" />
          <div class="form-hint">自定义 Provider 的 API 基础 URL</div>
        </el-form-item>
        
        <el-form-item label="API Key" required>
          <el-input v-model="formData.apiKey" type="password" placeholder="输入 API Key" show-password />
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
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import { getProviders, saveProvider, deleteProvider, getModelPriority, saveModelPriority } from '../api'
import { useConfigValidation } from '../composables/useConfigValidation'

const providerTypes = [
  { id: 'moonshot', name: 'Moonshot', icon: '🌙' },
  { id: 'openai', name: 'OpenAI', icon: '🤖' },
  { id: 'anthropic', name: 'Anthropic', icon: '🧠' },
  { id: 'custom', name: '其他 Provider', icon: '🔧' },
]

const loading = ref(false)
const saving = ref(false)
const savingPriority = ref(false)

// 配置验证
const { validating, validateAndShow } = useConfigValidation()
const currentType = ref('moonshot')
const isEditing = ref(false)

const instances = reactive<Record<string, any[]>>({
  moonshot: [], openai: [], anthropic: [], custom: [],
})

const modelPriority = reactive({
  primary: '', fallbacks: [] as string[]
})

const showAddDialog = ref(false)
const formData = reactive({
  id: '',
  name: '',
  version: 'coding',
  apiKey: '',
  baseUrl: '',
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
    instances.custom = []
    
    // 分类 provider
    providers.forEach((p: any) => {
      const id = p.id || ''
      const baseUrl = p.baseUrl || ''
      
      // 判断 provider 类型
      const isMoonshot = id.startsWith('moonshot-') || 
                         id.includes('kimi') || 
                         baseUrl.includes('kimi.com') || 
                         baseUrl.includes('moonshot.cn') || 
                         baseUrl.includes('moonshot.ai')
      
      const isOpenAI = id.startsWith('openai-') || baseUrl.includes('openai.com')
      const isAnthropic = id.startsWith('anthropic-') || id.includes('claude') || baseUrl.includes('anthropic.com')
      
      if (isMoonshot) {
        instances.moonshot.push(p)
      } else if (isOpenAI) {
        instances.openai.push(p)
      } else if (isAnthropic) {
        instances.anthropic.push(p)
      } else {
        // 其他所有 provider 归为 custom
        instances.custom.push(p)
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
    ],
    openai: [
      { value: 'gpt-4o', label: 'GPT-4o' },
      { value: 'gpt-4o-mini', label: 'GPT-4o Mini' },
    ],
    anthropic: [
      { value: 'claude-3-opus', label: 'Claude 3 Opus' },
      { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet' },
    ],
  }
  return opts[typeId] || []
}

// 分组模型选项，带描述信息
const getModelOptionGroups = (typeId: string) => {
  const groups: Record<string, { label: string; options: { value: string; label: string; desc?: string }[] }[]> = {
    moonshot: [
      {
        label: '推荐模型',
        options: [
          { value: 'kimi-k2.5', label: 'Kimi K2.5', desc: '通用对话，支持128K上下文' },
          { value: 'kimi-k2-thinking', label: 'Kimi K2 Thinking', desc: '深度思考，适合复杂推理' },
        ]
      },
      {
        label: '其他模型',
        options: [
          { value: 'moonshot-v1-8k', label: 'Moonshot v1 8K', desc: '基础模型' },
          { value: 'moonshot-v1-32k', label: 'Moonshot v1 32K', desc: '长上下文' },
          { value: 'moonshot-v1-128k', label: 'Moonshot v1 128K', desc: '超长上下文' },
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
  formData.version = 'coding'
  showAddDialog.value = true
}

const editInstance = (instance: any) => {
  // 根据 ID 前缀、名称或 baseUrl 推断类型
  const id = instance.id || ''
  const baseUrl = instance.baseUrl || ''
  
  const isMoonshot = id.startsWith('moonshot-') || 
                     id.includes('kimi') || 
                     baseUrl.includes('kimi.com') || 
                     baseUrl.includes('moonshot.cn') || 
                     baseUrl.includes('moonshot.ai')
  
  const isOpenAI = id.startsWith('openai-') || baseUrl.includes('openai.com')
  const isAnthropic = id.startsWith('anthropic-') || id.includes('claude') || baseUrl.includes('anthropic.com')
  
  if (isMoonshot) {
    currentType.value = 'moonshot'
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
  formData.version = instance.version || 'coding'
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
        moonshot: { coding: 'https://api.kimi.com/coding/', ai: 'https://api.moonshot.ai/v1', cn: 'https://api.moonshot.cn/v1' },
        openai: { default: 'https://api.openai.com/v1' },
        anthropic: { default: 'https://api.anthropic.com/v1' },
      }
      baseUrl = baseUrls[currentType.value]?.[formData.version] || baseUrls[currentType.value]?.default || ''
    }

    const data = {
      id, 
      name: currentType.value === 'custom' ? formData.name || id : formData.name,
      version: currentType.value === 'moonshot' ? formData.version : undefined,
      enabled: formData.enabled,
      apiKey: formData.apiKey,
      baseUrl: baseUrl,
      defaultModel: formData.defaultModel,
      api: currentType.value === 'anthropic' ? 'anthropic-messages' : 'openai-completions',
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
}

.section-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
}

.provider-icon {
  font-size: 24px;
}

.empty-state {
  padding: 20px 0;
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
</style>
