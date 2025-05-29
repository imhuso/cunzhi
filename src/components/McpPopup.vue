<template>
  <div class="fixed inset-0 flex flex-col z-50 dark bg-gray-900">
    <div class="relative w-full h-full flex flex-col bg-white dark:bg-gray-900 shadow-xl">

      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4 bg-gray-50 dark:bg-gray-900"
        @drop="handleImageDrop"
        @dragover.prevent
        @dragenter.prevent>


        <!-- 简化的加载状态 -->
        <div v-if="loading"
          class="flex items-center justify-center py-12">
          <div class="text-center">
            <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4">
            </div>
            <p class="text-gray-600 dark:text-gray-400">加载中...</p>
          </div>
        </div>

        <!-- 消息显示区域 -->
        <div v-else-if="request && request.message"
          class="mb-6">
          <div class="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm border border-gray-200 dark:border-gray-700">
            <div class="leading-relaxed text-base markdown-content text-gray-900 dark:text-gray-100">
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
          class="rounded-xl p-6 border bg-red-50 border-red-200 dark:bg-red-900/20 dark:border-red-700/30">
          <div class="text-red-800 dark:text-red-300">
            <h4 class="font-medium mb-2">❌ 数据加载错误</h4>
            <p class="text-sm">Request对象: {{ JSON.stringify(request) }}</p>
          </div>
        </div>

        <!-- 预定义选项 -->
        <div v-if="!loading && request.predefined_options && request.predefined_options.length > 0"
          class="mb-6">
          <h4 class="text-lg font-medium mb-4 text-gray-800 dark:text-gray-200">请选择选项</h4>

          <a-checkbox-group v-model:value="selectedOptions"
            class="w-full">
            <div class="grid gap-3">
              <a-checkbox v-for="(option, index) in request.predefined_options"
                :key="`option-${index}`"
                :value="option"
                class="!flex items-center p-4 rounded-lg transition-colors group border-0 bg-white hover:bg-blue-50 dark:bg-gray-800 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-600">
                <span
                  class="ml-3 text-base text-gray-800 group-hover:text-blue-700 dark:text-gray-200 dark:group-hover:text-blue-300">{{
                  option }}</span>
              </a-checkbox>
            </div>
          </a-checkbox-group>
        </div>

        <!-- 图片预览区域 -->
        <div v-if="!loading && draggedImages.length > 0"
          class="mb-4">
          <h4 class="text-lg font-medium mb-3 text-gray-800 dark:text-gray-200">已添加的图片</h4>
          <div class="grid grid-cols-2 gap-3">
            <div v-for="(image, index) in draggedImages"
              :key="index"
              class="relative rounded-lg overflow-hidden border-2 border-gray-300 dark:border-gray-600">
              <img :src="image"
                class="w-full h-24 object-cover" />
              <button @click="removeImage(index)"
                class="absolute top-1 right-1 w-6 h-6 rounded-full flex items-center justify-center text-xs bg-red-500 text-white dark:bg-red-600">×</button>
            </div>
          </div>
        </div>

        <!-- 通用回复输入 -->
        <div v-if="!loading"
          class="mb-6">
          <h4 class="text-lg font-medium mb-4 text-gray-800 dark:text-gray-200">
            {{ request.predefined_options ? '补充说明 (可选)' : '请输入您的回复' }}
          </h4>

          <div
            class="relative rounded-lg border-2 border-dashed p-4 mb-3 border-gray-300 bg-gray-100 dark:border-gray-600 dark:bg-gray-800/50">
            <p class="text-sm text-center text-gray-600 dark:text-gray-400">
              拖拽图片到此处或在输入框中粘贴图片 (⌘+V)
            </p>
          </div>

          <a-textarea ref="textareaRef"
            v-model:value="userInput"
            :placeholder="request.predefined_options ? '您可以在这里添加补充说明...' : '请输入您的回复...'"
            :rows="request.predefined_options ? 4 : 6"
            class="resize-none rounded-lg text-base bg-gray-100 border border-gray-300 text-gray-900 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-100 dark:placeholder-gray-400 dark:focus:border-blue-400 dark:focus:ring-1 dark:focus:ring-blue-400/20 transition-colors"
            :disabled="submitting"
            @keydown.meta.enter="submitInput"
            @paste="handleImagePaste" />
        </div>


      </div>

      <!-- 底部操作栏 -->
      <div class="border-t px-6 py-4 border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-900">
        <div v-if="!loading"
          class="flex justify-between items-center">
          <div class="flex items-center gap-4">


            <!-- 连接状态 -->
            <div class="flex items-center gap-2 text-sm">
              <div class="w-2 h-2 rounded-full bg-green-500"></div>
              <span class="text-gray-600 dark:text-gray-400">{{ connectionStatus }}</span>
              <span class="text-gray-400 dark:text-gray-500">|</span>
              <span class="text-gray-500 dark:text-gray-400">
                {{ request.predefined_options ? '选择选项或输入文本' : '⌘+回车 快速发送' }}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <!-- 继续按钮 -->
            <button @click="handleContinue"
              :disabled="submitting"
              class="flex items-center gap-2 px-4 py-2.5 rounded-lg font-medium transition-colors disabled:cursor-not-allowed bg-green-500 text-white hover:bg-green-600 shadow-md hover:shadow-lg"
              :style="!submitting ? 'color: white !important;' : ''"
              :class="submitting ? 'opacity-50' : ''">
              <!-- 继续图标 -->
              <svg class="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 5l7 7-7 7" />
              </svg>
              <span>继续</span>
            </button>

            <!-- 发送按钮 -->
            <button @click="handleSubmit"
              :disabled="!canSubmit || submitting"
              class="flex items-center gap-2 px-6 py-2.5 rounded-lg font-medium transition-colors disabled:cursor-not-allowed"
              :class="[
                canSubmit && !submitting
                  ? 'bg-blue-500 text-white hover:bg-blue-600 shadow-md hover:shadow-lg'
                  : 'bg-gray-300 text-gray-500 dark:bg-gray-600 dark:text-gray-400'
              ]"
              :style="canSubmit && !submitting ? 'color: white !important;' : ''">
              <!-- 加载动画 -->
              <div v-if="submitting"
                class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              <!-- 发送图标 -->
              <svg v-else
                class="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
              </svg>
              <span>{{ submitting ? '发送中...' : '发送' }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, computed } from 'vue'
