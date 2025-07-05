<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { onMounted, onUnmounted, ref } from 'vue'
import AudioSettings from '../settings/AudioSettings.vue'
import CustomPromptSettings from '../settings/CustomPromptSettings.vue'
import FontSettings from '../settings/FontSettings.vue'
import ReplySettings from '../settings/ReplySettings.vue'
import ShortcutSettings from '../settings/ShortcutSettings.vue'
import TelegramSettings from '../settings/TelegramSettings.vue'
import ThemeSettings from '../settings/ThemeSettings.vue'
import VersionChecker from '../settings/VersionChecker.vue'
import WindowSettings from '../settings/WindowSettings.vue'

interface Props {
  currentTheme: string
  alwaysOnTop: boolean
  audioNotificationEnabled: boolean
  audioUrl: string
  windowWidth: number
  windowHeight: number
  fixedWindowSize: boolean
}

defineProps<Props>()
const emit = defineEmits<Emits>()
const message = useMessage()
const isReloading = ref(false)
const configFilePath = ref('config.json')
let unlistenConfigReloaded: (() => void) | null = null

// 重新加载配置（通过重新加载设置实现）
async function reloadConfig() {
  if (isReloading.value)
    return

  isReloading.value = true
  try {
    // 触发重新加载设置的事件
    emit('configReloaded')
    message.success('配置已重新加载')
  }
  catch (error) {
    console.error('重新加载配置失败:', error)
    message.error('重新加载配置失败')
  }
  finally {
    isReloading.value = false
  }
}

// 获取配置文件路径
async function loadConfigFilePath() {
  try {
    const path = await invoke('get_config_file_path')
    configFilePath.value = path as string
    console.log('配置文件路径:', configFilePath.value)
  }
  catch (error) {
    console.error('获取配置文件路径失败:', error)
    configFilePath.value = 'config.json' // 使用默认值
  }
}

// 监听配置重载事件
onMounted(async () => {
  try {
    // 获取配置文件路径
    await loadConfigFilePath()

    unlistenConfigReloaded = await listen('config_reloaded', () => {
      // 配置重载后，重新加载设置而不是刷新整个页面
      console.log('收到配置重载事件，重新加载设置')
      // 触发重新加载设置的事件
      emit('configReloaded')
    })
  }
  catch (error) {
    console.error('设置配置重载监听器失败:', error)
  }
})

onUnmounted(() => {
  if (unlistenConfigReloaded) {
    unlistenConfigReloaded()
  }
})

interface Emits {
  themeChange: [theme: string]
  toggleAlwaysOnTop: []
  toggleAudioNotification: []
  updateAudioUrl: [url: string]
  testAudio: []
  stopAudio: []
  testAudioError: [error: any]
  updateWindowSize: [size: { width: number, height: number, fixed: boolean }]
  configReloaded: []
}

// 处理窗口尺寸更新
function handleWindowSizeUpdate(size: { width: number, height: number, fixed: boolean }) {
  emit('updateWindowSize', size)
}
</script>

