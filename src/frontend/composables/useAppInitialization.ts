import { ref } from 'vue'
import { useFontManager } from './useFontManager'
import { initMcpTools } from './useMcpTools'
import { useSettings } from './useSettings'
import { useVersionCheck } from './useVersionCheck'

/**
 * 应用初始化组合式函数
 */
export function useAppInitialization(mcpHandler: ReturnType<typeof import('./useMcpHandler').useMcpHandler>) {
  const isInitializing = ref(true)
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

      // 主题已在useTheme初始化时加载，这里不需要重复加载

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

      // 设置窗口焦点监听器，用于配置同步
      await settings.setupWindowFocusListener()

      // 在MCP模式下，确保前端状态与后端窗口状态同步
      if (isMcp) {
        console.log('MCP模式检测到，同步窗口状态...')
        try {
          await settings.syncWindowStateFromBackend()
        }
        catch (error) {
          console.warn('MCP模式状态同步失败，继续初始化:', error)
        }
      }

      // 初始化MCP工具配置（在非MCP模式下）
      if (!isMcp) {
        await initMcpTools()
        await setupMcpEventListener()
      }

      // 如果是首次启动，标记已初始化（主题已在上面加载过）
      if (isFirstRun) {
        console.log('检测到首次启动，标记应用已初始化')
        markAsInitialized()
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
