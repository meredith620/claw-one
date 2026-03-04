import axios from 'axios'
import type {
  Config,
  StateResponse,
  SnapshotsResponse,
  ApplyConfigRequest,
  ApplyConfigResponse,
  RollbackRequest,
} from '../types'

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
})

// 健康检查
export const getHealth = () => api.get('/health')

// 状态
export const getState = () => api.get<StateResponse>('/state')

// 配置
export const getConfig = () => api.get<Config>('/config')
export const applyConfig = (data: ApplyConfigRequest) =>
  api.post<ApplyConfigResponse>('/config', data)

// 快照
export const getSnapshots = () => api.get<SnapshotsResponse>('/snapshots')

// 回滚
export const rollback = (data: RollbackRequest) =>
  api.post('/rollback', data)

// 重启
export const restartService = () => api.post('/restart')

// 日志
export const getLogs = () => api.get('/logs')

// 首次启动向导
export const checkFirstSetup = () => api.get<{ is_first_setup: boolean }>('/setup/check')
export const completeSetup = () => api.post('/setup/complete')
export const resetToFactory = () => api.post('/setup/reset')

export default api