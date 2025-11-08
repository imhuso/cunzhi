<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-shell'
import { useMessage, useDialog } from 'naive-ui'
import { onMounted, ref, computed } from 'vue'
import { API_BASE_URL, API_EXAMPLES } from '../../constants/telegram'

interface TelegramBotConfig {
  name: string
  bot_token: string
  chat_id: string
  api_base_url: string
}

interface TelegramConfig {
  enabled: boolean
  hide_frontend_popup: boolean
  bots: TelegramBotConfig[]
  default_bot: string
}

const emit = defineEmits(['telegramConfigChange'])

// Naive UI 实例
const message = useMessage()
const dialog = useDialog()

// 配置状态
const telegramConfig = ref<TelegramConfig>({
  enabled: false,
  hide_frontend_popup: false,
  bots: [],
  default_bot: '',
})

// Bot 管理状态
const showBotDialog = ref(false)
const editingBot = ref<TelegramBotConfig | null>(null)
const editingBotOriginalName = ref('')
const botForm = ref<TelegramBotConfig>({
  name: '',
  bot_token: '',
  chat_id: '',
  api_base_url: API_BASE_URL,
})

// 会话映射管理
const showSessionMappingDialog = ref(false)
const sessionMappings = ref<Record<string, string>>({})
const newSessionId = ref('')
const newSessionBotName = ref('')

// 待配置会话管理
interface PendingSession {
  session_id: string
  first_seen: string
  last_seen: string
  request_count: number
}
const pendingSessions = ref<PendingSession[]>([])
const showPendingSessionsDialog = ref(false)
const configuringSession = ref<PendingSession | null>(null)
const showConfigureDialog = ref(false)
const configureForm = ref({
  useExistingBot: false, // 是否使用已有 Bot
  selectedBotName: '', // 选中的已有 Bot 名称
  botName: '',
  botToken: '',
  chatId: '',
  apiBaseUrl: API_BASE_URL,
})

// 测试状态
const isTesting = ref(false)

// Chat ID自动获取状态
const isDetectingChatId = ref(false)
const detectedChatInfo = ref<any>(null)

// 计算属性
const hasDefaultBot = computed(() => {
  return telegramConfig.value.bots.length > 0 && telegramConfig.value.default_bot
})

// 加载Telegram配置
async function loadTelegramConfig() {
  try {
    const config = await invoke('get_telegram_config') as TelegramConfig
    telegramConfig.value = config
  }
  catch (error) {
    console.error('加载Telegram配置失败:', error)
    message.error('加载Telegram配置失败')
  }
}

// 保存配置
async function saveTelegramConfig() {
  try {
    await invoke('set_telegram_config', { telegramConfig: telegramConfig.value })
    message.success('Telegram配置已保存')
    emit('telegramConfigChange', telegramConfig.value)
  }
  catch (error) {
    console.error('保存Telegram配置失败:', error)
    message.error('保存Telegram配置失败')
  }
}

// 切换启用状态
async function toggleTelegramEnabled() {
  // v-model 已经自动更新了值，这里只需要保存
  await saveTelegramConfig()
}

// 切换隐藏前端弹窗
async function toggleHideFrontendPopup() {
  // v-model 已经自动更新了值，这里只需要保存
  await saveTelegramConfig()
}

// Bot 管理函数
function openAddBotDialog() {
  editingBot.value = null
  editingBotOriginalName.value = ''
  botForm.value = {
    name: '',
    bot_token: '',
    chat_id: '',
    api_base_url: API_BASE_URL,
  }
  showBotDialog.value = true
}

function openEditBotDialog(bot: TelegramBotConfig) {
  editingBot.value = bot
  editingBotOriginalName.value = bot.name
  botForm.value = { ...bot }
  showBotDialog.value = true
}

