<template>
  <div class="min-h-screen bg-gray-50 dark:bg-dark-primary">
    <!-- 主界面 - 仅在非MCP模式下显示 -->
    <template v-if="!isMcpMode">
      <!-- 标题栏 -->
      <div class="bg-white dark:bg-dark-secondary border-b border-gray-200 dark:border-gray-700 px-4 py-2" data-tauri-drag-region>
        <div class="flex items-center justify-center">
          <div class="flex items-center gap-2">
            <span class="text-blue-500 text-base">🤖</span>
            <h1 class="text-sm font-medium text-gray-900 dark:text-gray-100">AI Review</h1>
          </div>
        </div>
      </div>

      <!-- 主内容区域 -->
      <div class="flex items-center justify-center min-h-[calc(100vh-46px)] p-4">
        <div class="max-w-lg w-full text-center">
          <div class="bg-white dark:bg-dark-secondary rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-8">
            <div class="mb-6">
              <span class="text-5xl text-blue-500 mb-4 block">🤖</span>
              <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">AI Review</h2>
              <p class="text-gray-600 dark:text-gray-400 text-sm">{{ appInfo }}</p>
            </div>

            <div class="space-y-4">
              <div class="alert alert-info">
                <div class="flex items-center gap-2">
                  <span class="text-lg">🚀</span>
                  <div class="text-left">
                    <h3 class="text-sm font-medium mb-1">服务状态</h3>
                    <p class="text-xs opacity-80">MCP服务器已启动，等待连接...</p>
                  </div>
                </div>
              </div>

              <div class="alert alert-success">
                <div class="flex items-center gap-2">
                  <span class="text-lg">📋</span>
                  <div class="text-left">
                    <h3 class="text-sm font-medium mb-1">支持的工具</h3>
                    <p class="text-xs opacity-80">ai_review_chat - 智能代码审查交互</p>
                  </div>
                </div>
              </div>

              <div class="alert alert-warning">
                <div class="flex items-center gap-2">
                  <span class="text-lg">💡</span>
                  <div class="text-left">
                    <h3 class="text-sm font-medium mb-1">使用提示</h3>
                    <p class="text-xs opacity-80">在Claude Desktop中通过MCP协议调用此服务</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- MCP弹窗 -->
    <McpPopup
      v-if="showMcpPopup && mcpRequest"
      :request="mcpRequest"
      @response="handleMcpResponse"
      @cancel="handleMcpCancel"
    />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import McpPopup from './components/McpPopup.vue'

// 响应式数据
const appInfo = ref('AI Review App v0.1.0')
const mcpRequest = ref(null)
const showMcpPopup = ref(false)
const isMcpMode = ref(false)

// 获取应用信息
async function getAppInfo() {
  try {
    appInfo.value = await invoke('get_app_info')
  } catch (error) {
    console.error('获取应用信息失败:', error)
    appInfo.value = 'AI Review App v0.1.0'
  }
}

// 统一的MCP响应处理
async function handleMcpResponse(response) {
  console.log('MCP响应:', response)

  try {
    if (isMcpMode.value) {
      // MCP模式下，通过Tauri命令发送响应并退出应用
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    } else {
      // 普通模式下，只关闭弹窗
      closeMcpPopup()
    }
  } catch (error) {
    console.error('处理MCP响应失败:', error)
  }
}

// 统一的MCP取消处理
async function handleMcpCancel() {
  console.log('MCP取消')

  try {
    if (isMcpMode.value) {
      // MCP模式下，发送取消信息并退出应用
      await invoke('send_mcp_response', { response: 'CANCELLED' })
      await invoke('exit_app')
    } else {
      // 普通模式下，只关闭弹窗
      closeMcpPopup()
    }
  } catch (error) {
    console.error('处理MCP取消失败:', error)
  }
}

// 关闭MCP弹窗
function closeMcpPopup() {
  showMcpPopup.value = false
  mcpRequest.value = null
}

// 显示MCP弹窗
function showMcpDialog(request) {
  console.log('显示MCP弹窗, request:', request)
  mcpRequest.value = request
  showMcpPopup.value = true
}

// 检查MCP模式
async function checkMcpMode() {
  try {
    const args = await invoke('get_cli_args')
    console.log('CLI参数:', args)

    if (args && args.mcp_request) {
      console.log('检测到MCP请求文件:', args.mcp_request)

      // 设置为MCP模式
      isMcpMode.value = true

      // 读取MCP请求文件
      const content = await invoke('read_mcp_request', { filePath: args.mcp_request })
      console.log('MCP请求内容:', content)

      if (content) {
        showMcpDialog(content)
      }
      return true // 表示是MCP模式
    }
  } catch (error) {
    console.error('检查MCP请求失败:', error)
  }
  return false // 表示不是MCP模式
}

// 设置MCP事件监听器
async function setupMcpEventListener() {
  try {
    await listen('mcp-request', (event) => {
      console.log('收到MCP请求:', event.payload)
      showMcpDialog(event.payload)
    })
    console.log('MCP事件监听器设置成功')
  } catch (error) {
    console.error('设置MCP事件监听器失败:', error)
  }
}

// 初始化
onMounted(async () => {
  // 检查是否是MCP模式
  const isMcp = await checkMcpMode()

  if (!isMcp) {
    // 非MCP模式：获取应用信息并设置事件监听器
    await getAppInfo()
    await setupMcpEventListener()
  }
})
</script>

<style>
/* 全局样式重置 */
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* Ant Design 样式覆盖 */
.ant-btn {
  border-radius: 6px;
}

.ant-modal {
  border-radius: 8px;
}

/* 代码样式 */
code {
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
}
</style>
