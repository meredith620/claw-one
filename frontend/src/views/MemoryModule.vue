<template>
  <div class="memory-module" v-loading="loading">
    <!-- 基础配置 -->
    <div class="memory-section">
      <div class="section-title">💾 Memory 基础配置</div>
      
      <el-form :model="memoryConfig" label-width="120px">
        <el-form-item label="启用 Memory">
          <el-switch v-model="memoryConfig.enabled" @change="onChange" />
        </el-form-item>

        <el-form-item label="Provider">
          <el-radio-group v-model="memoryConfig.provider" @change="onChange">
            <el-radio-button label="ollama">Ollama (本地)</el-radio-button>
            <el-radio-button label="openai">OpenAI</el-radio-button>
          </el-radio-group>
        </el-form-item>

        <!-- Ollama 配置 -->
        <template v-if="memoryConfig.provider === 'ollama'">
          <el-form-item label="Base URL">
            <el-input v-model="memoryConfig.remote.baseUrl" placeholder="http://localhost:11434" @change="onChange" />
          </el-form-item>
          <el-form-item label="Embedding 模型">
            <el-select v-model="memoryConfig.model" style="width: 100%" @change="onChange">
              <el-option label="qwen3-embedding:0.6b" value="qwen3-embedding:0.6b" />
              <el-option label="nomic-embed-text" value="nomic-embed-text" />
              <el-option label="mxbai-embed-large" value="mxbai-embed-large" />
            </el-select>
          </el-form-item>
        </template>

        <!-- OpenAI 配置 -->
        <template v-if="memoryConfig.provider === 'openai'">
          <el-form-item label="API Key">
            <el-input v-model="memoryConfig.remote.apiKey" type="password" show-password @change="onChange" />
          </el-form-item>
          <el-form-item label="Embedding 模型">
            <el-select v-model="memoryConfig.model" style="width: 100%" @change="onChange">
              <el-option label="text-embedding-3-small" value="text-embedding-3-small" />
              <el-option label="text-embedding-3-large" value="text-embedding-3-large" />
              <el-option label="text-embedding-ada-002" value="text-embedding-ada-002" />
            </el-select>
          </el-form-item>
        </template>
      </el-form>
    </div>

    <!-- 高级功能 -->
    <div class="memory-section">
      <div class="section-title">⚡ 高级功能</div>
      
      <el-form :model="memoryConfig" label-width="140px">
        <!-- 向量存储 -->
        <el-form-item>
          <template #label>
            <span>向量存储</span>
            <el-tooltip content="启用 SQLite 向量扩展，提升存储性能">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
          <el-switch v-model="memoryConfig.store.vector.enabled" @change="onVectorChange" />
        </el-form-item>

        <el-form-item v-if="memoryConfig.store?.vector?.enabled" label="扩展路径">
          <el-input v-model="memoryConfig.store.vector.extensionPath" placeholder="~/.openclaw/extensions/vec0.so" @change="onChange" />
        </el-form-item>

        <!-- 混合搜索 -->
        <el-form-item>
          <template #label>
            <span>混合搜索</span>
            <el-tooltip content="结合向量相似度和关键词匹配">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
          <el-switch v-model="memoryConfig.query.hybrid.enabled" :disabled="!memoryConfig.store?.vector?.enabled" @change="onHybridChange" />
        </el-form-item>

        <template v-if="memoryConfig.query?.hybrid?.enabled">
          <el-form-item label="向量权重">
            <el-slider v-model="memoryConfig.query.hybrid.vectorWeight" :max="1" :step="0.1" show-input @change="onChange" />
          </el-form-item>
          <el-form-item label="文本权重">
            <el-slider v-model="memoryConfig.query.hybrid.textWeight" :max="1" :step="0.1" show-input @change="onChange" />
          </el-form-item>
        </template>

        <!-- MMR 重排序 -->
        <el-form-item>
          <template #label>
            <span>MMR 重排序</span>
            <el-tooltip content="提升搜索结果多样性">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
          <el-switch v-model="memoryConfig.query.hybrid.mmr.enabled" :disabled="!memoryConfig.query?.hybrid?.enabled" @change="onChange" />
        </el-form-item>

        <!-- 时间衰减 -->
        <el-form-item>
          <template #label>
            <span>时间衰减</span>
            <el-tooltip content="优先返回近期记忆">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
          <el-switch v-model="memoryConfig.query.hybrid.temporalDecay.enabled" :disabled="!memoryConfig.query?.hybrid?.enabled" @change="onChange" />
        </el-form-item>
      </el-form>
    </div>

    <!-- 保存按钮 -->
    <div class="actions">
      <el-button type="primary" size="large" @click="saveMemory" :loading="saving">保存 Memory 配置</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { getMemory, saveMemory } from '../api'

const loading = ref(false)
const saving = ref(false)
const hasChanges = ref(false)

const defaultConfig = {
  enabled: true,
  provider: 'ollama',
  remote: { baseUrl: 'http://localhost:11434' },
  model: 'qwen3-embedding:0.6b',
  store: { vector: { enabled: false, extensionPath: '~/.openclaw/extensions/vec0.so' } },
  query: { 
    hybrid: { 
      enabled: false, 
      vectorWeight: 0.7, 
      textWeight: 0.3,
      mmr: { enabled: false },
      temporalDecay: { enabled: false, halfLifeDays: 30 }
    } 
  }
}

const memoryConfig = reactive({ ...defaultConfig })

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const res = await getMemory()
    const data = res.data || {}
    Object.assign(memoryConfig, { ...defaultConfig, ...data })
    // 确保嵌套对象存在
    if (!memoryConfig.remote) memoryConfig.remote = { baseUrl: 'http://localhost:11434' }
    if (!memoryConfig.store) memoryConfig.store = { vector: { enabled: false } }
    if (!memoryConfig.query) memoryConfig.query = { hybrid: { enabled: false } }
  } catch (error: any) {
    ElMessage.error('加载失败: ' + (error.response?.data?.error || error.message))
  } finally {
    loading.value = false
  }
}

const onChange = () => {
  hasChanges.value = true
}

const onVectorChange = () => {
  if (!memoryConfig.store.vector.enabled) {
    memoryConfig.query.hybrid.enabled = false
  }
  onChange()
}

const onHybridChange = () => {
  if (!memoryConfig.query.hybrid.enabled) {
    memoryConfig.query.hybrid.mmr.enabled = false
    memoryConfig.query.hybrid.temporalDecay.enabled = false
  }
  onChange()
}

const saveMemoryConfig = async () => {
  saving.value = true
  try {
    await saveMemory({ ...memoryConfig })
    ElMessage.success('保存成功')
    hasChanges.value = false
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
  } finally {
    saving.value = false
  }
}

onMounted(loadData)
</script>

<style scoped>
.memory-module { max-width: 900px; }
.memory-section { background: #fff; border-radius: 12px; padding: 20px; margin-bottom: 20px; }
.section-title { font-size: 16px; font-weight: 600; margin-bottom: 20px; display: flex; align-items: center; gap: 8px; }
.actions { display: flex; justify-content: center; margin-top: 20px; }
</style>
