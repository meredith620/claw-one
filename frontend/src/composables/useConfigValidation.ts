import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { validateConfig } from '../api'

export interface ValidationError {
  path: string
  message: string
}

export interface ValidationResult {
  valid: boolean
  errors: ValidationError[]
  warnings: ValidationError[]
}

export function useConfigValidation() {
  const validating = ref(false)
  const validationResult = ref<ValidationResult | null>(null)

  /**
   * 验证配置
   */
  async function validate(config: any): Promise<ValidationResult> {
    validating.value = true
    try {
      const response = await validateConfig(config)
      validationResult.value = response.data
      return response.data
    } catch (error: any) {
      const errorMsg = error.response?.data?.error || '验证失败'
      ElMessage.error(errorMsg)
      return {
        valid: false,
        errors: [{ path: '', message: errorMsg }],
        warnings: []
      }
    } finally {
      validating.value = false
    }
  }

  /**
   * 验证并显示结果
   */
  async function validateAndShow(config: any): Promise<boolean> {
    const result = await validate(config)
    
    if (!result.valid || result.errors.length > 0) {
      // 显示错误
      const errorMessages = result.errors.map(e => 
        e.path ? `${e.path}: ${e.message}` : e.message
      ).join('\n')
      
      ElMessage.error({
        message: `配置验证失败:\n${errorMessages}`,
        duration: 5000,
        showClose: true
      })
      return false
    }

    // 显示警告（如果有）
    if (result.warnings.length > 0) {
      const warningMessages = result.warnings.map(e => 
        e.path ? `${e.path}: ${e.message}` : e.message
      ).join('\n')
      
      ElMessage.warning({
        message: `配置警告:\n${warningMessages}`,
        duration: 5000,
        showClose: true
      })
    }

    return true
  }

  /**
   * 验证通过后才保存
   */
  async function validateBeforeSave(
    config: any, 
    saveFn: () => Promise<void>
  ): Promise<boolean> {
    const isValid = await validateAndShow(config)
    if (!isValid) {
      return false
    }

    try {
      await saveFn()
      ElMessage.success('配置保存成功')
      return true
    } catch (error: any) {
      ElMessage.error('保存失败: ' + (error.response?.data?.error || error.message))
      return false
    }
  }

  return {
    validating,
    validationResult,
    validate,
    validateAndShow,
    validateBeforeSave
  }
}
