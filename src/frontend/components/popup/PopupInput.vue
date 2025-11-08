<script setup lang="ts">
import type { CustomPrompt, McpRequest } from '../../types/popup'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useSortable } from '@vueuse/integrations/useSortable'
import { useMessage } from 'naive-ui'
import { computed, nextTick, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useKeyboard } from '../../composables/useKeyboard'

interface Props {
  request: McpRequest | null
  loading?: boolean
  submitting?: boolean
}

interface Emits {
  update: [data: {
    userInput: string
    selectedOptions: string[]
    draggedImages: string[]
  }]
  imageAdd: [image: string]
  imageRemove: [index: number]
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  submitting: false,
})

const emit = defineEmits<Emits>()

// 响应式数据
const userInput = ref('')
const selectedOptions = ref<string[]>([])
const uploadedImages = ref<string[]>([])
const textareaRef = ref<HTMLTextAreaElement | null>(null)

// 自定义prompt相关状态
const customPrompts = ref<CustomPrompt[]>([])
const customPromptEnabled = ref(true)
const showInsertDialog = ref(false)
const pendingPromptContent = ref('')

// 移除条件性prompt状态管理，直接使用prompt的current_state

// 分离普通prompt和条件性prompt
const normalPrompts = computed(() =>
  customPrompts.value.filter(prompt => prompt.type === 'normal' || !prompt.type),
)

const conditionalPrompts = computed(() =>
  customPrompts.value.filter(prompt => prompt.type === 'conditional'),
)

// 拖拽排序相关状态
const promptContainer = ref<HTMLElement | null>(null)
const sortablePrompts = shallowRef<CustomPrompt[]>([])
const { start, stop } = useSortable(promptContainer, sortablePrompts, {
  animation: 200,
  ghostClass: 'sortable-ghost',
  chosenClass: 'sortable-chosen',
  dragClass: 'sortable-drag',
  handle: '.drag-handle',
  forceFallback: true,
  fallbackTolerance: 3,
  onStart: (evt) => {
    console.log('PopupInput: 拖拽开始:', evt)
    console.log('PopupInput: 拖拽开始时的容器:', evt.from)
    console.log('PopupInput: 拖拽开始时的元素:', evt.item)
  },
  onEnd: (evt) => {
    console.log('PopupInput: 拖拽排序完成:', evt)
    console.log('PopupInput: 从索引', evt.oldIndex, '移动到索引', evt.newIndex)
    console.log('PopupInput: 拖拽后的sortablePrompts:', sortablePrompts.value.map(p => ({ id: p.id, name: p.name })))

    // 检查是否真的发生了位置变化
    if (evt.oldIndex !== evt.newIndex && evt.oldIndex !== undefined && evt.newIndex !== undefined) {
      // 手动重新排列数组
      const newList = [...sortablePrompts.value]
      const [movedItem] = newList.splice(evt.oldIndex, 1)
      newList.splice(evt.newIndex, 0, movedItem)

      // 更新sortablePrompts
      sortablePrompts.value = newList
      console.log('PopupInput: 手动更新后的sortablePrompts:', sortablePrompts.value.map(p => ({ id: p.id, name: p.name })))

      // 立即更新 customPrompts 的顺序，确保数据同步
      // 保留条件性prompt，只更新普通prompt的顺序
      const conditionalPromptsList = customPrompts.value.filter(prompt => prompt.type === 'conditional')
      customPrompts.value = [...sortablePrompts.value, ...conditionalPromptsList]
      console.log('PopupInput: 位置发生变化，保存新排序')

      // 立即保存排序
      savePromptOrder()
    }
    else {
      console.log('PopupInput: 位置未发生变化，无需保存')
    }
  },
  onMove: (evt) => {
    console.log('PopupInput: 拖拽移动中:', evt)
    return true // 允许移动
  },
  onChoose: (evt) => {
    console.log('PopupInput: 选择拖拽元素:', evt)
  },
  onUnchoose: (evt) => {
    console.log('PopupInput: 取消选择拖拽元素:', evt)
  },
})

// 使用键盘快捷键 composable
const { pasteShortcut } = useKeyboard()

const message = useMessage()

