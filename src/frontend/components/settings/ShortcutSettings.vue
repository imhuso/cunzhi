<script setup lang="ts">
import type { ShortcutBinding, ShortcutConfig } from '../../types/popup'
import { useMessage } from 'naive-ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useShortcuts } from '../../composables/useShortcuts'

const message = useMessage()

const {
  shortcutConfig,
  isMac,
  loadShortcutConfig,
  saveShortcutBinding,
  resetShortcutsToDefault,
  shortcutKeyToString,
  checkShortcutConflict,
} = useShortcuts()

const config = ref<ShortcutConfig>({
  shortcuts: {},
})

const showEditDialog = ref(false)
const editingBinding = ref<ShortcutBinding>({
  id: '',
  name: '',
  description: '',
  action: '',
  key_combination: {
    key: '',
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
  },
  enabled: true,
  scope: '',
})
const editingId = ref('')
const isRecording = ref(false)
const recordingTimeout = ref<number | null>(null)
const currentKeys = ref({
  ctrl: false,
  alt: false,
  shift: false,
  meta: false,
  key: '',
})

// 冲突检测
const conflictWarning = computed(() => {
  if (!editingBinding.value.key_combination.key)
    return null
  return checkShortcutConflict(editingBinding.value, editingId.value)
})

// 检查是否有按键被按下
const hasAnyKey = computed(() => {
  return currentKeys.value.ctrl || currentKeys.value.alt || currentKeys.value.shift
    || currentKeys.value.meta || currentKeys.value.key
})

// 获取作用域文本
function getScopeText(scope: string): string {
  switch (scope) {
    case 'global': return '全局'
    case 'popup': return '弹窗'
    case 'input': return '输入框'
    default: return scope
  }
}

// 编辑快捷键绑定
function editBinding(id: string, binding: ShortcutBinding) {
  editingId.value = id
  editingBinding.value = { ...binding }
  showEditDialog.value = true
}

// 保存快捷键绑定
async function saveBinding() {
  try {
    await saveShortcutBinding(editingId.value, editingBinding.value)
    config.value.shortcuts[editingId.value] = { ...editingBinding.value }
    showEditDialog.value = false
    message.success('快捷键已保存')
  }
  catch (error) {
    message.error(`保存失败: ${error}`)
  }
}

// 重置为默认值
async function handleReset() {
  try {
    await resetShortcutsToDefault()
    await loadShortcutConfig()
    config.value = { ...shortcutConfig.value }
    message.success('快捷键已重置为默认值')
  }
  catch (error) {
    message.error(`重置失败: ${error}`)
  }
}

// 监听配置变化
watch(shortcutConfig, (newConfig) => {
  config.value = { ...newConfig }
}, { deep: true })

// 开始录制快捷键
function startRecording() {
  isRecording.value = true

  // 清除之前的快捷键设置和当前按键状态
  editingBinding.value.key_combination = {
    key: '',
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
  }

  currentKeys.value = {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    key: '',
  }

  // 添加键盘事件监听器
  document.addEventListener('keydown', handleRecordingKeyDown, true)
  document.addEventListener('keyup', handleRecordingKeyUp, true)

  // 设置超时自动停止录制（10秒）
  recordingTimeout.value = window.setTimeout(() => {
    stopRecording()
    message.warning('录制超时，已自动停止')
  }, 10000)
}

// 停止录制快捷键
function stopRecording() {
  isRecording.value = false

  // 移除键盘事件监听器
  document.removeEventListener('keydown', handleRecordingKeyDown, true)
  document.removeEventListener('keyup', handleRecordingKeyUp, true)

  // 清除当前按键状态
  currentKeys.value = {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    key: '',
  }

  // 清除超时
  if (recordingTimeout.value) {
    clearTimeout(recordingTimeout.value)
    recordingTimeout.value = null
  }
}

