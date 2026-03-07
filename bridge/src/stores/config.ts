import { reactive } from 'vue'

// 全局配置状态管理
export const configState = reactive({
  hasChanges: false,
  lastSaved: null as Date | null,
  pendingReload: false,
})

export function markConfigChanged() {
  configState.hasChanges = true
}

export function markConfigSaved() {
  configState.hasChanges = false
  configState.lastSaved = new Date()
}

export function requestConfigReload() {
  configState.pendingReload = true
}

export function completeConfigReload() {
  configState.pendingReload = false
}
