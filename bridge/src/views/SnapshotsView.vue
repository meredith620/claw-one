<template>
  <div class="snapshots-view">
    <div class="page-header">
      <h2>📋 配置版本历史</h2>
      <p class="desc">查看所有配置变更历史，可以回滚到任意历史版本。</p>
    </div>

    <el-card v-loading="loading" class="snapshots-card">
      <div class="toolbar">
        <el-alert
          v-if="currentVersion"
          :title="`当前版本: ${currentVersion.id?.slice(0, 8)} - ${currentVersion.message}`"
          type="success"
          :closable="false"
          show-icon
        />
        <el-button @click="refreshSnapshots" :icon="Refresh">刷新</el-button>
      </div>

      <el-timeline v-if="snapshots.length > 0">
        <el-timeline-item
          v-for="(snapshot, index) in snapshots"
          :key="snapshot.id"
          :type="getTimelineType(index)"
          :timestamp="formatTime(snapshot.timestamp)"
        >
          <div class="snapshot-item">
            <div class="snapshot-header">
              <span class="snapshot-id">{{ snapshot.id?.slice(0, 8) }}</span>
              <el-tag v-if="index === 0" type="success" size="small">当前</el-tag>
              <el-tag v-else-if="index === 1" type="warning" size="small">上一个版本</el-tag>
            </div>
            
            <div class="snapshot-message">{{ snapshot.message }}</div>
            
            <div class="snapshot-meta">
              <span v-if="snapshot.author">👤 {{ snapshot.author }}</span>
              <span v-if="snapshot.changes">📝 {{ snapshot.changes }} 个文件变更</span>
            </div>

            <div class="snapshot-actions">
              <el-button 
                v-if="index > 0"
                type="primary" 
                size="small" 
                @click="confirmRollback(snapshot)"
                :loading="rollingBack === snapshot.id"
              >
                回滚到此版本
              </el-button>
              <el-button 
                v-else
                disabled
                size="small"
              >
                当前版本
              </el-button>
              
              <el-button 
                size="small" 
                @click="viewDiff(snapshot)"
                :disabled="index === snapshots.length - 1"
              >
                查看变更
              </el-button>
            </div>
          </div>
        </el-timeline-item>
      </el-timeline>

      <el-empty v-else description="暂无配置历史" />
    </el-card>

    <!-- 回滚确认对话框 -->
    <el-dialog v-model="rollbackVisible" title="确认回滚" width="500px">
      <div class="rollback-confirm">
        <el-alert
          title="警告：回滚将丢弃当前配置"
          type="warning"
          :closable="false"
          show-icon
        />
        
        <div class="confirm-details">
          <p><strong>目标版本：</strong></p>
          <p>提交 ID：{{ selectedSnapshot?.id?.slice(0, 8) }}</p>
          <p>提交时间：{{ formatTime(selectedSnapshot?.timestamp || '') }}</p>
          <p>提交信息：{{ selectedSnapshot?.message }}</p>
        </div>

        <el-divider />

        <p>回滚后将自动重启 OpenClaw 服务。</p>
        <p>如果回滚后服务仍无法启动，可以再次回滚到更早的版本。</p>
      </div>
      
      <template #footer>
        <el-button @click="rollbackVisible = false">取消</el-button>
        <el-button type="danger" @click="executeRollback" :loading="rollingBack === selectedSnapshot?.id">
          确认回滚
        </el-button>
      </template>
    </el-dialog>

    <!-- 变更详情对话框 -->
    <el-dialog v-model="diffVisible" title="配置变更详情" width="800px">
      <div v-if="diffLoading" class="diff-loading">
        <el-skeleton :rows="10" />
      </div>
      <div v-else-if="diffData" class="diff-content">
        <pre>{{ diffData }}</pre>
      </div>
      <template #footer>
        <el-button @click="diffVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { getSnapshots, rollback, getSnapshotDiff } from '../api'
import type { Snapshot } from '../types'

const router = useRouter()

const snapshots = ref<Snapshot[]>([])
const loading = ref(false)
const rollingBack = ref<string | null>(null)
const rollbackVisible = ref(false)
const selectedSnapshot = ref<Snapshot | null>(null)
const diffVisible = ref(false)
const diffLoading = ref(false)
const diffData = ref('')

const currentVersion = ref<Snapshot | null>(null)

const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const getTimelineType = (index: number) => {
  if (index === 0) return 'primary'
  if (index === 1) return 'warning'
  return 'info'
}

const refreshSnapshots = async () => {
  loading.value = true
  try {
    const response = await getSnapshots()
    snapshots.value = response.data.snapshots || []
    if (snapshots.value.length > 0) {
      currentVersion.value = snapshots.value[0]
    }
  } catch (error) {
    ElMessage.error('获取配置历史失败')
  } finally {
    loading.value = false
  }
}

const confirmRollback = (snapshot: Snapshot) => {
  selectedSnapshot.value = snapshot
  rollbackVisible.value = true
}

const executeRollback = async () => {
  if (!selectedSnapshot.value) return
  
  rollingBack.value = selectedSnapshot.value.id
  try {
    await rollback({ snapshot_id: selectedSnapshot.value.id })
    ElMessage.success('回滚成功，正在重启服务...')
    rollbackVisible.value = false
    
    // 等待几秒后跳转到状态页
    setTimeout(() => {
      router.push('/status')
    }, 3000)
  } catch (error) {
    ElMessage.error('回滚失败')
  } finally {
    rollingBack.value = null
  }
}

const viewDiff = async (snapshot: Snapshot) => {
  diffVisible.value = true
  diffLoading.value = true
  try {
    const response = await getSnapshotDiff(snapshot.id)
    diffData.value = response.data.diff || '无变更详情'
  } catch (error) {
    diffData.value = '获取变更详情失败'
  } finally {
    diffLoading.value = false
  }
}

onMounted(() => {
  refreshSnapshots()
})
</script>

<style scoped>
.snapshots-view { max-width: 900px; margin: 0 auto; padding: 20px; }
.page-header { margin-bottom: 24px; }
.page-header h2 { margin: 0 0 8px 0; }
.page-header .desc { color: #606266; margin: 0; }
.toolbar { margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center; }
.snapshot-item { padding: 8px; }
.snapshot-header { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; }
.snapshot-id { font-family: monospace; font-weight: 600; color: #409eff; }
.snapshot-message { color: #303133; margin-bottom: 8px; line-height: 1.5; }
.snapshot-meta { font-size: 13px; color: #909399; margin-bottom: 12px; display: flex; gap: 16px; }
.snapshot-actions { display: flex; gap: 8px; }
.rollback-confirm .confirm-details { margin: 16px 0; padding: 12px; background: #f5f7fa; border-radius: 4px; }
.rollback-confirm .confirm-details p { margin: 4px 0; }
.diff-content pre { background: #1e1e1e; color: #d4d4d4; padding: 16px; border-radius: 4px; overflow-x: auto; font-size: 12px; line-height: 1.5; max-height: 500px; overflow-y: auto; }
.diff-loading { padding: 20px; }
</style>
