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
  }
})

// 加载当前提示词
async function loadInitPrompt() {
  loading.value = true
  try {
    initPrompt.value = await invoke('get_init_prompt')
  }
  catch (error) {
    console.error('加载提示词失败:', error)
    message.error('加载提示词失败')
  }
  finally {
    loading.value = false
  }
}

// 保存提示词
async function saveInitPrompt() {
  if (!initPrompt.value.trim()) {
    message.warning('提示词不能为空')
    return
  }

  saving.value = true
  try {
    await invoke('set_init_prompt', { prompt: initPrompt.value.trim() })
    message.success('提示词保存成功')
    handleClose()
  }
  catch (error) {
    console.error('保存提示词失败:', error)
    message.error('保存提示词失败')
  }
  finally {
    saving.value = false
  }
}

// 重置为默认提示词
async function resetToDefault() {
  resetting.value = true
  try {
    const defaultPrompt = await invoke('reset_init_prompt')
    initPrompt.value = defaultPrompt
    message.success('已重置为默认提示词')
  }
  catch (error) {
    console.error('重置提示词失败:', error)
    message.error('重置提示词失败')
  }
  finally {
    resetting.value = false
  }
}

// 关闭弹窗
function handleClose() {
  emit('update:visible', false)
  emit('close')
}

// 键盘快捷键
function handleKeydown(event) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    saveInitPrompt()
  }
  else if (event.key === 'Escape') {
    event.preventDefault()
    handleClose()
  }
}
</script>

<template>
  <a-modal
    :open="visible"
    title="设置 Init 提示词"
    width="800px"
    :confirm-loading="saving"
    :mask-closable="false"
    :keyboard="false"
    centered
    @ok="saveInitPrompt"
    @cancel="handleClose"
  >
    <template #title>
      <div class="modal-title">
        <SettingOutlined class="title-icon" />
        <span>设置 Init 提示词</span>
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
          <a-button @click="handleClose">
            <template #icon>
              <CloseOutlined />
            </template>
            取消
          </a-button>
          <a-button
            type="primary"
            :loading="saving"
            :disabled="!initPrompt.trim()"
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

.reset-btn:hover {
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
