<template>
  <div class="channel-module" v-loading="loading">
    <!-- Mattermost 配置 -->
    <div class="channel-section">
      <div class="section-header">
        <div class="section-title">
          <span class="channel-icon">💬</span>
          <span>Mattermost</span>
        </div>
        <el-switch v-model="channels.mattermost.enabled" @change="onChange" />
      </div>

      <div v-if="channels.mattermost.enabled" class="channel-content">
        <el-form :model="channels.mattermost" label-width="120px">
          <el-form-item label="DM 策略">
            <el-select v-model="channels.mattermost.dmPolicy" style="width: 200px" @change="onChange">
              <el-option label="配对模式 (pairing)" value="pairing" />
              <el-option label="允许所有 (allowall)" value="allowall" />
            </el-select>
          </el-form-item>
          <el-form-item label="群组策略">
            <el-select v-model="channels.mattermost.groupPolicy" style="width: 200px" @change="onChange">
              <el-option label="允许列表 (allowlist)" value="allowlist" />
              <el-option label="允许所有 (allowall)" value="allowall" />
            </el-select>
          </el-form-item>
        </el-form>

        <div class="subsection-title">账号列表</div>
        <el-button type="primary" size="small" @click="openAddMattermostDialog">+ 添加账号</el-button>

        <div class="accounts-list">
          <div v-for="(account, id) in mattermostAccounts" :key="id" class="account-card">
            <div class="account-header">
              <div class="account-info">
                <span class="account-name">{{ account.name || id }}</span>
                <el-tag size="small" type="info">{{ id }}</el-tag>
              </div>
              <div class="account-actions">
                <el-button link type="primary" @click="editMattermostAccount(id, account)">配置</el-button>
                <el-button link type="danger" @click="deleteMattermostAccount(id)">删除</el-button>
              </div>
            </div>
            <div class="account-meta">
              <span>Base URL: {{ account.baseUrl }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 飞书配置 -->
    <div class="channel-section">
      <div class="section-header">
        <div class="section-title">
          <span class="channel-icon">📱</span>
          <span>飞书</span>
        </div>
        <el-switch v-model="channels.feishu.enabled" @change="onChange" />
      </div>

      <div v-if="channels.feishu.enabled" class="channel-content">
        <el-form :model="channels.feishu" label-width="120px">
          <el-form-item label="App ID">
            <el-input v-model="channels.feishu.appId" placeholder="cli_xxx" @change="onChange" />
          </el-form-item>
          <el-form-item label="App Secret">
            <el-input v-model="channels.feishu.appSecret" type="password" show-password @change="onChange" />
          </el-form-item>
          <el-form-item label="连接模式">
            <el-radio-group v-model="channels.feishu.connectionMode" @change="onChange">
              <el-radio-button label="websocket">WebSocket</el-radio-button>
              <el-radio-button label="webhook">Webhook</el-radio-button>
            </el-radio-group>
          </el-form-item>
          <el-form-item label="DM 策略">
            <el-select v-model="channels.feishu.dmPolicy" style="width: 200px" @change="onChange">
              <el-option label="配对模式 (pairing)" value="pairing" />
              <el-option label="允许所有 (allowall)" value="allowall" />
            </el-select>
          </el-form-item>
        </el-form>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div class="actions">
      <el-button type="primary" size="large" @click="saveChannels" :loading="saving">保存 Channel 配置</el-button>
    </div>

    <!-- 添加/编辑 Mattermost 账号对话框 -->
    <el-dialog v-model="showMattermostDialog" title="Mattermost 账号" width="500px">
      <el-form :model="mattermostForm" label-width="100px">
        <el-form-item label="账号 ID" required>
          <el-input v-model="mattermostForm.id" placeholder="如 default、work" :disabled="editingMattermost" />
        </el-form-item>
        <el-form-item label="显示名称" required>
          <el-input v-model="mattermostForm.name" placeholder="如 Main Bot" />
        </el-form-item>
        <el-form-item label="Bot Token" required>
          <el-input v-model="mattermostForm.botToken" type="password" show-password />
        </el-form-item>
        <el-form-item label="Base URL" required>
          <el-input v-model="mattermostForm.baseUrl" placeholder="https://mm.example.com" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showMattermostDialog = false">取消</el-button>
        <el-button type="primary" @click="saveMattermostAccount" :loading="saving">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getChannels, saveChannels } from '../api'

