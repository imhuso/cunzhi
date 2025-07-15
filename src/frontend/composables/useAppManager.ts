import { computed } from 'vue'
import { useAppInitialization } from './useAppInitialization'
import { useAudioManager } from './useAudioManager'
import { useMcpHandler } from './useMcpHandler'
import { useSettings } from './useSettings'
import { useTheme } from './useTheme'

/**
 * 统一的应用管理器
 * 封装所有组合式函数，提供简洁的API
 */
export function useAppManager() {
  // 初始化各个模块
  const theme = useTheme()
  const settings = useSettings()
  const audioManager = useAudioManager()
  const mcpHandler = useMcpHandler()
  const appInit = useAppInitialization(mcpHandler)

  // 创建统一的配置对象
  const appConfig = computed(() => {
    const config = {
      theme: theme.currentTheme.value,
      window: {
        alwaysOnTop: settings.alwaysOnTop.value,
        width: settings.windowWidth.value,
        height: settings.windowHeight.value,
        fixed: settings.fixedWindowSize.value,
      },
      audio: {
        enabled: settings.audioNotificationEnabled.value,
        url: settings.audioUrl.value,
      },
      reply: {
        enabled: settings.continueReplyEnabled.value,
        prompt: settings.continuePrompt.value,
      },
    }

    return config
  })

  // 创建统一的操作对象
  const actions = {
    // 主题操作
    theme: {
      setTheme: theme.setTheme,
    },
    // 设置操作
    settings: {
      toggleAlwaysOnTop: settings.toggleAlwaysOnTop,
      toggleAudioNotification: settings.toggleAudioNotification,
      updateAudioUrl: settings.updateAudioUrl,
      testAudio: settings.testAudioSound,
      stopAudio: settings.stopAudioSound,
      updateWindowSize: settings.updateWindowSize,
      updateReplyConfig: settings.updateReplyConfig,
      setMessageInstance: settings.setMessageInstance,
      reloadAllSettings: settings.reloadAllSettings,
    },
    // MCP操作
    mcp: {
      handleResponse: mcpHandler.handleMcpResponse,
      handleCancel: mcpHandler.handleMcpCancel,
    },
    // 音频操作
    audio: {
      handleTestError: audioManager.handleTestAudioError,
    },
    // 应用操作
    app: {
      initialize: appInit.initializeApp,
      cleanup: () => {
        // 清理窗口焦点监听器
        settings.removeWindowFocusListener()
      },
    },
  }

  // 返回状态和操作 - 保持响应式
  return {
    // 直接解构状态，Vue模板会自动处理响应式
    naiveTheme: theme.naiveTheme,
    mcpRequest: mcpHandler.mcpRequest,
    showMcpPopup: mcpHandler.showMcpPopup,
    appConfig,
    isInitializing: appInit.isInitializing,

    // 操作
    actions,
  }
}