async function saveBotConfig() {
  // 验证表单
  if (!botForm.value.name.trim()) {
    message.warning('请输入 Bot 名称')
    return
  }
  if (!botForm.value.bot_token.trim()) {
    message.warning('请输入 Bot Token')
    return
  }
  if (!botForm.value.chat_id.trim()) {
    message.warning('请输入 Chat ID')
    return
  }

  try {
    if (editingBot.value) {
      // 更新现有 bot
      await invoke('update_telegram_bot', {
        oldName: editingBotOriginalName.value,
        bot: botForm.value,
      })
      message.success('Bot 配置已更新')
    }
    else {
      // 添加新 bot
      await invoke('add_telegram_bot', { bot: botForm.value })
      message.success('Bot 配置已添加')
    }

    // 重新加载配置
    await loadTelegramConfig()
    showBotDialog.value = false
  }
  catch (error: any) {
    console.error('保存 Bot 配置失败:', error)
    message.error(error || '保存 Bot 配置失败')
  }
}

function deleteBot(botName: string) {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除 Bot "${botName}" 吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invoke('remove_telegram_bot', { botName })
        message.success('Bot 已删除')
        await loadTelegramConfig()
      }
      catch (error: any) {
        console.error('删除 Bot 失败:', error)
        message.error(error || '删除 Bot 失败')
      }
    },
  })
}

async function setDefaultBot(botName: string) {
  try {
    await invoke('set_default_telegram_bot', { botName })
    message.success(`已设置 "${botName}" 为默认 Bot`)
    await loadTelegramConfig()
  }
  catch (error: any) {
    console.error('设置默认 Bot 失败:', error)
    message.error(error || '设置默认 Bot 失败')
  }
}

// 测试 Bot 连接
async function testBotConnection(bot: TelegramBotConfig) {
  if (!bot.bot_token.trim()) {
    message.warning('请输入Bot Token')
    return
  }

  if (!bot.chat_id.trim()) {
    message.warning('请输入Chat ID')
    return
  }

  try {
    isTesting.value = true

    const result = await invoke('test_telegram_connection_cmd', {
      botToken: bot.bot_token,
      chatId: bot.chat_id,
    }) as string

    message.success(result)
  }
  catch (error) {
    console.error('测试Telegram连接失败:', error)
    message.error(typeof error === 'string' ? error : '测试连接失败')
  }
  finally {
    isTesting.value = false
  }
}

// 自动获取Chat ID（在 Bot 对话框中使用）
async function autoGetChatIdForBot() {
  if (!botForm.value.bot_token.trim()) {
    message.warning('请先输入Bot Token')
    return
  }

  try {
    isDetectingChatId.value = true
    detectedChatInfo.value = null

    // 定义清理函数数组
    const cleanupFunctions: (() => void)[] = []

    const unlistenStart = await listen('chat-id-detection-started', () => {
      message.info('开始监听消息，请向Bot发送任意消息...')
    })
    cleanupFunctions.push(unlistenStart)

    const unlistenDetected = await listen('chat-id-detected', (event: any) => {
      detectedChatInfo.value = event.payload
      message.success(`检测到Chat ID: ${event.payload.chat_id}`)
      isDetectingChatId.value = false

      // 自动填入Chat ID到表单
      botForm.value.chat_id = event.payload.chat_id

      // 清理所有监听器
      cleanupFunctions.forEach(cleanup => cleanup())
    })
    cleanupFunctions.push(unlistenDetected)

    const unlistenTimeout = await listen('chat-id-detection-timeout', () => {
      message.warning('检测超时，请确保Bot Token正确并向Bot发送消息')
      isDetectingChatId.value = false

      // 清理所有监听器
      cleanupFunctions.forEach(cleanup => cleanup())
    })
    cleanupFunctions.push(unlistenTimeout)

    // 开始自动获取
    await invoke('auto_get_chat_id', { botToken: botForm.value.bot_token })
  }
  catch (error) {
    console.error('自动获取Chat ID失败:', error)
    message.error('自动获取Chat ID失败')
    isDetectingChatId.value = false
  }
}

