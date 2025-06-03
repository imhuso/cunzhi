<template>
  <div class="fixed inset-0 flex flex-col z-50 dark bg-dark-primary">
    <div class="relative w-full h-full flex flex-col bg-white dark:bg-dark-primary shadow-xl">

      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3 bg-gray-50 dark:bg-dark-primary"
        @drop="handleImageDrop"
        @dragover.prevent
        @dragenter.prevent>

        <!-- 简化的加载状态 -->
        <div v-if="loading"
          class="flex items-center justify-center py-8">
          <div class="text-center">
            <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-3">
            </div>
            <p class="text-gray-600 dark:text-gray-400 text-sm">加载中...</p>
          </div>
        </div>

        <!-- 消息显示区域 -->
        <div v-else-if="request && request.message"
          class="mb-4">
          <div class="bg-white dark:bg-dark-secondary rounded-lg p-4 shadow-sm border border-gray-200 dark:border-gray-700">
            <div class="leading-relaxed text-sm markdown-content text-gray-900 dark:text-gray-100">
              <vue-markdown-it :source="request.message"
                :options="markdownOptions"
                v-if="request.is_markdown" />
              <div v-else
                class="whitespace-pre-wrap">{{ request.message }}</div>
            </div>
          </div>
        </div>

        <!-- 错误状态 -->
        <div v-else
          class="rounded-lg p-4 border bg-red-50 border-red-200 dark:bg-red-900/20 dark:border-red-700/30">
          <div class="text-red-800 dark:text-red-300">
            <h4 class="font-medium mb-1 text-sm">❌ 数据加载错误</h4>
            <p class="text-xs">Request对象: {{ JSON.stringify(request) }}</p>
          </div>
        </div>

        <!-- 预定义选项 -->
        <div v-if="!loading && request.predefined_options && request.predefined_options.length > 0"
          class="mb-4">
          <h4 class="text-sm font-medium mb-3 text-gray-800 dark:text-gray-200">请选择选项</h4>

          <div class="w-full">
            <div class="grid gap-2">
              <label v-for="(option, index) in request.predefined_options"
                :key="`option-${index}`"
                class="checkbox flex items-center p-3 rounded-lg transition-colors group bg-white hover:bg-blue-50 dark:bg-dark-secondary dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-600 cursor-pointer">
                <input 
                  type="checkbox"
                  :value="option"
                  v-model="selectedOptions"
                  class="sr-only" />
                <div class="checkbox-box"></div>
                <span class="ml-3 text-sm text-gray-800 group-hover:text-blue-700 dark:text-gray-200 dark:group-hover:text-blue-300">{{
                  option }}</span>
              </label>
            </div>
          </div>
        </div>

        <!-- 图片预览区域 -->
        <div v-if="!loading && draggedImages.length > 0"
          class="mb-3">
          <h4 class="text-sm font-medium mb-2 text-gray-800 dark:text-gray-200">已添加的图片</h4>
          <div class="grid grid-cols-2 gap-2">
            <div v-for="(image, index) in draggedImages"
              :key="index"
              class="relative rounded-md overflow-hidden border-2 border-gray-300 dark:border-gray-600">
              <img :src="image"
                class="w-full h-20 object-cover" />
              <button @click="removeImage(index)"
                class="absolute top-1 right-1 w-5 h-5 rounded-full flex items-center justify-center text-xs bg-red-500 text-white dark:bg-red-600">×</button>
            </div>
          </div>
        </div>

        <!-- 通用回复输入 -->
        <div v-if="!loading"
          class="mb-4">
          <h4 class="text-sm font-medium mb-3 text-gray-800 dark:text-gray-200">
            {{ request.predefined_options ? '补充说明 (可选)' : '请输入您的回复' }}
          </h4>

          <div
            class="relative rounded-md border-2 border-dashed p-3 mb-2 border-gray-300 bg-gray-100 dark:border-gray-600 dark:bg-gray-800/50">
            <p class="text-xs text-center text-gray-600 dark:text-gray-400">
              拖拽图片到此处或在输入框中粘贴图片 (⌘+V)
            </p>
          </div>

          <textarea 
            ref="textareaRef"
            v-model="userInput"
            :placeholder="request.predefined_options ? '您可以在这里添加补充说明...' : '请输入您的回复...'"
            :rows="request.predefined_options ? 3 : 5"
            class="textarea"
            :disabled="submitting"
            @keydown.meta.enter="submitInput"
            @paste="handleImagePaste" />
        </div>

      </div>

      <!-- 底部操作栏 -->
      <div class="border-t px-4 py-3 border-gray-200 bg-white dark:border-gray-700 dark:bg-dark-secondary">
        <div v-if="!loading"
          class="flex justify-between items-center">
          <div class="flex items-center gap-3">
            <!-- 连接状态 -->
            <div class="flex items-center gap-2 text-xs">
              <div class="w-2 h-2 rounded-full bg-green-500"></div>
              <span class="text-gray-600 dark:text-gray-400">{{ connectionStatus }}</span>
              <span class="text-gray-400 dark:text-gray-500">|</span>
              <span class="text-gray-500 dark:text-gray-400">
                {{ request.predefined_options ? '选择选项或输入文本' : '⌘+回车 快速发送' }}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <!-- 继续按钮 -->
            <button @click="handleContinue"
              :disabled="submitting"
              class="btn btn-success"
              :class="submitting ? 'opacity-50' : ''">
              <span class="text-xs">▶</span>
              <span class="text-sm">继续</span>
            </button>

            <!-- 发送按钮 -->
            <button @click="handleSubmit"
              :disabled="!canSubmit || submitting"
              :class="[
                'btn',
                canSubmit && !submitting ? 'btn-primary' : 'bg-gray-300 text-gray-500 dark:bg-gray-600 dark:text-gray-400'
              ]">
              <!-- 加载动画 -->
              <div v-if="submitting"
                class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              <!-- 发送图标 -->
              <span v-else class="text-xs">↗</span>
              <span class="text-sm">{{ submitting ? '发送中...' : '发送' }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, computed, watch } from 'vue'
