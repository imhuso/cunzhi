<script setup lang="ts">
import { useMessage } from 'naive-ui'
import { onMounted, onUnmounted, ref } from 'vue'
import { useUserGuide } from '../../composables/useUserGuide'

const message = useMessage()
const {
  isGuideActive,
  startGuide,
  guideFlows,
  isFirstTimeUser,
  markGuideAsSeen,
  resetGuideState,
} = useUserGuide()

// 检查是否需要显示首次使用提示
const showFirstTimeHint = ref(isFirstTimeUser())

// 可用的引导流程
const availableGuides = ref([
  {
    key: 'newUserOnboarding',
    name: '新用户入门引导',
    description: '了解寸止的基本功能和界面布局',
    icon: 'i-carbon-user-follow',
    color: 'primary',
  },
])

// 启动指定的引导流程
async function startGuideFlow(guideKey: string) {
  console.log('启动引导流程:', guideKey, '当前状态:', isGuideActive.value)

  if (isGuideActive.value) {
    console.log('引导正在进行中，强制清理后重试')
    forceCleanup()
    await new Promise(resolve => setTimeout(resolve, 200))
  }

  try {
    // 根据引导类型进行预处理
    if (guideKey === 'popupGuide') {
      // 检查是否有弹窗，如果没有则自动打开测试弹窗
      const popupContent = document.querySelector('[data-guide="popup-content"]')
      console.log('弹窗引导预处理 - 检查弹窗内容元素:', popupContent)

      if (!popupContent) {
        const testButton = document.querySelector('[data-guide="test-button"]') as HTMLElement
        console.log('未找到弹窗内容，查找测试按钮:', testButton)

        if (testButton) {
          console.log('找到测试按钮，点击打开弹窗')
          testButton.click()
          console.log('等待弹窗打开...')
          await new Promise(resolve => setTimeout(resolve, 1000)) // 增加等待时间到1秒

          // 再次检查弹窗是否打开
          const popupContentAfter = document.querySelector('[data-guide="popup-content"]')
          console.log('点击后检查弹窗内容元素:', popupContentAfter)
        }
        else {
          console.error('未找到测试按钮，无法自动打开弹窗')
        }
      }
      else {
        console.log('弹窗已存在，直接开始引导')
      }
    }

    const steps = guideFlows[guideKey as keyof typeof guideFlows]()
    console.log('获取引导步骤:', steps)

    if (!steps || steps.length === 0) {
      console.error('引导流程配置错误:', guideKey)
      message.error('引导流程配置错误')
      return
    }

    // 检查目标元素是否存在
    const missingElements = []
    console.log('开始检查引导元素是否存在...')

    for (const step of steps) {
      const element = document.querySelector(step.element)
      console.log(`检查元素 ${step.element}:`, element)

      if (!element) {
        missingElements.push(step.element)
      }
    }

    if (missingElements.length > 0) {
      console.warn('以下引导元素未找到:', missingElements)

      // 打印当前页面所有可能相关的元素
      console.log('当前页面所有data-guide元素:', document.querySelectorAll('[data-guide]'))

      // 根据引导类型给出具体提示
      if (guideKey === 'popupGuide') {
        message.warning('请先点击测试按钮打开 AI 助手弹窗，然后重试')
      }
      else {
        message.warning('部分引导元素不可见，请确保相关功能已打开后重试')
      }
      return
    }

    console.log('所有引导元素都已找到，开始启动引导')

    console.log('调用 startGuide 函数')

    // 尝试启动引导，如果失败则重试
    let retryCount = 0
    const maxRetries = 2

    while (retryCount <= maxRetries) {
      try {
        await startGuide(steps, {
          onDestroyed: () => {
            console.log('引导完成回调')
            message.success('引导完成！')
            // 如果是首次使用，标记为已看过引导
            if (showFirstTimeHint.value) {
              markGuideAsSeen()
              showFirstTimeHint.value = false
            }
          },
        })
        console.log('startGuide 函数调用完成')
        break // 成功启动，跳出循环
      }
      catch (error) {
        console.error(`引导启动失败 (尝试 ${retryCount + 1}/${maxRetries + 1}):`, error)
        retryCount++

        if (retryCount <= maxRetries) {
          console.log('等待后重试...')
          forceCleanup()
          resetGuideState()
          await new Promise(resolve => setTimeout(resolve, 300))
        }
        else {
          message.error('引导启动失败，请刷新页面后重试')
        }
      }
    }
  }
  catch (error) {
    console.error('启动引导失败:', error)
    message.error('启动引导失败')
  }
}

