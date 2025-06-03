<script setup>
import { invoke } from '@tauri-apps/api/core'
import { nextTick, ref, watch } from 'vue'
import { message } from '../utils/message.js'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['update:visible', 'close'])

// 响应式数据
const initPrompt = ref('')
const loading = ref(false)
const saving = ref(false)
const resetting = ref(false)
const textareaRef = ref(null)
const hasChanges = ref(false)
const originalPrompt = ref('')

// 监听弹窗显示状态
watch(() => props.visible, async (newVisible) => {
  if (newVisible) {
    await loadInitPrompt()
    // 等待DOM更新后聚焦
    nextTick(() => {
      if (textareaRef.value) {
        textareaRef.value.focus()
      }
    })
  } else {
    // 重置状态
    hasChanges.value = false
  }
})

// 监听提示词变化
watch(initPrompt, (newValue) => {
  hasChanges.value = newValue !== originalPrompt.value
})

// 加载当前提示词
async function loadInitPrompt() {
  loading.value = true
  try {
    const prompt = await invoke('get_init_prompt')
    initPrompt.value = prompt
    originalPrompt.value = prompt
    hasChanges.value = false
  } catch (error) {
    console.error('加载提示词失败:', error)
    message.error(`加载提示词失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 保存提示词
async function saveInitPrompt() {
  if (!initPrompt.value.trim()) {
    message.warning('提示词内容不能为空')
    return
  }

  saving.value = true
  try {
    await invoke('set_init_prompt', { prompt: initPrompt.value })
    originalPrompt.value = initPrompt.value
    hasChanges.value = false
    message.success('提示词保存成功')
    handleClose()
  } catch (error) {
    console.error('保存提示词失败:', error)
    message.error(`保存提示词失败: ${error}`)
  } finally {
    saving.value = false
  }
}

// 重置为默认
async function resetToDefault() {
  resetting.value = true
  try {
    await invoke('reset_init_prompt')
    await loadInitPrompt()
    message.success('已重置为默认提示词')
  } catch (error) {
    console.error('重置失败:', error)
    message.error(`重置失败: ${error}`)
  } finally {
    resetting.value = false
  }
}

// 处理取消
function handleCancel() {
  if (hasChanges.value) {
    if (confirm('您有未保存的更改，确定要关闭吗？')) {
      handleClose()
    }
  } else {
    handleClose()
  }
}

// 关闭弹窗
function handleClose() {
  emit('update:visible', false)
  emit('close')
}

// 快捷键处理
function handleKeydown(event) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault()
    saveInitPrompt()
  }
  if (event.key === 'Escape') {
    event.preventDefault()
    handleCancel()
  }
}
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click.self="handleCancel">
    <div class="modal">
      <!-- 模态框标题 -->
      <div class="modal-header">
        <div class="flex items-center gap-2">
          <span class="text-blue-500">⚙️</span>
          <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
            设置 Init 提示词
            <span v-if="hasChanges" class="text-yellow-500 text-lg ml-1">*</span>
          </h3>
        </div>
        <button
          type="button"
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          @click="handleCancel"
        >
          ✕
        </button>
      </div>

      <!-- 模态框内容 -->
      <div class="modal-body">
        <!-- 描述信息 -->
        <div class="alert alert-info mb-4">
          <div class="flex items-start gap-2">
            <span class="text-blue-500">ℹ️</span>
            <div>
              <p class="text-sm">当用户发送 'init' 命令时，系统将自动返回以下提示词内容</p>
            </div>
          </div>
        </div>

        <!-- 表单区域 -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            <div class="flex items-center gap-2">
              <span>📝</span>
              <span>提示词内容</span>
              <span v-if="hasChanges" class="text-yellow-500 text-xs">(已修改)</span>
            </div>
          </label>
          <textarea
            ref="textareaRef"
            v-model="initPrompt"
            class="textarea"
            rows="10"
            maxlength="2000"
            placeholder="请输入当用户发送 init 命令时要返回的提示词内容..."
            :disabled="loading || saving || resetting"
            @keydown="handleKeydown"
          />
          <div class="text-xs text-gray-500 dark:text-gray-400 mt-1 text-right">
            {{ initPrompt.length }}/2000
          </div>
        </div>

        <!-- 重置按钮 -->
        <div class="mb-4">
          <button
            type="button"
            class="btn btn-secondary"
            :disabled="loading || saving"
            @click="resetToDefault"
          >
            <span v-if="resetting">🔄</span>
            <span v-else>🔄</span>
            重置为默认
          </button>
        </div>
      </div>

      <!-- 模态框底部 -->
      <div class="modal-footer">
        <div class="flex items-center justify-between w-full">
          <div class="text-xs text-gray-500 dark:text-gray-400">
            💡 快捷键: Ctrl/Cmd + Enter 保存 | Escape 取消
          </div>
          <div class="flex gap-2">
            <button
              type="button"
              class="btn btn-secondary"
              :disabled="saving || resetting"
              @click="handleCancel"
            >
              ✕ 取消
            </button>
            <button
              type="button"
              class="btn btn-primary"
              :disabled="!initPrompt.trim() || loading || resetting || !hasChanges"
              @click="saveInitPrompt"
            >
              <span v-if="saving">⏳</span>
              <span v-else>✓</span>
              {{ saving ? '保存中...' : '保存设置' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 组件特定样式 */
.modal {
  max-width: 700px;
}

.textarea {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  line-height: 1.5;
  min-height: 200px;
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .modal {
    background-color: #272b3a;
  }
}
</style>
