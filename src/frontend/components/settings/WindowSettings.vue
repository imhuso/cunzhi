<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps({
  alwaysOnTop: {
    type: Boolean,
    required: true,
  },
  windowWidth: {
    type: Number,
    default: 600,
  },
  windowHeight: {
    type: Number,
    default: 900,
  },
  fixedWindowSize: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['toggleAlwaysOnTop', 'updateWindowSize'])

// 窗口设置状态 - 完全依赖后端
const localFixed = ref(props.fixedWindowSize)
const localWidth = ref(props.windowWidth)
const localHeight = ref(props.windowHeight)

// 实时窗口大小
const currentWidth = ref(0)
const currentHeight = ref(0)

// 窗口大小变化监听器
let windowResizeUnlisten: (() => void) | null = null

// 监听props变化，同步本地值

watch(() => props.windowWidth, (newWidth) => {
  localWidth.value = newWidth
})

watch(() => props.windowHeight, (newHeight) => {
  localHeight.value = newHeight
})

watch(() => props.fixedWindowSize, (newFixed) => {
  localFixed.value = newFixed
  loadWindowSettingsForMode(newFixed)
})

// 从后端加载指定模式的窗口设置
async function loadWindowSettingsForMode(fixed: boolean) {
  try {
    const result = await invoke('get_window_settings_for_mode', { fixed })
    if (result && typeof result === 'object') {
      const settings = result as any
      localWidth.value = settings.width
      localHeight.value = settings.height
      localFixed.value = settings.fixed
    }
  }
  catch (error) {
    console.error('加载窗口设置失败:', error)
  }
}

// 窗口约束和 UI 常量
const windowConstraints = ref({
  min_width: 600,
  min_height: 400,
  max_width: 1500,
  max_height: 1000,
  resize_step: 50,
  resize_throttle_ms: 1000,
  size_update_delay_ms: 500,
  size_check_delay_ms: 100,
})

// 加载窗口约束
async function loadWindowConstraints() {
  try {
    const constraints = await invoke('get_window_constraints_cmd')
    if (constraints) {
      windowConstraints.value = constraints as any
    }
  }
  catch (error) {
    console.error('加载窗口约束失败:', error)
  }
}

// 切换窗口模式（立即保存）
async function toggleWindowMode(fixed: boolean) {
  // 如果模式没有变化，直接返回
  if (localFixed.value === fixed) {
    return
  }

  // 先保存当前模式的尺寸到后端
  await saveCurrentModeSize()

  // 更新模式状态
  localFixed.value = fixed

  // 从后端加载新模式的尺寸设置
  await loadWindowSettingsForMode(fixed)

  // 立即保存模式切换
  emit('updateWindowSize', {
    width: localWidth.value,
    height: localHeight.value,
    fixed,
  })

  // 切换模式后刷新当前窗口大小显示
  setTimeout(() => {
    getCurrentWindowSize()
  }, windowConstraints.value.size_update_delay_ms)
}

// 保存当前模式的尺寸到后端
async function saveCurrentModeSize() {
  try {
    // 获取当前实际的窗口大小
    const result = await invoke('get_current_window_size')
    if (result && typeof result === 'object') {
      const { width, height } = result as any

      // 验证尺寸，如果小于最小限制则调整为最小尺寸
      let adjustedWidth = width
      let adjustedHeight = height
      let wasAdjusted = false

      if (width < windowConstraints.value.min_width) {
        adjustedWidth = windowConstraints.value.min_width
        wasAdjusted = true
      }
      if (height < windowConstraints.value.min_height) {
        adjustedHeight = windowConstraints.value.min_height
        wasAdjusted = true
      }

      if (wasAdjusted) {
        console.log(`窗口尺寸已调整: ${width}x${height} -> ${adjustedWidth}x${adjustedHeight}`)
      }

      const settings: any = {}

      if (localFixed.value) {
        settings.fixed_width = adjustedWidth
        settings.fixed_height = adjustedHeight
      }
      else {
        settings.free_width = adjustedWidth
        settings.free_height = adjustedHeight
      }

      await invoke('set_window_settings', { windowSettings: settings })
      console.log(`保存${localFixed.value ? '固定' : '自由'}模式尺寸: ${adjustedWidth}x${adjustedHeight}`)
    }
  }
  catch (error) {
    // 如果是窗口最小化或尺寸过小的错误，静默处理
    if (error && typeof error === 'string'
      && (error.includes('窗口已最小化') || error.includes('窗口尺寸过小'))) {
      console.debug('跳过窗口尺寸保存:', error)
    }
    else {
      console.error('保存当前模式尺寸失败:', error)
    }
  }
}

// 获取实时窗口大小
async function getCurrentWindowSize() {
  try {
    const result = await invoke('get_current_window_size')
    if (result && typeof result === 'object') {
      currentWidth.value = (result as any).width
      currentHeight.value = (result as any).height
    }
  }
  catch (error) {
    console.error('获取当前窗口大小失败:', error)
  }
}

// 保存窗口尺寸
async function saveWindowSize() {
  // 确保不小于最小值
  if (localWidth.value < windowConstraints.value.min_width)
    localWidth.value = windowConstraints.value.min_width
  if (localHeight.value < windowConstraints.value.min_height)
    localHeight.value = windowConstraints.value.min_height

  // 保存当前模式的尺寸到后端
  await saveCurrentModeSize()

  emit('updateWindowSize', {
    width: localWidth.value,
    height: localHeight.value,
    fixed: localFixed.value,
  })

  // 保存后刷新当前窗口大小显示
  setTimeout(() => {
    getCurrentWindowSize()
  }, windowConstraints.value.size_update_delay_ms)
}

// 设置窗口大小变化监听器
async function setupWindowResizeListener() {
  try {
    const webview = getCurrentWebviewWindow()

    // 移除之前的监听器
    if (windowResizeUnlisten) {
      windowResizeUnlisten()
    }

    // 监听窗口大小变化
    windowResizeUnlisten = await webview.onResized(() => {
      // 延迟获取窗口大小，确保变化已完成
      setTimeout(() => {
        getCurrentWindowSize()
      }, windowConstraints.value.size_check_delay_ms)
    })

    console.log('窗口大小变化监听器已设置')
  }
  catch (error) {
    console.error('设置窗口大小变化监听器失败:', error)
  }
}

// 移除窗口大小变化监听器
function removeWindowResizeListener() {
  if (windowResizeUnlisten) {
    windowResizeUnlisten()
    windowResizeUnlisten = null
  }
}

// 组件挂载时获取当前窗口大小并设置监听器
onMounted(async () => {
  await loadWindowConstraints()
  getCurrentWindowSize()
  loadWindowSettingsForMode(localFixed.value)
  setupWindowResizeListener()
})

// 组件卸载时移除监听器
onUnmounted(() => {
  removeWindowResizeListener()
})
</script>

<template>
  <!-- 设置内容 -->
  <n-space vertical size="large">
    <!-- 置顶显示设置 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-1.5 h-1.5 bg-success rounded-full mr-3 flex-shrink-0" />
        <div>
          <div class="text-sm font-medium leading-relaxed">
            总在最前
          </div>
          <div class="text-xs opacity-60">
            启用后窗口将始终保持在其他应用程序之上
          </div>
        </div>
      </div>
      <n-switch
        :value="alwaysOnTop"
        size="small"
        @update:value="$emit('toggleAlwaysOnTop')"
      />
    </div>

    <!-- 窗口尺寸设置 -->
    <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
      <div class="flex items-start">
        <div class="w-1.5 h-1.5 bg-success rounded-full mr-3 mt-2 flex-shrink-0" />
        <div class="flex-1">
          <div class="text-sm font-medium mb-3 leading-relaxed">
            窗口尺寸
          </div>

          <!-- 窗口模式选择 -->
          <div class="space-y-3">
            <!-- 自由拉伸模式 -->
            <div
              class="p-3 rounded-lg border cursor-pointer transition-all"
              :class="!localFixed ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20 shadow-sm' : 'border-gray-300 dark:border-gray-600 hover:border-primary-300 dark:hover:border-primary-500 hover:bg-gray-100'"
              @click="toggleWindowMode(false)"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div
                    class="w-4 h-4 rounded-full border-2 mr-3 flex items-center justify-center"
                    :class="!localFixed ? 'border-primary-500' : 'border-gray-400 dark:border-gray-500'"
                  >
                    <div
                      v-if="!localFixed"
                      class="w-2 h-2 bg-primary-500 rounded-full"
                    />
                  </div>
                  <div>
                    <div class="text-sm font-medium mb-1">
                      自由拉伸
                    </div>
                    <div class="text-xs opacity-60">
                      窗口可以自由拖拽调整大小
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 固定大小模式 -->
            <div
              class="p-3 rounded-lg border cursor-pointer transition-all"
              :class="localFixed ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20 shadow-sm' : 'border-gray-300 dark:border-gray-600 hover:border-primary-300 dark:hover:border-primary-500 hover:bg-gray-100'"
              @click="toggleWindowMode(true)"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div
                    class="w-4 h-4 rounded-full border-2 mr-3 flex items-center justify-center"
                    :class="localFixed ? 'border-primary-500' : 'border-gray-400 dark:border-gray-500'"
                  >
                    <div
                      v-if="localFixed"
                      class="w-2 h-2 bg-primary-500 rounded-full"
                    />
                  </div>
                  <div>
                    <div class="text-sm font-medium mb-1">
                      固定大小
                    </div>
                    <div class="text-xs opacity-60">
                      设置固定的窗口尺寸
                    </div>
                  </div>
                </div>
              </div>

              <!-- 固定模式的尺寸设置 -->
              <div v-if="localFixed" class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
                <div class="grid grid-cols-2 gap-3">
                  <!-- 宽度设置 -->
                  <div>
                    <div class="text-xs opacity-60 mb-2">
                      宽度
                    </div>
                    <n-input-number
                      v-model:value="localWidth"
                      :min="windowConstraints.min_width"
                      :max="windowConstraints.max_width"
                      :step="windowConstraints.resize_step"
                      size="small"
                      placeholder="宽度"
                      @click.stop
                      @mousedown.stop
                    />
                  </div>

                  <!-- 高度设置 -->
                  <div>
                    <div class="text-xs opacity-60 mb-2">
                      高度
                    </div>
                    <n-input-number
                      v-model:value="localHeight"
                      :min="windowConstraints.min_height"
                      :max="windowConstraints.max_height"
                      :step="windowConstraints.resize_step"
                      size="small"
                      placeholder="高度"
                      @click.stop
                      @mousedown.stop
                    />
                  </div>
                </div>

                <!-- 保存按钮 -->
                <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
                  <n-button
                    type="primary"
                    size="small"
                    @click="saveWindowSize"
                  >
                    保存窗口设置
                  </n-button>
                </div>
              </div>
            </div>
          </div>

          <!-- 窗口信息显示（弱化） -->
          <div class="mt-4 text-center space-y-1">
            <div class="text-xs opacity-60 text-gray-600 dark:text-gray-400">
              当前窗口：{{ currentWidth || localWidth }} × {{ currentHeight || localHeight }} px
              （{{ localFixed ? '固定大小' : '自由拉伸' }}）
            </div>
            <div class="text-xs opacity-50 text-gray-500 dark:text-gray-500">
              尺寸限制：宽度 {{ windowConstraints.min_width }}-{{ windowConstraints.max_width }}px，高度 {{ windowConstraints.min_height }}-{{ windowConstraints.max_height }}px
            </div>
          </div>
        </div>
      </div>
    </div>
  </n-space>
</template>
