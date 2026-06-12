<template>
  <div class="more-view page-enter">
    <div class="page-header anim-fade-left stagger-1">
      <span class="page-roman">IX.</span>
      <h1 class="page-header-title">更多 <PageHelp>
        <p><strong>设置与关于</strong></p>
        <ul>
          <li>开机自启：开启后每次电脑开机自动连接校园网</li>
          <li>桌面宠物：快速开关桌面宠物功能</li>
          <li>手动登录：直接打开校园网认证网页（备用方案）</li>
          <li>获取新版 / 检查更新：下载最新版本</li>
          <li>导出日志：打包日志文件用于反馈问题</li>
          <li>反馈建议：加入 QQ 群与作者交流</li>
        </ul>
        <p>本软件纯本地运行，不收集任何用户数据。</p>
      </PageHelp></h1>
      <p class="page-header-sub">M O R E</p>
    </div>

    <div class="more-scroll">
      <!-- 设置 -->
      <BezelCard class="more-card anim-fade-up stagger-2">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--accent);">
              <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>
            <span class="card-title">设置</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">开机自启</span>
            <span class="setting-desc">开机自动连接网络</span>
          </div>
          <button 
            class="toggle-switch"
            :class="{ active: autostart }"
            @click="toggleAutostart"
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">桌面宠物</span>
            <span class="setting-desc">在桌面显示一只可爱的宠物伙伴</span>
          </div>
          <button
            class="toggle-switch"
            :class="{ active: petEnabled }"
            @click="togglePet"
          >
            <span class="toggle-knob"></span>
          </button>
        </div>
      </BezelCard>

      <!-- 快捷入口 -->
      <BezelCard class="more-card anim-fade-up stagger-3">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--gold);">
              <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
            </svg>
            <span class="card-title">快捷入口</span>
          </div>
        </div>

        <div class="link-list">
          <div
            v-for="(item, i) in linkItems"
            :key="item.title"
            class="link-item"
            :class="{ 'anim-fade-up': true, 'stagger-3': i === 0, 'stagger-4': i === 1, 'stagger-5': i === 2, 'stagger-6': i === 3 }"
            @click="item.action"
          >
            <div class="link-icon">{{ item.icon }}</div>
            <div class="link-content">
              <span class="link-title">{{ item.title }}</span>
              <span class="link-desc">{{ item.desc }}</span>
            </div>
            <svg class="link-arrow" viewBox="0 0 24 24" fill="none">
              <path d="M9 18l6-6-6-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
        </div>
      </BezelCard>

      <!-- 关于 -->
      <BezelCard class="more-card anim-fade-up stagger-5" hoverable>
        <div class="about-section">
          <div class="about-brand">
            <svg class="about-logo" viewBox="0 0 20 20" fill="none">
              <rect x="6" y="6" width="8" height="8" rx="1" transform="rotate(45 10 10)"
                    stroke="var(--accent)" stroke-width="1.3"/>
              <circle cx="10" cy="10" r="1.5" fill="var(--gold)"/>
            </svg>
            <div>
              <span class="about-name">绍理闪连</span>
              <span class="about-ver">V3.0.0</span>
            </div>
          </div>
          <p class="about-copy">校园网络与效率工具</p>
          <p class="about-author">by 回</p>
          <p class="about-build">构建时间: {{ buildTime }}</p>
          <div class="about-links">
            <button class="about-link" @click="showDonate = true">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              请作者喝杯咖啡
            </button>
            <button class="about-link" @click="openUrl(GITHUB_URL)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
              </svg>
              GitHub
            </button>
          </div>
        </div>
      </BezelCard>

      <!-- 免责声明 -->
      <div class="disclaimer anim-fade-up stagger-6">
        <p class="disclaimer-text">
          本软件仅供绍兴理工学院学生内部交流与学习使用，严禁用于任何商业或非法用途。
          因使用本软件导致的账号封禁、数据丢失，作者概不负责。
          本软件纯本地运行，不收集、不上传任何用户隐私数据。
        </p>
      </div>
    </div>

    <!-- 赞赏码弹窗 -->
    <transition name="modal">
      <div v-if="showDonate" class="modal-overlay" @click.self="showDonate = false">
        <BezelCard class="modal-card donate-modal">
          <div class="modal-header">
            <span class="modal-title">请作者喝杯咖啡</span>
            <button class="modal-close" @click="showDonate = false">
              <svg viewBox="0 0 14 14" fill="none">
                <line x1="4" y1="4" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <line x1="10" y1="4" x2="4" y2="10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
          </div>
          <div class="donate-body">
            <div class="donate-item">
              <span class="donate-label">微信支付</span>
              <img src="/donate-wechat.jpg" alt="微信收款码" class="donate-qr" />
            </div>
            <div class="donate-item">
              <span class="donate-label">支付宝</span>
              <img src="/donate-alipay.jpg" alt="支付宝收款码" class="donate-qr" />
            </div>
            <p class="donate-thanks">感谢你的支持！</p>
          </div>
        </BezelCard>
      </div>
    </transition>

    <!-- Toast 提示 -->
    <transition name="slide">
      <div v-if="toastMsg" class="toast-bar" :class="toastType">
        <svg v-if="toastType === 'success'" width="14" height="14" viewBox="0 0 24 24" fill="none" style="flex-shrink:0;">
          <path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <svg v-else-if="toastType === 'error'" width="14" height="14" viewBox="0 0 24 24" fill="none" style="flex-shrink:0;">
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <span>{{ toastMsg }}</span>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
// 更多页：开机自启、宠物开关、检查更新、日志导出、赞赏、关于信息

