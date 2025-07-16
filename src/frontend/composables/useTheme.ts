import { invoke } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'
import { applyThemeVariables, getTheme } from '../theme'

export function useTheme() {
  // 使用默认深色主题，等待后端加载
  const currentTheme = ref('dark')

  // 立即应用默认主题，避免初始化闪烁
  applyThemeVariables('dark')

  // 计算 Naive UI 主题
  const naiveTheme = computed(() => {
    return getTheme(currentTheme.value)
  })

  // 应用主题
  function applyTheme(theme: string) {
    // 应用主题变量和类
    applyThemeVariables(theme)
    currentTheme.value = theme
  }

  // 切换主题
  async function setTheme(theme: string) {
    try {
      // 先保存到后端
      await invoke('set_theme', { theme })
      // 保存成功后应用前端主题
      applyTheme(theme)
    }
    catch (error) {
      console.error('保存主题设置失败:', error)
      // 保存失败时不应用主题，保持一致性
    }
  }

  // 加载主题设置
  async function loadTheme() {
    try {
      const theme = await invoke('get_theme')
      // 确保主题值有效
      const validTheme = (theme === 'light' || theme === 'dark') ? theme : 'dark'

      // 应用后端主题（无论是否与当前主题相同，确保状态同步）
      applyTheme(validTheme as string)
    }
    catch (error) {
      console.error('加载主题失败:', error)
      // 加载失败时保持默认深色主题
      applyTheme('dark')
    }
  }

  return {
    currentTheme,
    naiveTheme,
    setTheme,
    loadTheme,
  }
}