import { message } from 'ant-design-vue'
import { VueMarkdownIt } from '@f3ve/vue-markdown-it'
// import { useDark, useToggle } from '@vueuse/core'

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
const loading = ref(true)
const draggedImages = ref<string[]>([]) // 拖拽的图片
const connectionStatus = ref('已连接') // 连接状态



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
    const parts: string[] = []
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

// 处理继续按钮点击
const handleContinue = async () => {
  if (submitting.value) return
  
  // 直接发送固定文本
  await sendResponse('请按照最佳实践继续完成')
}



// 图片处理功能
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

const sendResponse = async (response: string) => {
  if (submitting.value) return

  submitting.value = true
  try {
    // 创建结构化的响应对象
    const responseData = {
      text: response,
      images: draggedImages.value.length > 0 ? draggedImages.value.map((image, index) => {
        const base64Data = image.replace(/^data:image\/[^;]+;base64,/, '')
        const mediaType = image.match(/^data:(image\/[^;]+);base64,/)?.[1] || 'image/jpeg'
        
        return {
          media_type: mediaType,
          data: base64Data
        }
      }) : null
    }

    // 为了向后兼容，仍然发送文本格式，但包含结构化数据
    let finalResponse = response

    // 如果有图片，添加图片信息（使用MCP标准格式）
    if (draggedImages.value.length > 0) {
      finalResponse += '\n\n'
      draggedImages.value.forEach((image, index) => {
        const base64Data = image.replace(/^data:image\/[^;]+;base64,/, '')
        const mediaType = image.match(/^data:(image\/[^;]+);base64,/)?.[1] || 'image/jpeg'

        finalResponse += `图片 ${index + 1}:\n`
        finalResponse += `{"type": "image", "source": {"type": "base64", "media_type": "${mediaType}", "data": "${base64Data}"}}\n\n`
      })
      finalResponse += `请分析以上 ${draggedImages.value.length} 张图片的内容。`
    }

    // 直接通过emit发送响应，让父组件处理
    emit('response', finalResponse)
  } catch (error) {
    console.error('发送响应失败:', error)
    message.error('发送响应失败，请重试')
    submitting.value = false
  }
}