import { ref, onMounted, onActivated, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { open as shellOpen } from '@tauri-apps/plugin-shell'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import BezelCard from '@/components/BezelCard.vue'
import PageHelp from '@/components/PageHelp.vue'

const showDonate = ref(false)
const autostart = ref(false)
const petEnabled = ref(false)
const toastMsg = ref('')
const toastType = ref<'success' | 'error' | 'info'>('info')
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string, type: 'success' | 'error' | 'info' = 'error') {
  toastMsg.value = msg
  toastType.value = type
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
}

const QQ_GROUP_URL = 'https://qm.qq.com/q/o21uCujbMG'
const DOWNLOAD_URL = 'https://wwbvh.lanzoum.com/b01bjfd91g'
const GITHUB_URL = 'https://github.com/xiaoyudecuicuisha/shaoli-flashlink'
const AUTH_URL = 'http://10.210.0.2/srun_portal_pc?ac_id=1&theme=pro'
const buildTime = ref('')

async function openUrl(url: string) {
  try {
    await shellOpen(url)
  } catch {
    try {
      await invoke('open_url', { url })
    } catch {
      showToast('无法打开链接: ' + url)
    }
  }
}

async function toggleAutostart() {
  try {
    if (autostart.value) {
      await invoke('remove_autostart')
    } else {
      await invoke('setup_autostart')
    }
    autostart.value = !autostart.value
    await invoke('save_config', { patch: { autostart: autostart.value } })
  } catch (e) {
    showToast('设置开机自启失败: ' + String(e))
  }
}

async function togglePet() {
  try {
    if (petEnabled.value) {
      await invoke('pet_close_window')
    } else {
      await invoke('pet_open_window')
    }
    petEnabled.value = !petEnabled.value
    // 只更新 enabled 状态，不覆盖其他宠物配置
    const currentConfig = await invoke<any>('pet_get_config')
    currentConfig.enabled = petEnabled.value
    await invoke('pet_save_config', { patch: currentConfig })
    await invoke('save_config', { patch: { pet_enabled: petEnabled.value } })
    showToast(petEnabled.value ? '宠物已开启' : '宠物已关闭', 'success')
  } catch (e) {
    showToast('操作失败: ' + String(e))
  }
}

