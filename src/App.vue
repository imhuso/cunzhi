<script setup>
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, ref } from 'vue'
import McpPopup from './components/McpPopup.vue'

// 响应式数据
const mcpRequest = ref(null)
const showMcpPopup = ref(false)
const isDarkMode = ref(false)
const alwaysOnTop = ref(true)

// 强制应用暗黑主题
function setupDarkMode() {
  // 始终应用暗黑主题
  isDarkMode.value = true
  document.documentElement.classList.add('dark')
  localStorage.setItem('theme', 'dark')
}

// 统一的MCP响应处理
async function handleMcpResponse(response) {
  try {
    // 通过Tauri命令发送响应并退出应用
    await invoke('send_mcp_response', { response })
    await invoke('exit_app')
  }
  catch (error) {
    console.error('处理MCP响应失败:', error)
  }
}

// 统一的MCP取消处理
async function handleMcpCancel() {
  try {
    // 发送取消信息并退出应用
    await invoke('send_mcp_response', { response: 'CANCELLED' })
    await invoke('exit_app')
  }
  catch (error) {
    console.error('处理MCP取消失败:', error)
  }
}

// 加载窗口设置
async function loadWindowSettings() {
  try {
    const enabled = await invoke('get_always_on_top')
    alwaysOnTop.value = enabled

    // 同步窗口状态
    await invoke('sync_window_state')
    console.log('窗口设置已加载并同步:', enabled)
  }
  catch (error) {
    console.error('加载窗口设置失败:', error)
  }
}

// 切换置顶设置
async function toggleAlwaysOnTop() {
  try {
    const newValue = !alwaysOnTop.value
    await invoke('set_always_on_top', { enabled: newValue })
    alwaysOnTop.value = newValue
    console.log('窗口置顶设置已更新:', newValue)
  }
  catch (error) {
    console.error('切换置顶设置失败:', error)
  }
}

// 显示MCP弹窗
function showMcpDialog(request) {
  mcpRequest.value = request
  showMcpPopup.value = true
}

// 检查MCP模式
async function checkMcpMode() {
  try {
    const args = await invoke('get_cli_args')

    if (args && args.mcp_request) {
      // 读取MCP请求文件
      const content = await invoke('read_mcp_request', { filePath: args.mcp_request })

      if (content) {
        showMcpDialog(content)
      }
      return true // 表示是MCP模式
    }
  }
  catch (error) {
    console.error('检查MCP请求失败:', error)
  }
  return false // 表示不是MCP模式
}

// 设置MCP事件监听器
async function setupMcpEventListener() {
  try {
    await listen('mcp-request', (event) => {
      showMcpDialog(event.payload)
    })
  }
  catch (error) {
    console.error('设置MCP事件监听器失败:', error)
  }
}

// 初始化
onMounted(async () => {
  // 首先设置暗黑主题
  setupDarkMode()

  // 检查是否为MCP模式
  const isMcp = await checkMcpMode()

  // 无论是否为MCP模式，都加载窗口设置
  await loadWindowSettings()

  // 如果不是MCP模式，设置事件监听器
  if (!isMcp) {
    await setupMcpEventListener()
  }
})
</script>

