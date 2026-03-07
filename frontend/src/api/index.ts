import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
})

// 健康检查
export const getHealth = () => api.get('/health')

// 状态
export const getState = () => api.get('/state')

// 完整配置
export const getConfig = () => api.get('/config')
export const applyConfig = (data: { config: any; message?: string }) =>
  api.post('/config', data)

// Provider 配置 API
export const getProviders = () => api.get('/providers')
export const getProvider = (id: string) => api.get(`/providers/${id}`)
export const saveProvider = (id: string, data: any) => api.post(`/providers/${id}`, data)
export const deleteProvider = (id: string) => api.delete(`/providers/${id}`)

// 模型优先级 API
export const getModelPriority = () => api.get('/model-priority')
export const saveModelPriority = (data: { primary: string; fallbacks: string[] }) => 
  api.post('/model-priority', data)

// Agent 配置 API
export const getAgents = () => api.get('/agents')
export const saveAgents = (data: any) => api.post('/agents', data)

// Memory 配置 API
export const getMemory = () => api.get('/memory')
export const saveMemory = (data: any) => api.post('/memory', data)

// Channel 配置 API
export const getChannels = () => api.get('/channels')
export const saveChannels = (data: any) => api.post('/channels', data)

// 快照
export const getSnapshots = () => api.get('/snapshots')
export const getSnapshotDiff = (id: string) => api.get(`/snapshots/${id}/diff`)

// 回滚
export const rollback = (data: { snapshot_id: string }) =>
  api.post('/rollback', data)

// 配置验证
export const validateConfig = (data: { config: any }) =>
  api.post('/config/validate', data)

// 重启
export const restartService = () => api.post('/restart')

// 日志
export const getLogs = () => api.get('/logs')

// 首次启动向导
export const checkFirstSetup = () => api.get('/setup/check')
export const completeSetup = () => api.post('/setup/complete')
export const resetToFactory = () => api.post('/setup/reset')

export default api