const loading = ref(false)
const saving = ref(false)
const showMattermostDialog = ref(false)
const editingMattermost = ref(false)

const defaultChannels = {
  mattermost: { enabled: false, dmPolicy: 'pairing', groupPolicy: 'allowlist', accounts: {} },
  feishu: { enabled: false, appId: '', appSecret: '', connectionMode: 'websocket', dmPolicy: 'pairing' }
}

const channels = reactive({ ...defaultChannels })

const mattermostAccounts = computed(() => channels.mattermost?.accounts || {})

const mattermostForm = reactive({ id: '', name: '', botToken: '', baseUrl: '' })

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const res = await getChannels()
    const data = res.data || {}
    Object.assign(channels, { ...defaultChannels, ...data })
  } catch (error: any) {
    ElMessage.error('加载失败: ' + (error.response?.data?.error || error.message))
  } finally {
    loading.value = false
  }
}

const onChange = () => {
  // 标记有变更
}

const openAddMattermostDialog = () => {
  editingMattermost.value = false
  mattermostForm.id = ''
  mattermostForm.name = ''
  mattermostForm.botToken = ''
  mattermostForm.baseUrl = ''
  showMattermostDialog.value = true
}

const editMattermostAccount = (id: string, account: any) => {
  editingMattermost.value = true
  mattermostForm.id = id
  mattermostForm.name = account.name || ''
  mattermostForm.botToken = account.botToken || ''
  mattermostForm.baseUrl = account.baseUrl || ''
  showMattermostDialog.value = true
}

const saveMattermostAccount = async () => {
  if (!mattermostForm.id || !mattermostForm.name) {
    ElMessage.error('请填写完整信息')
    return
  }
  
  if (!channels.mattermost.accounts) channels.mattermost.accounts = {}
  
  channels.mattermost.accounts[mattermostForm.id] = {
    name: mattermostForm.name,
    botToken: mattermostForm.botToken,
    baseUrl: mattermostForm.baseUrl
  }
  
  showMattermostDialog.value = false
  await saveChannelsData()
}

const deleteMattermostAccount = async (id: string) => {
  try {
    await ElMessageBox.confirm(`确定删除账号 "${id}" 吗？`, '确认删除', { type: 'warning' })
    delete channels.mattermost.accounts[id]
    await saveChannelsData()
  } catch (e) { /* cancel */ }
}

const saveChannelsData = async () => {
  saving.value = true
  try {
    await saveChannels({ ...channels })
    ElMessage.success('保存成功')
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
  } finally {
    saving.value = false
  }
}

const saveChannels = async () => {
  await saveChannelsData()
}

onMounted(loadData)
</script>

<style scoped>
.channel-module { max-width: 900px; }
.channel-section { background: #fff; border-radius: 12px; padding: 20px; margin-bottom: 20px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.section-title { display: flex; align-items: center; gap: 10px; font-size: 16px; font-weight: 600; }
.channel-icon { font-size: 24px; }
.subsection-title { font-size: 14px; font-weight: 600; margin: 20px 0 12px 0; color: #606266; }
.accounts-list { margin-top: 16px; display: grid; gap: 12px; }
.account-card { border: 1px solid #e4e7ed; border-radius: 8px; padding: 16px; }
.account-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.account-info { display: flex; align-items: center; gap: 10px; }
.account-name { font-weight: 600; }
.account-meta { font-size: 13px; color: #606266; }
.actions { display: flex; justify-content: center; margin-top: 20px; }
.channel-content { padding-top: 16px; border-top: 1px solid #e4e7ed; }
</style>
