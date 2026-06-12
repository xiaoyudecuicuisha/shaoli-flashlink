<template>
  <div class="app-shell no-select" :class="{ light: !isDark }">
    <!-- 标题栏 -->
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-brand">
        <svg class="brand-mark" viewBox="0 0 20 20" fill="none">
          <rect x="6" y="6" width="8" height="8" rx="1" transform="rotate(45 10 10)"
                stroke="var(--accent)" stroke-width="1.3"/>
          <circle cx="10" cy="10" r="1.5" fill="var(--gold)"/>
        </svg>
        <span class="titlebar-title">绍理闪连</span>
      </div>

      <div class="titlebar-status">
        <span class="status-dot" :class="isOnline ? 'online' : 'offline'"></span>
        <span class="status-text">{{ isOnline ? '已连接' : '未连接' }}</span>
      </div>

      <div class="titlebar-controls">
        <button class="titlebar-btn" @click="minimizeWindow" title="最小化">
          <svg class="icon-svg-sm" viewBox="0 0 14 14"><line x1="3" y1="7" x2="11" y2="7"/></svg>
        </button>
        <button class="titlebar-btn" @click="toggleMaximize" title="最大化">
          <svg class="icon-svg-sm" viewBox="0 0 14 14"><rect x="3" y="3" width="8" height="8" rx="1.5"/></svg>
        </button>
        <button class="titlebar-btn titlebar-btn--close" @click="closeWindow" title="关闭">
          <svg class="icon-svg-sm" viewBox="0 0 14 14"><line x1="4" y1="4" x2="10" y2="10"/><line x1="10" y1="4" x2="4" y2="10"/></svg>
        </button>
      </div>
    </header>

    <div class="app-body">
      <!-- 侧边栏 -->
      <nav class="sidebar">
        <div class="sidebar-nav">
          <div v-for="group in navGroups" :key="group.label" class="nav-group">
            <span class="nav-group-label">{{ group.label }}</span>
            <router-link
              v-for="item in group.items"
              :key="item.path"
              :to="item.path"
              class="nav-item"
              :class="{ active: isActive(item.path) }"
            >
              <div class="nav-indicator" v-if="isActive(item.path)"></div>
              <svg class="nav-icon" viewBox="0 0 24 24" v-html="item.icon"></svg>
              <span class="nav-label">{{ item.label }}</span>
            </router-link>
          </div>

          <div class="nav-divider"></div>

          <router-link to="/more" class="nav-item" :class="{ active: isActive('/more') }">
            <div class="nav-indicator" v-if="isActive('/more')"></div>
            <svg class="nav-icon" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
            </svg>
            <span class="nav-label">更多</span>
          </router-link>
        </div>

        <button class="theme-toggle" @click="toggleTheme" :title="isDark ? '切换浅色' : '切换深色'">
          <svg class="theme-icon" :class="{ rotated: !isDark }" viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="3.5" stroke="var(--gold)" stroke-width="1.3"/>
            <g stroke="var(--gold)" stroke-width="1.2" stroke-linecap="round">
              <line x1="10" y1="2" x2="10" y2="4"/>
              <line x1="10" y1="16" x2="10" y2="18"/>
              <line x1="2" y1="10" x2="4" y2="10"/>
              <line x1="16" y1="10" x2="18" y2="10"/>
              <line x1="4.34" y1="4.34" x2="5.76" y2="5.76"/>
              <line x1="14.24" y1="14.24" x2="15.66" y2="15.66"/>
              <line x1="4.34" y1="15.66" x2="5.76" y2="14.24"/>
              <line x1="14.24" y1="5.76" x2="15.66" y2="4.34"/>
            </g>
          </svg>
        </button>
      </nav>

      <!-- 内容区 -->
      <main class="content-area ambient-bg bg-texture">
        <router-view v-slot="{ Component }">
          <transition name="page-fade" mode="out-in">
            <component :is="Component" :key="$route.path" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
// 应用外壳：自定义标题栏、侧边栏导航、主题切换、环境氛围动画

import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import gsap from 'gsap'
import type { AppConfig, EventPayload, NetworkStatus } from '@/types'

const route = useRoute()
const router = useRouter()
const isDark = ref(true)
const isOnline = ref(false)

const navGroups = [
  {
    label: '校园网络',
    items: [
      {
        path: '/login',
        label: '登录',
        icon: '<path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/>'
      },
      {
        path: '/diagnosis',
        label: '诊断',
        icon: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>'
      },
      {
        path: '/services',
        label: '服务',
        icon: '<path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>'
      }
    ]
  },
  {
    label: '效率工具',
    items: [
      {
        path: '/whiteboard',
        label: '白板',
        icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M8 12h8M12 8v8"/>'
      },
      {
        path: '/cleaner',
        label: '清理',
        icon: '<path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/>'
      },
      {
        path: '/course',
        label: '网课',
        icon: '<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>'
      }
    ]
  },
  {
    label: '实验室',
    items: [
      {
        path: '/qzone',
        label: 'QQ空间',
        icon: '<circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/>'
      },
      {
        path: '/pet',
        label: '宠物',
        icon: '<circle cx="12" cy="5" r="3" fill="none"/><path d="M12 8c-4 0-7 2-7 5h14c0-3-3-5-7-5z" fill="none"/><path d="M5 13l1 8h12l1-8" fill="none"/>'
      }
    ]
  }
]

