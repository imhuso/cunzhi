import { nextTick, ref } from 'vue'
import { useFontManager } from './useFontManager'
import { initMcpTools } from './useMcpTools'
import { useSettings } from './useSettings'
import { useTheme } from './useTheme'
import { useVersionCheck } from './useVersionCheck'

/**
 * 应用初始化组合式函数
 */
export function useAppInitialization(mcpHandler: ReturnType<typeof import('./useMcpHandler').useMcpHandler>) {
  const isInitializing = ref(true)
  const { loadTheme } = useTheme()
  const { loadFontConfig, loadFontOptions } = useFontManager()
  const settings = useSettings()
  const { autoCheckUpdate } = useVersionCheck()
  const { checkMcpMode, setupMcpEventListener } = mcpHandler

  /**
   * 检查是否为首次启动
   */
  function checkFirstRun(): boolean {
    // 检查localStorage是否有初始化标记
    const hasInitialized = localStorage.getItem('app-initialized')
    return !hasInitialized
  }

  /**
   * 标记应用已初始化
   */
  function markAsInitialized() {
    localStorage.setItem('app-initialized', 'true')
  }

  /**
   * 初始化应用
   */
  async function initializeApp() {
    try {
      // 检查是否为首次启动
      const isFirstRun = checkFirstRun()

      // 加载主题设置
      await loadTheme()

      // 加载字体设置
      await Promise.all([
        loadFontConfig(),
        loadFontOptions(),
      ])

      // 检查是否为MCP模式
      const { isMcp, mcpContent } = await checkMcpMode()

      // 无论是否为MCP模式，都加载窗口设置
      await settings.loadWindowSettings()
      await settings.loadWindowConfig()

      // 在MCP模式下，确保前端状态与后端窗口状态同步
      if (isMcp) {
        console.log('MCP模式检测到，等待后端完全初始化...')
        // 给后端一些时间完全初始化
        await new Promise(resolve => setTimeout(resolve, 200))
        await settings.syncWindowStateFromBackend()

        // 在下一个tick再次验证状态，确保UI渲染时状态正确
        nextTick(() => {
          setTimeout(async () => {
            console.log('UI渲染后再次验证窗口状态...')
            await settings.syncWindowStateFromBackend()
          }, 100)
        })
      }

      // 初始化MCP工具配置（在非MCP模式下）
      if (!isMcp) {
        await initMcpTools()
        await setupMcpEventListener()
      }

      // 如果是首次启动，重新加载主题设置以确保正确
      if (isFirstRun) {
        console.log('检测到首次启动，重新加载主题设置以确保正确')
        try {
          // 重新加载主题设置
          await loadTheme()
          // 标记已初始化
          markAsInitialized()
        }
        catch (error) {
          console.warn('首次启动重新加载主题失败:', error)
          // 即使失败也标记为已初始化，避免每次都重试
          markAsInitialized()
        }
      }

      // 自动检查版本更新并弹窗（非阻塞）
      autoCheckUpdate().catch(() => {
        // 静默处理版本检查失败
      })

      // 结束初始化状态
      isInitializing.value = false

      return { isMcp, mcpContent }
    }
    catch (error) {
      console.error('应用初始化失败:', error)
      isInitializing.value = false
      throw error
    }
  }

  return {
    isInitializing,
    initializeApp,
  }
}
