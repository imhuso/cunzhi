<script setup lang="ts">
import type { CustomPrompt, CustomPromptConfig } from '../../types/popup'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'

const message = useMessage()

// 配置状态
const config = ref<CustomPromptConfig>({
  prompts: [],
  enabled: true,
  maxPrompts: 50,
})

// UI状态
const loading = ref(false)
const showAddDialog = ref(false)
const showEditDialog = ref(false)
const showDeleteDialog = ref(false)
const editingPrompt = ref<CustomPrompt | null>(null)
const deletingPromptId = ref<string>('')

// 新prompt表单
const newPrompt = ref({
  name: '',
  content: '',
  description: '',
  type: 'normal' as 'normal' | 'conditional',
  condition_text: '',
  template_true: '',
  template_false: '',
  current_state: false,
})

// 加载配置
async function loadConfig() {
  try {
    loading.value = true
    const result = await invoke('get_custom_prompt_config')
    config.value = result as CustomPromptConfig
    // 按sort_order排序
    config.value.prompts.sort((a, b) => a.sort_order - b.sort_order)
  }
  catch (error) {
    console.error('加载自定义prompt配置失败:', error)
    message.error('加载配置失败')
  }
  finally {
    loading.value = false
  }
}

// 切换启用状态
async function toggleEnabled() {
  try {
    await invoke('set_custom_prompt_enabled', { enabled: config.value.enabled })

    // 发送事件通知其他组件更新
    await emit('custom-prompt-updated')

    message.success(config.value.enabled ? '已启用快捷模板功能' : '已禁用快捷模板功能')
  }
  catch (error) {
    console.error('更新启用状态失败:', error)
    message.error('更新失败')
    // 回滚状态
    config.value.enabled = !config.value.enabled
  }
}

// 添加prompt
async function addPrompt() {
  if (!newPrompt.value.name.trim()) {
    message.warning('请填写名称')
    return
  }

  // 上下文追加的验证
  if (newPrompt.value.type === 'conditional') {
    if (!newPrompt.value.condition_text.trim()) {
      message.warning('请填写条件描述')
      return
    }
    if (!newPrompt.value.template_true.trim() && !newPrompt.value.template_false.trim()) {
      message.warning('请至少填写一个模板内容')
      return
    }
  }

  try {
    const prompt: CustomPrompt = {
      id: `custom_${Date.now()}`,
      name: newPrompt.value.name.trim(),
      content: newPrompt.value.content, // 允许空内容
      description: newPrompt.value.description.trim() || undefined,
      sort_order: config.value.prompts.length + 1,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      type: newPrompt.value.type,
      condition_text: newPrompt.value.type === 'conditional' ? newPrompt.value.condition_text.trim() || undefined : undefined,
      template_true: newPrompt.value.type === 'conditional' ? newPrompt.value.template_true.trim() || undefined : undefined,
      template_false: newPrompt.value.type === 'conditional' ? newPrompt.value.template_false.trim() || undefined : undefined,
      current_state: newPrompt.value.type === 'conditional' ? newPrompt.value.current_state : undefined,
    }

    await invoke('add_custom_prompt', { prompt })
    config.value.prompts.push(prompt)

    // 发送事件通知其他组件更新
    await emit('custom-prompt-updated')

    // 重置表单
    newPrompt.value = {
      name: '',
      content: '',
      description: '',
      type: 'normal',
      condition_text: '',
      template_true: '',
      template_false: '',
      current_state: false,
    }
    showAddDialog.value = false
    message.success('添加成功')
  }
  catch (error) {
    console.error('添加prompt失败:', error)
    message.error(`添加失败: ${error}`)
  }
}

// 编辑prompt
function editPrompt(prompt: CustomPrompt) {
  editingPrompt.value = { ...prompt }
  showEditDialog.value = true
}

// 更新prompt
async function updatePrompt() {
  if (!editingPrompt.value)
    return

  // 上下文追加的验证
  if (editingPrompt.value.type === 'conditional') {
    if (!editingPrompt.value.condition_text?.trim()) {
      message.warning('请填写条件描述')
      return
    }
    if (!editingPrompt.value.template_true?.trim() && !editingPrompt.value.template_false?.trim()) {
      message.warning('请至少填写一个模板内容')
      return
    }
  }

  try {
    editingPrompt.value.updated_at = new Date().toISOString()
    await invoke('update_custom_prompt', { prompt: editingPrompt.value })

    // 更新本地状态
    const index = config.value.prompts.findIndex(p => p.id === editingPrompt.value!.id)
    if (index !== -1) {
      config.value.prompts[index] = { ...editingPrompt.value }
    }

    // 发送事件通知其他组件更新
    await emit('custom-prompt-updated')

    showEditDialog.value = false
    editingPrompt.value = null
    message.success('更新成功')
  }
  catch (error) {
    console.error('更新prompt失败:', error)
    message.error(`更新失败: ${error}`)
  }
}

// 显示删除确认对话框
function showDeleteConfirm(promptId: string) {
  deletingPromptId.value = promptId
  showDeleteDialog.value = true
}

