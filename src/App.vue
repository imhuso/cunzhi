<script setup>
import {
  RobotOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { nextTick, onMounted, ref } from 'vue'
import RequestHandler from './components/RequestHandler.vue'
import SettingsModal from './components/SettingsModal.vue'

// 响应式数据
const appInfo = ref('')
const currentRequest = ref(null)
const isConnected = ref(false)
const chatHistory = ref([])
const showSettings = ref(false)
const replyText = ref('')
const inputFocused = ref(false)
const messagesContainer = ref(null)

// 聊天历史管理（限制数量以优化性能）
const MAX_HISTORY_ITEMS = 100

function addToHistory(type, content, id = null) {
  const message = {
    id: id || Date.now().toString(),
    type, // 'incoming' 或 'outgoing'
    content,
    timestamp: new Date(),
  }

  chatHistory.value.push(message)

  // 限制历史记录数量
  if (chatHistory.value.length > MAX_HISTORY_ITEMS) {
    chatHistory.value.shift()
  }

  // 滚动到底部
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

function formatTime(date) {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}



// 检查IPC连接状态
async function checkConnectionStatus() {
  try {
    const status = await invoke('check_ipc_status')
    isConnected.value = status
    return status
  }
  catch (error) {
    console.error('❌ 检查IPC状态失败:', error)
    isConnected.value = false
    return false
  }
}

// 初始化应用
onMounted(async () => {
  console.warn('🚀 AI Review Vue App 初始化中...')
  console.warn('🔧 Tauri API 可用性检查:', !!window.__TAURI__)

  try {
    // 获取应用信息
    appInfo.value = await invoke('get_app_info')
    console.warn('✅ 应用信息获取成功:', appInfo.value)
  }
  catch (error) {
    console.error('❌ 获取应用信息失败:', error)
    appInfo.value = 'AI Review App v0.1.0'
  }

  // 监听新请求事件
  try {
    console.warn('🔧 开始设置事件监听器...')
    await listen('new-request', (event) => {
      console.warn('🎯 收到新请求事件:', event)
      const message = event.payload
      console.warn('📨 解析后的消息:', message)

      // 添加到聊天历史
      addToHistory('incoming', message.content, message.id)

      // 设置当前请求
      currentRequest.value = message
      console.warn('📨 currentRequest已更新:', currentRequest.value)
    })
    console.warn('✅ 事件监听器设置成功')
  }
  catch (error) {
    console.error('❌ 设置事件监听器失败:', error)
  }

  // 初始检查连接状态
  await checkConnectionStatus()

  // 定期检查连接状态（每5秒）
  setInterval(checkConnectionStatus, 5000)
})

// 处理用户回复
async function handleResponse(response) {
  if (!currentRequest.value)
    return

  try {
    // 添加回复到聊天历史
    addToHistory('outgoing', response)

    await invoke('respond_to_request', {
      requestId: currentRequest.value.id,
      response,
    })
    console.warn('✅ 回复发送成功:', response)
    currentRequest.value = null
  }
  catch (error) {
    console.error('❌ 发送回复失败:', error)
    console.error(`发送回复失败: ${error}`)
  }
}

// 处理取消操作
async function handleCancel() {
  if (!currentRequest.value)
    return

  try {
    // 添加取消信息到聊天历史
    addToHistory('outgoing', '[已取消]')

    await invoke('respond_to_request', {
      requestId: currentRequest.value.id,
      response: '[我对你的问题没有更好的意见，你可以按照最佳实践来继续做]',
    })
    console.warn('✅ 请求已取消')
    currentRequest.value = null
  }
  catch (error) {
    console.error('❌ 取消请求失败:', error)
  }
}

// 打开设置弹窗
function openSettings() {
  showSettings.value = true
}

// 关闭设置弹窗
function closeSettings() {
  showSettings.value = false
}

// 输入框焦点处理
function onInputFocus() {
  inputFocused.value = true
}

function onInputBlur() {
  inputFocused.value = false
}

// 键盘事件处理
function handleEnterKey(event) {
  // 普通Enter键只是换行，不发送
  // 不阻止默认行为，允许换行
}

function handleCmdEnter(event) {
  // CMD+Enter 发送消息
  event.preventDefault()
  sendReply()
}

// 发送回复
async function sendReply() {
  if (!currentRequest.value || !replyText.value.trim()) {
    return
  }

  try {
    const response = await invoke('respond_to_request', {
      requestId: currentRequest.value.id,
      response: replyText.value.trim(),
    })

    // 清空输入框
    replyText.value = ''

    // 清除当前请求
    currentRequest.value = null

    console.warn('✅ 回复发送成功:', response)
  }
  catch (error) {
    console.error('❌ 发送回复失败:', error)
  }
}


</script>

<template>
  <div class="app-container">
    <!-- 自定义标题栏 -->
    <div class="title-bar" data-tauri-drag-region>
      <div class="title-content">
        <div class="app-title">
          <RobotOutlined class="app-icon" />
          <span class="app-name">AI Review</span>
        </div>
        <div class="status-indicator">
          <a-badge
            :status="isConnected ? 'success' : 'error'"
            :text="isConnected ? '已连接' : '连接中...'"
            class="status-badge"
          />
        </div>
      </div>
      <div class="toolbar">
        <a-button
          type="text"
          size="small"
          @click="openSettings"
          title="设置"
        >
          <SettingOutlined />
        </a-button>
      </div>
    </div>

    <!-- 聊天主界面 -->
    <div class="chat-container">
      <!-- 聊天消息区域 -->
      <div class="chat-messages" ref="messagesContainer">
        <!-- 欢迎消息 -->
        <div v-if="chatHistory.length === 0" class="welcome-message">
          <div class="welcome-content">
            <RobotOutlined class="welcome-icon" />
            <h3>AI Review 助手</h3>
            <p>{{ appInfo }}</p>
            <p class="status-text">
              等待命令行消息...
            </p>
          </div>
        </div>

        <!-- 聊天消息列表 -->
        <div v-for="item in chatHistory" :key="item.id" class="message-item" :class="item.type">
          <div class="message-bubble">
            <div class="message-content">{{ item.content }}</div>
            <div class="message-time">{{ formatTime(item.timestamp) }}</div>
          </div>
        </div>
      </div>

      <!-- 当前请求处理区域 - 固定在底部 -->
      <div v-if="currentRequest" class="current-request-area">
        <RequestHandler
          :request="currentRequest"
          @response="handleResponse"
          @cancel="handleCancel"
        />
      </div>

      <!-- 等待状态输入区域 - 固定在底部 -->
      <div v-else class="waiting-input-area">
        <div class="waiting-message">
          <span>等待新消息...</span>
        </div>
      </div>
    </div>

    <!-- 设置弹窗 -->
    <SettingsModal
      v-model:visible="showSettings"
      @close="closeSettings"
    />
  </div>
</template>

<style scoped>
.app-container {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  overflow: hidden;
  position: relative;
}

/* 自定义标题栏 */
.title-bar {
  height: 50px;
  background: #fafafa;
  border-bottom: 1px solid #d9d9d9;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  user-select: none;
  flex-shrink: 0;
}

.title-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #262626;
  font-weight: 500;
  font-size: 16px;
}

.app-icon {
  font-size: 18px;
  color: var(--ant-primary-color, #1890ff);
}

.app-name {
  color: #262626;
}

.status-indicator {
  display: flex;
  align-items: center;
}

.status-badge :deep(.ant-badge-status-text) {
  color: #595959;
  font-size: 13px;
  font-weight: 400;
}

.toolbar {
  display: flex;
  gap: 4px;
}

/* 聊天容器 */
.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

/* 聊天消息区域 */
.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background: #f5f5f5;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 欢迎消息 */
.welcome-message {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.welcome-content {
  text-align: center;
  color: #666;
}

.welcome-icon {
  font-size: 48px;
  color: var(--ant-primary-color, #1890ff);
  margin-bottom: 16px;
}

.welcome-content h3 {
  margin: 16px 0 8px 0;
  color: #333;
}

.welcome-content p {
  margin: 4px 0;
  color: #666;
}

.status-text {
  font-size: 14px;
  color: #999;
}

/* 消息气泡 */
.message-item {
  display: flex;
  max-width: 70%;
  margin-bottom: 8px;
}

.message-item.incoming {
  align-self: flex-start;
}

.message-item.outgoing {
  align-self: flex-end;
}

.message-bubble {
  padding: 12px 16px;
  border-radius: 18px;
  position: relative;
  word-wrap: break-word;
  max-width: 100%;
}

.message-item.incoming .message-bubble {
  background: #ffffff;
  border: 1px solid #e1e1e1;
  border-bottom-left-radius: 4px;
}

.message-item.outgoing .message-bubble {
  background: var(--ant-primary-color, #1890ff);
  color: white;
  border-bottom-right-radius: 4px;
}

.message-content {
  font-size: 14px;
  line-height: 1.4;
  margin-bottom: 4px;
}

.message-time {
  font-size: 11px;
  opacity: 0.7;
  text-align: right;
}

.message-item.incoming .message-time {
  color: #999;
}

.message-item.outgoing .message-time {
  color: rgba(255, 255, 255, 0.8);
}

/* 输入区域 */
.chat-input-area {
  background: #ffffff;
  border-top: 1px solid #e1e1e1;
  padding: 16px 20px;
  flex-shrink: 0;
}

.input-container {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  transition: all 0.3s ease;
}

.input-container.focused {
  transform: translateY(-2px);
}

.reply-input {
  flex: 1;
  border-radius: 20px;
  border: 1px solid #d9d9d9;
  transition: all 0.3s ease;
}

.reply-input:focus {
  border-color: var(--ant-primary-color, #1890ff);
  box-shadow: 0 0 0 2px var(--ant-primary-color-outline, rgba(24, 144, 255, 0.2));
}

.input-actions {
  display: flex;
  align-items: flex-end;
}

.send-button {
  border-radius: 20px;
  height: 40px;
  padding: 0 20px;
}

.input-hint {
  margin-top: 8px;
  font-size: 12px;
  color: #999;
  text-align: center;
}

/* 滚动条样式 */
.chat-messages::-webkit-scrollbar {
  width: 6px;
}

.chat-messages::-webkit-scrollbar-track {
  background: #f3f4f6;
  border-radius: 3px;
}

.chat-messages::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 3px;
}

.chat-messages::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}

/* 当前请求区域 - 固定在底部 */
.current-request-area {
  background: #ffffff;
  border-top: 1px solid #e1e1e1;
  flex-shrink: 0;
  max-height: 50vh;
  overflow-y: auto;
  position: sticky;
  bottom: 0;
  z-index: 10;
}

.current-request-area::-webkit-scrollbar {
  width: 6px;
}

.current-request-area::-webkit-scrollbar-track {
  background: #f3f4f6;
  border-radius: 3px;
}

.current-request-area::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 3px;
}

.current-request-area::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}

/* 等待状态区域 */
.waiting-input-area {
  background: #ffffff;
  border-top: 1px solid #e1e1e1;
  padding: 20px;
  text-align: center;
  flex-shrink: 0;
  position: sticky;
  bottom: 0;
  z-index: 10;
}

.waiting-message {
  color: #8c8c8c;
  font-size: 14px;
}
</style>
