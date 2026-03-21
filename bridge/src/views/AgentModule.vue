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

    <!-- A2A 协作配置 (Multi-Agent 模式显示) -->
    <div v-if="agentConfig.mode === 'multi'" class="agent-section">
      <div class="section-header">
        <div class="section-title">🔗 A2A 协作配置</div>
        <el-switch v-model="a2aConfig.enabled" active-text="启用" @change="onA2AChange" />
      </div>

      <!-- A2A 状态小组件 -->
      <div class="a2a-status-widget" :class="{ enabled: a2aConfig.enabled }">
        <div class="status-icon">{{ a2aConfig.enabled ? '✅' : '⚪' }}</div>
        <div class="status-info">
          <div class="status-title">{{ a2aConfig.enabled ? 'A2A 协作已启用' : 'A2A 协作未启用' }}</div>
          <div class="status-desc" v-if="a2aConfig.enabled">
            {{ agentList.length }} 个 Agent 可相互通信 | Session 可见性: {{ a2aConfig.visibility }}
          </div>
        </div>
        <el-button v-if="a2aConfig.enabled" type="primary" link @click="showA2AConfig = true">
          配置详情 →
        </el-button>
      </div>

      <!-- A2A 配置折叠面板 -->
      <el-collapse v-model="activeCollapse" v-if="a2aConfig.enabled">
        <el-collapse-item name="communication" title="Agent 间通信">
          <el-form :model="a2aConfig" label-width="140px">
            <el-form-item label="允许通信的 Agent">
              <el-checkbox-group v-model="a2aConfig.allowAgents">
                <el-checkbox v-for="agent in allAgents" :key="agent.id" :label="agent.id">
                  {{ agent.name }} ({{ agent.id }})
                </el-checkbox>
              </el-checkbox-group>
            </el-form-item>
          </el-form>
        </el-collapse-item>

        <el-collapse-item name="visibility" title="会话可见性">
          <el-form :model="a2aConfig" label-width="140px">
            <el-form-item label="Session 可见性">
              <el-select v-model="a2aConfig.visibility" style="width: 200px">
                <el-option label="仅当前会话 (self)" value="self" />
                <el-option label="当前+子会话 (tree)" value="tree" />
                <el-option label="同 Agent 会话 (agent)" value="agent" />
                <el-option label="任何会话 (all)" value="all" />
              </el-select>
              <el-tooltip content="控制 sessions_list/history/send 的访问范围">
                <el-icon class="help-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </el-form-item>
          </el-form>
        </el-collapse-item>

        <el-collapse-item name="subagents" title="Subagent 配置">
          <el-form :model="subagentConfig" label-width="180px">
            <el-form-item label="最大嵌套深度">
              <el-slider v-model="subagentConfig.maxSpawnDepth" :min="1" :max="5" show-stops style="width: 300px" />
              <span class="slider-value">{{ subagentConfig.maxSpawnDepth }} 级</span>
            </el-form-item>
            <el-form-item label="每 Agent 最大子代理">
              <el-input-number v-model="subagentConfig.maxChildrenPerAgent" :min="1" :max="20" />
            </el-form-item>
            <el-form-item label="全局并发限制">
              <el-input-number v-model="subagentConfig.maxConcurrent" :min="1" :max="20" />
            </el-form-item>
            <el-form-item>
              <el-button link type="primary" @click="showAdvancedSubagent = !showAdvancedSubagent">
                {{ showAdvancedSubagent ? '收起' : '高级设置...' }}
              </el-button>
            </el-form-item>
            <template v-if="showAdvancedSubagent">
              <el-form-item label="运行超时 (秒)">
                <el-input-number v-model="subagentConfig.runTimeoutSeconds" :min="0" :step="60" />
                <span class="form-hint">0 = 无限制</span>
              </el-form-item>
              <el-form-item label="自动归档 (分钟)">
                <el-input-number v-model="subagentConfig.archiveAfterMinutes" :min="1" :step="10" />
              </el-form-item>
            </template>
          </el-form>
        </el-collapse-item>

        <el-collapse-item name="threads" title="讨论线程配置">
          <el-form :model="threadConfig" label-width="140px">
            <el-form-item label="启用 Thread 绑定">
              <el-switch v-model="threadConfig.enabled" />
            </el-form-item>
            <template v-if="threadConfig.enabled">
              <el-form-item label="空闲保留时间">
                <el-input-number v-model="threadConfig.idleHours" :min="1" :max="168" />
                <span class="form-hint">小时</span>
              </el-form-item>
              <el-form-item label="最大生命周期">
                <el-input-number v-model="threadConfig.maxAgeHours" :min="0" />
                <span class="form-hint">0 = 无限制</span>
              </el-form-item>
            </template>
          </el-form>
        </el-collapse-item>
      </el-collapse>
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
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { getAgents, saveAgents as saveAgentsApi, getModuleConfig, saveModuleConfig } from '../api'

const loading = ref(false)
const saving = ref(false)
const showDialog = ref(false)
const isEditing = ref(false)
const showA2AConfig = ref(false)
const activeCollapse = ref(['communication'])
const showAdvancedSubagent = ref(false)

const agentConfig = reactive({
  mode: 'single',
  defaults: {
    workspace: '~/.openclaw/workspace',
    agentDir: '~/.openclaw/agent',
    model: { primary: '', fallbacks: [] }
  },
  list: [] as any[]
})

// A2A 协作配置
const a2aConfig = reactive({
  enabled: false,
  allowAgents: [] as string[],
  visibility: 'all' as 'self' | 'tree' | 'agent' | 'all'
})

