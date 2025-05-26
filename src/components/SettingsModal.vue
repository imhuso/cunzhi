<script setup>
import {
  CheckOutlined,
  CloseOutlined,
  EditOutlined,
  ReloadOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'
import { nextTick, ref, watch } from 'vue'

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
    message.warning('提示词不能为空')
    return
  }

  if (!hasChanges.value) {
    message.info('没有需要保存的更改')
    return
  }

  saving.value = true
  try {
    await invoke('set_init_prompt', { prompt: initPrompt.value.trim() })
    originalPrompt.value = initPrompt.value.trim()
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

// 重置为默认提示词
async function resetToDefault() {
  resetting.value = true
  try {
    const defaultPrompt = await invoke('reset_init_prompt')
    initPrompt.value = defaultPrompt
    originalPrompt.value = defaultPrompt
    hasChanges.value = false
    message.success('已重置为默认提示词')
  } catch (error) {
    console.error('重置提示词失败:', error)
    message.error(`重置提示词失败: ${error}`)
  } finally {
    resetting.value = false
  }
}

// 关闭弹窗
function handleClose() {
  if (hasChanges.value) {
    // 如果有未保存的更改，询问用户
    const confirmed = confirm('您有未保存的更改，确定要关闭吗？')
    if (!confirmed) {
      return
    }
  }
  
  emit('update:visible', false)
  emit('close')
}

// 键盘快捷键
function handleKeydown(event) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    saveInitPrompt()
  } else if (event.key === 'Escape') {
    event.preventDefault()
    handleClose()
  }
}

// 取消编辑
function handleCancel() {
  if (hasChanges.value) {
    // 恢复原始值
    initPrompt.value = originalPrompt.value
    hasChanges.value = false
  }
  handleClose()
}
</script>

<template>
  <a-modal
    :open="visible"
    title="设置 Init 提示词"
    width="800px"
    :confirm-loading="saving"
    :mask-closable="!hasChanges"
    :keyboard="false"
    centered
    @ok="saveInitPrompt"
    @cancel="handleCancel"
  >
    <template #title>
      <div class="modal-title">
        <SettingOutlined class="title-icon" />
        <span>设置 Init 提示词</span>
        <span v-if="hasChanges" class="changes-indicator">*</span>
      </div>
    </template>

    <div class="settings-content">
      <div class="description">
        <a-alert
          message="当用户发送 'init' 命令时，系统将自动返回以下提示词内容"
          type="info"
          show-icon
          class="info-alert"
        />
      </div>

      <div class="form-section">
        <a-form layout="vertical">
          <a-form-item label="提示词内容" class="prompt-form-item">
            <template #label>
              <div class="form-label">
                <EditOutlined />
                <span>提示词内容</span>
                <span v-if="hasChanges" class="text-orange-500 text-xs ml-2">(已修改)</span>
              </div>
            </template>
            <a-textarea
              ref="textareaRef"
              v-model:value="initPrompt"
              :rows="12"
              :max-length="2000"
              show-count
              placeholder="请输入当用户发送 init 命令时要返回的提示词内容..."
              :loading="loading"
              :disabled="loading || saving || resetting"
              class="prompt-textarea"
              @keydown="handleKeydown"
            />
          </a-form-item>
        </a-form>
      </div>

      <div class="action-section">
        <a-button
          type="default"
          :loading="resetting"
          :disabled="loading || saving"
          class="reset-btn"
          @click="resetToDefault"
        >
          <template #icon>
            <ReloadOutlined />
          </template>
          重置为默认
        </a-button>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <div class="shortcuts-hint">
          <span class="hint-text">
            💡 快捷键: Ctrl/Cmd + Enter 保存 | Escape 取消
          </span>
        </div>
        <div class="action-buttons">
          <a-button 
            @click="handleCancel"
            :disabled="saving || resetting">
            <template #icon>
              <CloseOutlined />
            </template>
            取消
          </a-button>
          <a-button
            type="primary"
            :loading="saving"
            :disabled="!initPrompt.trim() || loading || resetting || !hasChanges"
            @click="saveInitPrompt"
          >
            <template #icon>
              <CheckOutlined />
            </template>
            保存设置
          </a-button>
        </div>
      </div>
    </template>
  </a-modal>
</template>

<style scoped>
.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #1f2937;
  font-weight: 600;
}

.title-icon {
  color: #3b82f6;
  font-size: 16px;
}

.changes-indicator {
  color: #f59e0b;
  font-size: 18px;
  margin-left: 4px;
}

.settings-content {
  padding: 8px 0;
}

.description {
  margin-bottom: 20px;
}

.info-alert {
  border-radius: 8px;
}

.form-section {
  margin-bottom: 20px;
}

.prompt-form-item {
  margin-bottom: 0;
}

.form-label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #1f2937;
  font-weight: 600;
  font-size: 14px;
}

.prompt-textarea {
  border-radius: 8px;
  transition: all 0.3s ease;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  line-height: 1.6;
}

.prompt-textarea:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.action-section {
  display: flex;
  justify-content: flex-start;
}

.reset-btn {
  border-radius: 6px;
  font-weight: 500;
}

.reset-btn:hover:not(:disabled) {
  border-color: #f59e0b;
  color: #f59e0b;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.shortcuts-hint {
  flex: 1;
}

.hint-text {
  color: #6b7280;
  font-size: 12px;
  opacity: 0.8;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.action-buttons .ant-btn {
  border-radius: 6px;
  font-weight: 500;
  min-width: 80px;
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .modal-title {
    color: #f9fafb;
  }

  .form-label {
    color: #f9fafb;
  }

  .hint-text {
    color: #9ca3af;
  }
}
</style>
