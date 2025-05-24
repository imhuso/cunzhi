<script setup>
import {
  BulbOutlined,
  ClockCircleOutlined,
  CloseOutlined,
  EditOutlined,
  MailOutlined,
  MessageOutlined,
  SendOutlined,
} from '@ant-design/icons-vue'
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'

const props = defineProps({
  request: {
    type: Object,
    required: true,
  },
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
  if (!props.request.timeout)
    return 100
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
function startCountdown() {
  timeoutInterval = setInterval(() => {
    remainingTime.value--

    if (remainingTime.value <= 0) {
      clearInterval(timeoutInterval)
      // 自动取消
      handleCancel()
    }
  }, 1000)
}

function formatTime(date) {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

async function handleSend() {
  if (!responseText.value.trim() || isProcessing.value)
    return

  isProcessing.value = true
  try {
    emit('response', responseText.value.trim())
  }
  finally {
    isProcessing.value = false
  }
}

function handleCancel() {
  if (isProcessing.value)
    return
  emit('cancel')
}

function handleKeydown(event) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    handleSend()
  }
}

function handleGlobalKeydown(event) {
  if (event.key === 'Escape') {
    event.preventDefault()
    handleCancel()
  }
}
</script>

<template>
  <div class="request-container">
    <a-card class="request-card" :bordered="false">
      <!-- 请求头部 -->
      <template #title>
        <div class="request-header">
          <div class="header-content">
            <MailOutlined class="header-icon" />
            <div class="header-text">
              <h3>收到新消息</h3>
              <a-tag size="small" color="blue">
                ID: {{ request.id.slice(0, 8) }}...
              </a-tag>
            </div>
          </div>
        </div>
      </template>

      <!-- 可滚动内容区域 -->
      <div class="scrollable-content">
        <!-- 消息内容 -->
        <div class="message-section">
          <a-card size="small" class="message-bubble">
            <template #title>
              <div class="message-header">
                <MessageOutlined />
                <span>消息内容</span>
              </div>
            </template>
            <div class="message-content">
              {{ request.content }}
            </div>
            <template #extra>
              <div class="message-time">
                <ClockCircleOutlined />
                {{ formatTime(new Date()) }}
              </div>
            </template>
          </a-card>
        </div>

        <!-- 超时信息 -->
        <div v-if="request.timeout" class="timeout-section">
          <a-alert
            :type="remainingTime <= 10 ? 'error' : 'warning'"
            :message="`剩余时间: ${remainingTime}秒`"
            show-icon
            class="timeout-alert"
          >
            <template #icon>
              <ClockCircleOutlined :spin="remainingTime <= 10" />
            </template>
            <template #description>
              <a-progress
                :percent="progressPercentage"
                :status="remainingTime <= 10 ? 'exception' : 'active'"
                :stroke-color="remainingTime <= 10 ? '#ff4d4f' : '#1890ff'"
                size="small"
              />
            </template>
          </a-alert>
        </div>

        <!-- 回复区域 -->
        <div class="response-section">
          <a-form layout="vertical">
            <a-form-item label="您的回复" class="response-form-item">
              <template #label>
                <div class="response-label">
                  <EditOutlined />
                  <span>您的回复</span>
                </div>
              </template>
              <a-textarea
                ref="textareaRef"
                v-model:value="responseText"
                :rows="4"
                :max-length="1000"
                show-count
                placeholder="请输入您的回复..."
                :disabled="isProcessing"
                class="response-textarea"
                @keydown="handleKeydown"
              />
            </a-form-item>
          </a-form>
        </div>
      </div>

      <!-- 固定底部操作区域 -->
      <template #actions>
        <div class="action-footer">
          <!-- 操作按钮 -->
          <div class="action-buttons">
            <a-button
              size="large"
              :disabled="isProcessing"
              class="cancel-btn"
              @click="handleCancel"
            >
              <template #icon>
                <CloseOutlined />
              </template>
              取消
            </a-button>

            <a-button
              type="primary"
              size="large"
              :disabled="!responseText.trim() || isProcessing"
              :loading="isProcessing"
              class="send-btn"
              @click="handleSend"
            >
              <template v-if="!isProcessing" #icon>
                <SendOutlined />
              </template>
              {{ isProcessing ? '发送中...' : '发送回复' }}
            </a-button>
          </div>

          <!-- 快捷键提示 -->
          <div class="shortcuts-hint">
            <a-typography-text type="secondary" class="hint-text">
              <BulbOutlined />
              快捷键: Ctrl/Cmd + Enter 发送 | Escape 取消
            </a-typography-text>
          </div>
        </div>
      </template>
    </a-card>
  </div>
</template>

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
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.request-card :deep(.ant-card-head) {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border-bottom: 2px solid rgba(102, 126, 234, 0.1);
  border-radius: 20px 20px 0 0;
}

.request-card :deep(.ant-card-body) {
  flex: 1;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.request-card :deep(.ant-card-actions) {
  background: rgba(248, 249, 250, 0.8);
  border-top: 1px solid rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
  border-radius: 0 0 20px 20px;
}

.request-header {
  width: 100%;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  font-size: 24px;
  color: #667eea;
}

.header-text h3 {
  margin: 0 0 4px 0;
  color: #333;
  font-size: 18px;
  font-weight: 600;
}

.scrollable-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  padding-bottom: 10px;
}

.action-footer {
  padding: 20px;
  width: 100%;
}

.message-section {
  margin-bottom: 20px;
}

.message-bubble {
  border-left: 4px solid #667eea;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

.message-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #667eea;
  font-weight: 500;
}

.message-content {
  color: #333;
  line-height: 1.6;
  font-size: 16px;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.message-time {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #666;
  font-size: 12px;
}

.timeout-section {
  margin-bottom: 20px;
}

.timeout-alert {
  border-radius: 12px;
}

.response-section {
  margin-bottom: 0;
}

.response-form-item {
  margin-bottom: 0;
}

.response-label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #333;
  font-weight: 600;
  font-size: 16px;
}

.response-textarea {
  border-radius: 12px;
  transition: all 0.3s ease;
}

.action-buttons {
  display: flex;
  gap: 16px;
  justify-content: flex-end;
  margin-bottom: 16px;
}

.cancel-btn,
.send-btn {
  min-width: 120px;
  border-radius: 12px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.send-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

.send-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.3);
}

.cancel-btn:hover:not(:disabled) {
  transform: translateY(-1px);
}

.shortcuts-hint {
  text-align: center;
}

.hint-text {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 13px;
  opacity: 0.8;
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
  .scrollable-content {
    padding: 15px 20px 10px 20px;
  }

  .action-footer {
    padding: 15px 20px;
  }

  .response-textarea {
    min-height: 80px;
  }
}
</style>
