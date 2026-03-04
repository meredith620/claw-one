// API 类型定义

export interface Config {
  version: string
  gateway: GatewayConfig
  models: ModelConfig[]
  channels: ChannelConfig[]
}

export interface GatewayConfig {
  port: number
  bind: string
}

export interface ModelConfig {
  id: string
  provider: string
  apiKey?: string
}

export interface ChannelConfig {
  id: string
  type: string
  enabled: boolean
}

export type OpenClawState =
  | { type: 'running' }
  | { type: 'starting' }
  | { type: 'stopped' }
  | { type: 'unknown' }
  | { type: 'config_error'; error: string; auto_rolled_back: boolean }
  | { type: 'system_error'; error: string }

export interface StateResponse {
  state: OpenClawState
  current_version?: string
  last_error?: string
  can_rollback: boolean
}

export interface Snapshot {
  id: string
  timestamp: string
  message: string
}

export interface SnapshotsResponse {
  snapshots: Snapshot[]
}

export interface ApplyConfigRequest {
  config: Config
  message?: string
}

export interface ApplyConfigResponse {
  success: boolean
  message: string
  commit_id?: string
}

export interface RollbackRequest {
  snapshot_id: string
}

export interface LogsResponse {
  logs: string
}