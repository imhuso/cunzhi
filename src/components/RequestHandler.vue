<script setup>
import {
  ClockCircleOutlined,
  SendOutlined,
} from '@ant-design/icons-vue'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

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
const isReplied = ref(false) // 添加已回复状态

let timeoutInterval = null



// 监听request变化，重置状态
watch(() => props.request, (newRequest) => {
  if (newRequest) {
    // 重置状态
    responseText.value = ''
    isReplied.value = false
    remainingTime.value = newRequest.timeout || 30

    // 清除之前的倒计时
    if (timeoutInterval) {
      clearInterval(timeoutInterval)
      timeoutInterval = null
    }

    // 启动新的倒计时
    if (newRequest.timeout) {
      startCountdown()
    }

    // 自动聚焦到输入框
    nextTick(() => {
      if (textareaRef.value) {
        textareaRef.value.focus()
      }
    })
  }
}, { immediate: true })

// 生命周期
onMounted(() => {
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
  if (!responseText.value.trim() || isProcessing.value || isReplied.value)
    return

  isProcessing.value = true
  try {
    emit('response', responseText.value.trim())
    // 发送成功后清空输入框并标记为已回复
    responseText.value = ''
    isReplied.value = true
  }
  finally {
    isProcessing.value = false
  }
}

function handleCancel() {
  if (isProcessing.value || isReplied.value)
    return
  isReplied.value = true
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
    <!-- 回复区域 -->
    <div class="reply-section">
      <div class="input-container">
        <div class="textarea-container">
          <a-textarea
            ref="textareaRef"
            v-model:value="responseText"
            :auto-size="{ minRows: 2, maxRows: 4 }"
            :max-length="1000"
            :placeholder="isReplied ? '已回复' : '请输入您的回复...'"
            :disabled="isProcessing || isReplied"
            class="reply-textarea"
            @focus="onInputFocus"
            @blur="onInputBlur"
            @keydown="handleKeydown"
          />
          <div class="textarea-actions">
            <span class="char-count">{{ responseText.length }}/1000</span>

            <!-- 倒计时信息 - 放在中间 -->
            <div v-if="request.timeout && remainingTime > 0 && !isReplied" class="countdown-inline">
              <ClockCircleOutlined />
              <span>{{ remainingTime }}s</span>
            </div>

            <a-button
              :disabled="!responseText.trim() || isProcessing || isReplied"
              :loading="isProcessing"
              @click="handleSend"
              class="send-button"
              size="small"
              :title="isReplied ? '' : 'CMD+Enter 发送'"
            >
              <template v-if="!isProcessing" #icon>
                <SendOutlined />
              </template>
              {{ isReplied ? '已发送' : (isProcessing ? '发送中' : '发送') }}
            </a-button>
          </div>
        </div>
      </div>




    </div>
  </div>
</template>

<style scoped>
.request-container {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
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
  display: flex;
  flex-direction: column;
}

.input-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.textarea-container {
  position: relative;
  border: 1px solid #d9d9d9;
  border-radius: 8px;
  background: #ffffff;
  transition: all 0.2s ease;
}

.textarea-container:focus-within {
  border-color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

.reply-textarea {
  border: none;
  background: transparent;
  padding: 12px 12px 8px 12px;
  resize: none;
  font-size: 14px;
  line-height: 1.5;
  width: 100%;
}

.reply-textarea:focus {
  outline: none;
  box-shadow: none;
}

.textarea-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-top: 1px solid #f0f0f0;
  background: #fafafa;
  border-radius: 0 0 8px 8px;
}

.char-count {
  font-size: 12px;
  color: #999;
}

/* 内联倒计时 */
.countdown-inline {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #666;
  padding: 2px 8px;
  background: #f0f0f0;
  border-radius: 4px;
}

.countdown-inline span {
  font-weight: 500;
}



.send-button {
  border-radius: 6px !important;
  font-size: 12px !important;
  height: 32px !important;
  padding: 0 16px !important;
  transition: all 0.2s ease !important;
  flex-shrink: 0;
}

.send-button:disabled {
  cursor: not-allowed !important;
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
    padding: 12px;
  }

  .reply-section {
    padding: 12px;
  }
}

@media (max-width: 480px) {
  .textarea-actions {
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
  }

  .send-button {
    width: 100%;
  }
}
</style>