<template>
  <div class="max-w-3xl mx-auto tab-content">
    <n-collapse size="large" :default-expanded-names="[]" arrow-placement="right">
      <!-- 主题设置 -->
      <n-collapse-item name="theme">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center mr-4">
                <div class="i-carbon-color-palette text-lg text-primary-600 dark:text-primary-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  主题设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  选择您喜欢的界面主题
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <ThemeSettings :current-theme="currentTheme" @theme-change="$emit('themeChange', $event)" />
        </div>
      </n-collapse-item>



      <!-- 字体设置 -->
      <n-collapse-item name="font">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-orange-100 dark:bg-orange-900 flex items-center justify-center mr-4">
                <div class="i-carbon-text-font text-lg text-orange-600 dark:text-orange-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  字体设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  自定义应用字体系列和大小
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <FontSettings />
        </div>
      </n-collapse-item>

      <!-- 继续回复设置 -->
      <n-collapse-item name="reply">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900 flex items-center justify-center mr-4">
                <div class="i-carbon-continue text-lg text-blue-600 dark:text-blue-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  继续回复设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  配置AI继续回复的行为
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <ReplySettings />
        </div>
      </n-collapse-item>

      <!-- 窗口设置 -->
      <n-collapse-item name="window">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-green-100 dark:bg-green-900 flex items-center justify-center mr-4">
                <div class="i-carbon-application text-lg text-green-600 dark:text-green-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  窗口设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  调整窗口显示和行为
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <WindowSettings
            :always-on-top="alwaysOnTop"
            :window-width="windowWidth"
            :window-height="windowHeight"
            :fixed-window-size="fixedWindowSize"
            @toggle-always-on-top="$emit('toggleAlwaysOnTop')"
            @update-window-size="handleWindowSizeUpdate"
          />
        </div>
      </n-collapse-item>

      <!-- 音频设置 -->
      <n-collapse-item name="audio">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-yellow-100 dark:bg-yellow-900 flex items-center justify-center mr-4">
                <div class="i-carbon-volume-up text-lg text-yellow-600 dark:text-yellow-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  音频设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  配置音频通知和提示音
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <AudioSettings
            :audio-notification-enabled="audioNotificationEnabled"
            :audio-url="audioUrl"
            @toggle-audio-notification="$emit('toggleAudioNotification')"
            @update-audio-url="$emit('updateAudioUrl', $event)"
            @test-audio="$emit('testAudio')"
            @stop-audio="$emit('stopAudio')"
            @test-audio-error="$emit('testAudioError', $event)"
          />
        </div>
      </n-collapse-item>

      <!-- Telegram设置 -->
      <n-collapse-item name="telegram">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-cyan-100 dark:bg-cyan-900 flex items-center justify-center mr-4">
                <div class="i-carbon-send text-lg text-cyan-600 dark:text-cyan-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  Telegram设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  配置Telegram机器人集成
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <TelegramSettings />
        </div>
      </n-collapse-item>

      <!-- 快捷模板设置 -->
      <n-collapse-item name="custom-prompt">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-orange-100 dark:bg-orange-900 flex items-center justify-center mr-4">
                <div class="i-carbon-text-creation text-lg text-orange-600 dark:text-orange-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  提示词模板
                </div>
                <div class="text-sm opacity-60 font-normal">
                  管理快捷模板和上下文追加
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <CustomPromptSettings />
        </div>
      </n-collapse-item>

      <!-- 快捷键设置 -->
      <n-collapse-item name="shortcuts">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center mr-4">
                <div class="i-carbon-keyboard text-lg text-indigo-600 dark:text-indigo-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  快捷键设置
                </div>
                <div class="text-sm opacity-60 font-normal">
                  自定义应用快捷键绑定
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <ShortcutSettings />
        </div>
      </n-collapse-item>

      <!-- 配置管理 -->
      <n-collapse-item name="config">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900 flex items-center justify-center mr-4">
                <div class="i-carbon-settings-adjust text-lg text-blue-600 dark:text-blue-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  配置管理
                </div>
                <div class="text-sm opacity-60 font-normal">
                  重新加载配置文件和管理设置
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <n-space vertical size="large">
            <!-- 重新加载配置 -->
            <div class="flex items-center justify-between">
              <div class="flex items-center">
                <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0" />
                <div>
                  <div class="text-sm font-medium leading-relaxed">
                    重新加载配置文件
                  </div>
                  <div class="text-xs opacity-60">
                    从 config.json 重新加载所有配置设置
                  </div>
                </div>
              </div>
              <n-button
                size="small"
                type="primary"
                :loading="isReloading"
                @click="reloadConfig"
              >
                <template #icon>
                  <div class="i-carbon-restart w-4 h-4" />
                </template>
                重新加载
              </n-button>
            </div>

            <!-- 配置文件位置说明 -->
            <div class="flex items-start">
              <div class="w-1.5 h-1.5 bg-warning rounded-full mr-3 flex-shrink-0 mt-2" />
              <div>
                <div class="text-sm font-medium leading-relaxed mb-1">
                  配置文件位置
                </div>
                <div class="text-xs opacity-60 leading-relaxed">
                  配置文件路径：<br>
                  <code class="bg-black-100 px-1 rounded text-xs break-all">{{ configFilePath }}</code><br>
                  您可以直接编辑该文件，然后点击"重新加载"按钮使更改生效
                </div>
              </div>
            </div>
          </n-space>
        </div>
      </n-collapse-item>

      <!-- 版本检查 -->
      <n-collapse-item name="version">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center">
              <div class="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900 flex items-center justify-center mr-4">
                <div class="i-carbon-update-now text-lg text-purple-600 dark:text-purple-400" />
              </div>
              <div>
                <div class="text-lg font-medium tracking-tight mb-1">
                  版本检查
                </div>
                <div class="text-sm opacity-60 font-normal">
                  检查应用更新和版本信息
                </div>
              </div>
            </div>
          </div>
        </template>
        <div class="setting-content">
          <VersionChecker />
        </div>
      </n-collapse-item>
    </n-collapse>
  </div>
</template>

<style>
/* 移除子组件的卡片样式，因为现在由折叠面板提供容器 */
.setting-content :deep(.n-card) {
  background: transparent;
  border: none;
  box-shadow: none;
}

.setting-content :deep(.n-card .n-card-header) {
  display: none;
}
</style>
