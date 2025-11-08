import { invoke } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'

// MCP工具配置接口
export interface MCPToolConfig {
  id: string
  name: string
  description: string
  enabled: boolean
  can_disable: boolean
  icon: string
  icon_bg: string
  dark_icon_bg: string
  has_config?: boolean
}

// 全局MCP工具状态
const mcpTools = ref<MCPToolConfig[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// 计算属性：启用的工具
const enabledTools = computed(() => mcpTools.value.filter(tool => tool.enabled))

// 计算属性：工具统计
const toolStats = computed(() => ({
  total: mcpTools.value.length,
  enabled: enabledTools.value.length,
  disabled: mcpTools.value.length - enabledTools.value.length,
}))

// 加载MCP工具配置
async function loadMcpTools() {
  try {
    loading.value = true
    error.value = null
    const tools = await invoke('get_mcp_tools_config') as MCPToolConfig[]
    mcpTools.value = tools
    console.log('✅ MCP工具配置已加载:', tools)
  }
  catch (err) {
    error.value = `加载MCP工具配置失败: ${err}`
    console.error('❌ 加载MCP工具配置失败:', err)
    throw err
  }
  finally {
    loading.value = false
  }
}

// 切换工具启用状态
async function toggleTool(toolId: string) {
  const tool = mcpTools.value.find(t => t.id === toolId)
  if (!tool || !tool.can_disable) {
    throw new Error('工具不存在或不可禁用')
  }

  try {
    const newEnabled = !tool.enabled
    await invoke('set_mcp_tool_enabled', {
      toolId,
      enabled: newEnabled,
    })

    // 更新本地状态
    tool.enabled = newEnabled

    console.log(`✅ 工具 ${toolId} 状态已更新为: ${newEnabled}`)

    return {
      toolId,
      enabled: newEnabled,
      needsReconnect: true,
    }
  }
  catch (err) {
    error.value = `更新MCP工具状态失败: ${err}`
    console.error('❌ 更新MCP工具状态失败:', err)
    throw err
  }
}

// 重置MCP工具配置
async function resetMcpTools() {
  try {
    loading.value = true
    error.value = null
    await invoke('reset_mcp_tools_config')
    await loadMcpTools() // 重新加载配置
    console.log('✅ MCP工具配置已重置')
  }
  catch (err) {
    error.value = `重置MCP工具配置失败: ${err}`
    console.error('❌ 重置MCP工具配置失败:', err)
    throw err
  }
  finally {
    loading.value = false
  }
}

// 获取工具状态
function getToolStatus(toolId: string): boolean {
  const tool = mcpTools.value.find(t => t.id === toolId)
  return tool?.enabled ?? false
}

// 检查工具是否可禁用
function canDisableTool(toolId: string): boolean {
  const tool = mcpTools.value.find(t => t.id === toolId)
  return tool?.can_disable ?? false
}

// 全局MCP工具管理composable
export function useMcpTools() {
  return {
    // 状态
    mcpTools: mcpTools.value,
    loading: loading.value,
    error: error.value,

    // 计算属性
    enabledTools,
    toolStats,

    // 方法
    loadMcpTools,
    toggleTool,
    resetMcpTools,
    getToolStatus,
    canDisableTool,
  }
}

// 响应式状态访问（用于模板）
export function useMcpToolsReactive() {
  return {
    // 响应式状态
    mcpTools,
    loading,
    error,

    // 计算属性
    enabledTools,
    toolStats,

    // 方法
    loadMcpTools,
    toggleTool,
    resetMcpTools,
    getToolStatus,
    canDisableTool,
  }
}

// 初始化函数（在应用启动时调用）
export async function initMcpTools() {
  try {
    await loadMcpTools()
  }
  catch (err) {
    console.error('初始化MCP工具失败:', err)
  }
}