// 处理录制时的按键事件
function handleRecordingKeyDown(event: KeyboardEvent) {
  event.preventDefault()
  event.stopPropagation()

  // 更新当前按键状态显示
  currentKeys.value = {
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
    meta: event.metaKey,
    key: ['Control', 'Alt', 'Shift', 'Meta', 'Cmd', 'Command'].includes(event.key) ? '' : normalizeKey(event.key),
  }

  // ESC 键取消录制
  if (event.key === 'Escape') {
    stopRecording()
    message.info('已取消录制')
    return
  }

  // 忽略单独的修饰键
  if (['Control', 'Alt', 'Shift', 'Meta', 'Cmd', 'Command'].includes(event.key)) {
    return
  }

  // 记录快捷键组合
  const keyCombo = {
    key: normalizeKey(event.key),
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
    meta: event.metaKey,
  }

  // 验证快捷键是否有效（必须包含至少一个修饰键，除非是功能键）
  const isFunctionKey = /^F\d+$/.test(keyCombo.key) || ['Escape', 'Tab', 'Space', 'Enter'].includes(keyCombo.key)
  const hasModifier = keyCombo.ctrl || keyCombo.alt || keyCombo.shift || keyCombo.meta

  if (!hasModifier && !isFunctionKey) {
    message.warning('请使用修饰键组合（如 Ctrl、Alt、Shift）或功能键')
    return
  }

  // 设置录制的快捷键
  editingBinding.value.key_combination = keyCombo

  // 停止录制
  stopRecording()
  message.success(`已录制快捷键: ${shortcutKeyToString(keyCombo)}`)
}

// 处理录制时的按键释放事件（用于更新修饰键状态）
function handleRecordingKeyUp(event: KeyboardEvent) {
  event.preventDefault()
  event.stopPropagation()

  // 更新修饰键状态
  currentKeys.value.ctrl = event.ctrlKey
  currentKeys.value.alt = event.altKey
  currentKeys.value.shift = event.shiftKey
  currentKeys.value.meta = event.metaKey
}

// 标准化按键名称
function normalizeKey(key: string): string {
  // 处理特殊键名
  const keyMap: Record<string, string> = {
    ' ': 'Space',
    'ArrowUp': 'Up',
    'ArrowDown': 'Down',
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'Delete': 'Del',
    'Insert': 'Ins',
    'PageUp': 'PgUp',
    'PageDown': 'PgDn',
    'Home': 'Home',
    'End': 'End',
  }

  return keyMap[key] || key.toUpperCase()
}

// 组件挂载时加载配置
onMounted(async () => {
  await loadShortcutConfig()
  config.value = { ...shortcutConfig.value }
})

// 组件卸载时清理
onUnmounted(() => {
  if (isRecording.value) {
    stopRecording()
  }
})
</script>

