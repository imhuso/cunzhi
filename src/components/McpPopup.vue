<template>
  <div class="fixed inset-0 bg-white flex flex-col z-50">
    <div class="bg-white w-full h-full flex flex-col">
      <!-- 头部 -->
      <div class="flex items-center justify-between px-6 py-5 border-b border-gray-200 bg-gradient-to-r from-blue-500 to-blue-600">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 bg-white bg-opacity-20 rounded-xl flex items-center justify-center backdrop-blur-sm">
            <RobotOutlined class="text-white text-lg" />
          </div>
          <h3 class="text-xl font-semibold text-white">AI Review</h3>
        </div>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6 bg-gray-50">
        <!-- 消息显示区域 -->
        <div class="bg-white border border-gray-200 rounded-xl p-6 shadow-sm">
          <div class="text-gray-800 leading-relaxed text-sm">
            <vue-markdown-it :source="request.message"
              :options="markdownOptions"
              v-if="request.is_markdown" />
            <div v-else
              class="whitespace-pre-wrap">{{ request.message }}</div>
          </div>
        </div>

        <!-- 预定义选项 -->
        <div v-if="request.predefined_options && request.predefined_options.length > 0" class="bg-white rounded-xl p-6 shadow-sm border border-gray-200">
          <h4 class="text-base font-semibold text-gray-800 mb-4 flex items-center gap-3">
            <div class="w-6 h-6 bg-green-100 rounded-lg flex items-center justify-center">
              <CheckCircleOutlined class="text-green-600 text-sm" />
            </div>
            请选择选项 <span class="text-sm text-gray-500 font-normal">(可多选)</span>
          </h4>

          <a-checkbox-group v-model:value="selectedOptions" class="w-full">
            <div class="space-y-3">
              <a-checkbox v-for="(option, index) in request.predefined_options"
                :key="`option-${index}`"
                :value="option"
                class="!flex items-center p-4 border-2 border-gray-200 rounded-xl hover:border-blue-400 hover:bg-blue-50 hover:shadow-md transition-all duration-300 group">
                <span class="ml-3 text-base text-gray-800 group-hover:text-blue-800 font-medium">{{ option }}</span>
              </a-checkbox>
            </div>
          </a-checkbox-group>
        </div>

        <!-- 通用回复输入 -->
        <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-200">
          <h4 class="text-base font-semibold text-gray-800 mb-4 flex items-center gap-3">
            <div class="w-6 h-6 bg-purple-100 rounded-lg flex items-center justify-center">
              <EditOutlined class="text-purple-600 text-sm" />
            </div>
            {{ request.predefined_options ? '补充说明 (可选)' : '请输入您的回复' }}
          </h4>
          <a-textarea ref="textareaRef"
            v-model:value="userInput"
            :placeholder="request.predefined_options ? '您可以在这里添加补充说明...' : '请输入您的回复...'"
            :rows="request.predefined_options ? 4 : 6"
            class="resize-none rounded-lg border-gray-300 focus:border-blue-500 focus:ring-2 focus:ring-blue-200"
            :disabled="submitting"
            @keydown.ctrl.enter="submitInput" />
        </div>


      </div>

      <!-- 底部操作栏 -->
      <div class="border-t border-gray-200 px-6 py-5 bg-white">
        <div class="flex justify-between items-center">
          <div class="text-sm text-gray-600 flex items-center gap-2">
            <div class="w-4 h-4 bg-blue-100 rounded flex items-center justify-center">
              <span class="text-blue-600 text-xs">💡</span>
            </div>
            {{ request.predefined_options ? '选择选项或输入文本，至少一个' : 'Ctrl+Enter 快速发送' }}
          </div>
          <div class="flex justify-end">
            <a-button type="primary"
              @click="handleSubmit"
              :disabled="!canSubmit || submitting"
              :loading="submitting"
              size="large"
              class="px-8 font-medium">
              <template #icon>
                <SendOutlined />
              </template>
              发送
            </a-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  RobotOutlined,
  CheckCircleOutlined,
  EditOutlined,
  SendOutlined
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { VueMarkdownIt } from '@f3ve/vue-markdown-it'

// Props
const props = defineProps({
  request: {
    type: Object,
    required: true,
    validator: (value) => {
      return value && typeof value.message === 'string'
    }
  }
})

// Emits
const emit = defineEmits(['response', 'cancel'])

// 响应式数据
const userInput = ref('')
const selectedOptions = ref([])
const submitting = ref(false)
const textareaRef = ref(null)

// 计算属性
const canSubmit = computed(() => {
  if (props.request.predefined_options) {
    // 预定义选项模式：选择选项或输入文本，至少一个
    return selectedOptions.value.length > 0 || userInput.value.trim() !== ''
  } else {
    // 文本输入模式：必须有用户输入
    return userInput.value.trim() !== ''
  }
})

// Markdown配置 - 安全的配置
const markdownOptions = {
  html: false, // 禁用HTML标签以防止XSS
  linkify: true, // 自动转换URL为链接
  typographer: true, // 启用排版增强
  breaks: true // 转换换行符
}

// 方法
const handleSubmit = async () => {
  if (submitting.value || !canSubmit.value) return

  let response = ''
  if (props.request.predefined_options) {
    // 组合选项和文本输入
    const parts = []
    if (selectedOptions.value.length > 0) {
      parts.push(selectedOptions.value.join(', '))
    }
    if (userInput.value.trim()) {
      parts.push(userInput.value.trim())
    }
    response = parts.join('\n\n')
  } else {
    response = userInput.value.trim()
  }

  await sendResponse(response)
}

const submitInput = async () => {
  await handleSubmit()
}

const handleCancel = async () => {
  if (submitting.value) return
  emit('cancel')
}

const sendResponse = async (response) => {
  if (submitting.value) return

  submitting.value = true
  try {
    // 直接通过emit发送响应，让父组件处理
    emit('response', response)
  } catch (error) {
    console.error('发送响应失败:', error)
    message.error('发送响应失败，请重试')
    submitting.value = false
  }
}

// 生命周期
onMounted(() => {
  // 聚焦到输入框（如果有）
  nextTick(() => {
    if (!props.request.predefined_options && textareaRef.value) {
      textareaRef.value.focus()
    }
  })

  // 清空输入
  userInput.value = ''
  selectedOptions.value = []
})
</script>

<style scoped>
/* 确保Markdown内容样式正确 */
:deep(.prose) {
  max-width: none;
}

:deep(.prose h1) {
  font-size: 1.25rem;
  font-weight: 700;
  color: #2563eb;
  margin-bottom: 0.5rem;
}

:deep(.prose h2) {
  font-size: 1.125rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 0.5rem;
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 0.25rem;
}

:deep(.prose h3) {
  font-size: 1rem;
  font-weight: 500;
  color: #374151;
  margin-bottom: 0.25rem;
}

:deep(.prose strong) {
  font-weight: 600;
}

:deep(.prose em) {
  font-style: italic;
}

:deep(.prose code) {
  background-color: #f3f4f6;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-size: 0.875rem;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace;
}

:deep(.prose ul) {
  list-style-type: disc;
  margin-left: 1rem;
}

:deep(.prose ol) {
  list-style-type: decimal;
  margin-left: 1rem;
}

:deep(.prose li) {
  margin-bottom: 0.25rem;
}

:deep(.prose a) {
  color: #2563eb;
  text-decoration: underline;
}

:deep(.prose a:hover) {
  color: #1e40af;
}

:deep(.prose blockquote) {
  border-left: 4px solid #d1d5db;
  padding-left: 1rem;
  font-style: italic;
  color: #6b7280;
}
</style>
