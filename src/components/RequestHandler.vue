<template>
  <div class="request-container">
    <div class="request-card">
      <!-- 请求头部 -->
      <div class="request-header">
        <div class="header-icon">📨</div>
        <h3>收到新消息</h3>
        <div class="request-id">ID: {{ request.id.slice(0, 8) }}...</div>
      </div>

      <!-- 可滚动内容区域 -->
      <div class="scrollable-content">
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
            💬 您的回复:
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
            {{ responseText.length }}/1000 字符
          </div>
        </div>
      </div>

      <!-- 固定底部操作区域 -->
      <div class="action-footer">
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
  padding: 16px;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.request-card {
  width: 100%;
  height: 100%;
  background: rgba(255, 255, 255, 0.98);
  border-radius: 20px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(20px);
  animation: slideUp 0.4s ease-out;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.request-header {
  text-align: center;
  padding: 20px 20px 15px 20px;
  border-bottom: 2px solid rgba(102, 126, 234, 0.1);
  flex-shrink: 0;
}

.scrollable-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  padding-bottom: 10px;
}

.action-footer {
  flex-shrink: 0;
  background: rgba(248, 249, 250, 0.8);
  border-top: 1px solid rgba(0, 0, 0, 0.1);
  padding: 20px;
  backdrop-filter: blur(10px);
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
  margin-bottom: 20px;
}

.message-bubble {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border-radius: 18px;
  padding: 24px;
  border-left: 5px solid #667eea;
  position: relative;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
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
  margin-bottom: 20px;
}

.timeout-indicator {
  background: rgba(255, 193, 7, 0.12);
  border: 2px solid rgba(255, 193, 7, 0.4);
  border-radius: 16px;
  padding: 18px;
  text-align: center;
  transition: all 0.3s ease;
  box-shadow: 0 4px 15px rgba(255, 193, 7, 0.1);
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
  margin-bottom: 0;
}

.response-label {
  display: block;
  margin-bottom: 12px;
  color: #333;
  font-weight: 600;
  font-size: 16px;
}

.response-input {
  min-height: 100px;
  max-height: 200px;
  resize: vertical;
  font-size: 14px;
  line-height: 1.5;
  border-radius: 12px;
  border: 2px solid rgba(102, 126, 234, 0.2);
  transition: all 0.3s ease;
}

.response-input:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.1);
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
  gap: 16px;
  justify-content: flex-end;
  margin-bottom: 16px;
}

.action-buttons .btn {
  padding: 12px 24px;
  font-size: 15px;
  font-weight: 600;
  border-radius: 12px;
  min-width: 120px;
  transition: all 0.3s ease;
}

.action-buttons .btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.3);
}

.action-buttons .btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.4);
}

.action-buttons .btn-secondary {
  background: rgba(108, 117, 125, 0.1);
  color: #6c757d;
  border: 2px solid rgba(108, 117, 125, 0.2);
}

.action-buttons .btn-secondary:hover:not(:disabled) {
  background: rgba(108, 117, 125, 0.15);
  border-color: rgba(108, 117, 125, 0.3);
  transform: translateY(-1px);
}

.btn-icon {
  font-size: 16px;
  margin-right: 4px;
}

.shortcuts-hint {
  text-align: center;
  color: #666;
  font-size: 13px;
  opacity: 0.8;
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

/* 滚动条样式 */
.scrollable-content::-webkit-scrollbar {
  width: 8px;
}

.scrollable-content::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

.scrollable-content::-webkit-scrollbar-thumb {
  background: rgba(102, 126, 234, 0.3);
  border-radius: 4px;
  transition: background 0.3s ease;
}

.scrollable-content::-webkit-scrollbar-thumb:hover {
  background: rgba(102, 126, 234, 0.5);
}

/* 响应式设计 */
@media (max-height: 600px) {
  .request-header {
    padding: 15px 20px 10px 20px;
  }

  .scrollable-content {
    padding: 15px 20px 10px 20px;
  }

  .action-footer {
    padding: 15px 20px;
  }

  .response-input {
    min-height: 80px;
  }
}
</style>
