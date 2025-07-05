<script setup lang="ts">
import MainLayout from './MainLayout.vue'

interface AppConfig {
  theme: string
  window: {
    alwaysOnTop: boolean
    width: number
    height: number
    fixed: boolean
  }
  audio: {
    enabled: boolean
    url: string
  }
  reply: {
    enabled: boolean
    prompt: string
  }
}

interface Props {
  appConfig: AppConfig
}

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

defineProps<Props>()
defineEmits<Emits>()
</script>

<template>
  <MainLayout
    :current-theme="appConfig.theme"
    :always-on-top="appConfig.window.alwaysOnTop"
    :audio-notification-enabled="appConfig.audio.enabled"
    :audio-url="appConfig.audio.url"
    :window-width="appConfig.window.width"
    :window-height="appConfig.window.height"
    :fixed-window-size="appConfig.window.fixed"
    @theme-change="$emit('themeChange', $event)"
    @toggle-always-on-top="$emit('toggleAlwaysOnTop')"
    @toggle-audio-notification="$emit('toggleAudioNotification')"
    @update-audio-url="$emit('updateAudioUrl', $event)"
    @test-audio="$emit('testAudio')"
    @stop-audio="$emit('stopAudio')"
    @test-audio-error="$emit('testAudioError', $event)"
    @update-window-size="$emit('updateWindowSize', $event)"
    @config-reloaded="$emit('configReloaded')"
  />
</template>
