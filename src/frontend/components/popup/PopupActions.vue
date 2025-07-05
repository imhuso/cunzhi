<script setup lang="ts">
import type { McpRequest } from '../../types/popup'
import { computed, onMounted } from 'vue'
import { useShortcuts } from '../../composables/useShortcuts'

interface Props {
  request: McpRequest | null
  loading?: boolean
  submitting?: boolean
  canSubmit?: boolean
  connectionStatus?: string
  continueReplyEnabled?: boolean
  inputStatusText?: string
}

interface Emits {
  submit: []
  continue: []
  enhance: []
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  submitting: false,
  canSubmit: false,
  connectionStatus: '已连接',
  continueReplyEnabled: true,
  inputStatusText: '',
})

const emit = defineEmits<Emits>()

// 使用自定义快捷键系统
const {
  quickSubmitShortcutText,
  enhanceShortcutText,
  continueShortcutText,
  useQuickSubmitShortcut,
  useEnhanceShortcut,
  useContinueShortcut,
  loadShortcutConfig,
} = useShortcuts()

const shortcutText = quickSubmitShortcutText

const statusText = computed(() => {
  // 如果可以提交，直接显示快捷键提示
  if (props.canSubmit) {
    return shortcutText.value
  }

  // 如果有输入状态文本且不是默认状态，显示输入状态
  if (props.inputStatusText && props.inputStatusText !== '等待输入...') {
    return props.inputStatusText
  }

  // 根据请求类型显示不同的提示
  if (props.request?.predefined_options) {
    return '选择选项或输入文本'
  }
  return '请输入内容'
})

// 处理快捷键
useQuickSubmitShortcut(() => {
  if (props.canSubmit && !props.submitting) {
    handleSubmit()
  }
})

useEnhanceShortcut(() => {
  if (!props.submitting) {
    handleEnhance()
  }
})

useContinueShortcut(() => {
  if (!props.submitting) {
    handleContinue()
  }
})

function handleSubmit() {
  if (props.canSubmit && !props.submitting) {
    emit('submit')
  }
}

function handleContinue() {
  if (!props.submitting) {
    emit('continue')
  }
}

function handleEnhance() {
  if (!props.submitting) {
    emit('enhance')
  }
}

// 组件挂载时加载快捷键配置
onMounted(() => {
  loadShortcutConfig()
})
</script>

<template>
  <div class="px-4 py-3 bg-gray-100 min-h-[60px] select-none">
    <div v-if="!loading" class="flex justify-between items-center">
      <!-- 左侧状态信息 -->
      <div class="flex items-center">
        <div class="flex items-center gap-2 text-xs text-gray-600">
          <div class="w-2 h-2 rounded-full bg-primary-500" />
          <span class="font-medium">{{ connectionStatus }}</span>
          <span class="opacity-60">|</span>
          <span class="opacity-60">{{ statusText }}</span>
        </div>
      </div>

      <!-- 右侧操作按钮 -->
      <div class="flex items-center" data-guide="popup-actions">
        <n-space size="small">
          <!-- 增强按钮 -->
          <n-tooltip trigger="hover" placement="top">
            <template #trigger>
              <n-button
                :disabled="!canSubmit || submitting"
                size="medium"
                type="info"
                data-guide="enhance-button"
                @click="handleEnhance"
              >
                <template #icon>
                  <div class="i-carbon-magic-wand w-4 h-4" />
                </template>
                增强
              </n-button>
            </template>
            {{ enhanceShortcutText }}
          </n-tooltip>

          <!-- 继续按钮 -->
          <n-tooltip v-if="continueReplyEnabled" trigger="hover" placement="top">
            <template #trigger>
              <n-button
                :disabled="submitting"
                :loading="submitting"
                size="medium"
                type="default"
                data-guide="continue-button"
                @click="handleContinue"
              >
                <template #icon>
                  <div class="i-carbon-play w-4 h-4" />
                </template>
                继续
              </n-button>
            </template>
            {{ continueShortcutText }}
          </n-tooltip>

          <!-- 发送按钮 -->
          <n-tooltip trigger="hover" placement="top">
            <template #trigger>
              <n-button
                type="primary"
                :disabled="!canSubmit || submitting"
                :loading="submitting"
                size="medium"
                data-guide="submit-button"
                @click="handleSubmit"
              >
                <template #icon>
                  <div v-if="!submitting" class="i-carbon-send w-4 h-4" />
                </template>
                {{ submitting ? '发送中...' : '发送' }}
              </n-button>
            </template>
            {{ shortcutText }}
          </n-tooltip>
        </n-space>
      </div>
    </div>
  </div>
</template>