// 代码复制功能
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

      // 设置按钮样式 - 使用固定的dark mode class
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
  // 延迟加载完成，确保DOM完全渲染
  setTimeout(() => {
    loading.value = false

    // 等待下一个渲染周期后设置功能
    nextTick(() => {
      setupCodeCopy()

      // 再等待一个周期后处理焦点，避免与窗口焦点冲突
      setTimeout(() => {
        if (!props.request.predefined_options && textareaRef.value) {
          textareaRef.value.focus()
        }
      }, 100)
    })
  }, 150) // 给组件更多时间完成渲染

  // 清空输入
  userInput.value = ''
  selectedOptions.value = []
})
</script>

<style scoped>
/* 简化的自定义样式 */

/* Markdown样式 - 支持dark mode */
.markdown-content :deep(.prose) {
  max-width: none;
  color: #111827;
}

.dark .markdown-content :deep(.prose) {
  color: #f3f4f6;
}

/* 标题样式 - 紧凑间距 */
.markdown-content :deep(h1) {
  font-size: 1.25rem;
  font-weight: 700;
  color: #2563eb;
  margin: 0.5rem 0 0.25rem 0;
}

.dark .markdown-content :deep(h1) {
  color: #60a5fa;
}

.markdown-content :deep(h2) {
  font-size: 1.125rem;
  font-weight: 600;
  color: #111827;
  margin: 0.5rem 0 0.25rem 0;
  border-bottom: 1px solid #d1d5db;
  padding-bottom: 0.125rem;
}

.dark .markdown-content :deep(h2) {
  color: #f3f4f6;
  border-bottom-color: #4b5563;
}

.markdown-content :deep(h3) {
  font-size: 1rem;
  font-weight: 500;
  color: #1f2937;
  margin: 0.25rem 0 0.125rem 0;
}

.dark .markdown-content :deep(h3) {
  color: #e5e7eb;
}

/* 文本样式 */
.markdown-content :deep(p) {
  margin: 0.25rem 0;
  line-height: 1.5;
}

.markdown-content :deep(strong) {
  font-weight: 600;
  color: #111827;
}

.dark .markdown-content :deep(strong) {
  color: #f3f4f6;
}

.markdown-content :deep(em) {
  font-style: italic;
  color: #374151;
}

.dark .markdown-content :deep(em) {
  color: #d1d5db;
}

/* 代码样式 - 支持复制 */
.markdown-content :deep(code) {
  background-color: #f3f4f6;
  color: #d97706;
  padding: 0.125rem 0.375rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  border: 1px solid #d1d5db;
  cursor: pointer;
  transition: all 0.2s;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace;
}

.dark .markdown-content :deep(code) {
  background-color: #1f2937;
  color: #fbbf24;
  border-color: #4b5563;
}

.markdown-content :deep(code:hover) {
  background-color: #e5e7eb;
  border-color: #9ca3af;
}

.dark .markdown-content :deep(code:hover) {
  background-color: #374151;
  border-color: #6b7280;
}

.markdown-content :deep(pre) {
  background-color: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 0.5rem;
  padding: 1rem;
  margin: 0.5rem 0;
  overflow-x: auto;
  position: relative;
}

.dark .markdown-content :deep(pre) {
  background-color: #1f2937;
  border-color: #4b5563;
}

.markdown-content :deep(pre code) {
  background: transparent;
  border: none;
  padding: 0;
  color: #111827;
}

.dark .markdown-content :deep(pre code) {
  color: #e5e7eb;
}

/* 列表样式 - 紧凑 */
.markdown-content :deep(ul) {
  list-style-type: disc;
  margin: 0.25rem 0 0.25rem 1rem;
  padding: 0;
}

.markdown-content :deep(ol) {
  list-style-type: decimal;
  margin: 0.25rem 0 0.25rem 1rem;
  padding: 0;
}

.markdown-content :deep(li) {
  margin: 0.125rem 0;
  line-height: 1.4;
}

/* 链接样式 */
.markdown-content :deep(a) {
  color: #60a5fa;
  text-decoration: underline;
  transition: color 0.2s;
}

.markdown-content :deep(a:hover) {
  color: #93c5fd;
}

/* 引用样式 */
.markdown-content :deep(blockquote) {
  border-left: 4px solid #4b5563;
  padding-left: 1rem;
  margin: 0.5rem 0;
  font-style: italic;
  color: #9ca3af;
  background-color: #1f2937;
  border-radius: 0 0.375rem 0.375rem 0;
}

/* 表格样式 */
.markdown-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5rem 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid #374151;
  padding: 0.5rem;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: #1f2937;
  font-weight: 600;
}

/* 分隔线 */
.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid #374151;
  margin: 1rem 0;
}
</style>
