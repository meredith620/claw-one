<template>
  <div class="agent-module" v-loading="loading">
    <!-- 模式选择 -->
    <div class="agent-section">
      <div class="section-title">🤖 Agent 模式</div>
      <el-radio-group v-model="agentConfig.mode" size="large" @change="onModeChange">
        <el-radio-button label="single">单 Agent 模式</el-radio-button>
        <el-radio-button label="multi">Multi-Agent 模式</el-radio-button>
      </el-radio-group>
      <p class="mode-desc">{{ agentConfig.mode === 'single' ? '使用默认 Agent 配置，适合个人使用' : '支持多个独立 Agent，每个可配置不同工作区和模型' }}</p>
    </div>

    <!-- 默认 Agent 配置 -->
    <div class="agent-section">
      <div class="section-header">
        <div class="section-title">📁 默认 Agent 配置</div>
      </div>
      <el-form :model="agentConfig.defaults" label-width="120px">
        <el-form-item label="工作区目录">
          <el-input v-model="agentConfig.defaults.workspace" placeholder="~/.openclaw/workspace">
            <template #append><el-button>浏览</el-button></template>
          </el-input>
        </el-form-item>
        <el-form-item label="Agent 目录">
          <el-input v-model="agentConfig.defaults.agentDir" placeholder="~/.openclaw/agent" />
        </el-form-item>
      </el-form>
    </div>

    <!-- Multi-Agent 列表 -->
    <div v-if="agentConfig.mode === 'multi'" class="agent-section">
      <div class="section-header">
        <div class="section-title">👥 自定义 Agent 列表</div>
        <el-button type="primary" size="small" @click="openAddDialog">+ 添加 Agent</el-button>
      </div>

      <div v-if="agentList.length === 0" class="empty-state">
        <el-empty description="暂无自定义 Agent" :image-size="60" />
      </div>

      <div class="agents-grid">
        <div v-for="agent in agentList" :key="agent.id" class="agent-card">
          <div class="agent-header">
            <div class="agent-info">
              <span class="agent-name">{{ agent.name }}</span>
              <el-tag size="small" type="info">{{ agent.id }}</el-tag>
            </div>
            <div class="agent-actions">
              <el-button link type="primary" @click="editAgent(agent)">配置</el-button>
              <el-button link type="danger" @click="deleteAgent(agent)">删除</el-button>
            </div>
          </div>
          <div class="agent-meta">
            <div class="meta-item">
              <span class="meta-label">工作区:</span>
              <span class="meta-value">{{ agent.workspace }}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">Agent 目录:</span>
              <span class="meta-value">{{ agent.agentDir }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div class="actions">
      <el-button type="primary" size="large" @click="saveAgents" :loading="saving">保存 Agent 配置</el-button>
    </div>

    <!-- 添加/编辑 Agent 对话框 -->
    <el-dialog v-model="showDialog" :title="isEditing ? '编辑 Agent' : '添加 Agent'" width="500px">
      <el-form :model="formData" label-width="100px">
        <el-form-item label="Agent ID" required>
          <el-input v-model="formData.id" placeholder="如 developer、architecturer" :disabled="isEditing" />
        </el-form-item>
        <el-form-item label="显示名称" required>
          <el-input v-model="formData.name" placeholder="如 开发助手" />
        </el-form-item>
        <el-form-item label="工作区目录">
          <el-input v-model="formData.workspace" placeholder="默认: ~/.openclaw/workspace-{id}" />
        </el-form-item>
        <el-form-item label="Agent 目录">
          <el-input v-model="formData.agentDir" placeholder="默认: ~/.openclaw/agents/{id}/agent" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showDialog = false">取消</el-button>
        <el-button type="primary" @click="saveAgent" :loading="saving">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getAgents, saveAgents } from '../api'

const loading = ref(false)
const saving = ref(false)
const showDialog = ref(false)
const isEditing = ref(false)

