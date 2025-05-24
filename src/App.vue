<template>
  <div class="app-container">
    <!-- 窗口标题栏 -->
    <div class="title-bar" data-tauri-drag-region>
      <div class="title-content">
        <div class="app-title">
          <span class="app-icon">🤖</span>
          <span>AI Review</span>
        </div>
        <div class="status-indicator">
          <div
            class="status-dot"
            :class="{
              'connected': isConnected,
              'disconnected': !isConnected,
              'pulse': isConnected
            }"
          ></div>
          <span class="status-text">
            {{ isConnected ? '已连接' : '连接中...' }}
          </span>
        </div>
      </div>
      <div class="window-controls">
        <button class="control-btn minimize" @click="minimizeWindow">−</button>
        <button class="control-btn close" @click="closeWindow">×</button>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="main-content">
      <!-- 聊天历史区域 -->
      <div class="chat-history" ref="chatHistoryRef">
        <div v-if="chatHistory.length === 0" class="empty-state">
          <div class="empty-icon">💬</div>
          <p>暂无聊天记录</p>
          <small>等待命令行消息...</small>
        </div>

        <div v-else class="messages">
          <div
            v-for="message in chatHistory"
            :key="message.id"
            class="message-item"
            :class="message.type"
          >
            <div class="message-content">
              <div class="message-text">{{ message.content }}</div>
              <div class="message-time">{{ formatTime(message.timestamp) }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 当前请求处理区域 -->
      <div v-if="currentRequest" class="current-request">
        <RequestHandler
          :request="currentRequest"
          @response="handleResponse"
          @cancel="handleCancel"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'
import RequestHandler from './components/RequestHandler.vue'

// 响应式数据
const appInfo = ref('')
const currentRequest = ref(null)
const isConnected = ref(false)
const chatHistory = ref([])
const chatHistoryRef = ref(null)

// 聊天历史管理（限制数量以优化性能）
const MAX_HISTORY_ITEMS = 100

const addToHistory = (type, content, id = null) => {
  const message = {
    id: id || Date.now().toString(),
    type, // 'incoming' 或 'outgoing'
    content,
    timestamp: new Date()
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

const formatTime = (date) => {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 窗口控制
const minimizeWindow = async () => {
  try {
    await appWindow.minimize()
  } catch (error) {
    console.error('最小化窗口失败:', error)
  }
}

const closeWindow = async () => {
  try {
    await appWindow.close()
  } catch (error) {
    console.error('关闭窗口失败:', error)
  }
}

// 初始化应用
onMounted(async () => {
  console.log('🚀 AI Review Vue App 初始化中...')
  console.log('🔧 Tauri API 可用性检查:', !!window.__TAURI__)

  try {
    // 获取应用信息
    appInfo.value = await invoke('get_app_info')
    console.log('✅ 应用信息获取成功:', appInfo.value)
  } catch (error) {
    console.error('❌ 获取应用信息失败:', error)
    appInfo.value = 'AI Review App v0.1.0'
  }

  // 监听新请求事件
  try {
    console.log('🔧 开始设置事件监听器...')
    const unlisten = await listen('new-request', (event) => {
      console.log('🎯 收到新请求事件:', event)
      const message = event.payload
      console.log('📨 解析后的消息:', message)

      // 添加到聊天历史
      addToHistory('incoming', message.content, message.id)

      // 设置当前请求
      currentRequest.value = message
      console.log('📨 currentRequest已更新:', currentRequest.value)
    })
    console.log('✅ 事件监听器设置成功')
    isConnected.value = true

  } catch (error) {
    console.error('❌ 设置事件监听器失败:', error)
    isConnected.value = false
  }
})

// 处理用户回复
const handleResponse = async (response) => {
  if (!currentRequest.value) return

  try {
    // 添加回复到聊天历史
    addToHistory('outgoing', response)

    await invoke('respond_to_request', {
      id: currentRequest.value.id,
      response: response
    })
    console.log('✅ 回复发送成功:', response)
    currentRequest.value = null
  } catch (error) {
    console.error('❌ 发送回复失败:', error)
    alert('发送回复失败: ' + error)
  }
}

// 处理取消操作
const handleCancel = async () => {
  if (!currentRequest.value) return

  try {
    // 添加取消信息到聊天历史
    addToHistory('outgoing', '[已取消]')

    await invoke('respond_to_request', {
      id: currentRequest.value.id,
      response: '[用户取消了请求]'
    })
    console.log('✅ 请求已取消')
    currentRequest.value = null
  } catch (error) {
    console.error('❌ 取消请求失败:', error)
  }
}
</script>

<style scoped>
.app-container {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  margin: 8px;
  height: calc(100vh - 16px);
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
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  transition: all 0.3s ease;
}

.status-dot.connected {
  background: #4CAF50;
  box-shadow: 0 0 0 2px rgba(76, 175, 80, 0.3);
}

.status-dot.disconnected {
  background: #f44336;
  box-shadow: 0 0 0 2px rgba(244, 67, 54, 0.3);
}

.status-dot.pulse {
  animation: statusPulse 2s infinite;
}

.status-text {
  color: white;
  font-size: 12px;
  opacity: 0.9;
}

.window-controls {
  display: flex;
  gap: 8px;
}

.control-btn {
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 12px;
  font-weight: bold;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.1);
}

.control-btn.close:hover {
  background: #f44336;
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
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: rgba(255, 255, 255, 0.7);
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 16px;
  margin-bottom: 8px;
}

.empty-state small {
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
  animation: messageSlideIn 0.3s ease-out;
}

.message-item.incoming {
  align-self: flex-start;
}

.message-item.outgoing {
  align-self: flex-end;
}

.message-content {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 16px;
  padding: 12px 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  position: relative;
}

.message-item.incoming .message-content {
  background: rgba(255, 255, 255, 0.9);
  border-bottom-left-radius: 4px;
}

.message-item.outgoing .message-content {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-bottom-right-radius: 4px;
}

.message-text {
  font-size: 14px;
  line-height: 1.4;
  word-wrap: break-word;
  margin-bottom: 4px;
}

.message-time {
  font-size: 11px;
  opacity: 0.7;
  text-align: right;
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

/* 动画 */
@keyframes statusPulse {
  0%, 100% {
    box-shadow: 0 0 0 2px rgba(76, 175, 80, 0.3);
  }
  50% {
    box-shadow: 0 0 0 6px rgba(76, 175, 80, 0.1);
  }
}

@keyframes messageSlideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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
