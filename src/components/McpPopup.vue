<template>
  <div class="fixed inset-0 flex flex-col z-50 dark bg-dark-primary popup-container">
    <div class="relative w-full h-full flex flex-col bg-white dark:bg-dark-primary shadow-xl popup-content">

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
        <div v-else-if="request?.message"
          class="mb-4">
          <!-- 调试信息 -->
          <div v-if="false" class="mb-2 p-2 bg-yellow-100 dark:bg-yellow-900/20 rounded text-xs">
            <strong>调试信息:</strong> is_markdown = {{ request.is_markdown }},
            message length = {{ request.message?.length }}
          </div>

          <div
            class="bg-white dark:bg-dark-secondary rounded-lg p-4 shadow-sm border border-gray-200 dark:border-gray-700">
            <div class="leading-relaxed text-sm markdown-content text-gray-900 dark:text-gray-100 text-left">
              <div v-if="request.is_markdown"
                v-html="renderMarkdown(request.message)"
                class="markdown-rendered"></div>
              <div v-else
                class="whitespace-pre-wrap text-left">{{ request.message }}</div>
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
                class="checkbox flex items-center p-3 rounded-lg transition-all duration-200 group bg-white hover:bg-blue-50 dark:bg-dark-secondary dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-600 cursor-pointer smooth-option">
                <input type="checkbox"
                  :value="option"
                  v-model="selectedOptions"
                  class="sr-only" />
                <div class="checkbox-box"></div>
                <span
                  class="ml-3 text-sm text-gray-800 group-hover:text-blue-700 dark:text-gray-200 dark:group-hover:text-blue-300">{{
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

          <textarea ref="textareaRef"
            v-model="userInput"
            :placeholder="request.predefined_options ? '您可以在这里添加补充说明...' : '请输入您的回复...'"
            :rows="request.predefined_options ? 3 : 5"
            class="textarea smooth-textarea"
            :disabled="submitting"
            @keydown.meta.enter="submitInput"
            @keydown.stop
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
              class="btn btn-success smooth-button"
              :class="submitting ? 'opacity-50' : ''">
              <span class="text-xs">▶</span>
              <span class="text-sm">继续</span>
            </button>

            <!-- 发送按钮 -->
            <button @click="handleSubmit"
              :disabled="!canSubmit || submitting"
              :class="[
                'btn smooth-button',
                canSubmit && !submitting ? 'btn-primary' : 'bg-gray-300 text-gray-500 dark:bg-gray-600 dark:text-gray-400'
              ]">
              <!-- 加载动画 -->
              <div v-if="submitting"
                class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              <!-- 发送图标 -->
              <span v-else
                class="text-xs">↗</span>
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
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'

type McpRequest = {
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
  response: [response: any[]]
  cancel: []
}>()

// 响应式数据
const userInput = ref('')
const selectedOptions = ref<string[]>([])
const submitting = ref(false)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const loading = ref(false)
const draggedImages = ref<string[]>([])

// 计算属性 - 优化版本，减少不必要的计算
const canSubmit = computed(() => {
  const hasOptions = selectedOptions.value.length > 0
  const hasInput = userInput.value.trim().length > 0
  const hasImages = draggedImages.value.length > 0

  if (props.request?.predefined_options?.length) {
    return hasOptions || hasInput
  }
  return hasInput || hasImages
})

// 静态值，避免重复计算
const connectionStatus = '已连接'

// 创建 Markdown 实例
const md = new MarkdownIt({
  html: true, // 允许HTML标签
  xhtmlOut: false,
  breaks: true, // 换行符转换为<br>
  langPrefix: 'language-',
  linkify: true, // 自动转换链接
  typographer: true, // 启用智能引号等
  quotes: '""\'\'',
  // 启用代码高亮
  highlight: function (str: string, lang: string) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value
      } catch (__) {
        // 忽略错误
      }
    }
    return '' // 使用默认的转义
  }
})

// Markdown 渲染函数
const renderMarkdown = (content: string) => {
  try {
    return md.render(content)
  } catch (error) {
    console.error('Markdown 渲染失败:', error)
    return content // 如果渲染失败，返回原始内容
  }
}

// 提交输入
const submitInput = () => {
  if (canSubmit.value && !submitting.value) {
    handleSubmit()
  }
}

const buildTextContent = () => {
  const textParts: string[] = []
  if (selectedOptions.value.length > 0) {
    textParts.push(`选择的选项: ${selectedOptions.value.join(', ')}`)
  }
  if (userInput.value.trim()) {
    textParts.push(userInput.value.trim())
  }
  return textParts.length > 0 ? textParts.join('\n\n') : null
}