// 计算属性
const hasOptions = computed(() => (props.request?.predefined_options?.length ?? 0) > 0)
const canSubmit = computed(() => {
  const hasOptionsSelected = selectedOptions.value.length > 0
  const hasInputText = userInput.value.trim().length > 0
  const hasImages = uploadedImages.value.length > 0

  if (hasOptions.value) {
    return hasOptionsSelected || hasInputText || hasImages
  }
  return hasInputText || hasImages
})

// 工具栏状态文本
const statusText = computed(() => {
  // 检查是否有任何输入内容
  const hasInput = selectedOptions.value.length > 0
    || uploadedImages.value.length > 0
    || userInput.value.trim().length > 0

  // 如果有任何输入内容，返回空字符串让 PopupActions 显示快捷键
  if (hasInput) {
    return ''
  }

  return '等待输入...'
})

// 发送更新事件
function emitUpdate() {
  // 获取条件性prompt的追加内容
  const conditionalContent = generateConditionalContent()

  // 将条件性内容追加到用户输入
  const finalUserInput = userInput.value + conditionalContent

  emit('update', {
    userInput: finalUserInput,
    selectedOptions: selectedOptions.value,
    draggedImages: uploadedImages.value,
  })
}

// 处理选项变化
function handleOptionChange(option: string, checked: boolean) {
  if (checked) {
    selectedOptions.value.push(option)
  }
  else {
    const idx = selectedOptions.value.indexOf(option)
    if (idx > -1)
      selectedOptions.value.splice(idx, 1)
  }
  emitUpdate()
}

// 处理选项切换（整行点击）
function handleOptionToggle(option: string) {
  const idx = selectedOptions.value.indexOf(option)
  if (idx > -1) {
    selectedOptions.value.splice(idx, 1)
  }
  else {
    selectedOptions.value.push(option)
  }
  emitUpdate()
}

// 移除了所有拖拽和上传组件相关的代码

function handleImagePaste(event: ClipboardEvent) {
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

  if (hasImage) {
    event.preventDefault()
  }
}

