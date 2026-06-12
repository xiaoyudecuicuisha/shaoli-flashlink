/**
 * pet-engine.js — 桌面宠物交互引擎
 *
 * 功能：拖拽、点击穿透、右键菜单、心情系统、跨模块联动、气泡消息
 * 运行环境：宠物 WebView 窗口（透明、置顶、无装饰）
 */
(function () {
  'use strict'

  const { invoke } = window.__TAURI__.core
  const { listen } = window.__TAURI__.event
  const { getCurrentWindow, LogicalSize } = window.__TAURI__.window

  // 窗口尺寸 = size + 120 像素（与后端 pet_open_window 保持一致）
  const WIN_PADDING = 120

  // ── DOM ──
  const petImg = document.getElementById('pet-img')
  const bubble = document.getElementById('bubble')
  const contextMenu = document.getElementById('context-menu')

  // ── 状态 ──
  let config = null
  let bubbleTimer = null
  let isDragging = false
  let mouseDownPos = null
  let currentMood = 'normal' // normal | happy | sad | busy | sleep
  let idleTimer = null
  let pollTimers = []
  const IDLE_TIMEOUT = 5 * 60 * 1000 // 5 分钟无操作 → 打瞌睡

  // ── 窗口引用 ──
  const win = getCurrentWindow()

  // ── 默认配置（完整结构） ──
  function getDefaultConfig() {
    return {
      enabled: true,
      current_pet: 'default',
      size: 180,
      custom_pets: [],
      reaction_network: true,
      reaction_course: true,
      reaction_qzone: true,
      reaction_convert: true,
      click_messages: ['喵~', '汪!', '嘿嘿', '(害羞)', '❤️', '✨', '你好呀~', '(开心)'],
      shake_message: '别摇啦~',
      mood_messages: { happy: ['开心~', '✨', '❤️', '嘿嘿'], sad: ['呜呜...', '(难过)', '网络断开了...'], busy: ['忙碌中...', '稍等~'], sleep: ['困了...', '💤', 'zzZ'] },
      click_animation: 'random',
      bubble_style: { duration: 3000, font_size: 12, bg_color: '#1e1e1e', text_color: '#ffffff' },
      menu_items: [
        { id: 'interact', label: '摸摸我', action: 'interact', icon: '' },
        { id: 'shake', label: '摇一摇', action: 'shake', icon: '' },
        { id: 'settings', label: '打开设置', action: 'settings', icon: '' },
        { id: 'close', label: '关闭宠物', action: 'close', icon: '' }
      ]
    }
  }
  // ── 初始化 ──
  async function init() {
    // 交互系统不依赖配置，必须最先设置
    setupInteraction()
    setupContextMenu()

    // 配置加载和联动系统可以失败而不影响基础交互
    try {
      config = await invoke('pet_get_config')
      // 补全缺失字段（旧配置兼容）
      const defaults = getDefaultConfig()
      for (const key of Object.keys(defaults)) {
        if (config[key] === undefined) config[key] = defaults[key]
      }
    } catch (e) {
      console.error('加载宠物配置失败:', e)
      config = getDefaultConfig()
    }

    try {
      await loadPetImage()
    } catch (e) {
      console.error('加载宠物图片失败:', e)
      petImg.src = 'assets/default.gif'
    }

    try {
      setupIntegration()
    } catch (e) {
      console.error('设置联动系统失败:', e)
    }

    resetIdleTimer()
  }

  // ── 加载宠物图片 ──
  async function loadPetImage() {
    if (!config) return

    if (config.current_pet === 'default') {
      petImg.src = 'assets/default.gif'
    } else {
      try {
        const dataUrl = await invoke('pet_read_file', { pet_id: config.current_pet })
        petImg.src = dataUrl
      } catch (e) {
        // 回退到默认
        petImg.src = 'assets/default.gif'
      }
    }
  }

  // ── 交互系统 ──
  function setupInteraction() {
    // 拖拽检测
    petImg.addEventListener('mousedown', (e) => {
      if (e.button === 2) return // 右键不拖拽
      mouseDownPos = { x: e.screenX, y: e.screenY }
      isDragging = false

      const onMove = (moveEvent) => {
        if (!mouseDownPos) return
        const dx = Math.abs(moveEvent.screenX - mouseDownPos.x)
        const dy = Math.abs(moveEvent.screenY - mouseDownPos.y)
        if (dx > 5 || dy > 5) {
          isDragging = true
          win.startDragging().catch(() => {})
          cleanup()
        }
      }

      const onUp = () => {
        if (!isDragging && mouseDownPos) {
          // 点击互动
          triggerInteraction()
        }
        cleanup()
      }

      function cleanup() {
        document.removeEventListener('mousemove', onMove)
        document.removeEventListener('mouseup', onUp)
        mouseDownPos = null
      }

      document.addEventListener('mousemove', onMove)
      document.addEventListener('mouseup', onUp)
    })

    // 点击穿透：使用后端轮询鼠标位置方案
    // 当鼠标在宠物图片区域内时，窗口不穿透（可交互）
    // 当鼠标不在宠物图片区域内时，窗口穿透（点击穿过到下面的窗口）
    startCursorTracking()
  }

  // ── 鼠标位置追踪（用于点击穿透） ──
  let cursorTrackingInterval = null

  async function startCursorTracking() {
    // 先等待窗口和图片加载完成
    await new Promise(r => setTimeout(r, 500))

    cursorTrackingInterval = setInterval(async () => {
      try {
        // 通过后端获取全局鼠标位置
        const [cursorX, cursorY] = await invoke('pet_get_cursor_pos')
        // 获取宠物窗口位置和大小
        const pos = await win.outerPosition()
        const size = await win.innerSize()
        const scaleFactor = await win.scaleFactor()

        // 计算宠物图片在窗口内的位置（居中显示）
        const petRect = petImg.getBoundingClientRect()

        // 将全局鼠标位置转换为窗口内坐标
        // GetCursorPos 返回物理像素，outerPosition 返回逻辑像素
        // 需要除以 scaleFactor 转换
        const relX = cursorX / scaleFactor - pos.x
        const relY = cursorY / scaleFactor - pos.y

        // 检查鼠标是否在宠物图片区域内
        const isOverPet = (
          relX >= petRect.left && relX <= petRect.right &&
          relY >= petRect.top && relY <= petRect.bottom
        )

        // 检查鼠标是否在气泡区域内
        const bubbleRect = bubble.getBoundingClientRect()
        const isOverBubble = (
          bubble.classList.contains('visible') &&
          relX >= bubbleRect.left && relX <= bubbleRect.right &&
          relY >= bubbleRect.top && relY <= bubbleRect.bottom
        )

        // 检查鼠标是否在右键菜单上
        const menuRect = contextMenu.getBoundingClientRect()
        const isOverMenu = (
          contextMenu.classList.contains('visible') &&
          relX >= menuRect.left && relX <= menuRect.right &&
          relY >= menuRect.top && relY <= menuRect.bottom
        )

        if (isOverPet || isOverBubble || isOverMenu) {
          // 鼠标在交互元素上 → 不穿透
          win.setIgnoreCursorEvents(false).catch(() => {})
        } else {
          // 鼠标在空白区域 → 穿透
          win.setIgnoreCursorEvents(true).catch(() => {})
        }
      } catch {}
    }, 120) // 120ms 轮询间隔
  }

  // ── 点击互动 ──
  function triggerInteraction() {
    const anims = ['bounce', 'shake', 'spin', 'flash']
    let anim = config?.click_animation || 'random'
    if (anim === 'random') {
      anim = anims[Math.floor(Math.random() * anims.length)]
    }

    petImg.classList.remove('bounce', 'shake', 'spin', 'flash')
    void petImg.offsetWidth
    petImg.classList.add(anim)

    setTimeout(() => {
      petImg.classList.remove(anim)
    }, 600)

    const messages = config?.click_messages?.length ? config.click_messages : ['喵~', '你好呀~']
    showBubble(messages[Math.floor(Math.random() * messages.length)])
    resetIdleTimer()
  }

  // ── 右键菜单 ──
  function renderMenu() {
    const items = config?.menu_items?.length ? config.menu_items : getDefaultConfig().menu_items
    contextMenu.innerHTML = ''
    for (let i = 0; i < items.length; i++) {
      const item = items[i]
      // 在 settings/close 前加分隔线（不在第一项前加）
      if (i > 0 && (item.action === 'settings' || item.action === 'close')) {
        const div = document.createElement('div')
        div.className = 'menu-divider'
        contextMenu.appendChild(div)
      }
      const el = document.createElement('div')
      el.className = 'menu-item'
      el.dataset.action = item.action
      el.textContent = (item.icon ? item.icon + ' ' : '') + item.label
      contextMenu.appendChild(el)
    }
  }

  function setupContextMenu() {
    renderMenu()

    petImg.addEventListener('contextmenu', (e) => {
      e.preventDefault()
      e.stopPropagation()
      showContextMenu(e.clientX, e.clientY)
    })

    document.addEventListener('click', (e) => {
      if (!contextMenu.contains(e.target)) {
        hideContextMenu()
      }
    })

    contextMenu.addEventListener('click', async (e) => {
      const item = e.target.closest('.menu-item')
      if (!item) return

      const action = item.dataset.action
      hideContextMenu()

      switch (action) {
        case 'interact':
          triggerInteraction()
          break
        case 'shake':
          petImg.classList.remove('shake')
          void petImg.offsetWidth
          petImg.classList.add('shake')
          setTimeout(() => petImg.classList.remove('shake'), 500)
          showBubble(config?.shake_message || '别摇啦~')
          break
        case 'settings':
          try {
            await invoke('pet_open_settings')
          } catch (e) {
            console.error('打开设置失败:', e)
          }
          break
        case 'close':
          try {
            await invoke('pet_close_window')
          } catch (e) {
            console.error('关闭宠物失败:', e)
            try { win.close() } catch (e2) {}
          }
          break
      }
      resetIdleTimer()
    })
  }

  function showContextMenu(x, y) {
    // 先显示菜单以获取其尺寸
    contextMenu.style.left = x + 'px'
    contextMenu.style.top = y + 'px'
    contextMenu.classList.add('visible')

    // 获取窗口和菜单尺寸，防止溢出
    requestAnimationFrame(() => {
      const menuRect = contextMenu.getBoundingClientRect()
      const winWidth = window.innerWidth
      const winHeight = window.innerHeight

      let left = x
      let top = y

      // 右侧溢出 → 向左偏移
      if (left + menuRect.width > winWidth) {
        left = Math.max(0, winWidth - menuRect.width)
      }
      // 底部溢出 → 向上偏移
      if (top + menuRect.height > winHeight) {
        top = Math.max(0, winHeight - menuRect.height)
      }

      contextMenu.style.left = left + 'px'
      contextMenu.style.top = top + 'px'
    })
  }

  function hideContextMenu() {
    contextMenu.classList.remove('visible')
  }

  // ── 气泡消息 ──
  function showBubble(text, duration) {
    const style = config?.bubble_style || {}
    duration = duration || style.duration || 3000
    bubble.textContent = text
    bubble.style.fontSize = (style.font_size || 12) + 'px'
    bubble.style.background = style.bg_color || '#1e1e1e'
    bubble.style.color = style.text_color || '#ffffff'
    bubble.classList.add('visible')

    if (bubbleTimer) clearTimeout(bubbleTimer)
    bubbleTimer = setTimeout(() => {
      bubble.classList.remove('visible')
    }, duration)
  }

  // ── 联动系统 ──
  function setupIntegration() {
    // 网络状态变化
    listen('network-lost', () => {
      if (!config?.reaction_network) return
      setMood('sad')
      showBubble(pickMoodMessage('sad') || '❓ 网络断开了...')
    })

    listen('network-restored', () => {
      if (!config?.reaction_network) return
      setMood('happy')
      showBubble(pickMoodMessage('happy') || '✅ 网络恢复啦!')
      setTimeout(() => setMood('normal'), 5000)
    })

    listen('network-status', (e) => {
      if (!config?.reaction_network) return
      if (!e.payload.is_online && currentMood !== 'sad') {
        setMood('sad')
      }
    })

    // 网课进度
    listen('course-progress', (e) => {
      if (!config?.reaction_course) return
      const p = e.payload
      if (p.videos_completed > 0) {
        showBubble(`📚 已完成 ${p.videos_completed} 个视频`)
        triggerBounce()
      }
      if (p.quizzes_answered > 0) {
        showBubble(`✏️ 答对 ${p.quizzes_answered} 题`)
      }
    })

    // Qzone 获取进度（轮询）
    startPolling('qzone', async () => {
      if (!config?.reaction_qzone) return
      try {
        const p = await invoke('qzone_get_progress')
        if (p.is_running) {
          setMood('busy')
          showBubble(pickMoodMessage('busy') || `📥 获取中 ${p.current}/${p.total}`)
        } else if (p.status === '获取完成' && currentMood === 'busy') {
          setMood('happy')
          showBubble(pickMoodMessage('happy') || `🎉 获取完成! ${p.moments_count} 条动态`)
          setTimeout(() => setMood('normal'), 5000)
        }
      } catch {}
    }, 5000)

    // 文档转换（轮询）
    startPolling('convert', async () => {
      if (!config?.reaction_convert) return
      try {
        const tasks = await invoke('get_all_convert_status')
        const running = tasks?.filter(t => t.status === 'Running' || t.status === 'Pending')
        if (running && running.length > 0) {
          setMood('busy')
          showBubble(pickMoodMessage('busy') || `⏳ 转换中... (${running.length})`)
        }
      } catch {}
    }, 3000)
  }

  function startPolling(key, fn, interval) {
    const timer = setInterval(fn, interval)
    pollTimers.push(timer)
  }

  // ── 心情系统 ──
  function pickMoodMessage(mood) {
    const pool = config?.mood_messages?.[mood]
    if (pool && pool.length > 0) return pool[Math.floor(Math.random() * pool.length)]
    return null
  }

  function setMood(mood) {
    currentMood = mood
    // 通过 CSS filter 暗示心情
    switch (mood) {
      case 'happy':
        petImg.style.filter = 'saturate(1.3) brightness(1.1)'
        break
      case 'sad':
        petImg.style.filter = 'saturate(0.5) brightness(0.8)'
        break
      case 'busy':
        petImg.style.filter = 'hue-rotate(20deg)'
        break
      case 'sleep':
        petImg.style.filter = 'brightness(0.6) saturate(0.7)'
        break
      default:
        petImg.style.filter = 'none'
    }
  }

  // ── 闲置检测 ──
  function resetIdleTimer() {
    if (idleTimer) clearTimeout(idleTimer)
    if (currentMood === 'sleep') {
      setMood('normal')
    }
    idleTimer = setTimeout(() => {
      setMood('sleep')
      showBubble(pickMoodMessage('sleep') || '💤 困了...', 5000)
    }, IDLE_TIMEOUT)
  }

  function triggerBounce() {
    petImg.classList.remove('bounce')
    void petImg.offsetWidth
    petImg.classList.add('bounce')
    setTimeout(() => petImg.classList.remove('bounce'), 500)
  }

  // ── 监听配置更新（从 PetView 发来） ──
  listen('pet-config-updated', async () => {
    try {
      const oldSize = config?.size
      const oldPet = config?.current_pet
      config = await invoke('pet_get_config')
      if (!config) return

      // 补全缺失字段（与 init 一致）
      const defaults = getDefaultConfig()
      for (const key of Object.keys(defaults)) {
        if (config[key] === undefined) config[key] = defaults[key]
      }

      // 切换了宠物 → 重新加载图片
      if (config.current_pet !== oldPet) {
        await loadPetImage()
      }

      // 菜单项可能已变更 → 重新渲染
      renderMenu()

      // 尺寸发生变化 → 调整悬浮窗大小
      if (typeof config.size === 'number' && config.size !== oldSize) {
        const winSize = config.size + WIN_PADDING
        await win.setSize(new LogicalSize(winSize, winSize))
      }
    } catch (e) {
      console.error('应用配置更新失败:', e)
    }
  })

  // ── 全局鼠标移动重置闲置 ──
  document.addEventListener('mousemove', () => {
    resetIdleTimer()
  })

  // ── 启动 ──
  init()
})()