// 构建图片内容
const buildImageContent = () => {
  const imageContent: any[] = []
  for (const imageDataUrl of draggedImages.value) {
    const matches = imageDataUrl.match(/^data:(image\/[^;]+);base64,(.+)$/)
    if (matches) {
      const [, mediaType, base64Data] = matches
      imageContent.push({
        type: "image",
        source: { type: "base64", media_type: mediaType, data: base64Data }
      })
    }
  }
  return imageContent
}

// 处理提交
const handleSubmit = async () => {
  if (!canSubmit.value || submitting.value) return

  submitting.value = true
  try {
    const responseContent: any[] = []

    const textContent = buildTextContent()
    if (textContent) {
      responseContent.push({ type: "text", text: textContent })
    }

    responseContent.push(...buildImageContent())

    if (responseContent.length === 0) {
      responseContent.push({ type: "text", text: "用户确认继续" })
    }

    emit('response', responseContent)
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
  const continueResponse = [{
    type: "text",
    text: "请按照最佳实践继续完成"
  }]
  emit('response', continueResponse)
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
  let hasImage = false

  if (items) {
    for (const item of items) {
      if (item.type.includes('image')) {
        hasImage = true
        const file = item.getAsFile()
        if (file) {
          handleImageFiles([file])
        }
      }
    }
  }

  // 如果检测到图片，阻止默认的粘贴行为（防止文件名被粘贴到输入框）
  if (hasImage) {
    event.preventDefault()
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

// 创建复制按钮
const createCopyButton = (preEl: Element) => {
  if (preEl.querySelector('.copy-button')) return

  const copyButton = document.createElement('button')
  copyButton.className = 'copy-button absolute top-2 right-2 px-2 py-1 text-xs rounded transition-colors'
  copyButton.innerHTML = '📋 复制'
  copyButton.classList.add('bg-gray-200', 'text-gray-700', 'hover:bg-gray-300', 'dark:bg-gray-700', 'dark:text-gray-300', 'dark:hover:bg-gray-600')

  copyButton.addEventListener('click', async (e) => {
    e.stopPropagation()
    try {
      const codeEl = preEl.querySelector('code')
      const text = codeEl?.textContent || preEl.textContent || ''
      await navigator.clipboard.writeText(text)
      copyButton.innerHTML = '✅ 已复制'
      setTimeout(() => { copyButton.innerHTML = '📋 复制' }, 2000)
      message.success('代码已复制到剪贴板')
    } catch (err) {
      message.error('复制失败')
    }
  })

    ; (preEl as HTMLElement).style.position = 'relative'
  preEl.appendChild(copyButton)
}

// 设置内联代码复制
const setupInlineCodeCopy = () => {
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
}

// 设置代码复制功能 - 优化版本，使用防抖避免重复执行
let setupCodeCopyTimer: number | null = null
const setupCodeCopy = () => {
  // 清除之前的定时器，避免重复执行
  if (setupCodeCopyTimer) {
    clearTimeout(setupCodeCopyTimer)
  }

  setupCodeCopyTimer = window.setTimeout(() => {
    nextTick(() => {
      const preElements = document.querySelectorAll('.markdown-content pre')
      preElements.forEach(createCopyButton)
      setupInlineCodeCopy()
    })
  }, 50) // 短暂延迟，让DOM稳定后再执行
}

// 生命周期 - 优化版本，确保丝滑体验
onMounted(() => {
  // 使用 requestIdleCallback 在浏览器空闲时设置代码复制功能
  // 如果不支持则回退到 requestAnimationFrame
  if (window.requestIdleCallback) {
    window.requestIdleCallback(() => {
      setupCodeCopy()
    })
  } else {
    requestAnimationFrame(() => {
      setupCodeCopy()
    })
  }
})

// 监听request变化，重新设置代码复制
watch(() => props.request, () => {
  setupCodeCopy()
  // 移除自动焦点设置，让用户自然交互
}, { deep: true })
</script>

<style scoped>
/* 弹窗动画 - 确保丝滑体验 */
.popup-container {
  animation: fadeIn 0.15s ease-out;
}

.popup-content {
  animation: slideIn 0.2s ease-out;
  transform-origin: center;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* 按钮流畅动画 */
.smooth-button {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  transform: translateZ(0); /* 启用硬件加速 */
}

.smooth-button:hover:not(:disabled) {
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.smooth-button:active:not(:disabled) {
  transform: translateY(0) translateZ(0);
  transition-duration: 0.1s;
}

/* 输入框流畅动画 */
.smooth-textarea {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  transform: translateZ(0); /* 启用硬件加速 */
}

.smooth-textarea:focus {
  transform: translateZ(0);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

/* 选项卡流畅动画 */
.smooth-option {
  transform: translateZ(0); /* 启用硬件加速 */
}

.smooth-option:hover {
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.smooth-option:active {
  transform: translateY(0) translateZ(0);
  transition-duration: 0.1s;
}

/* 组件特定样式 */
.markdown-content {
  text-align: left;
  line-height: 1.6;
  color: inherit;
}

/* 标题样式 */
.markdown-content h1,
.markdown-content h2,
.markdown-content h3,
.markdown-content h4,
.markdown-content h5,
.markdown-content h6 {
  text-align: left;
  margin-top: 1.5em;
  margin-bottom: 0.75em;
  font-weight: 600;
  color: inherit;
}

.markdown-content h1 { font-size: 1.5em; }
.markdown-content h2 { font-size: 1.3em; }
.markdown-content h3 { font-size: 1.2em; }
.markdown-content h4 { font-size: 1.1em; }
.markdown-content h5 { font-size: 1em; }
.markdown-content h6 { font-size: 0.9em; }

/* 段落样式 */
.markdown-content p {
  text-align: left;
  margin-bottom: 1em;
  color: inherit;
}

/* 列表样式 */
.markdown-content ul,
.markdown-content ol {
  text-align: left;
  margin-left: 1.5em;
  margin-bottom: 1em;
  padding-left: 0;
}

.markdown-content li {
  text-align: left;
  margin-bottom: 0.5em;
  color: inherit;
}

/* 引用样式 */
.markdown-content blockquote {
  border-left: 4px solid #e5e7eb;
  margin: 1em 0;
  padding-left: 1em;
  color: #6b7280;
  font-style: italic;
}

.dark .markdown-content blockquote {
  border-left-color: #4b5563;
  color: #9ca3af;
}

/* 代码块样式 */
.markdown-content pre {
  background-color: #f6f8fa;
  border-radius: 6px;
  padding: 16px;
  overflow-x: auto;
  position: relative;
  margin: 1em 0;
  border: 1px solid #e1e4e8;
}

.dark .markdown-content pre {
  background-color: #161b22 !important;
  border-color: #30363d;
  color: #e6edf3 !important;
}

/* 确保代码块内的所有文本都可见 */
.dark .markdown-content pre code,
.dark .markdown-content pre .hljs {
  background-color: transparent !important;
  color: #e6edf3 !important;
}

/* 覆盖 highlight.js 的暗色主题样式 */
.dark .markdown-content .hljs {
  background: #161b22 !important;
  color: #e6edf3 !important;
}

.dark .markdown-content .hljs-keyword,
.dark .markdown-content .hljs-selector-tag,
.dark .markdown-content .hljs-literal,
.dark .markdown-content .hljs-section,
.dark .markdown-content .hljs-link {
  color: #ff7b72 !important;
}

.dark .markdown-content .hljs-string {
  color: #a5d6ff !important;
}

.dark .markdown-content .hljs-comment {
  color: #8b949e !important;
}

.dark .markdown-content .hljs-number {
  color: #79c0ff !important;
}

.dark .markdown-content .hljs-function,
.dark .markdown-content .hljs-title {
  color: #d2a8ff !important;
}

/* 行内代码样式 */
.markdown-content code {
  background-color: rgba(175, 184, 193, 0.2);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  cursor: pointer;
  color: #d73a49;
}

.dark .markdown-content code {
  background-color: rgba(110, 118, 129, 0.4);
  color: #f85149;
}

/* 代码块内的代码不需要背景 */
.markdown-content pre code {
  background-color: transparent;
  padding: 0;
  border-radius: 0;
  font-size: inherit;
}

/* 链接样式 */
.markdown-content a {
  color: #3b82f6;
  text-decoration: underline;
  cursor: pointer;
}

.markdown-content a:hover {
  color: #1d4ed8;
}

.dark .markdown-content a {
  color: #60a5fa;
}

.dark .markdown-content a:hover {
  color: #93c5fd;
}

/* 强调样式 */
.markdown-content strong {
  font-weight: 600;
}

.markdown-content em {
  font-style: italic;
}

/* 分隔线样式 */
.markdown-content hr {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 2em 0;
}

.dark .markdown-content hr {
  border-top-color: #4b5563;
}

.checkbox-box {
  position: relative;
}

.checkbox input:checked+.checkbox-box::after {
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
