<template>
  <div class="provider-config">
    <div class="provider-sidebar">
      <h3>🧠 Provider 配置</h3>
      <div class="provider-types">
        <div 
          v-for="type in providerTypes" 
          :key="type.id"
          class="provider-type"
          :class="{ active: selectedType === type.id }"
          @click="selectType(type.id)"
        >
          <span class="icon">{{ type.icon }}</span>
          <span class="name">{{ type.name }}</span>
          <span class="count">({{ getInstanceCount(type.id) }})</span>
        </div>
      </div>
    </div>

    <div class="provider-content">
      <div v-if="selectedType" class="instance-section">
        <div class="section-header">
          <h4>{{ getTypeName(selectedType) }} 实例</h4>
          <el-button type="primary" size="small" @click="showAddDialog = true">
            <el-icon><Plus /></el-icon>
            添加实例
          </el-button>
        </div>

        <div v-if="getInstances(selectedType).length === 0" class="empty-state">
          <el-empty description="暂无实例，点击上方按钮添加" />
        </div>

        <div 
          v-for="instance in getInstances(selectedType)" 
          :key="instance.id"
          class="instance-card"
          :class="{ disabled: !instance.enabled }"
        >
          <div class="instance-header">
            <div class="instance-info">
              <span class="instance-id">{{ instance.id }}</span>
              <el-tag :type="instance.enabled ? 'success' : 'info'" size="small">
                {{ instance.enabled ? '已启用' : '未启用' }}
              </el-tag>
            </div>
            <div class="instance-actions">
              <el-button link type="primary" @click="editInstance(instance)">配置</el-button>
              <el-button link type="danger" @click="deleteInstance(instance)">删除</el-button>
            </div>
          </div>
          <div class="instance-details">
            <span v-if="instance.version">版本: {{ instance.version }}</span>
            <span v-if="instance.defaultModel">模型: {{ instance.defaultModel }}</span>
          </div>
        </div>
      </div>

      <div class="model-priority">
        <h4>模型优先级设置</h4>
        <div class="priority-list">
          <div class="priority-item">
            <label>Primary (主模型):</label>
            <el-select v-model="modelPriority.primary" placeholder="选择主模型" class="priority-select">
              <el-option 
                v-for="model in availableModels" 
                :key="model.value"
                :label="model.label"
                :value="model.value"
              />
            </el-select>
          </div>
          <div 
            v-for="(fallback, index) in modelPriority.fallbacks" 
            :key="index"
            class="priority-item"
          >
            <label>Fallback {{ index + 1 }}:</label>
            <el-select v-model="modelPriority.fallbacks[index]" placeholder="选择备用模型" class="priority-select" clearable>
              <el-option 
                v-for="model in availableModels" 
                :key="model.value"
                :label="model.label"
                :value="model.value"
              />
            </el-select>
            <el-button link type="danger" @click="removeFallback(index)">删除</el-button>
          </div>
          <el-button v-if="modelPriority.fallbacks.length < 3" link type="primary" @click="addFallback">
            <el-icon><Plus /></el-icon>添加 Fallback
          </el-button>
        </div>
      </div>
    </div>

    <!-- 添加实例对话框 -->
    <el-dialog v-model="showAddDialog" :title="`添加 ${getTypeName(selectedType)} 实例`" width="500px">
      <el-form :model="newInstance" label-width="100px">
        <el-form-item label="实例名称" required>
          <el-input v-model="newInstance.name" placeholder="如 work、personal" />
          <div class="form-hint">将生成 Provider ID: {{ selectedType }}-{{ newInstance.name || 'xxx' }}</div>
        </el-form-item>

        <el-form-item v-if="selectedType === 'moonshot'" label="版本">
          <el-radio-group v-model="newInstance.version">
            <el-radio label="coding">Kimi Coding (默认)</el-radio>
            <el-radio label="ai">Kimi API (.ai)</el-radio>
            <el-radio label="cn">Kimi API (.cn)</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item label="API Key" required>
          <el-input v-model="newInstance.apiKey" type="password" placeholder="输入 API Key" show-password />
        </el-form-item>

        <el-form-item label="默认模型" required>
          <el-select v-model="newInstance.defaultModel" filterable allow-create>
            <el-option 
              v-for="model in getModelOptions(selectedType)" 
              :key="model.value"
              :label="model.label"
              :value="model.value"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="addInstance">保存</el-button>
      </template>
    </el-dialog>

    <!-- 编辑实例对话框 -->
    <el-dialog v-model="showEditDialog" :title="`配置 ${editingInstance?.id}`" width="500px">
      <el-form v-if="editingInstance" :model="editingInstance" label-width="100px">
        <el-form-item label="启用">
          <el-switch v-model="editingInstance.enabled" />
        </el-form-item>
        <el-form-item label="API Key">
          <el-input v-model="editingInstance.apiKey" type="password" placeholder="输入 API Key" show-password />
        </el-form-item>
        <el-form-item label="默认模型">
          <el-select v-model="editingInstance.defaultModel" filterable allow-create>
            <el-option 
              v-for="model in getModelOptions(selectedType)" 
              :key="model.value"
              :label="model.label"
              :value="model.value"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showEditDialog = false">取消</el-button>
        <el-button type="primary" @click="saveInstance">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

const providerTypes = [
  { id: 'moonshot', name: 'Moonshot', icon: '🌙' },
  { id: 'openai', name: 'OpenAI', icon: '🤖' },
  { id: 'anthropic', name: 'Anthropic', icon: '🧠' },
]