// 加载会话映射
async function loadSessionMappings() {
  try {
    const mappings = await invoke('get_session_bot_mappings') as Record<string, string>
    sessionMappings.value = mappings
  }
  catch (error) {
    console.error('加载会话映射失败:', error)
  }
}

// 打开会话映射管理对话框
function openSessionMappingDialog() {
  loadSessionMappings()
  showSessionMappingDialog.value = true
}

// 添加会话映射
async function addSessionMapping(sessionId: string, botName: string) {
  try {
    await invoke('set_session_bot_mapping', { sessionId, botName })
    message.success('会话映射已添加')
    await loadSessionMappings()
  }
  catch (error: any) {
    console.error('添加会话映射失败:', error)
    message.error(error || '添加会话映射失败')
  }
}

// 删除会话映射
async function removeSessionMapping(sessionId: string) {
  try {
    await invoke('remove_session_bot_mapping', { sessionId })
    message.success('会话映射已删除')
    await loadSessionMappings()
  }
  catch (error: any) {
    console.error('删除会话映射失败:', error)
    message.error(error || '删除会话映射失败')
  }
}

// 添加新的会话映射
async function addNewSessionMapping() {
  if (!newSessionId.value.trim() || !newSessionBotName.value) {
    message.warning('请填写完整信息')
    return
  }

  await addSessionMapping(newSessionId.value.trim(), newSessionBotName.value)
  newSessionId.value = ''
  newSessionBotName.value = ''
}

// 加载待配置会话
async function loadPendingSessions() {
  try {
    const sessions = await invoke('get_pending_sessions') as PendingSession[]
    pendingSessions.value = sessions
  }
  catch (error) {
    console.error('加载待配置会话失败:', error)
  }
}

// 打开待配置会话对话框
function openPendingSessionsDialog() {
  loadPendingSessions()
  showPendingSessionsDialog.value = true
}

// 开始配置会话
function startConfigureSession(session: PendingSession) {
  configuringSession.value = session
  // 从会话 ID 提取目录名作为默认 bot 名称
  const pathParts = session.session_id.split('/')
  const dirName = pathParts[pathParts.length - 1] || pathParts[pathParts.length - 2]
  configureForm.value.useExistingBot = false
  configureForm.value.selectedBotName = ''
  configureForm.value.botName = `${dirName} Bot`
  configureForm.value.botToken = ''
  configureForm.value.chatId = ''
  configureForm.value.apiBaseUrl = API_BASE_URL
  showConfigureDialog.value = true
}

// 打开 BotFather 创建 Bot
async function openBotFather() {
  try {
    await open('https://t.me/BotFather')
    message.success('已打开 Telegram BotFather')
  }
  catch (error) {
    console.error('打开 BotFather 失败:', error)
    message.error('打开 BotFather 失败，请手动在 Telegram 中搜索 @BotFather')
  }
}

// 自动获取 Chat ID（用于会话配置）
async function autoGetChatIdForSession() {
  if (!configureForm.value.botToken.trim()) {
    message.warning('请先输入 Bot Token')
    return
  }

  try {
    // 定义清理函数数组
    const cleanupFunctions: (() => void)[] = []

    const unlistenStart = await listen('chat-id-detection-started', () => {
      message.info('开始监听消息，请向Bot发送任意消息...')
    })
    cleanupFunctions.push(unlistenStart)

    const unlistenDetected = await listen('chat-id-detected', (event: any) => {
      message.success(`检测到Chat ID: ${event.payload.chat_id}`)

      // 自动填入Chat ID到表单
      configureForm.value.chatId = event.payload.chat_id

      // 清理所有监听器
      cleanupFunctions.forEach(cleanup => cleanup())
    })
    cleanupFunctions.push(unlistenDetected)

    const unlistenTimeout = await listen('chat-id-detection-timeout', () => {
      message.warning('检测超时，请确保Bot Token正确并向Bot发送消息')

      // 清理所有监听器
      cleanupFunctions.forEach(cleanup => cleanup())
    })
    cleanupFunctions.push(unlistenTimeout)

    // 开始自动获取
    await invoke('auto_get_chat_id', {
      botToken: configureForm.value.botToken.trim(),
    })
  }
  catch (error: any) {
    console.error('启动自动获取 Chat ID 失败:', error)
    message.error(error || '启动失败')
  }
}

