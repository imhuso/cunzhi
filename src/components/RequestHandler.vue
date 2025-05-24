<template>
  <div class="request-container">
    <div class="request-card">
      <!-- 请求头部 -->
      <div class="request-header">
        <div class="header-icon">📨</div>
        <h3>正在处理请求</h3>
        <div class="request-id">ID: {{ request.id.slice(0, 8) }}...</div>
      </div>

      <!-- 消息内容 -->
      <div class="message-section">
        <div class="message-bubble">
          <div class="message-content">
            {{ request.content }}
          </div>
          <div class="message-time">
            {{ formatTime(new Date()) }}
          </div>
        </div>
      </div>

      <!-- 超时信息 -->
      <div v-if="request.timeout" class="timeout-section">
        <div class="timeout-indicator" :class="{ 'urgent': remainingTime <= 10 }">
          <span class="timeout-icon">⏱️</span>
          <span class="timeout-text">
            剩余时间: {{ remainingTime }}秒
          </span>
          <div class="timeout-progress">
            <div
              class="progress-bar"
              :style="{ width: progressPercentage + '%' }"
            ></div>
          </div>
        </div>
      </div>

      <!-- 回复区域 -->
      <div class="response-section">
        <label for="response-input" class="response-label">
          您的回复:
        </label>
        <textarea
          id="response-input"
          v-model="responseText"
          class="input textarea response-input"
          placeholder="请输入您的回复..."
          :disabled="isProcessing"
          @keydown="handleKeydown"
          ref="textareaRef"
        ></textarea>

        <div class="char-count" :class="{ 'warning': responseText.length > 1000 }">
          {{ responseText.length }}/1000
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="action-buttons">
        <button
          class="btn btn-secondary"
          @click="handleCancel"
          :disabled="isProcessing"
        >
          <span class="btn-icon">❌</span>
          取消
        </button>

        <button
          class="btn btn-primary"
          @click="handleSend"
          :disabled="!responseText.trim() || isProcessing"
        >
          <span class="btn-icon">✅</span>
          {{ isProcessing ? '发送中...' : '发送回复' }}
        </button>
      </div>

      <!-- 快捷键提示 -->
      <div class="shortcuts-hint">
        <small>
          💡 快捷键: Ctrl/Cmd + Enter 发送 | Escape 取消
        </small>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps({
  request: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['response', 'cancel'])

// 响应式数据
const responseText = ref('')
const isProcessing = ref(false)
const remainingTime = ref(props.request.timeout || 30)
const textareaRef = ref(null)

let timeoutInterval = null

// 计算属性
const progressPercentage = computed(() => {
  if (!props.request.timeout) return 100
  return (remainingTime.value / props.request.timeout) * 100
})

// 生命周期
onMounted(() => {
  // 自动聚焦到输入框
  nextTick(() => {
    if (textareaRef.value) {
      textareaRef.value.focus()
    }
  })

  // 启动倒计时
  if (props.request.timeout) {
    startCountdown()
  }

  // 添加键盘事件监听
  document.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  if (timeoutInterval) {
    clearInterval(timeoutInterval)
  }
  document.removeEventListener('keydown', handleGlobalKeydown)
})

// 方法
const startCountdown = () => {
  timeoutInterval = setInterval(() => {
    remainingTime.value--

    if (remainingTime.value <= 0) {
      clearInterval(timeoutInterval)
      // 自动取消
      handleCancel()
    }
  }, 1000)
}

const formatTime = (date) => {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const handleSend = async () => {
  if (!responseText.value.trim() || isProcessing.value) return

  isProcessing.value = true
  try {
    emit('response', responseText.value.trim())
  } finally {
    isProcessing.value = false
  }
}

const handleCancel = () => {
  if (isProcessing.value) return
  emit('cancel')
}

const handleKeydown = (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    handleSend()
  }
}

const handleGlobalKeydown = (event) => {
  if (event.key === 'Escape') {
    event.preventDefault()
    handleCancel()
  }
}
</script>

<style scoped>
.request-container {
  padding: 20px;
  width: 100%;
}

.request-card {
  width: 100%;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 16px;
  padding: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
  animation: slideUp 0.4s ease-out;
}

.request-header {
  text-align: center;
  margin-bottom: 25px;
  padding-bottom: 20px;
  border-bottom: 2px solid rgba(102, 126, 234, 0.1);
}

.header-icon {
  font-size: 32px;
  margin-bottom: 10px;
}

.request-header h3 {
  color: #333;
  margin-bottom: 8px;
  font-size: 18px;
  font-weight: 600;
}

.request-id {
  color: #999;
  font-size: 12px;
  font-family: monospace;
}

.message-section {
  margin-bottom: 25px;
}

.message-bubble {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border-radius: 16px;
  padding: 20px;
  border-left: 4px solid #667eea;
  position: relative;
}

.message-content {
  color: #333;
  line-height: 1.6;
  font-size: 16px;
  margin-bottom: 10px;
  word-wrap: break-word;
}

.message-time {
  color: #666;
  font-size: 12px;
  text-align: right;
}

.timeout-section {
  margin-bottom: 25px;
}

.timeout-indicator {
  background: rgba(255, 193, 7, 0.1);
  border: 1px solid rgba(255, 193, 7, 0.3);
  border-radius: 12px;
  padding: 15px;
  text-align: center;
  transition: all 0.3s ease;
}

.timeout-indicator.urgent {
  background: rgba(220, 53, 69, 0.1);
  border-color: rgba(220, 53, 69, 0.3);
  animation: urgentPulse 1s infinite;
}

.timeout-icon {
  font-size: 18px;
  margin-right: 8px;
}

.timeout-text {
  font-weight: 500;
  color: #333;
}

.timeout-progress {
  width: 100%;
  height: 4px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
  margin-top: 10px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #28a745, #ffc107, #dc3545);
  transition: width 1s linear;
  border-radius: 2px;
}

.response-section {
  margin-bottom: 25px;
}

.response-label {
  display: block;
  margin-bottom: 10px;
  color: #333;
  font-weight: 500;
  font-size: 16px;
}

.response-input {
  min-height: 120px;
  resize: vertical;
  font-size: 14px;
  line-height: 1.5;
}

.char-count {
  text-align: right;
  font-size: 12px;
  color: #666;
  margin-top: 5px;
}

.char-count.warning {
  color: #dc3545;
  font-weight: 500;
}

.action-buttons {
  display: flex;
  gap: 15px;
  justify-content: flex-end;
  margin-bottom: 15px;
}

.btn-icon {
  font-size: 14px;
}

.shortcuts-hint {
  text-align: center;
  color: #666;
  padding-top: 15px;
  border-top: 1px solid rgba(0, 0, 0, 0.1);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes urgentPulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(220, 53, 69, 0.4);
  }
  50% {
    box-shadow: 0 0 0 10px rgba(220, 53, 69, 0);
  }
}
</style>
