import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { ref } from 'vue'

// 单例实例
let settingsInstance: ReturnType<typeof createSettings> | null = null

function createSettings() {
  const alwaysOnTop = ref(true) // 与后端默认值保持一致
  const audioNotificationEnabled = ref(true)
  const audioUrl = ref('')
  const windowConfig = ref({
    auto_resize: true,
    max_width: 1500,
    max_height: 1000,
    min_width: 600,
    min_height: 400,
  })
  const windowWidth = ref(600)
  const windowHeight = ref(900)
  const fixedWindowSize = ref(false)

  // 继续回复设置
  const continueReplyEnabled = ref(true)
  const continuePrompt = ref('请按照最佳实践继续')

  // 窗口约束和 UI 常量
  const windowConstraints = ref({
    min_width: 600,
    min_height: 400,
    max_width: 1500,
    max_height: 1000,
    resize_step: 50,
    resize_throttle_ms: 1000,
    size_update_delay_ms: 500,
    size_check_delay_ms: 100,
  })

  // Naive UI 消息实例
  let message: any = null

  // 节流定时器
  let resizeThrottleTimer: number | null = null

  // 窗口大小变化监听器
  let windowResizeUnlisten: (() => void) | null = null

  function setMessageInstance(messageInstance: any) {
    message = messageInstance
  }

  function getMessageInstance() {
    return message
  }

  // 加载窗口约束
  async function loadWindowConstraints() {
    try {
      const constraints = await invoke('get_window_constraints_cmd')
      if (constraints) {
        windowConstraints.value = constraints as any
      }
    }
    catch (error) {
      console.error('加载窗口约束失败:', error)
    }
  }

  // 加载窗口设置
  async function loadWindowSettings() {
    try {
      // 首先加载窗口约束
      await loadWindowConstraints()

      const enabled = await invoke('get_always_on_top')
      alwaysOnTop.value = enabled as boolean

      // 加载音频通知设置
      const audioEnabled = await invoke('get_audio_notification_enabled')
      audioNotificationEnabled.value = audioEnabled as boolean

      // 加载音效URL设置
      const audioUrlValue = await invoke('get_audio_url')
      audioUrl.value = audioUrlValue as string

      // 加载窗口尺寸和模式设置
      try {
        const windowSettings = await invoke('get_window_settings')
        if (windowSettings) {
          const settings = windowSettings as any
          windowWidth.value = settings.current_width || 600
          windowHeight.value = settings.current_height || 900
          fixedWindowSize.value = settings.fixed || false
        }
      }
      catch {
        console.log('窗口设置不存在，使用默认值')
      }

      // 加载继续回复设置
      try {
        const replyConfig = await invoke('get_reply_config')
        if (replyConfig) {
          const config = replyConfig as any
          continueReplyEnabled.value = config.enable_continue_reply || true
          continuePrompt.value = config.continue_prompt || '请按照最佳实践继续'
        }
      }
      catch {
        console.log('继续回复设置不存在，使用默认值')
      }

      // 同步窗口状态
      await invoke('sync_window_state')

      // 根据当前模式设置监听器
      if (!fixedWindowSize.value) {
        await setupWindowResizeListener()
      }
    }
    catch (error) {
      console.error('加载窗口设置失败:', error)
    }
  }

  // 切换置顶设置
  async function toggleAlwaysOnTop() {
    try {
      const newValue = !alwaysOnTop.value
      await invoke('set_always_on_top', { enabled: newValue })
      alwaysOnTop.value = newValue
    }
    catch (error) {
      console.error('切换置顶设置失败:', error)
    }
  }

  // 从后端同步窗口状态（用于MCP弹窗启动时确保状态一致）
  async function syncWindowStateFromBackend() {
    try {
      console.log('同步窗口状态...')

      // 重新获取后端的置顶状态
      const backendAlwaysOnTop = await invoke('get_always_on_top')
      alwaysOnTop.value = backendAlwaysOnTop as boolean

      // 同步窗口状态到前端
      await invoke('sync_window_state')

      console.log('窗口状态已从后端同步:', {
        alwaysOnTop: alwaysOnTop.value,
      })
    }
    catch (error) {
      console.error('同步窗口状态失败:', error)
      throw error
    }
  }

  // 切换音频通知设置
  async function toggleAudioNotification() {
    try {
      const newValue = !audioNotificationEnabled.value
      await invoke('set_audio_notification_enabled', { enabled: newValue })
      audioNotificationEnabled.value = newValue
    }
    catch (error) {
      console.error('切换音频通知设置失败:', error)
    }
  }

  // 更新音效URL
  async function updateAudioUrl(url: string) {
    try {
      await invoke('set_audio_url', { url })
      audioUrl.value = url
    }
    catch (error) {
      console.error('更新音效URL失败:', error)
    }
  }

  // 测试音效
  async function testAudioSound() {
    try {
      await invoke('test_audio_sound')
      // 只有在成功时才显示成功提示
      if (message) {
        message.success('音效测试播放成功')
      }
    }
    catch (error) {
      // 显示错误提示给用户
      console.error('音效测试失败:', error)
      if (message) {
        message.error(`音效测试失败: ${error}`)
      }
    }
  }

  // 停止音效
  async function stopAudioSound() {
    try {
      await invoke('stop_audio_sound')
    }
    catch (error) {
      console.error('停止音效失败:', error)
    }
  }

  // 更新窗口大小
  async function updateWindowSize(size: { width: number, height: number, fixed: boolean }) {
    try {
      // update_window_size 命令已经会同时更新 WindowConfig 中的所有相关设置
      await invoke('update_window_size', { sizeUpdate: size })

      // 更新本地状态
      windowWidth.value = size.width
      windowHeight.value = size.height
      fixedWindowSize.value = size.fixed

      // 根据模式设置监听器
      if (size.fixed) {
        // 固定模式：移除监听器
        removeWindowResizeListener()
      }
      else {
        // 自由拉伸模式：启用监听器
        await setupWindowResizeListener()
      }

      if (message) {
        const mode = size.fixed ? '固定大小' : '自由拉伸'
        message.success(`窗口设置已更新：${mode} ${size.width}x${size.height}`)
      }
    }
    catch (error) {
      console.error('更新窗口大小失败:', error)
      if (message) {
        message.error(`更新窗口大小失败: ${error}`)
      }
    }
  }

  // 节流保存窗口尺寸
  function throttledSaveWindowSize() {
    if (resizeThrottleTimer) {
      clearTimeout(resizeThrottleTimer)
    }

    resizeThrottleTimer = window.setTimeout(async () => {
      try {
        // 只在自由拉伸模式下保存
        if (!fixedWindowSize.value) {
          // 获取当前窗口的逻辑尺寸
          const result = await invoke('get_current_window_size')
          if (result && typeof result === 'object') {
            const { width, height } = result as any

            // 验证尺寸，如果小于最小限制则调整为最小尺寸
            let adjustedWidth = width
            let adjustedHeight = height
            let wasAdjusted = false

            if (width < windowConstraints.value.min_width) {
              adjustedWidth = windowConstraints.value.min_width
              wasAdjusted = true
            }
            if (height < windowConstraints.value.min_height) {
              adjustedHeight = windowConstraints.value.min_height
              wasAdjusted = true
            }

            if (wasAdjusted) {
              console.log(`窗口尺寸已调整: ${width}x${height} -> ${adjustedWidth}x${adjustedHeight}`)
            }

            await invoke('set_window_settings', {
              windowSettings: {
                free_width: adjustedWidth,
                free_height: adjustedHeight,
                fixed: false,
              },
            })

            // 更新本地状态
            windowWidth.value = adjustedWidth
            windowHeight.value = adjustedHeight

            console.log(`窗口尺寸已保存: ${adjustedWidth}x${adjustedHeight}`)
          }
        }
      }
      catch (error) {
        // 如果是窗口最小化或尺寸过小的错误，静默处理
        if (error && typeof error === 'string'
          && (error.includes('窗口已最小化') || error.includes('窗口尺寸过小'))) {
          console.debug('跳过窗口尺寸保存:', error)
        }
        else {
          console.error('保存窗口尺寸失败:', error)
        }
      }
    }, windowConstraints.value.resize_throttle_ms) // 使用配置的节流时间
  }

  // 设置窗口大小变化监听器
  async function setupWindowResizeListener() {
    try {
      const webview = getCurrentWebviewWindow()

      // 移除之前的监听器
      if (windowResizeUnlisten) {
        windowResizeUnlisten()
      }

      // 监听窗口大小变化
      windowResizeUnlisten = await webview.onResized(() => {
        throttledSaveWindowSize()
      })

      console.log('窗口大小变化监听器已设置')
    }
    catch (error) {
      console.error('设置窗口大小变化监听器失败:', error)
    }
  }

  // 移除窗口大小变化监听器
  function removeWindowResizeListener() {
    if (windowResizeUnlisten) {
      windowResizeUnlisten()
      windowResizeUnlisten = null
    }

    if (resizeThrottleTimer) {
      clearTimeout(resizeThrottleTimer)
      resizeThrottleTimer = null
    }
  }

  // 加载窗口配置
  async function loadWindowConfig() {
    try {
      const config = await invoke('get_window_config')
      windowConfig.value = config as any
    }
    catch (error) {
      console.error('加载窗口配置失败:', error)
    }
  }

  // 更新继续回复设置
  async function updateReplyConfig(config: { enable_continue_reply?: boolean, continue_prompt?: string }) {
    try {
      await invoke('set_reply_config', { config })

      if (config.enable_continue_reply !== undefined) {
        continueReplyEnabled.value = config.enable_continue_reply
      }
      if (config.continue_prompt !== undefined) {
        continuePrompt.value = config.continue_prompt
      }

      if (message) {
        message.success('继续回复设置已更新')
      }
    }
    catch (error) {
      console.error('更新继续回复设置失败:', error)
      if (message) {
        message.error('更新继续回复设置失败')
      }
    }
  }

  // 重新加载所有设置
  async function reloadAllSettings() {
    try {
      // 首先重新加载后端配置文件到内存
      await invoke('reload_config')

      // 然后重新加载前端设置
      await loadWindowSettings()
      await loadWindowConfig()
    }
    catch (error) {
      console.error('重新加载设置失败:', error)
    }
  }

  // 窗口焦点监听器
  let windowFocusUnlisten: (() => void) | null = null

  // 设置窗口焦点监听
  async function setupWindowFocusListener() {
    try {
      const webview = getCurrentWebviewWindow()

      // 监听窗口获得焦点事件
      windowFocusUnlisten = await webview.onFocusChanged(({ payload: focused }) => {
        if (focused) {
          reloadAllSettings()
        }
      })
    }
    catch (error) {
      console.error('设置窗口焦点监听器失败:', error)
    }
  }

  // 移除窗口焦点监听器
  function removeWindowFocusListener() {
    if (windowFocusUnlisten) {
      windowFocusUnlisten()
      windowFocusUnlisten = null
    }
  }

  return {
    // 状态
    alwaysOnTop,
    audioNotificationEnabled,
    audioUrl,
    windowConfig,
    windowWidth,
    windowHeight,
    fixedWindowSize,
    windowConstraints,
    continueReplyEnabled,
    continuePrompt,

    // 方法
    setMessageInstance,
    getMessageInstance,
    loadWindowSettings,
    loadWindowConstraints,
    toggleAlwaysOnTop,
    syncWindowStateFromBackend,
    toggleAudioNotification,
    updateAudioUrl,
    testAudioSound,
    stopAudioSound,
    updateWindowSize,
    updateReplyConfig,
    loadWindowConfig,
    setupWindowResizeListener,
    removeWindowResizeListener,
    reloadAllSettings,
    setupWindowFocusListener,
    removeWindowFocusListener,
  }
}

export function useSettings() {
  if (!settingsInstance) {
    settingsInstance = createSettings()
  }
  return settingsInstance
}