<template>
  <div
    id="app"
    class="min-h-screen bg-dark-primary transition-colors duration-300"
  >
    <!-- MCP弹窗 -->
    <McpPopup
      v-if="showMcpPopup && mcpRequest"
      :request="mcpRequest"
      @response="handleMcpResponse"
      @cancel="handleMcpCancel"
    />

    <!-- MCP功能展示界面 -->
    <div
      v-else
      class="flex items-center justify-center min-h-screen p-6"
    >
      <div class="max-w-6xl w-full">
        <!-- 主标题 -->
        <div class="text-center mb-8">
          <div
            class="w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-4 shadow-lg"
          >
            <span class="text-white text-3xl">🤖</span>
          </div>
          <h1 class="text-3xl font-bold text-gray-100 mb-2">
            AI Review MCP
          </h1>
          <p class="text-lg text-gray-400">
            智能代码审查与交互工具
          </p>
        </div>

        <!-- 功能卡片 -->
        <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          <!-- MCP服务器功能 -->
          <div class="bg-dark-secondary rounded-xl p-6 shadow-lg border border-gray-700">
            <div class="flex items-center mb-4">
              <div class="w-12 h-12 bg-blue-900 rounded-lg flex items-center justify-center mr-4">
                <span class="text-2xl">🔧</span>
              </div>
              <div>
                <h3 class="text-lg font-semibold text-gray-100">
                  MCP 服务器
                </h3>
                <p class="text-sm text-gray-400">
                  Model Context Protocol
                </p>
              </div>
            </div>
            <ul class="space-y-2 text-sm text-gray-400">
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                智能代码审查交互
              </li>
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                支持文本和图片输入
              </li>
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                预定义选项支持
              </li>
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                Markdown 渲染
              </li>
            </ul>
          </div>

          <!-- 记忆管理功能 -->
          <div class="bg-dark-secondary rounded-xl p-6 shadow-lg border border-gray-700">
            <div class="flex items-center mb-4">
              <div class="w-12 h-12 bg-purple-900 rounded-lg flex items-center justify-center mr-4">
                <span class="text-2xl">🧠</span>
              </div>
              <div>
                <h3 class="text-lg font-semibold text-gray-100">
                  记忆管理
                </h3>
                <p class="text-sm text-gray-400">
                  智能记忆系统
                </p>
              </div>
            </div>
            <ul class="space-y-2 text-sm text-gray-400">
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                开发规范存储
              </li>
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                用户偏好记录
              </li>
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                项目信息管理
              </li>
              <li class="flex items-center">
                <span class="w-2 h-2 bg-green-500 rounded-full mr-2" />
                最佳实践收集
              </li>
            </ul>
          </div>

          <!-- 窗口设置功能 -->
          <div class="bg-dark-secondary rounded-xl p-6 shadow-lg border border-gray-700">
            <div class="flex items-center mb-4">
              <div class="w-12 h-12 bg-green-900 rounded-lg flex items-center justify-center mr-4">
                <span class="text-2xl">⚙️</span>
              </div>
              <div>
                <h3 class="text-lg font-semibold text-gray-100">
                  窗口设置
                </h3>
                <p class="text-sm text-gray-400">
                  显示偏好配置
                </p>
              </div>
            </div>
            <div class="space-y-4">
              <!-- 置顶显示切换开关 -->
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <span class="w-2 h-2 bg-blue-500 rounded-full mr-2" />
                  <span class="text-sm text-gray-300">总在最前</span>
                </div>
                <button
                  @click="toggleAlwaysOnTop"
                  :class="[
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800',
                    alwaysOnTop ? 'bg-blue-600' : 'bg-gray-600'
                  ]"
                >
                  <span
                    :class="[
                      'inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200 ease-in-out',
                      alwaysOnTop ? 'translate-x-6' : 'translate-x-1'
                    ]"
                  />
                </button>
              </div>
              <p class="text-xs text-gray-500">
                启用后窗口将始终保持在其他应用程序之上
              </p>
            </div>
          </div>
        </div>

        <!-- 使用说明 -->
        <div class="mt-8 bg-gradient-to-r from-blue-900/20 to-purple-900/20 rounded-xl p-6 border border-blue-800">
          <h3 class="text-lg font-semibold text-gray-100 mb-4 flex items-center">
            <span class="text-2xl mr-2">📋</span>
            使用方法
          </h3>
          <div class="grid gap-4 md:grid-cols-2 text-sm">
            <div>
              <h4 class="font-medium text-gray-100 mb-2">
                命令行工具
              </h4>
              <div class="space-y-1 text-gray-400 font-mono">
                <div>ai-review-mcp</div>
                <div>ai-review-ui --mcp-request file</div>
              </div>
            </div>
            <div>
              <h4 class="font-medium text-gray-100 mb-2">
                MCP 配置
              </h4>
              <div class="text-gray-400">
                将 ai-review-mcp 添加到您的<br />
                MCP 客户端配置中使用
              </div>
            </div>
          </div>
        </div>

        <!-- 状态指示 -->
        <div class="mt-6 text-center">
          <div class="inline-flex items-center px-4 py-2 bg-green-900/30 text-green-300 rounded-full text-sm">
            <div class="w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse" />
            MCP 服务器就绪
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 确保平滑的主题切换动画 */
#app {
  transition:
    background-color 0.3s ease,
    color 0.3s ease;
}

/* 加载动画 */
@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.5;
  }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>