async function exportLogs() {
  try {
    const savePath = await save({
      defaultPath: 'shaoli-logs.zip',
      filters: [{ name: 'ZIP', extensions: ['zip'] }]
    })
    if (savePath) {
      await invoke('export_logs', { targetPath: savePath })
      showToast('日志已导出', 'success')
    }
  } catch (e) {
    showToast('导出日志失败: ' + String(e))
  }
}

async function checkForUpdates() {
  try {
    showToast('正在检查更新...', 'info')
    const msg = await invoke<string>('check_for_updates')
    showToast(msg, 'success')
  } catch (e) {
    showToast('检查更新失败: ' + String(e), 'error')
  }
}

async function loadPetState() {
  try {
    const config: any = await invoke('load_config')
    petEnabled.value = config?.pet_enabled ?? false
  } catch {}
}

let petUnlisten: UnlistenFn | null = null

onMounted(async () => {
  try {
    const config: any = await invoke('load_config')
    autostart.value = config?.autostart ?? false
    petEnabled.value = config?.pet_enabled ?? false
  } catch (e) {
    console.warn('加载配置失败:', e)
  }

  // 获取真实自启状态（覆盖配置文件中的值）
  try {
    const realAutostart = await invoke<boolean>('check_autostart')
    autostart.value = realAutostart
  } catch {}

  // 从后端获取构建时间
  try {
    buildTime.value = await invoke<string>('get_build_time')
  } catch {
    buildTime.value = '未知'
  }

  // 监听宠物状态变化事件
  petUnlisten = await listen('pet-config-updated', () => {
    loadPetState()
  })
})

onUnmounted(() => {
  if (petUnlisten) petUnlisten()
})

// 每次页面激活时重新读取状态
onActivated(() => {
  loadPetState()
})

const linkItems = [
  {
    icon: '🔗',
    title: '手动登录',
    desc: '打开网络认证页面',
    action: () => openUrl(AUTH_URL),
  },
  {
    icon: '📥',
    title: '获取新版',
    desc: '下载或分享给同学（提取码：sxlg）',
    action: () => openUrl(DOWNLOAD_URL),
  },
  {
    icon: '🔄',
    title: '检查更新',
    desc: '检查是否有新版本可用',
    action: checkForUpdates,
  },
  {
    icon: '💬',
    title: '反馈建议',
    desc: '加入 QQ 群反馈问题或建议',
    action: () => openUrl(QQ_GROUP_URL),
  },
  {
    icon: '📋',
    title: '导出日志',
    desc: '导出日志文件用于反馈问题',
    action: exportLogs,
  },
]
</script>

<style scoped>
.more-view {
  height: 100%;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

.page-header {
  margin-bottom: 18px;
  flex-shrink: 0;
}

.more-scroll {
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
  flex: 1;
  padding-bottom: 16px;
}

.more-card {
  padding: 18px;
  border-radius: var(--radius-lg);
  flex-shrink: 0;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.card-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-primary);
}

/* ── 设置项 ── */
.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-label {
  font-size: 0.88rem;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 0.72rem;
  color: var(--text-hint);
}