// 保存会话配置
async function saveSessionConfiguration() {
  if (!configuringSession.value)
    return

  let botName: string
  let botToken: string
  let chatId: string
  let apiBaseUrl: string | null

  if (configureForm.value.useExistingBot) {
    // 使用已有 Bot
    if (!configureForm.value.selectedBotName) {
      message.warning('请选择一个 Bot')
      return
    }
    const selectedBot = telegramConfig.value.bots.find(b => b.name === configureForm.value.selectedBotName)
    if (!selectedBot) {
      message.error('选中的 Bot 不存在')
      return
    }
    botName = selectedBot.name
    botToken = selectedBot.bot_token
    chatId = selectedBot.chat_id
    apiBaseUrl = selectedBot.api_base_url || null
  }
  else {
    // 创建新 Bot
    if (!configureForm.value.botName.trim() || !configureForm.value.botToken.trim() || !configureForm.value.chatId.trim()) {
      message.warning('请填写完整信息')
      return
    }
    botName = configureForm.value.botName.trim()
    botToken = configureForm.value.botToken.trim()
    chatId = configureForm.value.chatId.trim()
    apiBaseUrl = configureForm.value.apiBaseUrl === API_BASE_URL ? null : configureForm.value.apiBaseUrl
  }

  try {
    if (configureForm.value.useExistingBot) {
      // 使用已有 Bot：只设置映射
      await invoke('set_session_bot_mapping', {
        sessionId: configuringSession.value.session_id,
        botName,
      })
    }
    else {
      // 创建新 Bot：创建 Bot 并设置映射
      await invoke('configure_session_bot', {
        sessionId: configuringSession.value.session_id,
        botName,
        botToken,
        chatId,
        apiBaseUrl,
      })
    }

    message.success('会话配置成功')
    showConfigureDialog.value = false
    configuringSession.value = null
    await loadPendingSessions()
    await loadTelegramConfig()
    await loadSessionMappings()
  }
  catch (error: any) {
    console.error('配置会话失败:', error)
    message.error(error || '配置会话失败')
  }
}

// 忽略待配置会话
async function ignoreSession(session: PendingSession) {
  try {
    await invoke('ignore_pending_session', {
      sessionId: session.session_id,
    })

    message.success('已忽略该会话')
    await loadPendingSessions()
  }
  catch (error: any) {
    console.error('忽略会话失败:', error)
    message.error(error || '忽略会话失败')
  }
}

