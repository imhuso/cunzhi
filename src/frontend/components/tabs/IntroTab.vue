<script setup lang="ts">
import FeatureCard from '../common/FeatureCard.vue'
import ProjectInfoCard from '../common/ProjectInfoCard.vue'

const features = [
  {
    icon: 'i-carbon-chat text-lg text-blue-600 dark:text-blue-400',
    title: 'Zhi 智能审查工具',
    subtitle: '寸止交互系统',
    iconBg: 'bg-blue-100',
    features: [
      '智能代码审查交互',
      '支持文本和图片输入',
      '预定义选项支持',
      'Markdown 渲染',
      '跨平台快捷键支持',
    ],
  },
  {
    icon: 'i-carbon-search text-lg text-green-600 dark:text-green-400',
    title: '代码搜索工具',
    subtitle: '智能代码检索',
    iconBg: 'bg-green-100',
    features: [
      '语义代码搜索',
      '增量索引支持',
      '多文件类型支持',
      '自动索引更新',
      '快速代码定位',
    ],
  },
  {
    icon: 'i-carbon-volume-up text-lg text-orange-600 dark:text-orange-400',
    title: '音频通知系统',
    subtitle: '智能音效管理',
    iconBg: 'bg-orange-100',
    features: [
      '多种内置音效',
      '自定义音频支持',
      '网络音频播放',
      '动态资源加载',
      '音效试听功能',
    ],
  },
  {
    icon: 'i-carbon-settings text-lg text-green-600 dark:text-green-400',
    title: '个性化设置',
    subtitle: '全面配置选项',
    iconBg: 'bg-green-100',
    features: [
      '深色/浅色主题',
      '窗口大小控制',
      '置顶显示设置',
      'MCP工具管理',
      '继续回复配置',
    ],
  },
]
</script>

<template>
  <div class="tab-content">
    <!-- 项目信息卡片 -->
    <div class="mb-5">
      <ProjectInfoCard />
    </div>

    <!-- 功能卡片 -->
    <n-grid :cols="2" :x-gap="20" :y-gap="20" responsive="screen">
      <n-grid-item
        v-for="(feature, index) in features"
        :key="index"
        :span="1"
      >
        <FeatureCard :feature="feature" />
      </n-grid-item>
    </n-grid>
  </div>
</template>