import { message } from '../utils/message.js'
import { VueMarkdownIt } from '@f3ve/vue-markdown-it'

interface McpRequest {
  id: string
  message: string
  predefined_options?: string[]
  is_markdown?: boolean
}

// Props
const props = defineProps<{
  request: McpRequest
}>()

// Emits
const emit = defineEmits<{
  response: [response: string]
  cancel: []
}>()

// 响应式数据
const userInput = ref('')
const selectedOptions = ref<string[]>([])
const submitting = ref(false)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const loading = ref(false)
const draggedImages = ref<string[]>([])

// 计算属性
const canSubmit = computed(() => {
  if (props.request?.predefined_options && props.request.predefined_options.length > 0) {
    return selectedOptions.value.length > 0 || userInput.value.trim()
  }
  return userInput.value.trim() || draggedImages.value.length > 0
})

const connectionStatus = computed(() => '已连接')

// Markdown 配置
const markdownOptions = {
  html: false,
  xhtmlOut: false,
  breaks: true,
  langPrefix: 'language-',
  linkify: true,
  typographer: true,
  quotes: '""\'\'',
}

// 提交输入
const submitInput = () => {
  if (canSubmit.value && !submitting.value) {
    handleSubmit()
  }
}

// 处理提交
const handleSubmit = async () => {
  if (!canSubmit.value || submitting.value) return

  submitting.value = true

  try {
    let response = ''
    
    // 组合响应内容
    const parts: string[] = []
    
    if (selectedOptions.value.length > 0) {
      parts.push(`选择的选项: ${selectedOptions.value.join(', ')}`)
    }
    
    if (userInput.value.trim()) {
      parts.push(userInput.value.trim())
    }
    
    if (draggedImages.value.length > 0) {
      parts.push(`[图片数量: ${draggedImages.value.length}]`)
      // 这里可以添加图片处理逻辑
    }
    
    response = parts.join('\n\n')
    
    if (response) {
      emit('response', response)
    }
  } catch (error) {
    console.error('提交失败:', error)
    message.error('提交失败，请重试')
  } finally {
    submitting.value = false
  }
}

// 处理继续
const handleContinue = () => {
  if (submitting.value) return
  emit('response', '请按照最佳实践继续完成')
}