const agentConfig = reactive({
  mode: 'single',
  defaults: {
    workspace: '~/.openclaw/workspace',
    agentDir: '~/.openclaw/agent',
    model: { primary: '', fallbacks: [] }
  },
  list: [] as any[]
})

const agentList = computed(() => agentConfig.list || [])

const formData = reactive({ id: '', name: '', workspace: '', agentDir: '' })

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const res = await getAgents()
    const data = res.data || {}
    agentConfig.mode = data.mode || 'single'
    agentConfig.defaults = { ...agentConfig.defaults, ...data.defaults }
    agentConfig.list = data.list || []
  } catch (error: any) {
    ElMessage.error('加载失败: ' + (error.response?.data?.error || error.message))
  } finally {
    loading.value = false
  }
}

const onModeChange = () => {
  // 只切换模式，不自动保存，避免递归
  // 用户点击保存按钮时才保存
}

const openAddDialog = () => {
  isEditing.value = false
  formData.id = ''
  formData.name = ''
  formData.workspace = ''
  formData.agentDir = ''
  showDialog.value = true
}

const editAgent = (agent: any) => {
  isEditing.value = true
  formData.id = agent.id
  formData.name = agent.name
  formData.workspace = agent.workspace || ''
  formData.agentDir = agent.agentDir || ''
  showDialog.value = true
}

const saveAgent = () => {
  if (!formData.id || !formData.name) {
    ElMessage.error('请填写完整信息')
    return
  }
  
  const newAgent = {
    id: formData.id,
    name: formData.name,
    workspace: formData.workspace || `~/.openclaw/workspace-${formData.id}`,
    agentDir: formData.agentDir || `~/.openclaw/agents/${formData.id}/agent`,
  }
  
  if (isEditing.value) {
    const idx = agentConfig.list.findIndex(a => a.id === formData.id)
    if (idx > -1) agentConfig.list[idx] = newAgent
  } else {
    if (agentConfig.list.find(a => a.id === formData.id)) {
      ElMessage.error('Agent ID 已存在')
      return
    }
    agentConfig.list.push(newAgent)
  }
  
  showDialog.value = false
  saveAgents()
}

const deleteAgent = async (agent: any) => {
  try {
    await ElMessageBox.confirm(`确定删除 Agent "${agent.name}" 吗？`, '确认删除', { type: 'warning' })
    agentConfig.list = agentConfig.list.filter(a => a.id !== agent.id)
    saveAgents()
  } catch (e) { /* cancel */ }
}

const saveAgents = async () => {
  saving.value = true
  try {
    await saveAgents({
      mode: agentConfig.mode,
      defaults: agentConfig.defaults,
      list: agentConfig.list
    })
    ElMessage.success('保存成功')
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
  } finally {
    saving.value = false
  }
}

onMounted(loadData)
</script>

<style scoped>
.agent-module { max-width: 900px; }
.agent-section { background: #fff; border-radius: 12px; padding: 20px; margin-bottom: 20px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.section-title { font-size: 16px; font-weight: 600; display: flex; align-items: center; gap: 8px; }
.mode-desc { color: #606266; font-size: 13px; margin-top: 8px; }
.empty-state { padding: 20px 0; }
.agents-grid { display: grid; gap: 12px; }
.agent-card { border: 1px solid #e4e7ed; border-radius: 8px; padding: 16px; }
.agent-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
.agent-info { display: flex; align-items: center; gap: 10px; }
.agent-name { font-weight: 600; font-size: 15px; }
.agent-actions { display: flex; gap: 8px; }
.agent-meta { display: flex; flex-direction: column; gap: 6px; }
.meta-item { display: flex; gap: 8px; font-size: 13px; }
.meta-label { color: #909399; min-width: 70px; }
.meta-value { color: #606266; font-family: monospace; }
.actions { display: flex; justify-content: center; margin-top: 20px; }
</style>