// Subagent 配置
const subagentConfig = reactive({
  maxSpawnDepth: 2,
  maxChildrenPerAgent: 5,
  maxConcurrent: 8,
  runTimeoutSeconds: 300,
  archiveAfterMinutes: 60
})

// Thread 配置
const threadConfig = reactive({
  enabled: true,
  idleHours: 24,
  maxAgeHours: 0
})

const agentList = computed(() => agentConfig.list || [])

// 所有 Agent 列表（包括 main）
const allAgents = computed(() => {
  const agents = [...agentConfig.list]
  if (!agents.find(a => a.id === 'main')) {
    agents.unshift({ id: 'main', name: '主代理' })
  }
  return agents
})

const formData = reactive({ id: '', name: '', workspace: '', agentDir: '' })

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    // 加载 Agent 配置
    const res = await getAgents()
    const data = res.data || {}
    
    // 推断模式
    const agentList = data.list || []
    const hasCustomAgents = agentList.some((a: any) => a.id !== 'main')
    const inferredMode = (agentList.length > 1 || hasCustomAgents) ? 'multi' : 'single'
    
    agentConfig.mode = inferredMode
    agentConfig.defaults = { ...agentConfig.defaults, ...data.defaults }
    agentConfig.list = agentList

    // 加载 A2A 配置
    try {
      const toolsRes = await getModuleConfig('tools')
      const toolsData = toolsRes.data || {}
      
      // agentToAgent 配置
      if (toolsData.agentToAgent) {
        a2aConfig.enabled = toolsData.agentToAgent.enabled || false
        a2aConfig.allowAgents = toolsData.agentToAgent.allow || []
      }
      
      // sessions visibility
      if (toolsData.sessions?.visibility) {
        a2aConfig.visibility = toolsData.sessions.visibility
      }

      // subagents 配置
      if (data.defaults?.subagents) {
        subagentConfig.maxSpawnDepth = data.defaults.subagents.maxSpawnDepth || 2
        subagentConfig.maxChildrenPerAgent = data.defaults.subagents.maxChildrenPerAgent || 5
        subagentConfig.maxConcurrent = data.defaults.subagents.maxConcurrent || 8
        subagentConfig.runTimeoutSeconds = data.defaults.subagents.runTimeoutSeconds || 300
        subagentConfig.archiveAfterMinutes = data.defaults.subagents.archiveAfterMinutes || 60
      }
    } catch (e) {
      console.log('A2A config not found, using defaults')
    }

    // 加载 Thread 配置
    try {
      const sessionRes = await getModuleConfig('session')
      const sessionData = sessionRes.data || {}
      if (sessionData.threadBindings) {
        threadConfig.enabled = sessionData.threadBindings.enabled || false
        threadConfig.idleHours = sessionData.threadBindings.idleHours || 24
        threadConfig.maxAgeHours = sessionData.threadBindings.maxAgeHours || 0
      }
    } catch (e) {
      console.log('Thread config not found, using defaults')
    }
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

const onA2AChange = () => {
  if (a2aConfig.enabled && a2aConfig.allowAgents.length === 0) {
    // 默认选中所有 Agent
    a2aConfig.allowAgents = allAgents.value.map(a => a.id)
  }
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
    // 构建发送给后端的数据
    const defaultsToSend = { ...agentConfig.defaults }
    delete (defaultsToSend as any).agentDir

    // 添加 subagent 配置
    defaultsToSend.subagents = {
      maxSpawnDepth: subagentConfig.maxSpawnDepth,
      maxChildrenPerAgent: subagentConfig.maxChildrenPerAgent,
      maxConcurrent: subagentConfig.maxConcurrent,
      runTimeoutSeconds: subagentConfig.runTimeoutSeconds,
      archiveAfterMinutes: subagentConfig.archiveAfterMinutes
    }
    
    await saveAgentsApi({
      defaults: defaultsToSend,
      list: agentConfig.list
    })

    // 保存 A2A 配置
    if (agentConfig.mode === 'multi') {
      const toolsConfig = {
        agentToAgent: {
          enabled: a2aConfig.enabled,
          allow: a2aConfig.allowAgents
        },
        sessions: {
          visibility: a2aConfig.visibility
        }
      }
      await saveModuleConfig('tools', toolsConfig)

      // 保存 Thread 配置
      const sessionConfig = {
        threadBindings: {
          enabled: threadConfig.enabled,
          idleHours: threadConfig.idleHours,
          maxAgeHours: threadConfig.maxAgeHours
        }
      }
      await saveModuleConfig('session', sessionConfig)
    }

    ElMessage.success('保存成功')
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
  } finally {
    saving.value = false
  }
}

// 监听全局保存事件
const handleSaveAll = () => {
  saveAgents()
}

onMounted(() => {
  loadData()
  window.addEventListener('claw:save-all', handleSaveAll)
})

onUnmounted(() => {
  window.removeEventListener('claw:save-all', handleSaveAll)
})
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

/* A2A 样式 */
.a2a-status-widget {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
  margin-bottom: 16px;
  border: 1px solid #e4e7ed;
}

.a2a-status-widget.enabled {
  background: #f0f9ff;
  border-color: #bae6fd;
}

.status-icon {
  font-size: 24px;
}

.status-info {
  flex: 1;
}

.status-title {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
}

.status-desc {
  font-size: 13px;
  color: #606266;
  margin-top: 4px;
}

.help-icon {
  margin-left: 8px;
  color: #909399;
  cursor: help;
}

.slider-value {
  margin-left: 12px;
  color: #606266;
  font-size: 13px;
}

.form-hint {
  margin-left: 8px;
  color: #909399;
  font-size: 13px;
}
</style>