const selectedType = ref('moonshot')
const showAddDialog = ref(false)
const showEditDialog = ref(false)

const instances = reactive<Record<string, any[]>>({
  moonshot: [
    { 
      id: 'moonshot-work', 
      name: 'work',
      version: 'ai',
      enabled: true, 
      apiKey: 'sk-xxx',
      baseUrl: 'https://api.moonshot.ai/v1',
      defaultModel: 'kimi-k2.5'
    },
  ],
  openai: [],
  anthropic: [],
})

const modelPriority = reactive({
  primary: 'moonshot-work/kimi-k2.5',
  fallbacks: ['']
})

const newInstance = reactive({
  name: '',
  version: 'coding',
  apiKey: '',
  defaultModel: ''
})

const editingInstance = ref<any>(null)

const availableModels = computed(() => {
  const models: { value: string; label: string }[] = []
  Object.entries(instances).forEach(([type, typeInstances]) => {
    typeInstances.forEach(inst => {
      if (inst.enabled) {
        models.push({
          value: `${inst.id}/${inst.defaultModel}`,
          label: `${inst.id} - ${inst.defaultModel}`
        })
      }
    })
  })
  return models
})

const getInstanceCount = (typeId: string) => instances[typeId]?.length || 0
const getInstances = (typeId: string) => instances[typeId] || []
const getTypeName = (typeId: string) => providerTypes.find(t => t.id === typeId)?.name || typeId
const selectType = (typeId: string) => { selectedType.value = typeId }

const getModelOptions = (typeId: string) => {
  const options: Record<string, { value: string; label: string }[]> = {
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
  return options[typeId] || []
}

const addInstance = () => {
  if (!newInstance.name) { ElMessage.error('请输入实例名称'); return }
  if (!newInstance.apiKey) { ElMessage.error('请输入 API Key'); return }
  if (!newInstance.defaultModel) { ElMessage.error('请选择默认模型'); return }

  const id = `${selectedType.value}-${newInstance.name}`
  const exists = Object.values(instances).flat().some(i => i.id === id)
  if (exists) { ElMessage.error(`Provider ID "${id}" 已存在`); return }

  const baseUrls: Record<string, Record<string, string>> = {
    moonshot: { coding: 'https://api.kimi.com/coding/', ai: 'https://api.moonshot.ai/v1', cn: 'https://api.moonshot.cn/v1' },
    openai: { default: 'https://api.openai.com/v1' },
    anthropic: { default: 'https://api.anthropic.com/v1' },
  }

  instances[selectedType.value].push({
    id, name: newInstance.name, version: selectedType.value === 'moonshot' ? newInstance.version : undefined,
    enabled: true, apiKey: newInstance.apiKey,
    baseUrl: baseUrls[selectedType.value][newInstance.version] || baseUrls[selectedType.value].default,
    defaultModel: newInstance.defaultModel,
  })

  ElMessage.success(`实例 "${id}" 添加成功`)
  showAddDialog.value = false
  newInstance.name = ''; newInstance.apiKey = ''; newInstance.defaultModel = ''; newInstance.version = 'coding'
}

const editInstance = (instance: any) => {
  editingInstance.value = { ...instance }
  showEditDialog.value = true
}

const saveInstance = () => {
  if (!editingInstance.value) return
  const idx = instances[selectedType.value].findIndex(i => i.id === editingInstance.value.id)
  if (idx !== -1) {
    instances[selectedType.value][idx] = { ...editingInstance.value }
    ElMessage.success('配置已保存')
    showEditDialog.value = false
  }
}

const deleteInstance = async (instance: any) => {
  try {
    await ElMessageBox.confirm(`确定删除实例 "${instance.id}" 吗？`, '确认删除', { type: 'warning' })
    const idx = instances[selectedType.value].findIndex(i => i.id === instance.id)
    if (idx !== -1) {
      instances[selectedType.value].splice(idx, 1)
      ElMessage.success('实例已删除')
    }
  } catch { /* cancelled */ }
}

const addFallback = () => { if (modelPriority.fallbacks.length < 3) modelPriority.fallbacks.push('') }
const removeFallback = (index: number) => { modelPriority.fallbacks.splice(index, 1) }
</script>

<style scoped>
.provider-config {
  display: flex;
  height: 100%;
  background: #f5f7fa;
}

.provider-sidebar {
  width: 240px;
  background: #fff;
  border-right: 1px solid #e4e7ed;
  padding: 20px;
}

.provider-sidebar h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
}

.provider-types {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.provider-type {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
}

.provider-type:hover {
  background: #f5f7fa;
}

.provider-type.active {
  background: #ecf5ff;
  color: #409eff;
}

.provider-type .icon {
  font-size: 20px;
}

.provider-type .name {
  flex: 1;
  font-weight: 500;
}

.provider-type .count {
  font-size: 12px;
  color: #909399;
}

.provider-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.instance-section {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h4 {
  margin: 0;
}

.empty-state {
  padding: 40px 0;
}

.instance-card {
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
  transition: all 0.3s;
}

.instance-card:hover {
  box-shadow: 0 2px 12px 0 rgba(0,0,0,0.1);
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

.instance-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.instance-id {
  font-weight: 600;
  font-size: 16px;
}

.instance-details {
  display: flex;
  gap: 20px;
  color: #606266;
  font-size: 14px;
}

.model-priority {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
}

.model-priority h4 {
  margin: 0 0 20px 0;
}

.priority-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
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

.form-hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}
</style>