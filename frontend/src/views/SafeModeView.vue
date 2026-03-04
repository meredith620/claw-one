<template>
  <div class="safe-mode-view">
    <el-result
      icon="warning"
      title="Safe Mode"
      :sub-title="errorMessage"
      status="warning"
    >
      <template #extra>
        <!-- 配置错误场景（已回滚）-->
        <el-row v-if="isConfigError && autoRolledBack" :gutter="10">
          <el-col>
            <el-button type="primary" @click="goToConfig">重新编辑配置</el-button>
          </el-col>
          <el-col>
            <el-button @click="showLogs">查看日志</el-button>
          </el-col>
          <el-col>
            <el-dropdown>
              <el-button>更多操作</el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item @click="forceApply">强制使用新配置</el-dropdown-item>
                  <el-dropdown-item divided type="danger" @click="resetToFactory"
                    >恢复出厂设置</el-dropdown-item
                  >
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </el-col>
        </el-row>

        <!-- 配置错误场景（未回滚）-->
        <el-row v-else-if="isConfigError && !autoRolledBack" :gutter="10">
          <el-col>
            <el-button type="primary" @click="goToConfig">重新编辑配置</el-button>
          </el-col>
          <el-col>
            <el-button @click="showSnapshots">回滚到历史版本</el-button>
          </el-col>
          <el-col>
            <el-button @click="showLogs">查看日志</el-button>
          </el-col>
        </el-row>

        <!-- 系统错误场景 -->
        <el-row v-else-if="isSystemError" :gutter="10">
          <el-col>
            <el-button type="primary" @click="goToConfig">重新编辑配置</el-button>
          </el-col>
          <el-col>
            <el-button @click="showSnapshots">回滚到历史版本</el-button>
          </el-col>
          <el-col>
            <el-button @click="showLogs">查看日志</el-button>
          </el-col>
        </el-row>

        <!-- 手动触发 SafeMode -->
        <el-row v-else :gutter="10">
          <el-col>
            <el-button type="primary" @click="recover">尝试恢复</el-button>
          </el-col>
          <el-col>
            <el-button @click="showSnapshots">回滚到历史版本</el-button>
          </el-col>
          <el-col>
            <el-button @click="showLogs">查看日志</el-button>
          </el-col>
        </el-row>
      </template>
    </el-result>

    <!-- 日志对话框 -->
    <el-dialog v-model="logsVisible" title="服务日志" width="800px">
      <pre class="logs-content">{{ logs }}</pre>
      <template #footer>
        <el-button @click="logsVisible = false">关闭</el-button>
      </template>
    </el-dialog>

    <!-- 历史版本对话框 -->
    <el-dialog v-model="snapshotsVisible" title="历史版本" width="600px">
      <el-table :data="snapshots" v-loading="loadingSnapshots">
        <el-table-column prop="timestamp" label="时间" width="180" />
        <el-table-column prop="message" label="提交信息" />
        <el-table-column label="操作" width="100">
          <template #default="{ row }">
            <el-button size="small" type="danger" @click="confirmRollback(row)"
              >回滚</el-button
            >
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getState, getLogs, getSnapshots, rollback, restartService, resetToFactory as apiResetToFactory } from '../api'
import type { StateResponse, Snapshot } from '../types'

const router = useRouter()

// 状态数据
const stateData = ref<StateResponse | null>(null)
const logs = ref('')
const logsVisible = ref(false)
const snapshots = ref<Snapshot[]>([])
const snapshotsVisible = ref(false)
const loadingSnapshots = ref(false)

// 计算属性
const errorMessage = computed(() => {
  if (!stateData.value?.last_error) {
    return '系统处于安全模式，请检查配置或查看日志'
  }
  return stateData.value.last_error
})

const isConfigError = computed(() => {
  return stateData.value?.state.type === 'config_error'
})

const isSystemError = computed(() => {
  return stateData.value?.state.type === 'system_error'
})

const autoRolledBack = computed(() => {
  const state = stateData.value?.state
  if (state?.type === 'config_error') {
    return state.auto_rolled_back
  }
  return false
})

// 获取状态
const fetchState = async () => {
  try {
    const response = await getState()
    stateData.value = response.data

    // 如果状态正常，跳转到状态页面
    if (stateData.value?.state.type === 'running') {
      router.push('/status')
    }
  } catch (error) {
    console.error('Failed to fetch state:', error)
  }
}

// 显示日志
const showLogs = async () => {
  try {
    const response = await getLogs()
    logs.value = response.data.logs || '暂无日志'
    logsVisible.value = true
  } catch (error) {
    ElMessage.error('获取日志失败')
  }
}

// 显示历史版本
const showSnapshots = async () => {
  snapshotsVisible.value = true
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

// 确认回滚
const confirmRollback = async (snapshot: Snapshot) => {
  try {
    await ElMessageBox.confirm(
      `确定要回滚到版本 ${snapshot.id.slice(0, 8)} 吗？`,
      '确认回滚',
      {
        confirmButtonText: '确定回滚',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    await rollback({ snapshot_id: snapshot.id })
    ElMessage.success('回滚成功，正在重启服务...')
    snapshotsVisible.value = false

    // 等待几秒后检查状态
    setTimeout(() => {
      fetchState()
    }, 5000)
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

// 强制使用新配置
const forceApply = async () => {
  try {
    await ElMessageBox.confirm(
      '强制使用新配置可能导致服务无法启动，确定继续吗？',
      '警告',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    // TODO: 实现强制应用逻辑
    ElMessage.info('功能开发中...')
  } catch (error) {
    // 取消
  }
}

// 恢复出厂设置
const resetToFactory = async () => {
  try {
    await ElMessageBox.confirm(
      '恢复出厂设置将清除所有配置并重启服务，确定继续吗？',
      '警告',
      {
        confirmButtonText: '确定恢复',
        cancelButtonText: '取消',
        type: 'danger',
      }
    )

    const response = await apiResetToFactory()
    ElMessage.success('已恢复出厂设置，正在重启服务...')

    // 等待几秒后检查状态
    setTimeout(() => {
      fetchState()
    }, 5000)
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('恢复出厂设置失败')
    }
  }
}

// 尝试恢复
const recover = async () => {
  try {
    await restartService()
    ElMessage.success('恢复指令已发送，请稍候...')

    // 等待几秒后检查状态
    setTimeout(() => {
      fetchState()
    }, 5000)
  } catch (error) {
    ElMessage.error('恢复失败')
  }
}

// 生命周期
onMounted(() => {
  fetchState()
  // 每 5 秒刷新状态
  const timer = setInterval(fetchState, 5000)
  return () => clearInterval(timer)
})
</script>

<style scoped>
.safe-mode-view {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 20px;
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
</style>