<script setup>
import {
  BulbOutlined,
  ClockCircleOutlined,
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
const inputFocused = ref(false)

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

// 输入框焦点处理
function onInputFocus() {
  inputFocused.value = true
}

function onInputBlur() {
  inputFocused.value = false
}

function handleKeydown(event) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    handleSend()
  }
  // 普通Enter键允许换行，不做处理
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
    <!-- 消息内容 -->
    <div class="message-content">
      {{ request.content }}
    </div>

    <!-- 回复区域 -->
    <div class="reply-section">
      <div class="input-container">
        <a-textarea
          ref="textareaRef"
          v-model:value="responseText"
          :auto-size="inputFocused ? { minRows: 3, maxRows: 8 } : { minRows: 1, maxRows: 1 }"
          :max-length="1000"
          placeholder="请输入您的回复..."
          :disabled="isProcessing"
          class="reply-textarea"
          @focus="onInputFocus"
          @blur="onInputBlur"
          @keydown="handleKeydown"
        />
        <div class="input-actions">
          <span class="char-count">{{ responseText.length }}/1000</span>
          <a-button
            :disabled="!responseText.trim() || isProcessing"
            :loading="isProcessing"
            @click="handleSend"
            class="send-button"
            size="small"
          >
            <template v-if="!isProcessing" #icon>
              <SendOutlined />
            </template>
            {{ isProcessing ? '发送中' : '发送' }}
          </a-button>
        </div>
      </div>

      <!-- 倒计时信息 - 只在有超时参数时显示 -->
      <div v-if="request.timeout && remainingTime > 0" class="countdown-info">
        <ClockCircleOutlined
          :spin="remainingTime <= 10"
          :style="{ color: remainingTime <= 10 ? '#ff4d4f' : 'var(--ant-primary-color, #1890ff)' }"
        />
        <span
          class="countdown-text"
          :style="{ color: remainingTime <= 10 ? '#ff4d4f' : '#666' }"
        >
          剩余 {{ remainingTime }}秒
        </span>
        <a-progress
          :percent="progressPercentage"
          :status="remainingTime <= 10 ? 'exception' : 'active'"
          :stroke-color="remainingTime <= 10 ? '#ff4d4f' : 'var(--ant-primary-color, #1890ff)'"
          size="small"
          :show-info="false"
          class="countdown-progress"
        />
      </div>

      <!-- 快捷键提示 -->
      <div class="shortcuts-hint" v-if="inputFocused">
        <BulbOutlined />
        <span>CMD+Enter 发送，Enter 换行</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.request-container {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

/* 消息内容 */
.message-content {
  color: #262626;
  line-height: 1.6;
  font-size: 14px;
  word-wrap: break-word;
  white-space: pre-wrap;
  padding: 16px;
  background: #f9f9f9;
  border-radius: 12px;
  border-left: 4px solid var(--ant-primary-color, #1890ff);
  margin-bottom: 16px;
}

/* 回复区域 */
.reply-section {
  background: #ffffff;
  border: 1px solid #e1e1e1;
  border-radius: 12px;
  padding: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.input-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.reply-textarea {
  border: none;
  border-radius: 8px;
  background: #f7f7f7;
  padding: 12px;
  resize: none;
  transition: all 0.2s ease;
  font-size: 14px;
  line-height: 1.5;
}

.reply-textarea:focus {
  background: #ffffff;
  border: 1px solid #e8e8e8;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  outline: none;
}

.input-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px;
}

.char-count {
  font-size: 12px;
  color: #999;
}

/* 倒计时信息 */
.countdown-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 8px;
  background: #f9f9f9;
  border-radius: 6px;
  margin-top: 8px;
}

.countdown-text {
  font-size: 13px;
  font-weight: 500;
}

.countdown-progress {
  width: 120px;
  margin-left: 8px;
}

.send-button {
  background: #f0f0f0 !important;
  border: 1px solid #e0e0e0 !important;
  color: #333 !important;
  border-radius: 6px !important;
  font-size: 12px !important;
  height: 28px !important;
  padding: 0 12px !important;
  transition: all 0.2s ease !important;
}

.send-button:hover:not(:disabled) {
  background: #e8f4fd !important;
  border-color: #91d5ff !important;
  color: #1890ff !important;
}

.send-button:disabled {
  background: #f5f5f5 !important;
  border-color: #e0e0e0 !important;
  color: #bbb !important;
  cursor: not-allowed !important;
}

/* 快捷键提示 */
.shortcuts-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: #8c8c8c;
  font-size: 12px;
  padding-top: 8px;
  border-top: 1px solid #f0f0f0;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .action-area {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .action-left {
    order: 2;
  }

  .action-buttons {
    order: 1;
    justify-content: flex-end;
  }

  .countdown-info {
    justify-content: center;
  }
}

@media (max-height: 600px) {
  .request-container {
    max-height: 50vh;
  }

  .reply-textarea {
    min-height: 60px;
  }
}
</style>