async function handleImageFiles(files: FileList | File[]): Promise<void> {
  console.log('=== 处理图片文件 ===')
  console.log('文件数量:', files.length)

  for (const file of files) {
    console.log('处理文件:', file.name, '类型:', file.type, '大小:', file.size)

    if (file.type.startsWith('image/')) {
      try {
        console.log('开始转换为 Base64...')
        const base64 = await fileToBase64(file)
        console.log('Base64转换成功，长度:', base64.length)

        // 检查是否已存在相同图片，避免重复添加
        if (!uploadedImages.value.includes(base64)) {
          uploadedImages.value.push(base64)
          console.log('图片已添加到数组，当前数量:', uploadedImages.value.length)
          message.success(`图片 ${file.name} 已添加`)
          emitUpdate()
        }
        else {
          console.log('图片已存在，跳过:', file.name)
          message.warning(`图片 ${file.name} 已存在`)
        }
      }
      catch (error) {
        console.error('图片处理失败:', error)
        message.error(`图片 ${file.name} 处理失败`)
        throw error
      }
    }
    else {
      console.log('跳过非图片文件:', file.type)
    }
  }

  console.log('=== 图片文件处理完成 ===')
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

function removeImage(index: number) {
  uploadedImages.value.splice(index, 1)
  emit('imageRemove', index)
  emitUpdate()
}

// 移除自定义图片预览功能，改用 Naive UI 的内置预览

// 加载自定义prompt配置
async function loadCustomPrompts() {
  try {
    console.log('PopupInput: 开始加载自定义prompt配置')
    const config = await invoke('get_custom_prompt_config')
    if (config) {
      const promptConfig = config as any

      // 按sort_order排序
      customPrompts.value = (promptConfig.prompts || []).sort((a: CustomPrompt, b: CustomPrompt) => a.sort_order - b.sort_order)
      customPromptEnabled.value = promptConfig.enabled ?? true
      console.log('PopupInput: 加载到的prompt数量:', customPrompts.value.length)
      console.log('PopupInput: 条件性prompt列表:', customPrompts.value.filter(p => p.type === 'conditional'))

      // 同步到拖拽列表（只包含普通prompt）
      sortablePrompts.value = [...normalPrompts.value]
      console.log('PopupInput: 同步到sortablePrompts:', sortablePrompts.value.length)

      // 延迟初始化拖拽功能，等待组件完全挂载
      if (customPrompts.value.length > 0) {
        console.log('PopupInput: 准备启动拖拽功能')
        initializeDragSort()
      }
      else {
        console.log('PopupInput: 没有prompt，跳过拖拽初始化')
      }
    }
  }
  catch (error) {
    console.error('PopupInput: 加载自定义prompt失败:', error)
  }
}

// 处理自定义prompt点击
function handlePromptClick(prompt: CustomPrompt) {
  // 如果prompt内容为空或只有空格，直接清空输入框
  if (!prompt.content || prompt.content.trim() === '') {
    userInput.value = ''
    emitUpdate()
    return
  }

  if (userInput.value.trim()) {
    // 如果输入框有内容，显示插入选择对话框
    pendingPromptContent.value = prompt.content
    showInsertDialog.value = true
  }
  else {
    // 如果输入框为空，直接插入
    insertPromptContent(prompt.content)
  }
}

// 处理引用消息内容
function handleQuoteMessage(messageContent: string) {
  if (userInput.value.trim()) {
    // 输入框有内容，显示插入选择对话框
    pendingPromptContent.value = messageContent
    showInsertDialog.value = true
  }
  else {
    // 输入框为空，直接插入
    insertPromptContent(messageContent)
    message.success('原文内容已引用到输入框')
  }
}

// 插入prompt内容
function insertPromptContent(content: string, mode: 'replace' | 'append' = 'replace') {
  if (mode === 'replace') {
    userInput.value = content
  }
  else {
    userInput.value = userInput.value.trim() + (userInput.value.trim() ? '\n\n' : '') + content
  }

  // 聚焦到输入框
  setTimeout(() => {
    if (textareaRef.value) {
      textareaRef.value.focus()
      // 尝试将光标移到末尾（对于Naive UI组件）
      try {
        const inputElement = textareaRef.value.$el?.querySelector('textarea') || textareaRef.value.inputElRef
        if (inputElement && typeof inputElement.setSelectionRange === 'function') {
          inputElement.setSelectionRange(inputElement.value.length, inputElement.value.length)
        }
      }
      catch (error) {
        console.log('设置光标位置失败:', error)
      }
    }
  }, 100)

  emitUpdate()
}

// 处理插入模式选择
function handleInsertMode(mode: 'replace' | 'append') {
  insertPromptContent(pendingPromptContent.value, mode)
  showInsertDialog.value = false
  pendingPromptContent.value = ''
}

// 处理条件性prompt开关变化
async function handleConditionalToggle(promptId: string, value: boolean) {
  // 先更新本地状态
  const prompt = customPrompts.value.find(p => p.id === promptId)
  if (prompt) {
    prompt.current_state = value
  }

  // 保存到后端
  try {
    await invoke('update_conditional_prompt_state', {
      promptId,
      newState: value,
    })
    message.success('上下文追加状态已保存')
  }
  catch (error) {
    console.error('保存条件性prompt状态失败:', error)
    message.error(`保存设置失败: ${(error as any)?.message}` || error)

    // 回滚本地状态
    if (prompt) {
      prompt.current_state = !value
    }
  }
}

// 生成条件性prompt的追加内容
function generateConditionalContent(): string {
  const conditionalTexts: string[] = []

  conditionalPrompts.value.forEach((prompt) => {
    const isEnabled = prompt.current_state ?? false
    const template = isEnabled ? prompt.template_true : prompt.template_false

    if (template && template.trim()) {
      conditionalTexts.push(template.trim())
    }
  })

  return conditionalTexts.length > 0 ? `\n\n${conditionalTexts.join('\n')}` : ''
}

// 获取条件性prompt的自适应描述
function getConditionalDescription(prompt: CustomPrompt): string {
  const isEnabled = prompt.current_state ?? false
  const template = isEnabled ? prompt.template_true : prompt.template_false

  // 如果有对应状态的模板，显示模板内容，否则显示原始描述
  if (template && template.trim()) {
    return template.trim()
  }

  return prompt.description || ''
}

// 移除拖拽排序初始化函数

// 初始化拖拽排序功能
async function initializeDragSort() {
  console.log('PopupInput: initializeDragSort 被调用')

  // 等待多个tick确保DOM完全渲染
  await nextTick()
  await nextTick()

  // 使用更长的延迟
  setTimeout(async () => {
    console.log('PopupInput: 开始查找容器')

    // 尝试多种方式查找容器
    let targetContainer = promptContainer.value

    if (!targetContainer) {
      targetContainer = document.querySelector('[data-prompt-container]') as HTMLElement
      console.log('PopupInput: querySelector结果:', targetContainer)
    }

    if (!targetContainer) {
      // 尝试通过类名查找
      const containers = document.querySelectorAll('.flex.flex-wrap')
      console.log('PopupInput: 找到的flex容器数量:', containers.length)
      for (let i = 0; i < containers.length; i++) {
        const container = containers[i] as HTMLElement
        if (container.querySelector('.sortable-item')) {
          targetContainer = container
          console.log('PopupInput: 通过sortable-item找到容器')
          break
        }
      }
    }

    if (targetContainer) {
      console.log('PopupInput: 找到目标容器:', targetContainer)
      const dragHandles = targetContainer.querySelectorAll('.drag-handle')
      console.log('PopupInput: 找到拖拽手柄数量:', dragHandles.length)

      const sortableItems = targetContainer.querySelectorAll('.sortable-item')
      console.log('PopupInput: 找到可排序项数量:', sortableItems.length)

      // 更新容器引用
      promptContainer.value = targetContainer

      console.log('PopupInput: 调用start()函数')
      start()
      console.log('PopupInput: start()函数调用完成')
    }
    else {
      console.log('PopupInput: 无法找到容器，DOM可能还没有渲染')
      console.log('PopupInput: 当前页面所有带data-prompt-container的元素:', document.querySelectorAll('[data-prompt-container]'))
      console.log('PopupInput: 当前页面所有.sortable-item元素:', document.querySelectorAll('.sortable-item'))
    }
  }, 500) // 增加延迟时间
}

// 保存prompt排序
async function savePromptOrder() {
  try {
    console.log('savePromptOrder被调用')
    console.log('当前sortablePrompts:', sortablePrompts.value.map(p => ({ id: p.id, name: p.name })))
    const promptIds = sortablePrompts.value.map(p => p.id)
    console.log('开始保存排序，prompt IDs:', promptIds)

    const startTime = Date.now()
    await invoke('update_custom_prompt_order', { promptIds })
    const endTime = Date.now()

    console.log(`排序已保存，耗时: ${endTime - startTime}ms`)
    message.success('排序已保存')
  }
  catch (error) {
    console.error('保存排序失败:', error)
    message.error('保存排序失败')
    // 重新加载以恢复原始顺序
    loadCustomPrompts()
  }
}

// 监听用户输入变化
watch(userInput, () => {
  emitUpdate()
})

// 移除拖拽相关的监听器

// 事件监听器引用
let unlistenCustomPromptUpdate: (() => void) | null = null
let unlistenWindowMove: (() => void) | null = null

// 修复输入法候选框位置的函数
function fixIMEPosition() {
  if (textareaRef.value) {
    try {
      // 获取实际的 textarea 元素（Naive UI 的 n-input）
      const inputElement = (textareaRef.value as any).$el?.querySelector('textarea') || (textareaRef.value as any).inputElRef
      
      if (inputElement && document.activeElement === inputElement) {
        // 先失焦再聚焦，让输入法重新计算位置
        inputElement.blur()
        setTimeout(() => {
          inputElement.focus()
        }, 10)
      }
    }
    catch (error) {
      console.debug('修复IME位置失败:', error)
    }
  }
}

// 设置窗口移动监听器
async function setupWindowMoveListener() {
  try {
    const webview = getCurrentWebviewWindow()
    
    // 监听窗口移动事件
    unlistenWindowMove = await webview.onMoved(() => {
      // 窗口移动后修复输入法位置
      fixIMEPosition()
    })
    
    console.log('窗口移动监听器已设置')
  }
  catch (error) {
    console.error('设置窗口移动监听器失败:', error)
  }
}

// 组件挂载时加载自定义prompt
onMounted(async () => {
  console.log('组件挂载，开始加载prompt')
  await loadCustomPrompts()

  // 监听自定义prompt更新事件
  unlistenCustomPromptUpdate = await listen('custom-prompt-updated', () => {
    console.log('收到自定义prompt更新事件，重新加载数据')
    loadCustomPrompts()
  })
  
  // 设置窗口移动监听器
  setupWindowMoveListener()
})

onUnmounted(() => {
  // 清理事件监听器
  if (unlistenCustomPromptUpdate) {
    unlistenCustomPromptUpdate()
  }
  
  // 清理窗口移动监听器
  if (unlistenWindowMove) {
    unlistenWindowMove()
  }

  // 停止拖拽功能
  stop()
})

// 重置数据
function reset() {
  userInput.value = ''
  selectedOptions.value = []
  uploadedImages.value = []
  emitUpdate()
}

// 更新数据（用于外部同步）
function updateData(data: { userInput?: string, selectedOptions?: string[], draggedImages?: string[] }) {
  if (data.userInput !== undefined) {
    userInput.value = data.userInput
  }
  if (data.selectedOptions !== undefined) {
    selectedOptions.value = data.selectedOptions
  }
  if (data.draggedImages !== undefined) {
    uploadedImages.value = data.draggedImages
  }

  emitUpdate()
}

// 移除了文件选择和测试图片功能

// 暴露方法给父组件
defineExpose({
  reset,
  canSubmit,
  statusText,
  updateData,
  handleQuoteMessage,
})
</script>

<template>
  <div class="space-y-3">
    <!-- 预定义选项 -->
    <div v-if="!loading && hasOptions" class="space-y-3" data-guide="predefined-options">
      <div>
        <h4 class="text-sm font-medium text-white">
          请选择选项
        </h4>
        <div v-if="request?.session_id" class="text-xs opacity-60 mt-1">
          工作目录: {{ request.session_id }}
        </div>
      </div>
      <n-space vertical size="small">
        <div
          v-for="(option, index) in request!.predefined_options"
          :key="`option-${index}`"
          class="rounded-lg p-3 border border-gray-600 bg-gray-100 cursor-pointer hover:opacity-80 transition-opacity"
          @click="handleOptionToggle(option)"
        >
          <n-checkbox
            :value="option"
            :checked="selectedOptions.includes(option)"
            :disabled="submitting"
            size="medium"
            @update:checked="(checked: boolean) => handleOptionChange(option, checked)"
            @click.stop
          >
            {{ option }}
          </n-checkbox>
        </div>
      </n-space>
    </div>

    <!-- 图片预览区域 -->
    <div v-if="!loading && uploadedImages.length > 0" class="space-y-3">
      <h4 class="text-sm font-medium text-white">
        已添加的图片 ({{ uploadedImages.length }})
      </h4>

      <!-- 使用 Naive UI 的图片组件，支持预览和放大 -->
      <n-image-group>
        <div class="flex flex-wrap gap-3">
          <div
            v-for="(image, index) in uploadedImages"
            :key="`image-${index}`"
            class="relative"
          >
            <!-- 使用 n-image 组件，启用预览功能 -->
            <n-image
              :src="image"
              width="100"
              height="100"
              object-fit="cover"
              class="rounded-lg border-2 border-gray-300 hover:border-primary-400 transition-all duration-200 cursor-pointer"
            />

            <!-- 删除按钮 -->
            <n-button
              class="absolute -top-2 -right-2 z-10"
              size="tiny"
              type="error"
              circle
              @click="removeImage(index)"
            >
              <template #icon>
                <div class="i-carbon-close w-3 h-3" />
              </template>
            </n-button>

            <!-- 序号 -->
            <div class="absolute bottom-1 left-1 w-5 h-5 bg-primary-500 text-white text-xs rounded-full flex items-center justify-center font-bold shadow-sm z-5">
              {{ index + 1 }}
            </div>
          </div>
        </div>
      </n-image-group>
    </div>

    <!-- 文本输入区域 -->
    <div v-if="!loading" class="space-y-3">
      <h4 class="text-sm font-medium text-white">
        {{ hasOptions ? '补充说明 (可选)' : '请输入您的回复' }}
      </h4>

      <!-- 自定义prompt按钮区域 -->
      <div v-if="customPromptEnabled && customPrompts.length > 0" class="space-y-2" data-guide="custom-prompts">
        <div class="text-xs text-on-surface-secondary flex items-center gap-2">
          <div class="i-carbon-bookmark w-3 h-3 text-primary-500" />
          <span>快捷模板 (拖拽调整顺序):</span>
        </div>
        <div
          ref="promptContainer"
          data-prompt-container
          class="flex flex-wrap gap-2"
        >
          <div
            v-for="prompt in sortablePrompts"
            :key="prompt.id"
            :title="prompt.description || (prompt.content.trim() ? prompt.content : '清空输入框')"
            class="inline-flex items-center gap-1 px-2 py-1 text-xs bg-container-secondary hover:bg-container-tertiary rounded transition-all duration-200 select-none border border-gray-600 text-on-surface sortable-item"
          >
            <!-- 拖拽手柄 -->
            <div class="drag-handle cursor-move p-0.5 rounded hover:bg-container-tertiary transition-colors">
              <div class="i-carbon-drag-horizontal w-3 h-3 text-on-surface-secondary" />
            </div>

            <!-- 按钮内容 -->
            <div
              class="inline-flex items-center cursor-pointer"
              @click="handlePromptClick(prompt)"
            >
              <span>{{ prompt.name }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 上下文追加区域 -->
      <div v-if="customPromptEnabled && conditionalPrompts.length > 0" class="space-y-2" data-guide="context-append">
        <div class="text-xs text-on-surface-secondary flex items-center gap-2">
          <div class="i-carbon-settings-adjust w-3 h-3 text-primary-500" />
          <span>上下文追加:</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div
            v-for="prompt in conditionalPrompts"
            :key="prompt.id"
            class="flex items-center justify-between p-2 bg-container-secondary rounded border border-gray-600 hover:bg-container-tertiary transition-colors text-xs"
          >
            <div class="flex-1 min-w-0 mr-2">
              <div class="text-xs text-on-surface truncate font-medium" :title="prompt.condition_text || prompt.name">
                {{ prompt.condition_text || prompt.name }}
              </div>
              <div v-if="getConditionalDescription(prompt)" class="text-xs text-primary-600 dark:text-primary-400 opacity-50 dark:opacity-60 mt-0.5 truncate leading-tight" :title="getConditionalDescription(prompt)">
                {{ getConditionalDescription(prompt) }}
              </div>
            </div>
            <n-switch
              :value="prompt.current_state ?? false"
              size="small"
              @update:value="(value: boolean) => handleConditionalToggle(prompt.id, value)"
            />
          </div>
        </div>
      </div>

      <!-- 图片提示区域 -->
      <div v-if="uploadedImages.length === 0" class="text-center">
        <div class="text-xs text-on-surface-secondary">
          💡 提示：可以在输入框中粘贴图片 ({{ pasteShortcut }})
        </div>
      </div>

      <!-- 文本输入框 -->
      <n-input
        ref="textareaRef"
        v-model:value="userInput"
        type="textarea"
        size="small"
        :placeholder="hasOptions ? `您可以在这里添加补充说明... (支持粘贴图片 ${pasteShortcut})` : `请输入您的回复... (支持粘贴图片 ${pasteShortcut})`"
        :disabled="submitting"
        :autosize="{ minRows: 3, maxRows: 6 }"
        data-guide="popup-input"
        @paste="handleImagePaste"
      />
    </div>

    <!-- 插入模式选择对话框 -->
    <n-modal v-model:show="showInsertDialog" preset="dialog" title="插入模式选择">
      <template #header>
        <div class="flex items-center gap-2">
          <div class="i-carbon-text-creation w-4 h-4" />
          <span>插入Prompt</span>
        </div>
      </template>
      <div class="space-y-4">
        <p class="text-sm text-on-surface-secondary">
          输入框中已有内容，请选择插入模式：
        </p>
        <div class="bg-container-secondary p-3 rounded text-sm">
          {{ pendingPromptContent }}
        </div>
      </div>
      <template #action>
        <div class="flex gap-2">
          <n-button @click="showInsertDialog = false">
            取消
          </n-button>
          <n-button type="warning" @click="handleInsertMode('replace')">
            替换内容
          </n-button>
          <n-button type="primary" @click="handleInsertMode('append')">
            追加内容
          </n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
/* Sortable.js 拖拽样式 */
.sortable-ghost {
  opacity: 0.5;
  transform: scale(0.95);
}

.sortable-chosen {
  cursor: grabbing !important;
}

.sortable-drag {
  opacity: 0.8;
  transform: rotate(5deg);
}
</style>
