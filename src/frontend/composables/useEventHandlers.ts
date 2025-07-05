/**
 * 事件处理器封装
 * 将复杂的事件传递简化为可复用的处理器
 */
export function useEventHandlers(actions: any) {
  return {
    // MCP 事件
    onMcpResponse: actions.mcp.handleResponse,
    onMcpCancel: actions.mcp.handleCancel,

    // 主题事件
    onThemeChange: actions.theme.setTheme,

    // 设置事件
    onToggleAlwaysOnTop: actions.settings.toggleAlwaysOnTop,
    onToggleAudioNotification: actions.settings.toggleAudioNotification,
    onUpdateAudioUrl: actions.settings.updateAudioUrl,
    onTestAudio: actions.settings.testAudio,
    onStopAudio: actions.settings.stopAudio,
    onUpdateWindowSize: actions.settings.updateWindowSize,
    onUpdateReplyConfig: actions.settings.updateReplyConfig,
    onMessageReady: actions.settings.setMessageInstance,

    // 音频事件
    onTestAudioError: actions.audio.handleTestError,

    // 配置事件
    onConfigReloaded: actions.settings.reloadAllSettings,
  }
}