// 处理图片拖拽
const handleImageDrop = (event: DragEvent) => {
  event.preventDefault()
  const files = event.dataTransfer?.files
  if (files) {
    handleImageFiles(files)
  }
}

const handleImagePaste = (event: ClipboardEvent) => {
  const items = event.clipboardData?.items
  if (items) {
    for (let i = 0; i < items.length; i++) {
      const item = items[i]
      if (item.type.indexOf('image') !== -1) {
        const file = item.getAsFile()
        if (file) {
          handleImageFiles([file])
        }
      }
    }
  }
}

const handleImageFiles = async (files: FileList | File[]) => {
  for (const file of files) {
    if (file.type.startsWith('image/')) {
      try {
        const base64 = await fileToBase64(file)
        draggedImages.value.push(base64)
        message.success(`图片 ${file.name} 已添加`)
      } catch (error) {
        message.error(`图片 ${file.name} 处理失败`)
      }
    }
  }
}

const fileToBase64 = (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

const removeImage = (index: number) => {
  draggedImages.value.splice(index, 1)
}

// 设置代码复制功能
const setupCodeCopy = () => {
  nextTick(() => {
    // 处理pre元素，添加复制按钮
    const preElements = document.querySelectorAll('.markdown-content pre')
    preElements.forEach((preEl) => {
      // 检查是否已经添加了复制按钮
      if (preEl.querySelector('.copy-button')) return

      // 创建复制按钮
      const copyButton = document.createElement('button')
      copyButton.className = 'copy-button absolute top-2 right-2 px-2 py-1 text-xs rounded transition-colors'
      copyButton.innerHTML = '📋 复制'

      // 设置按钮样式 
      copyButton.classList.add('bg-gray-200', 'text-gray-700', 'hover:bg-gray-300', 'dark:bg-gray-700', 'dark:text-gray-300', 'dark:hover:bg-gray-600')

      // 添加点击事件
      copyButton.addEventListener('click', async (e) => {
        e.stopPropagation()
        try {
          const codeEl = preEl.querySelector('code')
          const text = codeEl?.textContent || preEl.textContent || ''
          await navigator.clipboard.writeText(text)
          copyButton.innerHTML = '✅ 已复制'
          setTimeout(() => {
            copyButton.innerHTML = '📋 复制'
          }, 2000)
          message.success('代码已复制到剪贴板')
        } catch (err) {
          message.error('复制失败')
        }
      })

      // 设置pre元素为相对定位
      ; (preEl as HTMLElement).style.position = 'relative'
      preEl.appendChild(copyButton)
    })

    // 处理内联代码
    const inlineCodeElements = document.querySelectorAll('.markdown-content p code, .markdown-content li code')
    inlineCodeElements.forEach((codeEl) => {
      codeEl.addEventListener('click', async () => {
        try {
          await navigator.clipboard.writeText(codeEl.textContent || '')
          message.success('代码已复制到剪贴板')
        } catch (err) {
          message.error('复制失败')
        }
      })
    })
  })
}

// 生命周期
onMounted(() => {
  // 等待DOM更新后设置焦点和代码复制
  nextTick(() => {
    if (textareaRef.value) {
      textareaRef.value.focus()
    }
    setupCodeCopy()
  })
})

// 监听request变化，重新设置代码复制
watch(() => props.request, () => {
  setupCodeCopy()
}, { deep: true })
</script>

<style scoped>
/* 组件特定样式 */
.markdown-content {
  /* Markdown 内容样式 */
}

.markdown-content pre {
  background-color: #f6f8fa;
  border-radius: 6px;
  padding: 12px;
  overflow-x: auto;
  position: relative;
}

.dark .markdown-content pre {
  background-color: #2d2d2d;
}

.markdown-content code {
  background-color: rgba(175, 184, 193, 0.2);
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 0.85em;
  cursor: pointer;
}

.dark .markdown-content code {
  background-color: rgba(125, 125, 125, 0.3);
}

.checkbox-box {
  position: relative;
}

.checkbox input:checked + .checkbox-box::after {
  font-size: 10px;
  line-height: 1;
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .bg-gray-50 {
    background-color: #1f1f2b !important;
  }
  
  .bg-white {
    background-color: #272b3a !important;
  }
}
</style>