// 格式化时间
function formatTime(isoString: string) {
  const date = new Date(isoString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// 组件挂载时加载配置
onMounted(() => {
  loadTelegramConfig()
  loadSessionMappings()
  loadPendingSessions()
})
</script>

<template>
  <!-- 设置内容 -->
  <n-space vertical size="large">
    <!-- 启用Telegram Bot -->
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0" />
        <div>
          <div class="text-sm font-medium leading-relaxed">
            启用Telegram机器人
          </div>
          <div class="text-xs opacity-60">
            启用后可以通过Telegram Bot接收通知消息
          </div>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <n-switch v-model:value="telegramConfig.enabled" size="small" @update:value="toggleTelegramEnabled" />
      </div>
    </div>

    <!-- 配置项区域 - 条件显示 -->
    <n-collapse-transition :show="telegramConfig.enabled">
      <n-space vertical size="large">
        <!-- Bot 列表 -->
        <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
          <div class="flex items-start">
            <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 mt-2 flex-shrink-0" />
            <div class="flex-1">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <div class="text-sm font-medium leading-relaxed">
                    Bot 配置列表
                  </div>
                  <div class="text-xs opacity-60">
                    管理多个 Telegram Bot，支持为不同对话使用不同的 Bot
                  </div>
                </div>
                <n-button size="small" type="primary" @click="openAddBotDialog">
                  ➕ 添加 Bot
                </n-button>
              </div>

              <!-- Bot 列表 -->
              <n-space v-if="telegramConfig.bots.length > 0" vertical size="small">
                <div
                  v-for="bot in telegramConfig.bots" :key="bot.name"
                  class="p-3 rounded border border-gray-200 dark:border-gray-700"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-2 mb-1">
                        <span class="text-sm font-medium">{{ bot.name }}</span>
                        <n-tag v-if="bot.name === telegramConfig.default_bot" size="small" type="success">
                          默认
                        </n-tag>
                      </div>
                      <div class="text-xs opacity-60">
                        Token: {{ bot.bot_token.substring(0, 20) }}... | Chat ID: {{ bot.chat_id }}
                      </div>
                    </div>
                    <n-space size="small">
                      <n-button
                        v-if="bot.name !== telegramConfig.default_bot"
                        size="tiny" @click="setDefaultBot(bot.name)"
                      >
                        设为默认
                      </n-button>
                      <n-button size="tiny" @click="testBotConnection(bot)">
                        测试
                      </n-button>
                      <n-button size="tiny" @click="openEditBotDialog(bot)">
                        编辑
                      </n-button>
                      <n-button size="tiny" type="error" @click="deleteBot(bot.name)">
                        删除
                      </n-button>
                    </n-space>
                  </div>
                </div>
              </n-space>
              <n-empty v-else description="暂无 Bot 配置，点击上方按钮添加" size="small" />
            </div>
          </div>
        </div>

        <!-- 待配置会话提示 -->
        <div v-if="pendingSessions.length > 0" class="pt-4 border-t border-gray-200 dark:border-gray-700">
          <div class="flex items-start">
            <div class="w-1.5 h-1.5 bg-warning rounded-full mr-3 mt-2 flex-shrink-0" />
            <div class="flex-1">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <div class="text-sm font-medium leading-relaxed">
                    🔔 发现新的工作目录
                  </div>
                  <div class="text-xs opacity-60">
                    检测到 {{ pendingSessions.length }} 个新的工作目录，建议为它们配置专属 Bot
                  </div>
                </div>
                <n-button type="warning" size="small" @click="openPendingSessionsDialog">
                  立即配置
                </n-button>
              </div>
            </div>
          </div>
        </div>

        <!-- 会话映射管理 -->
        <div v-if="telegramConfig.bots.length > 0" class="pt-4 border-t border-gray-200 dark:border-gray-700">
          <div class="flex items-start">
            <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 mt-2 flex-shrink-0" />
            <div class="flex-1">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <div class="text-sm font-medium leading-relaxed">
                    会话自动映射
                  </div>
                  <div class="text-xs opacity-60">
                    根据工作目录自动选择对应的 Bot，无需手动切换
                  </div>
                </div>
                <n-button size="small" @click="openSessionMappingDialog">
                  ⚙️ 管理映射
                </n-button>
              </div>

              <!-- 映射列表预览 -->
              <div v-if="Object.keys(sessionMappings).length > 0" class="text-xs opacity-60">
                已配置 {{ Object.keys(sessionMappings).length }} 个会话映射
              </div>
              <div v-else class="text-xs opacity-60">
                暂无会话映射，点击"管理映射"添加
              </div>
            </div>
          </div>
        </div>

        <!-- 隐藏前端弹窗设置 -->
        <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
          <div class="flex items-center justify-between">
            <div class="flex items-center">
              <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0" />
              <div>
                <div class="text-sm font-medium leading-relaxed">
                  隐藏前端弹窗
                </div>
                <div class="text-xs opacity-60">
                  启用后仅通过Telegram交互，不显示前端弹窗界面
                </div>
              </div>
            </div>
            <n-switch
              v-model:value="telegramConfig.hide_frontend_popup" size="small"
              @update:value="toggleHideFrontendPopup"
            />
          </div>
        </div>

      </n-space>
    </n-collapse-transition>
  </n-space>

  <!-- Bot 编辑对话框 -->
  <n-modal v-model:show="showBotDialog" preset="card" :title="editingBot ? '编辑 Bot' : '添加 Bot'" style="width: 600px; margin: 0 20px;">
    <n-space vertical size="large">
      <!-- Bot 名称 -->
      <div>
        <div class="text-sm font-medium mb-2">
          Bot 名称
        </div>
        <n-input
          v-model:value="botForm.name" type="text"
          placeholder="例如: 工作Bot、个人Bot" size="small"
        />
        <div class="text-xs opacity-60 mt-1">
          用于区分不同的 Bot，建议使用有意义的名称
        </div>
      </div>

      <!-- Bot Token -->
      <div>
        <div class="text-sm font-medium mb-2">
          Bot Token
        </div>
        <n-input
          v-model:value="botForm.bot_token" type="text"
          placeholder="例如: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz" size="small"
        />
        <div class="text-xs opacity-60 mt-1">
          从 @BotFather 获取的 Bot Token
        </div>
      </div>

      <!-- Chat ID -->
      <div>
        <div class="text-sm font-medium mb-2">
          Chat ID
        </div>
        <n-input
          v-model:value="botForm.chat_id" type="text"
          placeholder="例如: 123456789" size="small"
        />
        <n-button
          size="small" type="primary" :loading="isDetectingChatId"
          :disabled="!botForm.bot_token.trim()" @click="autoGetChatIdForBot"
          class="mt-2"
        >
          {{ isDetectingChatId ? '监听中...' : '自动获取 Chat ID' }}
        </n-button>
        <div v-if="detectedChatInfo" class="text-xs text-success-600 dark:text-success-400 mt-1">
          ✅ 已检测到: {{ detectedChatInfo.chat_id }}
        </div>
        <div class="text-xs opacity-60 mt-1">
          目标聊天的 ID，点击"自动获取"后向 Bot 发送消息即可
        </div>
      </div>

      <!-- API 基础 URL -->
      <div>
        <div class="text-sm font-medium mb-2">
          API 基础 URL
        </div>
        <n-input
          v-model:value="botForm.api_base_url" type="text"
          :placeholder="API_BASE_URL" size="small"
        />
        <div class="text-xs opacity-60 mt-1">
          Telegram API 地址，默认使用官方 API，也可配置代理
        </div>
      </div>
    </n-space>

    <template #footer>
      <n-space justify="end">
        <n-button @click="showBotDialog = false">
          取消
        </n-button>
        <n-button type="primary" @click="saveBotConfig">
          保存
        </n-button>
      </n-space>
    </template>
  </n-modal>

  <!-- 会话映射管理对话框 -->
  <n-modal v-model:show="showSessionMappingDialog" preset="card" title="会话自动映射管理" style="width: 700px; margin: 0 20px;">
    <n-space vertical size="large">
      <!-- 说明 -->
      <n-alert type="info" title="自动映射说明">
        <div class="text-sm space-y-2">
          <p>• 系统会根据当前工作目录自动选择对应的 Bot</p>
          <p>• 例如：在 <code>/Users/you/project-a</code> 目录下使用寸止时，会自动使用"项目A Bot"</p>
          <p>• 如果没有配置映射，则使用默认 Bot</p>
        </div>
      </n-alert>

      <!-- 映射列表 -->
      <div>
        <div class="text-sm font-medium mb-3">
          当前映射 ({{ Object.keys(sessionMappings).length }})
        </div>
        <n-space v-if="Object.keys(sessionMappings).length > 0" vertical size="small">
          <div
            v-for="(botName, sessionId) in sessionMappings" :key="sessionId"
            class="p-3 rounded border border-gray-200 dark:border-gray-700 flex items-center justify-between"
          >
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium truncate">
                {{ sessionId }}
              </div>
              <div class="text-xs opacity-60 mt-1">
                → {{ botName }}
              </div>
            </div>
            <n-button size="tiny" type="error" @click="removeSessionMapping(sessionId)">
              删除
            </n-button>
          </div>
        </n-space>
        <n-empty v-else description="暂无会话映射" size="small" />
      </div>

      <!-- 添加新映射 -->
      <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
        <div class="text-sm font-medium mb-3">
          添加新映射
        </div>
        <n-space vertical size="small">
          <n-input
            v-model:value="newSessionId" type="text"
            placeholder="会话 ID（例如：/Users/you/project-a）" size="small"
          />
          <n-select
            v-model:value="newSessionBotName"
            :options="telegramConfig.bots.map(bot => ({ label: bot.name, value: bot.name }))"
            placeholder="选择 Bot" size="small"
          />
          <n-button
            type="primary" size="small"
            :disabled="!newSessionId.trim() || !newSessionBotName"
            @click="addNewSessionMapping"
          >
            添加映射
          </n-button>
        </n-space>
      </div>
    </n-space>

    <template #footer>
      <n-space justify="end">
        <n-button @click="showSessionMappingDialog = false">
          关闭
        </n-button>
      </n-space>
    </template>
  </n-modal>

  <!-- 待配置会话对话框 -->
  <n-modal v-model:show="showPendingSessionsDialog" preset="card" title="待配置的工作目录" style="width: 800px; margin: 0 20px;">
    <n-space vertical size="large">
      <!-- 说明 -->
      <n-alert type="info" title="自动识别说明">
        <div class="text-sm space-y-2">
          <p>• 系统已自动识别到以下工作目录使用了寸止工具</p>
          <p>• 建议为每个目录配置专属的 Telegram Bot，实现消息隔离</p>
          <p>• 如果不需要单独配置，可以点击"忽略"使用默认 Bot</p>
        </div>
      </n-alert>

      <!-- 待配置会话列表 -->
      <div>
        <div class="text-sm font-medium mb-3">
          待配置目录 ({{ pendingSessions.length }})
        </div>
        <n-space v-if="pendingSessions.length > 0" vertical size="small">
          <div
            v-for="session in pendingSessions" :key="session.session_id"
            class="p-4 rounded border border-gray-200 dark:border-gray-700"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium truncate mb-2">
                  📁 {{ session.session_id }}
                </div>
                <div class="text-xs opacity-60 space-y-1">
                  <div>首次使用：{{ formatTime(session.first_seen) }}</div>
                  <div>最后使用：{{ formatTime(session.last_seen) }}</div>
                  <div>使用次数：{{ session.request_count }} 次</div>
                </div>
              </div>
              <n-space size="small">
                <n-button size="small" type="primary" @click="startConfigureSession(session)">
                  配置 Bot
                </n-button>
                <n-button size="small" @click="ignoreSession(session)">
                  忽略
                </n-button>
              </n-space>
            </div>
          </div>
        </n-space>
        <n-empty v-else description="暂无待配置的会话" size="small" />
      </div>
    </n-space>

    <template #footer>
      <n-space justify="end">
        <n-button @click="showPendingSessionsDialog = false">
          关闭
        </n-button>
      </n-space>
    </template>
  </n-modal>

  <!-- 配置会话 Bot 对话框 -->
  <n-modal v-model:show="showConfigureDialog" preset="card" title="配置专属 Bot" style="width: 700px; margin: 0 20px;">
    <n-space vertical size="large">
      <!-- 会话信息 -->
      <n-alert v-if="configuringSession" type="info">
        <div class="text-sm">
          <div class="font-medium mb-1">工作目录</div>
          <div class="opacity-80">{{ configuringSession.session_id }}</div>
        </div>
      </n-alert>

      <!-- 选择 Bot 方式 -->
      <n-radio-group v-model:value="configureForm.useExistingBot">
        <n-space>
          <n-radio :value="false">创建新 Bot</n-radio>
          <n-radio :value="true" :disabled="telegramConfig.bots.length === 0">
            使用已有 Bot
            <span v-if="telegramConfig.bots.length === 0" class="text-xs opacity-60">(暂无可用 Bot)</span>
          </n-radio>
        </n-space>
      </n-radio-group>

      <!-- 使用已有 Bot -->
      <template v-if="configureForm.useExistingBot">
        <n-alert v-if="telegramConfig.bots.length === 0" type="warning">
          暂无可用的 Bot，请先在"Bot 管理"中添加 Bot
        </n-alert>
        <div v-else>
          <div class="text-sm font-medium mb-3">选择 Bot</div>
          <n-radio-group v-model:value="configureForm.selectedBotName">
            <n-space vertical>
              <n-radio
                v-for="bot in telegramConfig.bots"
                :key="bot.name"
                :value="bot.name"
              >
                <div class="flex items-center">
                  <span class="font-medium">{{ bot.name }}</span>
                  <span v-if="bot.is_default" class="ml-2 text-xs opacity-60">(默认)</span>
                </div>
              </n-radio>
            </n-space>
          </n-radio-group>

          <!-- 显示选中的 Bot 信息 -->
          <n-alert v-if="configureForm.selectedBotName" type="info" class="mt-3">
            <div class="text-sm">
              <div class="font-medium mb-1">已选择 Bot: {{ configureForm.selectedBotName }}</div>
              <div class="opacity-80 text-xs">
                该会话的消息将发送到此 Bot
              </div>
            </div>
          </n-alert>
        </div>
      </template>

      <!-- 创建新 Bot -->
      <div v-else>
        <!-- 创建 Bot 指引 -->
        <n-alert type="success" title="📝 创建 Bot 步骤">
          <div class="text-sm space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <p class="font-medium mb-1">1. 打开 Telegram，找到 @BotFather</p>
                <p class="opacity-80">2. 发送 <code>/newbot</code> 命令</p>
                <p class="opacity-80">3. 按提示设置 Bot 名称和用户名</p>
                <p class="opacity-80">4. 复制获得的 Bot Token</p>
              </div>
              <n-button type="success" size="small" @click="openBotFather">
                打开 BotFather
              </n-button>
            </div>
          </div>
        </n-alert>

        <!-- Bot 配置表单 -->
        <n-form label-placement="left" label-width="100">
          <n-form-item label="Bot 名称">
            <n-input v-model:value="configureForm.botName" placeholder="例如：项目A Bot" />
          </n-form-item>

        <n-form-item label="Bot Token">
          <n-input-group>
            <n-input
              v-model:value="configureForm.botToken" type="password"
              show-password-on="click" placeholder="从 @BotFather 获取的 Token"
              style="flex: 1;"
            />
          </n-input-group>
        </n-form-item>

        <n-form-item label="Chat ID">
          <n-input-group>
            <n-input
              v-model:value="configureForm.chatId"
              placeholder="Telegram 聊天 ID" style="flex: 1;"
            />
            <n-button type="primary" @click="autoGetChatIdForSession">
              自动获取
            </n-button>
          </n-input-group>
          <template #feedback>
            <div class="text-xs opacity-60 mt-1">
              点击"自动获取"后，在 Telegram 中向 Bot 发送任意消息
            </div>
          </template>
        </n-form-item>

          <n-form-item label="API 基础 URL">
            <n-input v-model:value="configureForm.apiBaseUrl" placeholder="默认使用官方 API" />
          </n-form-item>
        </n-form>
      </div>
    </n-space>

    <template #footer>
      <n-space justify="end">
        <n-button @click="showConfigureDialog = false">
          取消
        </n-button>
        <n-button type="primary" @click="saveSessionConfiguration">
          保存配置
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>
