<template>
  <div class="safe-mode-view">
    <el-result icon="warning" :title="title" :sub-title="subTitle" status="warning">
      <template #extra>
        <!-- 配置错误场景 -->
        <template v-if="isConfigError">
          <div class="action-desc">
            <p><strong>配置验证失败，OpenClaw 无法启动。</strong></p>
            <p>你可以选择：</p>
            <ul class="option-list">
              <li>📝 在 Safe Mode 下继续编辑配置并修复问题</li>
              <li>⏪ 回滚到上一个工作版本</li>
              <li>🏭 恢复出厂设置</li>
            </ul>
          </div>
          <el-row :gutter="10" class="actions">
            <el-col>
              <el-button type="primary" size="large" @click="goToConfig">
                <el-icon><Edit /></el-icon> 继续编辑配置
              </el-button>
            </el-col>
            <el-col>
              <el-button size="large" @click="showSnapshots">
                <el-icon><RefreshLeft /></el-icon> 回滚到历史版本
              </el-button>
            </el-col>
            <el-col>
              <el-button size="large" @click="showLogs">
                <el-icon><Document /></el-icon> 查看日志
              </el-button>
            </el-col>
          </el-row>
          <div class="danger-actions">
            <el-button link type="danger" @click="resetToFactory"><el-icon><Delete /></el-icon> 恢复出厂设置</el-button>
          </div>
        </template>

        <!-- 系统错误场景 -->
        <template v-else-if="isSystemError">
          <div class="action-desc">
            <p>检测到系统错误，这可能是由于资源不足或依赖服务异常导致的。</p>
          </div>
          <el-row :gutter="10" class="actions">
            <el-col>
              <el-button type="primary" size="large" @click="recover">
                <el-icon><Refresh /></el-icon> 尝试恢复
              </el-button>
            </el-col>
            <el-col>
              <el-button size="large" @click="showLogs">
                <el-icon><Document /></el-icon> 查看日志
              </el-button>
            </el-col>
          </el-row>
        </template>
      </template>
    </el-result>

    <!-- 历史版本对话框 -->
    <el-dialog v-model="snapshotsVisible" title="历史版本 - 选择回滚目标" width="700px">
      <div class="snapshot-desc">
        <p>选择一个历史版本回滚，回滚后会自动重启服务。</p>
      </div>
      <el-table :data="snapshots" v-loading="loadingSnapshots" class="snapshot-table">
        <el-table-column type="index" width="50" />
        <el-table-column label="时间" width="180">
          <template #default="{ row }">{{ formatTime(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column prop="message" label="提交信息" show-overflow-tooltip />
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row, $index }">
            <el-button size="small" :type="$index === 0 ? 'primary' : 'danger'" @click="confirmRollback(row)">
              {{ $index === 0 ? '恢复' : '回滚' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>

    <!-- 日志对话框 -->
    <el-dialog v-model="logsVisible" title="服务日志" width="900px">
      <div class="logs-toolbar">
        <el-button size="small" @click="refreshLogs"><el-icon><Refresh /></el-icon> 刷新</el-button>
        <el-button size="small" @click="copyLogs"><el-icon><CopyDocument /></el-icon> 复制</el-button>
      </div>
      <pre class="logs-content">{{ logs || '暂无日志' }}</pre>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Edit, RefreshLeft, Document, Refresh, CopyDocument, Delete } from '@element-plus/icons-vue'
import { getState, getLogs, getSnapshots, rollback, restartService, resetToFactory as apiResetToFactory } from '../api'
import type { StateResponse, Snapshot } from '../types'

const router = useRouter()

const stateData = ref<StateResponse | null>(null)
const logs = ref('')
const logsVisible = ref(false)
const snapshots = ref<Snapshot[]>([])
const snapshotsVisible = ref(false)
const loadingSnapshots = ref(false)
let stateTimer: number | null = null

const title = computed(() => {
  if (isConfigError.value) return 'Safe Mode - 配置错误'
  if (isSystemError.value) return 'Safe Mode - 系统错误'
  return 'Safe Mode'
})

const subTitle = computed(() => {
  return stateData.value?.last_error || 'OpenClaw 服务未正常运行'
})

const isConfigError = computed(() => stateData.value?.state.type === 'config_error')
const isSystemError = computed(() => stateData.value?.state.type === 'system_error')

const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

const fetchState = async () => {
  try {
    const response = await getState()
    stateData.value = response.data
    if (stateData.value?.state.type === 'running') {
      router.push('/status')
    }
  } catch (error) {
    console.error('Failed to fetch state:', error)
  }
}

const showLogs = async () => {
  await refreshLogs()
  logsVisible.value = true
}

const refreshLogs = async () => {
  const response = await getLogs()
  logs.value = response.data.logs || '暂无日志'
}

const copyLogs = () => {
  navigator.clipboard.writeText(logs.value).then(() => ElMessage.success('已复制')).catch(() => ElMessage.error('复制失败'))
}

const showSnapshots = async () => {
  snapshotsVisible.value = true
  loadingSnapshots.value = true
  try {
    const response = await getSnapshots()
    snapshots.value = response.data.snapshots || []
  } finally {
    loadingSnapshots.value = false
  }
}

const confirmRollback = async (snapshot: Snapshot) => {
  try {
    await ElMessageBox.confirm(`确定要回滚到 ${formatTime(snapshot.timestamp)} 的版本吗？`, '确认回滚', { type: 'warning' })
    await rollback({ snapshot_id: snapshot.id })
    ElMessage.success('回滚成功，正在重启服务...')
    snapshotsVisible.value = false
    setTimeout(fetchState, 5000)
  } catch (error: any) {
    if (error !== 'cancel') ElMessage.error('回滚失败')
  }
}

const goToConfig = () => router.push('/config')

const recover = async () => {
  try {
    await restartService()
    ElMessage.success('恢复指令已发送')
    setTimeout(fetchState, 5000)
  } catch {
    ElMessage.error('恢复失败')
  }
}

const resetToFactory = async () => {
  try {
    await ElMessageBox.confirm('恢复出厂设置将清除所有配置，确定继续吗？', '警告', { type: 'danger' })
    await apiResetToFactory()
    ElMessage.success('已恢复出厂设置')
    setTimeout(fetchState, 5000)
  } catch (e: any) {
    if (e !== 'cancel') ElMessage.error('恢复失败')
  }
}

onMounted(() => {
  fetchState()
  stateTimer = window.setInterval(fetchState, 5000)
})

onUnmounted(() => {
  if (stateTimer) clearInterval(stateTimer)
})
</script>

<style scoped>
.safe-mode-view { max-width: 800px; margin: 0 auto; padding: 40px 20px; }
.action-desc { text-align: left; margin: 20px 0; color: #606266; }
.action-desc ul { margin: 10px 0; padding-left: 20px; }
.action-desc li { margin: 5px 0; }
.actions { margin-top: 20px; }
.danger-actions { margin-top: 20px; padding-top: 20px; border-top: 1px solid #e4e7ed; }
.snapshot-desc { margin-bottom: 15px; color: #606266; }
.snapshot-table { margin-top: 10px; }
.logs-toolbar { margin-bottom: 10px; }
.logs-content { background: #1e1e1e; color: #d4d4d4; padding: 15px; border-radius: 4px; max-height: 400px; overflow-y: auto; font-family: monospace; font-size: 12px; white-space: pre-wrap; }
</style>
