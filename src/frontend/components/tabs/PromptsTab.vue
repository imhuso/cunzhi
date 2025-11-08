<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useMcpToolsReactive } from '../../composables/useMcpTools'
import { generateFullPrompt } from '../../constants/prompts'

// 使用全局MCP工具状态
const { mcpTools, loading: mcpLoading, loadMcpTools, enabledTools } = useMcpToolsReactive()

// 根据MCP工具状态动态生成提示词
const promptContent = computed(() => {
  // 将后端数据格式转换为前端格式
  const frontendTools = mcpTools.value.map(tool => ({
    id: tool.id === 'ji' ? 'memory' : tool.id, // 后端用ji，前端用memory
    name: tool.name,
    description: tool.description,
    enabled: tool.enabled,
    canDisable: tool.can_disable,
    icon: tool.icon,
    iconBg: tool.icon_bg,
    darkIconBg: tool.dark_icon_bg,
  })).filter((tool) => {
    // 只包含有提示词配置的工具
    return tool.id === 'zhi' || tool.id === 'memory' || tool.id === 'sou'
  })

  return generateFullPrompt(frontendTools)
})

const copyButtonText = ref('复制')

// 复制参考提示词内容
async function copyPromptContent() {
  try {
    await navigator.clipboard.writeText(promptContent.value)
    copyButtonText.value = '已复制'
    setTimeout(() => {
      copyButtonText.value = '复制'
    }, 2000)
  }
  catch (error) {
    copyButtonText.value = '复制失败'
    setTimeout(() => {
      copyButtonText.value = '复制'
    }, 2000)
    console.error('复制失败:', error)
  }
}

// 组件挂载时加载MCP工具配置
onMounted(async () => {
  if (mcpTools.value.length === 0) {
    try {
      await loadMcpTools()
    }
    catch (error) {
      console.error('加载MCP工具配置失败:', error)
    }
  }
})
</script>

<template>
  <div class="max-w-3xl mx-auto tab-content">
    <n-space
      vertical
      size="medium"
    >
      <!-- 参考提示词卡片 -->
      <n-card size="small">
        <!-- 卡片头部 -->
        <template #header>
          <n-space
            align="center"
            justify="space-between"
          >
            <n-space align="center">
              <!-- 图标 -->
              <div class="w-10 h-10 rounded-lg bg-orange-100 dark:bg-orange-900 flex items-center justify-center">
                <div class="i-carbon-document text-lg text-orange-600 dark:text-orange-400" />
              </div>

              <!-- 标题信息 -->
              <div>
                <div class="text-lg font-medium mb-1 tracking-tight">
                  参考提示词
                </div>
                <div class="text-sm opacity-60 font-normal">
                  基于MCP工具配置动态生成的系统提示词
                </div>
              </div>
            </n-space>

            <!-- 复制按钮 -->
            <n-button
              type="primary"
              size="small"
              @click="copyPromptContent"
            >
              <template #icon>
                <div class="i-carbon-copy text-sm" />
              </template>
              {{ copyButtonText }}
            </n-button>
          </n-space>
        </template>

        <!-- 工具状态说明 -->
        <div class="flex items-center text-sm leading-relaxed mb-4">
          <div
            class="w-1.5 h-1.5 rounded-full mr-3 flex-shrink-0"
            :class="mcpLoading ? 'bg-yellow-500' : 'bg-green-500'"
          />
          <span class="opacity-90">
            <template v-if="mcpLoading">
              正在加载MCP工具配置...
            </template>
            <template v-else>
              当前已启用 {{ enabledTools.length }} / {{ mcpTools.length }} 个MCP工具，
              可在"MCP工具"页面管理工具开关
            </template>
          </span>
        </div>

        <!-- 启用工具列表 -->
        <div class="mb-4">
          <div class="text-sm font-medium mb-2 opacity-80">
            已启用的工具模块：
          </div>
          <n-space v-if="!mcpLoading && enabledTools.length > 0">
            <n-tag
              v-for="tool in enabledTools"
              :key="tool.id"
              size="small"
              type="success"
              :bordered="false"
            >
              <template #icon>
                <div
                  :class="tool.icon"
                  class="text-xs"
                />
              </template>
              {{ tool.name }}
            </n-tag>
          </n-space>
          <div
            v-else-if="!mcpLoading && enabledTools.length === 0"
            class="text-sm opacity-60"
          >
            暂无启用的工具
          </div>
          <n-skeleton
            v-else
            text
            :repeat="2"
          />
        </div>

        <!-- 内容区域 -->
        <n-card embedded>
          <div class="text-sm font-mono leading-relaxed">
            <pre class="whitespace-pre-wrap my-0 opacity-90">{{ promptContent }}</pre>
          </div>
        </n-card>
      </n-card>

      <!-- 使用说明卡片 -->
      <n-card size="small">
        <template #header>
          <n-space align="center">
            <!-- 图标 -->
            <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
              <div class="i-carbon-information text-lg text-blue-600 dark:text-blue-400" />
            </div>

            <!-- 标题信息 -->
            <div>
              <div class="text-lg font-medium mb-1 tracking-tight">
                使用说明
              </div>
              <div class="text-sm opacity-60 font-normal">
                如何使用和配置提示词
              </div>
            </div>
          </n-space>
        </template>

        <n-space
          vertical
          size="small"
        >
          <div class="flex items-center text-sm leading-relaxed">
            <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-3 flex-shrink-0" />
            <span class="opacity-90">将此提示词添加到您的AI助手系统提示中，以获得最佳的交互体验</span>
          </div>
          <div class="flex items-center text-sm leading-relaxed">
            <div class="w-1.5 h-1.5 bg-blue-500 rounded-full mr-3 flex-shrink-0" />
            <span class="opacity-90">提示词内容会根据"MCP工具"页面的开关设置自动更新</span>
          </div>
          <div class="flex items-center text-sm leading-relaxed">
            <div class="w-1.5 h-1.5 bg-purple-500 rounded-full mr-3 flex-shrink-0" />
            <span class="opacity-90">关闭的工具不会包含在生成的提示词中，对应的MCP服务也不会启用</span>
          </div>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>
