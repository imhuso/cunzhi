<script setup>
import {
  ClockCircleOutlined,
  CloseOutlined,
  MessageOutlined,
  MinusOutlined,
  RobotOutlined,
} from '@ant-design/icons-vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from '@tauri-apps/api/window'
import { nextTick, onMounted, ref } from 'vue'
import RequestHandler from './components/RequestHandler.vue'

// 响应式数据
const appInfo = ref('')
const currentRequest = ref(null)
const isConnected = ref(false)
const chatHistory = ref([])
const chatHistoryRef = ref(null)

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
    if (chatHistoryRef.value) {
      chatHistoryRef.value.scrollTop = chatHistoryRef.value.scrollHeight
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

// 窗口控制
async function minimizeWindow() {
  try {
    await appWindow.minimize()
  }
  catch (error) {
    console.error('最小化窗口失败:', error)
  }
}

async function closeWindow() {
  try {
    await appWindow.close()
  }
  catch (error) {
    console.error('关闭窗口失败:', error)
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
    isConnected.value = true
  }
  catch (error) {
    console.error('❌ 设置事件监听器失败:', error)
    isConnected.value = false
  }
})

// 处理用户回复
async function handleResponse(response) {
  if (!currentRequest.value)
    return

  try {
    // 添加回复到聊天历史
    addToHistory('outgoing', response)

    await invoke('respond_to_request', {
      id: currentRequest.value.id,
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
      id: currentRequest.value.id,
      response: '[用户取消了请求]',
    })
    console.warn('✅ 请求已取消')
    currentRequest.value = null
  }
  catch (error) {
    console.error('❌ 取消请求失败:', error)
  }
}
</script>

<template>
  <div class="app-container">
    <!-- 窗口标题栏 -->
    <div class="title-bar window-drag-region">
      <div class="title-content">
        <div class="app-title">
          <RobotOutlined class="app-icon" />
          <span class="app-name">AI Review</span>
        </div>
        <div class="status-indicator">
          <a-badge
            :status="isConnected ? 'processing' : 'error'"
            :text="isConnected ? '已连接' : '连接中...'"
            class="status-badge"
          />
        </div>
      </div>
      <div class="window-controls window-no-drag">
        <a-button
          type="text"
          size="small"
          class="control-btn minimize"
          @click="minimizeWindow"
        >
          <MinusOutlined />
        </a-button>
        <a-button
          type="text"
          size="small"
          class="control-btn close"
          @click="closeWindow"
        >
          <CloseOutlined />
        </a-button>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="main-content">
      <!-- 聊天历史区域 -->
      <div ref="chatHistoryRef" class="chat-history">
        <a-empty
          v-if="chatHistory.length === 0"
          class="empty-state"
          description="暂无聊天记录"
        >
          <template #image>
            <MessageOutlined class="empty-icon" />
          </template>
          <template #description>
            <span class="empty-description">
              暂无聊天记录<br />
              <small>等待命令行消息...</small>
            </span>
          </template>
        </a-empty>

        <div v-else class="messages">
          <div
            v-for="message in chatHistory"
            :key="message.id"
            class="message-item fade-in-up"
            :class="message.type"
          >
            <a-card
              :bordered="false"
              size="small"
              class="message-card"
              :class="`message-${message.type}`"
            >
              <div class="message-content">
                <div class="message-text">
                  {{ message.content }}
                </div>
                <div class="message-time">
                  <ClockCircleOutlined class="time-icon" />
                  {{ formatTime(message.timestamp) }}
                </div>
              </div>
            </a-card>
          </div>
        </div>
      </div>

      <!-- 当前请求处理区域 -->
      <div v-if="currentRequest" class="current-request slide-in-right">
        <RequestHandler
          :request="currentRequest"
          @response="handleResponse"
          @cancel="handleCancel"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-container {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
  position: relative;
}

/* 标题栏 */
.title-bar {
  height: 40px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  user-select: none;
  flex-shrink: 0;
}

.title-content {
  display: flex;
  align-items: center;
  gap: 20px;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: white;
  font-weight: 600;
  font-size: 14px;
}

.app-icon {
  font-size: 16px;
  color: white;
}

.app-name {
  color: white;
}

.status-indicator {
  display: flex;
  align-items: center;
}

.status-badge :deep(.ant-badge-status-text) {
  color: white;
  font-size: 12px;
  opacity: 0.9;
}

.status-badge :deep(.ant-badge-status-dot) {
  width: 8px;
  height: 8px;
}

.window-controls {
  display: flex;
  gap: 4px;
}

.control-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  color: white !important;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.2) !important;
  transform: scale(1.1);
}

.control-btn.close:hover {
  background: #f44336 !important;
  color: white !important;
}

.control-btn.minimize:hover {
  background: rgba(255, 255, 255, 0.3) !important;
}

/* 主要内容区域 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 聊天历史区域 */
.chat-history {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background: rgba(255, 255, 255, 0.05);
}

.empty-state {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state :deep(.ant-empty-image) {
  margin-bottom: 16px;
}

.empty-icon {
  font-size: 48px;
  color: rgba(255, 255, 255, 0.5);
}

.empty-description {
  color: rgba(255, 255, 255, 0.7);
  font-size: 16px;
}

.empty-description small {
  font-size: 12px;
  opacity: 0.7;
}

/* 消息列表 */
.messages {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message-item {
  display: flex;
  max-width: 80%;
}

.message-item.incoming {
  align-self: flex-start;
}

.message-item.outgoing {
  align-self: flex-end;
}

.message-card {
  width: 100%;
  border-radius: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.message-incoming .message-card {
  background: rgba(255, 255, 255, 0.95);
}

.message-outgoing .message-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.message-outgoing .message-content {
  color: white;
}

.message-outgoing .message-time {
  color: rgba(255, 255, 255, 0.8);
}

.message-content {
  padding: 0;
}

.message-text {
  font-size: 14px;
  line-height: 1.5;
  word-wrap: break-word;
  margin-bottom: 8px;
}

.message-time {
  font-size: 11px;
  opacity: 0.7;
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
}

.time-icon {
  font-size: 10px;
}

/* 当前请求区域 */
.current-request {
  background: rgba(255, 255, 255, 0.08);
  border-top: 1px solid rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(15px);
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 滚动条样式 */
.chat-history::-webkit-scrollbar {
  width: 6px;
}

.chat-history::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

.chat-history::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 3px;
}

.chat-history::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.5);
}
</style>