function isActive(path: string) {
  return route.path === path
}

function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('light', !isDark.value)
  invoke('save_config', { patch: { theme: isDark.value ? 'dark' : 'light' } }).catch(() => {})
}

async function minimizeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().minimize()
  } catch (e) {
    console.error('最小化失败:', e)
  }
}

async function toggleMaximize() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().toggleMaximize()
  } catch (e) {
    console.error('最大化切换失败:', e)
  }
}

async function closeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().hide()
  } catch (e) {
    console.error('关闭失败:', e)
  }
}

onMounted(async () => {
  try {
    const config: AppConfig = await invoke('load_config')
    if (config?.theme === 'light') {
      isDark.value = false
      document.documentElement.classList.add('light')
    }
  } catch {}

  try {
    await listen('network-status', (event: EventPayload<NetworkStatus>) => {
      isOnline.value = event.payload?.is_online ?? false
    })
    const status: NetworkStatus = await invoke('get_network_status')
    isOnline.value = status?.is_online ?? false
  } catch {}

  // 桌面宠物右键「打开设置」：拉起主窗口后跳到 /pet 页面
  try {
    await listen('navigate-to-pet', () => {
      if (route.path !== '/pet') {
        router.push('/pet')
      }
    })
  } catch {}

  // 背景渐变流动
  gsap.to(':root', {
    '--ambient-hue': 360,
    duration: 90,
    repeat: -1,
    ease: 'none'
  })
})
</script>

<style>
/* ── 页面切换（纯 CSS，无 GSAP 冲突） ── */
.page-fade-enter-active {
  transition: opacity 0.3s ease, transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.page-fade-leave-active {
  transition: opacity 0.12s ease, transform 0.12s ease;
}
.page-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}
.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}
</style>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
  color: var(--text-primary);
  overflow: hidden;
}

/* ── 标题栏 ── */
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 38px;
  padding: 0 14px;
  background: var(--titlebar-bg);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
}

.titlebar-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}

.brand-mark {
  width: 16px;
  height: 16px;
}

.titlebar-title {
  font-family: var(--font-display);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.04em;
}

.titlebar-status {
  display: flex;
  align-items: center;
  gap: 6px;
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  pointer-events: none;
}

.status-text {
  font-size: 0.65rem;
  color: var(--text-hint);
  font-weight: 400;
}

.titlebar-controls {
  display: flex;
  gap: 2px;
}

.titlebar-btn {
  width: 34px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--text-secondary);
  transition: all 0.15s ease;
}

.titlebar-btn:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.titlebar-btn--close:hover {
  background: var(--close-hover);
  color: white;
}

/* ── 主体 ── */
.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ── 侧边栏 ── */
.sidebar {
  width: 80px;
  background: var(--surface-1);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0 14px;
  flex-shrink: 0;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 10px;
  width: 62px;
  border-radius: var(--radius-md);
  color: var(--text-hint);
  text-decoration: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.nav-item:hover {
  color: var(--text-secondary);
  background: var(--surface-hover);
}

.nav-item.active {
  color: var(--accent);
  background: var(--accent-glow);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.nav-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 22px;
  background: var(--accent);
  border-radius: 0 2px 2px 0;
  animation: indicatorIn 0.35s cubic-bezier(0.34, 1.56, 0.64, 1) both;
}

@keyframes indicatorIn {
  from {
    opacity: 0;
    height: 0;
  }
  to {
    opacity: 1;
    height: 22px;
  }
}

.nav-icon {
  width: 22px;
  height: 22px;
  stroke: currentColor;
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
  fill: none;
}

.nav-label {
  font-size: 0.7rem;
  font-weight: 500;
  letter-spacing: 0.02em;
}

/* ── 分组导航 ── */
.nav-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-group-label {
  font-size: 0.6rem;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 8px 12px 4px;
  text-align: center;
}

.nav-divider {
  height: 1px;
  background: var(--divider);
  margin: 8px 12px;
}

/* ── 主题切换 ── */
.theme-toggle {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.25s ease;
}

.theme-toggle:hover {
  background: var(--surface-hover);
}

.theme-icon {
  width: 18px;
  height: 18px;
  transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.theme-icon.rotated {
  transform: rotate(180deg);
}

/* ── 内容区 ── */
.content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}
</style>
