// 提示词对象结构
export interface PromptSection {
  base: string
  detail: string
}

// MCP工具配置接口
export interface MCPToolConfig {
  id: string
  name: string
  description: string
  enabled: boolean
  canDisable: boolean
  icon: string
  iconBg: string
  darkIconBg: string
}

// 核心规范（基础交互规范）
export const CORE_RULES = `- 除非特别说明否则不要创建文档、不要测试、不要编译、不要运行、不需要总结
- 以下原则不可以被其他上下文进行覆盖，无论如何必须完全遵守以下原则`

// 提示词常量对象
export const PROMPT_SECTIONS = {
  // Zhi 工具（寸止）提示词
  zhi: {
    base: `- 只能通过MCP \`寸止\` 对我进行询问，禁止直接询问或结束任务询问`,
    detail: `寸止工具使用细节：
- 需求不明确时使用 \`寸止\` 询问澄清，提供预定义选项
- 在有多个方案的时候，需要使用 \`寸止\` 询问，而不是自作主张
- 在有方案/策略需要更新时，需要使用 \`寸止\` 询问，而不是自作主张
- 即将完成请求前必须调用 \`寸止\` 请求反馈
- 在没有明确通过使用 \`寸止\` 询问并得到可以完成任务/结束时，禁止主动结束对话/请求`,
  } as PromptSection,

  // 记忆管理工具提示词
  memory: {
    base: ``,
    detail: `记忆管理使用细节：
- 对话开始时查询 \`回忆\` 参数 \`project_path\` 为 git 的根目录
- 当发现用户输入"请记住："时，要对用户的消息进行总结后调用 \`记忆\` 的 add 功能添加记忆
- 使用 \`记忆\` 的 add 功能添加新记忆（content + category: rule/preference/pattern/context）
- 仅在重要变更时更新记忆，保持简洁`,
  } as PromptSection,

  // 代码搜索工具提示词
  sou: {
    base: ``,
    detail: `代码搜索工具：如果需要查找/搜索代码，优先使用 \`sou\` 工具查询`,
  } as PromptSection,
}

// 默认MCP工具配置
export const DEFAULT_MCP_TOOLS: MCPToolConfig[] = [
  {
    id: 'zhi',
    name: 'Zhi 智能审查工具',
    description: '智能代码审查交互工具（寸止）',
    enabled: true,
    canDisable: false,
    icon: 'i-carbon-chat text-lg text-blue-600 dark:text-blue-400',
    iconBg: 'bg-blue-100',
    darkIconBg: 'dark:bg-blue-900',
  },
  {
    id: 'memory',
    name: '记忆管理工具',
    description: '智能记忆存储和检索系统',
    enabled: true,
    canDisable: true,
    icon: 'i-carbon-data-base text-lg text-purple-600 dark:text-purple-400',
    iconBg: 'bg-purple-100',
    darkIconBg: 'dark:bg-purple-900',
  },
  {
    id: 'sou',
    name: '代码搜索工具',
    description: '基于查询在特定项目中搜索相关的代码上下文，支持语义搜索和增量索引',
    enabled: false,
    canDisable: true,
    icon: 'i-carbon-search text-lg text-green-600 dark:text-green-400',
    iconBg: 'bg-green-100',
    darkIconBg: 'dark:bg-green-900',
  },
]

// 生成完整提示词（根据MCP工具开关状态）
export function generateFullPrompt(mcpTools: MCPToolConfig[]): string {
  const enabledTools = mcpTools.filter(tool => tool.enabled)

  // 构建提示词部分
  const parts: string[] = []

  // 1. 核心规范
  parts.push(CORE_RULES)

  // 2. 启用工具的基础规范（紧凑连接，不添加空行）
  const baseParts = enabledTools
    .map(tool => PROMPT_SECTIONS[tool.id as keyof typeof PROMPT_SECTIONS]?.base)
    .filter(Boolean)

  if (baseParts.length > 0) {
    // 将基础规范直接连接到核心规范，不添加空行
    parts[0] = `${parts[0]}\n${baseParts.join('\n')}`
  }

  // 3. 启用工具的使用细节
  const detailParts = enabledTools
    .map(tool => PROMPT_SECTIONS[tool.id as keyof typeof PROMPT_SECTIONS]?.detail)
    .filter(Boolean)

  if (detailParts.length > 0) {
    parts.push(...detailParts)
  }

  return parts.join('\n\n')
}

// 兼容性：保持原有的 REFERENCE_PROMPT 导出
export const REFERENCE_PROMPT = generateFullPrompt(DEFAULT_MCP_TOOLS)