<template>
  <div class="space-y-4">
    <!-- 页面标题 -->
    <div class="mb-4">
      <h3 class="text-base font-medium text-on-surface">
        自定义快捷键
      </h3>
      <p class="text-sm text-on-surface-secondary">
        自定义应用快捷键绑定
      </p>
    </div>

    <!-- 快捷键列表 -->
    <div class="space-y-3">
      <div
        v-for="(binding, id) in config.shortcuts"
        :key="id"
        class="p-3 border border-border rounded-lg"
      >
        <div class="flex items-center justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <span class="text-base font-medium text-on-surface">{{ binding.name }}</span>
            </div>
            <p class="text-xs text-on-surface-secondary mt-1">
              {{ binding.description }}
            </p>
            <div class="flex items-center gap-2 mt-2">
              <span class="text-xs text-on-surface-secondary">作用域:</span>
              <n-tag size="small">
                {{ getScopeText(binding.scope) }}
              </n-tag>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <n-button
              size="small"
              type="primary"
              @click="editBinding(id, binding)"
            >
              编辑
            </n-button>
          </div>
        </div>
        <div class="mt-2 p-2 bg-surface-variant rounded text-center">
          <span class="font-mono text-sm">{{ shortcutKeyToString(binding.key_combination) }}</span>
        </div>
      </div>
    </div>

    <!-- 重置按钮 -->
    <div class="pt-4 border-t border-border">
      <n-button
        type="warning"
        size="small"
        @click="handleReset"
      >
        重置为默认值
      </n-button>
    </div>

    <!-- 编辑快捷键对话框 -->
    <n-modal v-model:show="showEditDialog" preset="dialog" title="编辑快捷键">
      <div class="space-y-4">
        <n-form-item label="快捷键名称">
          <n-input v-model:value="editingBinding.name" placeholder="输入快捷键名称" />
        </n-form-item>

        <n-form-item label="描述">
          <n-input v-model:value="editingBinding.description" placeholder="输入描述" />
        </n-form-item>

        <n-form-item label="快捷键设置">
          <!-- 快捷键录制区域 -->
          <div
            class="border-2 border-dashed rounded-lg p-6 text-center transition-all duration-300"
            :class="isRecording ? 'border-primary bg-primary-50 dark:bg-primary-950' : 'border-border hover:border-primary'"
          >
            <div v-if="!isRecording" class="space-y-3">
              <div class="text-lg text-on-surface">
                🎹 快捷键录制器
              </div>
              <div class="text-sm text-on-surface-secondary">
                点击下方按钮，然后按下您想要的快捷键组合
              </div>
              <n-button
                type="primary"
                size="medium"
                @click="startRecording"
              >
                🎙️ 开始录制快捷键
              </n-button>
            </div>

            <div v-else class="space-y-4">
              <div class="flex items-center justify-center gap-2">
                <div class="w-3 h-3 bg-primary rounded-full animate-pulse" />
                <div class="text-lg text-primary font-medium">
                  正在录制... 请按下您想要的快捷键组合
                </div>
                <div class="w-3 h-3 bg-primary rounded-full animate-pulse" />
              </div>

              <!-- 实时按键状态显示 -->
              <div class="flex items-center justify-center gap-3 min-h-12 p-3 bg-surface rounded-lg">
                <n-tag v-if="currentKeys.ctrl" size="medium" type="info">
                  {{ isMac ? '⌃' : 'Ctrl' }}
                </n-tag>
                <n-tag v-if="currentKeys.alt" size="medium" type="info">
                  {{ isMac ? '⌥' : 'Alt' }}
                </n-tag>
                <n-tag v-if="currentKeys.shift" size="medium" type="info">
                  {{ isMac ? '⇧' : 'Shift' }}
                </n-tag>
                <n-tag v-if="currentKeys.meta && isMac" size="medium" type="info">
                  ⌘
                </n-tag>
                <n-tag v-if="currentKeys.key" size="medium" type="primary">
                  {{ currentKeys.key }}
                </n-tag>
                <span v-if="!hasAnyKey" class="text-on-surface-secondary">
                  等待按键...
                </span>
              </div>

              <div class="text-sm text-on-surface-secondary space-y-1">
                <div>💡 必须包含修饰键（Ctrl、Alt、Shift）或使用功能键</div>
                <div>⚠️ 按 ESC 取消录制</div>
              </div>

              <n-button
                size="medium"
                type="warning"
                @click="stopRecording"
              >
                取消录制
              </n-button>
            </div>
          </div>
        </n-form-item>

        <div class="p-3 bg-surface-variant rounded text-center">
          <span class="text-sm text-on-surface-secondary">预览: </span>
          <span class="font-mono">{{ shortcutKeyToString(editingBinding.key_combination) }}</span>
        </div>

        <!-- 冲突检测 -->
        <div v-if="conflictWarning" class="p-3 bg-error-container rounded">
          <p class="text-sm text-error">
            ⚠️ 快捷键冲突：与 "{{ conflictWarning }}" 冲突
          </p>
        </div>
      </div>

      <template #action>
        <div class="flex gap-2">
          <n-button @click="showEditDialog = false">
            取消
          </n-button>
          <n-button
            type="primary"
            :disabled="!!conflictWarning"
            @click="saveBinding"
          >
            保存
          </n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>