// 删除prompt
async function deletePrompt() {
  if (!deletingPromptId.value)
    return

  try {
    await invoke('delete_custom_prompt', { promptId: deletingPromptId.value })
    config.value.prompts = config.value.prompts.filter(p => p.id !== deletingPromptId.value)

    // 发送事件通知其他组件更新
    await emit('custom-prompt-updated')

    message.success('删除成功')
  }
  catch (error) {
    console.error('删除prompt失败:', error)
    message.error(`删除失败: ${error}`)
  }
  finally {
    showDeleteDialog.value = false
    deletingPromptId.value = ''
  }
}

// 取消编辑
function cancelEdit() {
  showEditDialog.value = false
  editingPrompt.value = null
}

// 组件挂载时加载配置
onMounted(() => {
  loadConfig()
})
</script>

<template>
  <div class="p-4">
    <!-- 启用开关 -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <div class="text-sm opacity-60">
          是否开启快捷模板功能
        </div>
      </div>
      <n-switch
        v-model:value="config.enabled"
        @update:value="toggleEnabled"
      />
    </div>

    <div v-if="config.enabled" data-guide="custom-prompt-settings">
      <!-- 添加按钮 -->
      <div class="flex justify-between items-center mb-4">
        <div class="text-sm opacity-60">
          已创建 {{ config.prompts.length }} 个模板
        </div>
        <n-button
          type="primary"
          size="small"
          :disabled="config.prompts.length >= config.maxPrompts"
          data-guide="add-prompt-button"
          @click="showAddDialog = true"
        >
          <template #icon>
            <div class="i-carbon-add w-4 h-4" />
          </template>
          添加模板
        </n-button>
      </div>

      <!-- Prompt列表 -->
      <div v-if="loading" class="text-center py-8">
        <n-spin size="medium" />
      </div>

      <div v-else-if="config.prompts.length === 0" class="text-center py-8 opacity-60">
        <div class="i-carbon-document-blank text-4xl mb-2" />
        <div>暂无快捷模板</div>
      </div>

      <div v-else class="space-y-3">
        <div class="space-y-3">
          <div
            v-for="prompt in config.prompts"
            :key="prompt.id"
            class="bg-black-50 rounded-lg p-4 border border-black-200 shadow-sm hover:border-black-300 transition-colors"
          >
            <div class="flex justify-between items-start mb-2">
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                  <span class="font-medium text-white">{{ prompt.name }}</span>
                  <!-- 类型标识 -->
                  <n-tag v-if="prompt.type === 'conditional'" size="small" type="info">
                    上下文追加
                  </n-tag>
                  <n-tag v-else size="small" type="default">
                    快捷模板
                  </n-tag>
                </div>
                <div v-if="prompt.description" class="text-sm opacity-60 mb-2">
                  {{ prompt.description }}
                </div>

                <!-- 快捷模板内容显示 -->
                <div v-if="prompt.type !== 'conditional'" class="text-sm bg-black-100 p-2 rounded border border-black-200">
                  <span v-if="prompt.content.trim()">{{ prompt.content }}</span>
                  <span v-else class="italic opacity-60">（空内容 - 清空输入框）</span>
                </div>

                <!-- 上下文追加内容显示 -->
                <div v-else class="space-y-2">
                  <div class="text-sm bg-black-100 p-2 rounded border border-black-200">
                    <div class="font-medium mb-1">
                      条件：{{ prompt.condition_text }}
                    </div>
                    <div class="space-y-1 text-xs">
                      <div v-if="prompt.template_true">
                        <span class="text-green-400">✓ 开启：</span>{{ prompt.template_true }}
                      </div>
                      <div v-if="prompt.template_false">
                        <span class="text-red-400">✗ 关闭：</span>{{ prompt.template_false }}
                      </div>
                      <div class="text-gray-700 dark:text-white">
                        当前状态：{{ prompt.current_state ? '开启' : '关闭' }}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <div class="flex gap-1 ml-4">
                <n-button size="small" quaternary @click="editPrompt(prompt)">
                  <template #icon>
                    <div class="i-carbon-edit w-4 h-4" />
                  </template>
                </n-button>
                <n-button size="small" quaternary type="error" @click="showDeleteConfirm(prompt.id)">
                  <template #icon>
                    <div class="i-carbon-trash-can w-4 h-4" />
                  </template>
                </n-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加对话框 -->
    <n-modal v-model:show="showAddDialog" preset="card" title="添加快捷模板" style="width: 600px">
      <n-form :model="newPrompt" label-placement="top">
        <n-form-item label="名称" required>
          <n-input v-model:value="newPrompt.name" placeholder="输入模板名称" />
        </n-form-item>
        <n-form-item label="描述">
          <n-input v-model:value="newPrompt.description" placeholder="简短描述这个模板的用途" />
        </n-form-item>

        <!-- 模板类型选择 -->
        <n-form-item label="类型">
          <n-radio-group v-model:value="newPrompt.type">
            <n-radio value="normal">
              快捷模板
            </n-radio>
            <n-radio value="conditional">
              上下文追加
            </n-radio>
          </n-radio-group>
        </n-form-item>

        <!-- 快捷模板内容 -->
        <n-form-item v-if="newPrompt.type === 'normal'" label="内容">
          <n-input
            v-model:value="newPrompt.content"
            type="textarea"
            placeholder="输入模板内容（留空可实现清空输入框效果）"
            :autosize="{ minRows: 4, maxRows: 8 }"
          />
        </n-form-item>

        <!-- 上下文追加字段 -->
        <template v-if="newPrompt.type === 'conditional'">
          <n-form-item label="条件描述" required>
            <n-input v-model:value="newPrompt.condition_text" placeholder="例如：是否使用TypeScript" />
          </n-form-item>
          <n-form-item label="开启时的内容">
            <n-input
              v-model:value="newPrompt.template_true"
              type="textarea"
              placeholder="例如：✔️需要使用TypeScript"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
          </n-form-item>
          <n-form-item label="关闭时的内容">
            <n-input
              v-model:value="newPrompt.template_false"
              type="textarea"
              placeholder="例如：❌切记，不要使用TypeScript"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
          </n-form-item>
          <n-form-item label="当前状态">
            <n-switch v-model:value="newPrompt.current_state">
              <template #checked>
                开启
              </template>
              <template #unchecked>
                关闭
              </template>
            </n-switch>
          </n-form-item>
        </template>
      </n-form>
      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button @click="showAddDialog = false">
            取消
          </n-button>
          <n-button type="primary" @click="addPrompt">
            添加
          </n-button>
        </div>
      </template>
    </n-modal>

    <!-- 编辑对话框 -->
    <n-modal v-model:show="showEditDialog" preset="card" title="编辑快捷模板" style="width: 600px">
      <n-form v-if="editingPrompt" :model="editingPrompt" label-placement="top">
        <n-form-item label="名称" required>
          <n-input v-model:value="editingPrompt.name" placeholder="输入模板名称" />
        </n-form-item>
        <n-form-item label="描述">
          <n-input v-model:value="editingPrompt.description" placeholder="简短描述这个模板的用途" />
        </n-form-item>

        <!-- 模板类型选择 -->
        <n-form-item label="类型">
          <n-radio-group v-model:value="editingPrompt.type">
            <n-radio value="normal">
              快捷模板
            </n-radio>
            <n-radio value="conditional">
              上下文追加
            </n-radio>
          </n-radio-group>
        </n-form-item>

        <!-- 快捷模板内容 -->
        <n-form-item v-if="editingPrompt.type === 'normal' || !editingPrompt.type" label="内容">
          <n-input
            v-model:value="editingPrompt.content"
            type="textarea"
            placeholder="输入模板内容（留空可实现清空输入框效果）"
            :autosize="{ minRows: 4, maxRows: 8 }"
          />
        </n-form-item>

        <!-- 上下文追加字段 -->
        <template v-if="editingPrompt.type === 'conditional'">
          <n-form-item label="条件描述" required>
            <n-input v-model:value="editingPrompt.condition_text" placeholder="例如：是否使用TypeScript" />
          </n-form-item>
          <n-form-item label="开启时的内容">
            <n-input
              v-model:value="editingPrompt.template_true"
              type="textarea"
              placeholder="例如：✔️需要使用TypeScript"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
          </n-form-item>
          <n-form-item label="关闭时的内容">
            <n-input
              v-model:value="editingPrompt.template_false"
              type="textarea"
              placeholder="例如：❌切记，不要使用TypeScript"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
          </n-form-item>
          <n-form-item label="当前状态">
            <n-switch v-model:value="editingPrompt.current_state">
              <template #checked>
                开启
              </template>
              <template #unchecked>
                关闭
              </template>
            </n-switch>
          </n-form-item>
        </template>
      </n-form>
      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button @click="cancelEdit">
            取消
          </n-button>
          <n-button type="primary" @click="updatePrompt">
            保存
          </n-button>
        </div>
      </template>
    </n-modal>

    <!-- 删除确认对话框 -->
    <n-modal v-model:show="showDeleteDialog" preset="dialog" title="确认删除">
      <div>确定要删除这个模板吗？此操作无法撤销。</div>
      <template #action>
        <div class="flex justify-end gap-2">
          <n-button @click="showDeleteDialog = false">
            取消
          </n-button>
          <n-button type="error" @click="deletePrompt">
            确定删除
          </n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
/* 拖拽排序样式 */
.sortable-item {
  cursor: default;
  transition: all 0.2s ease;
}

.sortable-ghost {
  opacity: 0.5;
  transform: scale(0.95);
  background: rgba(59, 130, 246, 0.1) !important;
  border: 2px dashed rgba(59, 130, 246, 0.3) !important;
}

.sortable-chosen {
  cursor: grabbing !important;
  transform: scale(1.02);
}

.sortable-drag {
  opacity: 0.8;
  transform: rotate(2deg);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
  z-index: 1000;
}
</style>