// 获取引导按钮的类型
function getButtonType(color: string) {
  switch (color) {
    case 'primary': return 'primary'
    case 'success': return 'success'
    case 'info': return 'info'
    case 'warning': return 'warning'
    case 'error': return 'error'
    default: return 'default'
  }
}

// 快捷键处理
function handleKeydown(event: KeyboardEvent) {
  // Ctrl + Shift + H 启动新用户引导
  if (event.ctrlKey && event.shiftKey && event.key === 'H') {
    event.preventDefault()
    startGuideFlow('newUserOnboarding')
  }
}

// 添加快捷键监听
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

// 清理快捷键监听
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// 重置引导状态
function resetGuideStatus() {
  try {
    localStorage.removeItem('cunzhi-has-seen-guide')
    localStorage.removeItem('cunzhi-guide-preferences')
    showFirstTimeHint.value = true
    message.success('引导状态已重置')
  }
  catch (error) {
    console.error('重置引导状态失败:', error)
    message.error('重置失败')
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- 首次使用提示 -->
    <div v-if="showFirstTimeHint && !isGuideActive" class="p-4 bg-success-50 dark:bg-success-900/20 border border-success-200 dark:border-success-800 rounded-lg">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="i-carbon-user-follow w-5 h-5 text-success-600 dark:text-success-400" />
          <div>
            <div class="font-medium text-success-900 dark:text-success-100">
              欢迎首次使用寸止！
            </div>
            <div class="text-sm text-success-700 dark:text-success-300">
              建议先体验"新用户入门引导"来了解基本功能
            </div>
          </div>
        </div>
        <n-button
          size="small"
          type="success"
          @click="startGuideFlow('newUserOnboarding')"
        >
          开始引导
        </n-button>
      </div>
    </div>

    <!-- 引导流程列表 -->
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <div class="text-sm font-medium text-on-surface">
          选择引导流程
        </div>
        <n-button
          size="tiny"
          type="tertiary"
          :disabled="isGuideActive"
          @click="resetGuideStatus"
        >
          <template #icon>
            <div class="i-carbon-reset w-3 h-3" />
          </template>
          重置状态
        </n-button>
      </div>

      <div class="grid gap-3">
        <div
          v-for="guide in availableGuides"
          :key="guide.key"
          class="p-4 border border-border rounded-lg hover:bg-surface-50 dark:hover:bg-surface-800 transition-colors"
        >
          <div class="flex items-start justify-between">
            <div class="flex items-start gap-3 flex-1">
              <div class="w-5 h-5 mt-0.5" :class="[guide.icon]" :style="{ color: `var(--${guide.color}-color, #3b82f6)` }" />
              <div class="flex-1 min-w-0">
                <div class="font-medium text-on-surface mb-1">
                  {{ guide.name }}
                </div>
                <div class="text-sm text-on-surface-secondary leading-relaxed">
                  {{ guide.description }}
                </div>
              </div>
            </div>
            <n-button
              size="small"
              :type="getButtonType(guide.color)"
              :disabled="isGuideActive"
              @click="startGuideFlow(guide.key)"
            >
              <template #icon>
                <div class="i-carbon-play w-4 h-4" />
              </template>
              开始
            </n-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 使用提示 -->
    <div class="p-3 bg-surface-50 dark:bg-surface-800 rounded-lg">
      <div class="flex items-start gap-2">
        <div class="i-carbon-idea w-4 h-4 text-warning-500 mt-0.5 flex-shrink-0" />
        <div class="text-xs text-on-surface-secondary leading-relaxed">
          <div class="font-medium mb-1">
            使用提示：
          </div>
          <ul class="space-y-1 list-disc list-inside">
            <li>引导过程中可以使用键盘方向键或点击按钮导航</li>
            <li>按 <kbd class="px-1 py-0.5 bg-surface-200 dark:bg-surface-700 rounded text-xs">ESC</kbd> 键或点击关闭按钮可以随时退出引导</li>
            <li>按 <kbd class="px-1 py-0.5 bg-surface-200 dark:bg-surface-700 rounded text-xs">Ctrl+Shift+H</kbd> 可以快速启动新用户引导</li>
            <li>某些引导需要特定页面或功能处于打开状态</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
