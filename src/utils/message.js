// 自定义消息提示系统
let messageContainer = null

// 创建消息容器
function createMessageContainer() {
  if (messageContainer)
    return messageContainer

  messageContainer = document.createElement('div')
  messageContainer.className = 'fixed top-4 right-4 z-[9999] flex flex-col gap-2'
  document.body.appendChild(messageContainer)
  return messageContainer
}

// 创建单个消息
function createMessage(content, type = 'info', duration = 3000) {
  const container = createMessageContainer()

  const messageEl = document.createElement('div')
  messageEl.className = `
    px-4 py-2 rounded-lg shadow-lg text-sm font-medium
    transform translate-x-full opacity-0 transition-all duration-300
    flex items-center gap-2 max-w-sm
    ${getTypeStyles(type)}
  `

  // 添加图标和内容
  const icon = getIcon(type)
  messageEl.innerHTML = `
    <span class="flex-shrink-0">${icon}</span>
    <span class="flex-1">${content}</span>
  `

  container.appendChild(messageEl)

  // 触发进入动画
  requestAnimationFrame(() => {
    messageEl.style.transform = 'translateX(0)'
    messageEl.style.opacity = '1'
  })

  // 自动移除
  setTimeout(() => {
    messageEl.style.transform = 'translateX(full)'
    messageEl.style.opacity = '0'
    setTimeout(() => {
      if (messageEl.parentNode) {
        messageEl.parentNode.removeChild(messageEl)
      }
    }, 300)
  }, duration)
}

function getTypeStyles(type) {
  switch (type) {
    case 'success':
      return 'bg-green-500 text-white'
    case 'error':
      return 'bg-red-500 text-white'
    case 'warning':
      return 'bg-yellow-500 text-white'
    default:
      return 'bg-blue-500 text-white'
  }
}

function getIcon(type) {
  switch (type) {
    case 'success':
      return '✅'
    case 'error':
      return '❌'
    case 'warning':
      return '⚠️'
    default:
      return 'ℹ️'
  }
}

// 导出消息API
export const message = {
  success: (content, duration) => createMessage(content, 'success', duration),
  error: (content, duration) => createMessage(content, 'error', duration),
  warning: (content, duration) => createMessage(content, 'warning', duration),
  info: (content, duration) => createMessage(content, 'info', duration),
}
