<template>
  <div class="status-view">
    <!-- 状态卡片 -->
    <el-card class="status-card">
      <template #header>
        <div class="card-header">
          <span>系统状态</span>
          <el-tag :type="stateType">{{ stateText }}</el-tag>
        </div>
      </template>

      <div v-if="stateData" class="status-content">
        <!-- 当前版本 -->
        <div v-if="stateData.current_version" class="info-row">
          <span class="label">当前版本:</span>
          <span class="value mono">{{ stateData.current_version.slice(0, 8) }}</span>
        </div>

        <!-- 错误信息 -->
        <div v-if="stateData.last_error" class="info-row error">
          <span class="label">错误信息:</span>
          <span class="value">{{ stateData.last_error }}</span>
        </div>

        <!-- 操作按钮 -->
        <div class="actions">
          <el-button type="primary" @click="goToConfig">
            <el-icon><Edit /></el-icon>
            修改配置
          </el-button>

          <el-button @click="showSnapshots = true" :disabled="!stateData.can_rollback">
            <el-icon><Clock /></el-icon>
            历史版本
          </el-button>

          <el-button @click="restartService" :loading="restarting">
            <el-icon><Refresh /></el-icon>
            重启服务
          </el-button>
        </div>
      </div>

      <div v-else class="loading">
        <el-skeleton :rows="3" animated />
      </div>
    </el-card>

    <!-- 日志卡片 -->
    <el-card class="logs-card">
      <template #header>
        <div class="card-header">
          <span>服务日志</span>
          <el-button size="small" @click="refreshLogs">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </template>

      <pre v-if="logs" class="logs-content">{{ logs }}</pre>
      <div v-else class="loading">加载中...</div>
    </el-card>

    <!-- 历史版本对话框 -->
    <el-dialog v-model="showSnapshots" title="历史版本" width="600px">
      <el-table :data="snapshots" v-loading="loadingSnapshots">
        <el-table-column prop="timestamp" label="时间" width="180" />
        <el-table-column prop="message" label="提交信息" />
        <el-table-column label="操作" width="100">
          <template #default="{ row }">
            <el-button size="small" type="danger" @click="confirmRollback(row)">
              回滚
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Edit, Clock, Refresh } from '@element-plus/icons-vue'
import { getState, getLogs, getSnapshots, restartService as apiRestart, rollback } from '../api'
import type { StateResponse, Snapshot } from '../types'

const router = useRouter()

// 状态数据
const stateData = ref<StateResponse | null>(null)
const logs = ref('')
const snapshots = ref<Snapshot[]>([])
const showSnapshots = ref(false)
const loadingSnapshots = ref(false)
const restarting = ref(false)

// 定时刷新
let refreshTimer: ReturnType<typeof setInterval> | null = null

// 状态显示
const stateType = computed(() => {
  if (!stateData.value) return 'info'
  const state = stateData.value.state
  switch (state.type) {
    case 'running':
      return 'success'
    case 'starting':
      return 'warning'
    case 'stopped':
    case 'unknown':
      return 'info'
    case 'config_error':
    case 'system_error':
      return 'danger'
    default:
      return 'info'
  }
})

const stateText = computed(() => {
  if (!stateData.value) return '加载中...'
  const state = stateData.value.state
  switch (state.type) {
    case 'running':
      return '运行中'
    case 'starting':
      return '启动中'
    case 'stopped':
      return '已停止'
    case 'unknown':
      return '未知'
    case 'config_error':
      return state.auto_rolled_back ? '配置错误（已回滚）' : '配置错误'
    case 'system_error':
      return '系统错误'
    default:
      return '未知'
  }
})

// 获取状态
const fetchState = async () => {
  try {
    const response = await getState()
    stateData.value = response.data

    // 如果是 SafeMode，跳转到 SafeMode 页面
    if (stateData.value?.state.type === 'config_error' ||
        stateData.value?.state.type === 'system_error') {
      router.push('/safe-mode')
    }
  } catch (error) {
    console.error('Failed to fetch state:', error)
  }
}

// 获取日志
const fetchLogs = async () => {
  try {
    const response = await getLogs()
    logs.value = response.data.logs || '暂无日志'
  } catch (error) {
    logs.value = '获取日志失败'
  }
}

// 刷新日志
const refreshLogs = () => {
  fetchLogs()
  ElMessage.success('日志已刷新')
}

// 获取快照列表
const fetchSnapshots = async () => {
  loadingSnapshots.value = true
  try {
    const response = await getSnapshots()
    snapshots.value = response.data.snapshots
  } catch (error) {
    ElMessage.error('获取历史版本失败')
  } finally {
    loadingSnapshots.value = false
  }
}

// 重启服务
const restartService = async () => {
  try {
    await ElMessageBox.confirm('确定要重启服务吗？', '确认', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })

    restarting.value = true
    await apiRestart()
    ElMessage.success('服务重启指令已发送')

    // 等待几秒后刷新状态
    setTimeout(() => {
      fetchState()
      restarting.value = false
    }, 3000)
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('重启失败')
    }
    restarting.value = false
  }
}

// 确认回滚
const confirmRollback = async (snapshot: Snapshot) => {
  try {
    await ElMessageBox.confirm(
      `确定要回滚到版本 ${snapshot.id.slice(0, 8)} 吗？当前配置将会丢失。`,
      '确认回滚',
      {
        confirmButtonText: '确定回滚',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    await rollback({ snapshot_id: snapshot.id })
    ElMessage.success('回滚成功')
    showSnapshots.value = false
    fetchState()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('回滚失败')
    }
  }
}

// 跳转配置
const goToConfig = () => {
  router.push('/config')
}

// 监听对话框
watch(showSnapshots, (val) => {
  if (val) {
    fetchSnapshots()
  }
})

// 生命周期
onMounted(() => {
  fetchState()
  fetchLogs()
  refreshTimer = setInterval(() => {
    fetchState()
  }, 5000) // 每 5 秒刷新状态
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})

// 导入 watch
import { watch } from 'vue'
</script>

<style scoped>
.status-view {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.status-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-content {
  padding: 10px 0;
}

.info-row {
  margin-bottom: 15px;
  display: flex;
  align-items: flex-start;
}

.info-row.error {
  color: #f56c6c;
}

.label {
  font-weight: bold;
  margin-right: 10px;
  min-width: 100px;
}

.value {
  flex: 1;
}

.value.mono {
  font-family: monospace;
  background: #f5f7fa;
  padding: 2px 6px;
  border-radius: 4px;
}

.actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
}

.logs-card {
  margin-top: 20px;
}

.logs-content {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 15px;
  border-radius: 4px;
  max-height: 400px;
  overflow-y: auto;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.loading {
  text-align: center;
  padding: 20px;
  color: #909399;
}
</style>