import { driver } from 'driver.js'
import { nextTick, ref, watch } from 'vue'
import 'driver.js/dist/driver.css'

export interface GuideStep {
  element: string
  popover: {
    title: string
    description: string
    side?: 'left' | 'right' | 'top' | 'bottom'
    align?: 'start' | 'center' | 'end'
  }
}

export interface GuideConfig {
  showProgress?: boolean
  allowClose?: boolean
  overlayClickNext?: boolean
  smoothScroll?: boolean
  animate?: boolean
  nextBtnText?: string
  prevBtnText?: string
  doneBtnText?: string
  closeBtnText?: string
  progressText?: string
  popoverOffset?: number
  stagePadding?: number
  onDestroyStarted?: (element?: Element, step?: any, options?: any) => void
  onDestroyed?: (element?: Element, step?: any, options?: any) => void
  onHighlightStarted?: (element?: Element, step?: any, options?: any) => void
  onHighlighted?: (element?: Element, step?: any, options?: any) => void
  onDeselected?: (element?: Element, step?: any, options?: any) => void
}

export function useUserGuide() {
  const isGuideActive = ref(false)
  const currentGuide = ref<any>(null)
  const currentStep = ref(0)
  const currentGuideKey = ref('')

  // 强制清理引导（紧急情况使用）
  function forceCleanup() {
    console.log('强制清理引导')

    // 先尝试正常销毁当前实例
    if (currentGuide.value) {
      try {
        console.log('尝试正常销毁当前实例')
        currentGuide.value.destroy()
      }
      catch (error) {
        console.log('正常销毁失败:', error)
      }
    }

    // 清理状态
    currentGuide.value = null
    isGuideActive.value = false
    currentStep.value = 0
    currentGuideKey.value = ''

    // 清理所有可能的 driver.js DOM 元素
    const elements = [
      '.driver-overlay',
      '.driver-popover',
      '.driver-highlighted-element',
      '[data-driver-popover]',
      '[data-driver-overlay]',
      '.driver-popover-wrapper',
      '.driver-stage',
    ]

    elements.forEach((selector) => {
      const els = document.querySelectorAll(selector)
      els.forEach((el) => {
        console.log('移除元素:', selector)
        el.remove()
      })
    })

    // 移除可能的类名
    document.body.classList.remove('driver-active', 'driver-fade', 'driver-fix')
    document.documentElement.classList.remove('driver-active', 'driver-fade', 'driver-fix')

    // 重置可能的样式
    document.body.style.overflow = ''
    document.documentElement.style.overflow = ''

    console.log('强制清理完成')
  }

  // 监听引导状态变化
  watch(isGuideActive, (newValue, oldValue) => {
    console.log('引导状态变化:', oldValue, '->', newValue)
    if (!newValue && oldValue) {
      // 引导被关闭，确保清理
      setTimeout(() => {
        if (!isGuideActive.value) {
          console.log('确认引导已关闭，执行清理')
          const overlay = document.querySelector('.driver-overlay')
          const popover = document.querySelector('.driver-popover')
          if (overlay || popover) {
            console.log('发现残留元素，强制清理')
            forceCleanup()
          }
        }
      }, 200)
    }
  })

  // 键盘事件处理 - 移除自定义键盘处理，让 driver.js 自己处理
  // function handleKeydown(event: KeyboardEvent) {
  //   if (event.key === 'Escape' && isGuideActive.value) {
  //     event.preventDefault()
  //     event.stopPropagation()
  //     stopGuide()
  //   }
  // }

  // // 添加键盘监听
  // onMounted(() => {
  //   document.addEventListener('keydown', handleKeydown, true)
  // })

  // // 清理键盘监听
  // onUnmounted(() => {
  //   document.removeEventListener('keydown', handleKeydown, true)
  //   if (isGuideActive.value) {
  //     stopGuide()
  //   }
  // })

  // 默认配置
  const defaultConfig = {
    showProgress: true,
    allowClose: true,
    overlayClickNext: true,
    smoothScroll: true,
    animate: true,
    nextBtnText: '下一步',
    prevBtnText: '上一步',
    doneBtnText: '完成',
    popoverOffset: 40, // 增加弹出框与目标元素之间的距离
    stagePadding: 5, // 增加高亮区域的内边距
  }

  // 重置引导状态
  function resetGuideState() {
    console.log('重置引导状态')
    currentGuide.value = null
    isGuideActive.value = false
    currentStep.value = 0
    currentGuideKey.value = ''
  }

  // 检查环境是否干净
  function checkEnvironment() {
    const overlay = document.querySelector('.driver-overlay')
    const popover = document.querySelector('.driver-popover')
    const highlighted = document.querySelector('.driver-highlighted-element')

    console.log('环境检查:', {
      overlay: !!overlay,
      popover: !!popover,
      highlighted: !!highlighted,
      isGuideActive: isGuideActive.value,
      currentGuide: !!currentGuide.value,
    })

    return !overlay && !popover && !highlighted && !isGuideActive.value && !currentGuide.value
  }

  // 启动引导
  async function startGuide(steps: GuideStep[], config: GuideConfig = {}) {
    console.log('尝试启动引导, 当前状态:', isGuideActive.value)

    // 强制清理和重置状态
    console.log('强制清理和重置状态')
    forceCleanup()
    resetGuideState()

    // 等待清理完成
    await new Promise(resolve => setTimeout(resolve, 100))
    await nextTick()

    // 检查环境是否干净
    if (!checkEnvironment()) {
      console.warn('环境不干净，再次清理')
      forceCleanup()
      resetGuideState()
      await new Promise(resolve => setTimeout(resolve, 200))
    }

    try {
      console.log('开始创建新的引导实例')

      // 创建引导实例
      const finalConfig = {
        ...defaultConfig,
        ...config,
        steps,
        onDestroyStarted: () => {
          console.log('引导开始销毁')
          isGuideActive.value = false
          if (config.onDestroyStarted) {
            config.onDestroyStarted()
          }
        },
        onDestroyed: () => {
          console.log('引导已销毁回调')
          // 延迟清理状态，确保 driver.js 完全清理完毕
          setTimeout(() => {
            console.log('延迟清理状态')
            currentGuide.value = null
            isGuideActive.value = false

            // 确保清理残留元素
            const overlay = document.querySelector('.driver-overlay')
            const popover = document.querySelector('.driver-popover')
            if (overlay)
              overlay.remove()
            if (popover)
              popover.remove()

            if (config.onDestroyed) {
              config.onDestroyed()
            }
          }, 50)
        },
      }

      console.log('创建引导配置:', finalConfig)
      currentGuide.value = driver(finalConfig)
      isGuideActive.value = true
      console.log('引导实例已创建，开始驱动')
      currentGuide.value.drive()
      console.log('引导已启动')
    }
    catch (error) {
      console.error('启动引导失败:', error)
      isGuideActive.value = false
      currentGuide.value = null
      forceCleanup()
    }
  }

  // 停止引导
  function stopGuide() {
    console.log('手动停止引导')
    try {
      if (currentGuide.value) {
        console.log('调用 destroy()')
        currentGuide.value.destroy()
      }
    }
    catch (error) {
      console.error('停止引导时出错:', error)
    }

    // 强制清理状态
    console.log('强制清理状态')
    currentGuide.value = null
    isGuideActive.value = false

    // 清理可能残留的 DOM 元素
    setTimeout(() => {
      const overlay = document.querySelector('.driver-overlay')
      const popover = document.querySelector('.driver-popover')
      if (overlay) {
        overlay.remove()
        console.log('手动移除 overlay')
      }
      if (popover) {
        popover.remove()
        console.log('手动移除 popover')
      }
    }, 100)
  }

  // 下一步
  function nextStep() {
    if (currentGuide.value) {
      currentGuide.value.moveNext()
    }
  }

  // 上一步
  function previousStep() {
    if (currentGuide.value) {
      currentGuide.value.movePrevious()
    }
  }

  // 跳转到指定步骤
  function goToStep(stepIndex: number) {
    if (currentGuide.value) {
      currentGuide.value.moveTo(stepIndex)
    }
  }

  // 预定义的引导流程
  const guideFlows = {
    // 新用户入门引导
    newUserOnboarding: (): GuideStep[] => [
      {
        element: '[data-guide="app-logo"]',
        popover: {
          title: '欢迎使用寸止！',
          description: '寸止是一个强大的 AI 助手 MCP 工具，告别 AI 提前终止烦恼，助力 AI 更加持久，让我们开始探索它的功能吧！点击"下一步"继续，或按 ESC 键退出引导。',
          side: 'bottom',
          align: 'center',
        },
      },
      {
        element: '[data-guide="test-button"]',
        popover: {
          title: '测试功能',
          description: '点击这个按钮可以测试 MCP 弹窗功能，体验和 AI 的交互。',
          side: 'bottom',
          align: 'center',
        },
      },
      {
        element: '[data-guide="tabs"]',
        popover: {
          title: '功能标签页',
          description: '这里有四个主要功能区域：介绍、MCP 工具、参考提示词和设置。每个标签页都有不同的功能。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="settings-tab"]',
        popover: {
          title: '设置页面',
          description: '在设置页面，您可以自定义主题、字体、提示词、快捷键，设置音频提醒，甚至配置Telegram机器人等功能。点击"完成"结束引导！',
          side: 'top',
          align: 'center',
        },
      },
    ],

    // 弹窗功能引导
    popupGuide: (): GuideStep[] => [
      {
        element: '[data-guide="popup-content"]',
        popover: {
          title: 'AI 消息区域',
          description: '这里显示 AI 发送的消息内容，您可以选中文本进行复制。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="quote-message"]',
        popover: {
          title: '引用原文',
          description: '点击此按钮可以将 AI 的回复内容引用到输入框中，方便您基于 AI 的回复进行进一步的对话，最重要的用途就是提示词增强后，引用原文。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="predefined-options"]',
        popover: {
          title: '预定义选项',
          description: '当AI提供多个选择时，您可以在这里勾选需要的选项。可以选择多个选项，也可以不选择任何选项直接在输入框输入文本。',
          side: 'bottom',
          align: 'center',
        },
      },
      {
        element: '[data-guide="custom-prompts"]',
        popover: {
          title: '快捷模板',
          description: '这些是您的自定义提示词模板，点击可以快速插入到输入框中。您还可以拖拽调整顺序。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="context-append"]',
        popover: {
          title: '上下文追加',
          description: '这些开关控制条件性提示词，开启后会根据设定的条件自动在您的输入后追加相应的上下文内容。请注意：开启和关闭都会追加相应的上下文，请根据您的需求进行设置。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="popup-input"]',
        popover: {
          title: '输入区域',
          description: '在这里输入您的问题或指令，与 AI 进行交互。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="enhance-button"]',
        popover: {
          title: '增强按钮',
          description: '点击此按钮可以让 AI 对您的输入进行增强和优化，使指令更加清晰和具体，配合上面的“引用原文”一起食用，风味更佳。支持快捷键操作。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="continue-button"]',
        popover: {
          title: '继续按钮',
          description: '当 AI 回复的内容符合您的心意，不需要修改的时候，点击此按钮让AI继续当牛马。支持快捷键操作。',
          side: 'top',
          align: 'center',
        },
      },
      {
        element: '[data-guide="submit-button"]',
        popover: {
          title: '发送按钮',
          description: '点击此按钮发送输入框的内容给 AI。这是最常用的操作按钮，支持快捷键操作。',
          side: 'top',
          align: 'center',
        },
      },
    ],
  }

  // 保存引导偏好设置
  function saveGuidePreferences(preferences: any) {
    try {
      localStorage.setItem('cunzhi-guide-preferences', JSON.stringify(preferences))
    }
    catch (error) {
      console.error('保存引导偏好设置失败:', error)
    }
  }

  // 加载引导偏好设置
  function loadGuidePreferences() {
    try {
      const saved = localStorage.getItem('cunzhi-guide-preferences')
      return saved ? JSON.parse(saved) : {}
    }
    catch (error) {
      console.error('加载引导偏好设置失败:', error)
      return {}
    }
  }

  // 检查是否是第一次使用
  function isFirstTimeUser() {
    try {
      const hasSeenGuide = localStorage.getItem('cunzhi-has-seen-guide')
      return !hasSeenGuide
    }
    catch (error) {
      console.error('检查首次使用状态失败:', error)
      return true
    }
  }

  // 标记用户已看过引导
  function markGuideAsSeen() {
    try {
      localStorage.setItem('cunzhi-has-seen-guide', 'true')
    }
    catch (error) {
      console.error('标记引导状态失败:', error)
    }
  }

  return {
    isGuideActive,
    currentGuide,
    currentStep,
    currentGuideKey,
    startGuide,
    stopGuide,
    nextStep,
    previousStep,
    goToStep,
    guideFlows,
    saveGuidePreferences,
    loadGuidePreferences,
    isFirstTimeUser,
    markGuideAsSeen,
    forceCleanup,
    resetGuideState,
    checkEnvironment,
  }
}