/* ── 开关 ── */
.toggle-switch {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  background: var(--surface-3);
  border: 1px solid var(--border);
  cursor: pointer;
  position: relative;
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.toggle-switch.active {
  background: var(--accent);
  border-color: var(--accent);
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: white;
  box-shadow: 0 1px 3px rgba(21, 20, 15, 0.15);
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.toggle-switch.active .toggle-knob {
  transform: translateX(20px);
}

/* ── 链接列表 ── */
.link-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.link-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.link-item:hover {
  background: var(--surface-hover);
  transform: translateX(2px);
}

.link-item:active {
  transform: scale(0.98);
  background: var(--surface-active);
}

.link-icon {
  font-size: 1.15rem;
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--surface-2), var(--surface-3));
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.link-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.link-title {
  font-size: 0.84rem;
  font-weight: 600;
  color: var(--text-primary);
}

.link-desc {
  font-size: 0.72rem;
  color: var(--text-hint);
}

.link-arrow {
  width: 16px;
  height: 16px;
  color: var(--text-muted);
  flex-shrink: 0;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.link-item:hover .link-arrow {
  color: var(--accent);
  transform: translateX(3px);
}

/* ── 关于 ── */
.about-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  text-align: center;
  padding: 8px 0;
}

.about-brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.about-logo {
  width: 24px;
  height: 24px;
}

.about-name {
  font-family: var(--font-display);
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.about-ver {
  font-size: 0.65rem;
  color: var(--text-hint);
  margin-left: 6px;
  font-family: var(--font-mono);
}

.about-copy {
  font-size: 0.72rem;
  color: var(--text-secondary);
}

.about-author {
  font-size: 0.65rem;
  color: var(--text-hint);
}

.about-build {
  font-size: 0.6rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
  margin-top: 4px;
}

/* ── 弹窗 ── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(21, 20, 15, 0.5);
  backdrop-filter: blur(4px);
  z-index: 9998;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-card {
  width: 380px;
  max-height: 80vh;
  border-radius: var(--radius-xl);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--divider);
}

.modal-title {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-hint);
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.modal-close svg {
  width: 14px;
  height: 14px;
}

.modal-footer {
  padding: 14px 20px;
  border-top: 1px solid var(--divider);
  display: flex;
  justify-content: flex-end;
}

.modal-btn {
  padding: 8px 20px;
  font-size: 0.8rem;
  font-weight: 600;
  border-radius: var(--radius-md);
  background: var(--accent);
  color: white;
  transition: all 0.2s ease;
}

.modal-btn:hover {
  background: var(--accent-hover);
}

/* 弹窗动画 */
.modal-enter-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.modal-leave-active {
  transition: all 0.2s ease;
}
.modal-enter-from {
  opacity: 0;
}
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .modal-card {
  transform: scale(0.95) translateY(10px);
}
.modal-leave-to .modal-card {
  transform: scale(0.97) translateY(5px);
}

/* ── Toast 提示 ── */
.toast-bar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: var(--surface-1);
  border-radius: var(--radius-lg);
  font-size: 0.82rem;
  color: var(--text-primary);
  white-space: nowrap;
  z-index: 9999;
  box-shadow: var(--shadow-lg);
}

.toast-bar.success {
  border: 1px solid var(--success);
  color: var(--success);
}

.toast-bar.error {
  border: 1px solid var(--error);
  color: var(--error);
}

.toast-bar.info {
  border: 1px solid var(--accent);
}

/* ── 免责声明 ── */
.disclaimer {
  padding: 12px 16px;
  border-radius: var(--radius-md);
  background: var(--surface-2);
  border: 1px solid var(--border-subtle);
}

.disclaimer-text {
  font-size: 0.7rem;
  color: var(--text-hint);
  line-height: 1.6;
  text-align: center;
}

/* ── 关于链接 ── */
.about-links {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

.about-link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: var(--radius-md);
  background: var(--surface-2);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  font-size: 0.72rem;
  transition: all 0.2s ease;
}

.about-link:hover {
  border-color: var(--accent);
  color: var(--accent);
}

/* ── 赞赏码弹窗 ── */
.donate-modal {
  width: 520px;
  max-width: calc(100vw - 40px);
}

.donate-body {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  justify-content: center;
  gap: 24px;
  padding: 20px 24px;
  flex-wrap: wrap;
}

.donate-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 200px;
}

.donate-label {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.donate-qr {
  width: 100%;
  max-width: 220px;
  height: auto;
  object-fit: contain;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: #fff;
}

.donate-thanks {
  width: 100%;
  text-align: center;
  font-size: 0.75rem;
  color: var(--text-hint);
  margin-top: 4px;
}
</style>
