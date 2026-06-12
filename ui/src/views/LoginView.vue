<template>
  <div class="login-view page-enter">
    <!-- 大标题 -->
    <div class="login-hero anim-fade-up stagger-1">
      <span class="page-roman">I.</span>
      <h1 class="login-title">绍理闪连 <PageHelp>
        <p><strong>校园网络自动登录</strong></p>
        <p>选择运营商（移动/联通/电信），输入学号和密码，点击登录即可连接校园网。支持保存多个账号，点击账号输入框可切换历史账号。</p>
        <ul>
          <li>首次登录成功后会提示开启「开机自启」，开启后每次开机会自动连接</li>
          <li>网络断开恢复时会自动尝试重连，无需手动操作</li>
          <li>密码使用 AES-256 加密存储在本地，不会上传</li>
        </ul>
      </PageHelp></h1>
      <div class="accent-line-wide"></div>
    </div>

    <!-- 登录表单 -->
    <BezelCard class="login-form anim-fade-up stagger-2">
      <!-- 运营商 -->
      <div class="form-section">
        <label class="form-label">运营商</label>
        <div class="capsule-switcher" ref="switcherRef">
          <div class="capsule-slider" :style="{ left: sliderLeft + 'px', width: sliderWidth + 'px' }"></div>
          <div
            v-for="op in operators"
            :key="op.value"
            class="capsule-item"
            :class="{ active: selectedOperator === op.value }"
            @click="selectOperator(op.value)"
          >
            {{ op.label }}
          </div>
        </div>
      </div>

      <!-- 账号（带下拉历史） -->
      <div class="form-section" style="position: relative;">
        <label class="form-label">账号</label>
        <div class="input-underline-wrapper">
          <input
            v-model="username"
            type="text"
            class="input-underline"
            placeholder="请输入学号"
            @keyup.enter="focusPassword"
            @focus="onUsernameFocus"
            @blur="onUsernameBlur"
          />
        </div>
        <!-- 账号下拉 -->
        <transition name="dropdown">
          <div v-if="showDropdown && filteredAccounts.length > 0" class="account-dropdown glass">
            <div
              v-for="acc in filteredAccounts"
              :key="acc.username"
              class="account-item"
              :class="{ active: acc.username === username }"
              @mousedown.prevent="selectAccount(acc)"
            >
              <span class="account-username">{{ acc.username }}</span>
              <span class="account-operator">{{ operatorLabel(acc.operator) }}</span>
            </div>
          </div>
        </transition>
      </div>

      <!-- 密码 -->
      <div class="form-section">
        <label class="form-label">密码</label>
        <div class="input-underline-wrapper">
          <input v-model="password" type="password" class="input-underline" placeholder="请输入密码" ref="passwordInput" @keyup.enter="handleLogin" />
        </div>
      </div>

      <!-- 登录按钮 -->
      <BtnIsland
        class="login-btn"
        :loading="isLoading"
        :success="isSuccess"
        :error="isError"
        :disabled="isLoading"
        @click="handleLogin"
        ref="loginBtnRef"
        :show-icon="!isLoading && !isSuccess && !isError"
      >
        登 录
      </BtnIsland>

      <!-- 状态栏：仅保留自启开关，连接信息在 titlebar 已显示 -->
      <div class="login-status-bar anim-fade-up stagger-3">
        <button class="autostart-badge" :class="{ enabled: autostart }" @click="toggleAutostart">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path v-if="autostart" d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <template v-else>
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="12" y1="16" x2="12.01" y2="16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </template>
          </svg>
          <span>{{ autostart ? '已开启自启' : '自启已关闭' }}</span>
        </button>
      </div>
    </BezelCard>

    <!-- 错误提示 -->
    <transition name="slide">
      <div v-if="errorMsg" class="error-toast glass">
        <svg class="icon-svg-sm" viewBox="0 0 24 24" style="color: var(--error); flex-shrink: 0;">
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <span>{{ errorMsg }}</span>
      </div>
    </transition>

    <!-- 登录失败友好页面 -->
    <transition name="modal-fade">
      <div v-if="showErrorPage" class="modal-overlay" @click.self="dismissErrorPage">
        <div class="modal-card error-modal">
          <div class="modal-icon error-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="var(--error)" stroke-width="1.5"/>
              <line x1="12" y1="8" x2="12" y2="12" stroke="var(--error)" stroke-width="1.5" stroke-linecap="round"/>
              <line x1="12" y1="16" x2="12.01" y2="16" stroke="var(--error)" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <h3 class="modal-title">登录失败</h3>
          <p class="modal-desc">
            学校认证接口可能已更新，作者正在处理中。<br>
            您可以先使用网页版登录，或稍后重试。
          </p>
          <div class="modal-actions">
            <button class="modal-btn primary" @click="openWebLogin">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <polyline points="15 3 21 3 21 9" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <line x1="10" y1="14" x2="21" y2="3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              使用网页版登录
            </button>
            <button class="modal-btn secondary" @click="retryLogin">重新尝试</button>
            <button class="modal-btn secondary" @click="dismissErrorPage">关闭</button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 开机自启确认弹窗 -->
    <transition name="modal-fade">
      <div v-if="showAutostartModal" class="modal-overlay" @click.self="dismissAutostartModal">
        <div class="modal-card">
          <div class="modal-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <h3 class="modal-title">开启开机自动登录？</h3>
          <p class="modal-desc">每次开机自动连接校园网，无需手动操作。<br>登录即用，省心省力。</p>
          <div class="modal-actions">
            <button class="modal-btn primary" @click="enableAutostart">开启自动登录</button>
            <button class="modal-btn secondary" @click="dismissAutostartModal">稍后再说</button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
// 登录页：运营商选择、账号密码输入、开机自启、网络状态监听、断网自动重连

import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import BezelCard from '@/components/BezelCard.vue'
import BtnIsland from '@/components/BtnIsland.vue'
import PageHelp from '@/components/PageHelp.vue'
import type { AppConfig, EventPayload, NetworkStatus } from '@/types'

interface AccountEntry {
  username: string
  password: string
  operator: string
}

const username = ref('')
const password = ref('')
const selectedOperator = ref('cmcc')
const autostart = ref(false)
const isLoading = ref(false)
const isSuccess = ref(false)
const isError = ref(false)
const showAutostartModal = ref(false)
const showErrorPage = ref(false)
const errorMsg = ref('')
const ip = ref('')
const isOnline = ref(false)

const accounts = ref<AccountEntry[]>([])
const showDropdown = ref(false)
let blurTimer: ReturnType<typeof setTimeout> | null = null

const switcherRef = ref<HTMLElement | null>(null)
const passwordInput = ref<HTMLInputElement | null>(null)

const sliderLeft = ref(0)
const sliderWidth = ref(0)

const operators = [
  { value: 'cmcc', label: '中国移动' },
  { value: 'unicom', label: '中国联通' },
  { value: 'telecom', label: '中国电信' }
]

const operatorMap: Record<string, string> = {
  cmcc: '中国移动',
  unicom: '中国联通',
  telecom: '中国电信',
}

function operatorLabel(op: string): string {
  return operatorMap[op] || op
}

// 过滤：当输入框有内容时，显示匹配的已保存账号
const filteredAccounts = computed(() => {
  if (!username.value.trim()) return accounts.value
  const q = username.value.trim().toLowerCase()
  return accounts.value.filter(a => a.username.toLowerCase().includes(q))
})

function selectOperator(value: string) {
  selectedOperator.value = value
  updateSliderPosition()
}

function updateSliderPosition() {
  const container = switcherRef.value
  if (!container) return
  const items = container.querySelectorAll('.capsule-item')
  const idx = operators.findIndex(op => op.value === selectedOperator.value)
  if (idx >= 0 && items[idx]) {
    const item = items[idx] as HTMLElement
    sliderLeft.value = item.offsetLeft
    sliderWidth.value = item.offsetWidth
  }
}

function focusPassword() {
  passwordInput.value?.focus()
}

function onUsernameFocus() {
  if (blurTimer) clearTimeout(blurTimer)
  if (accounts.value.length > 0) {
    showDropdown.value = true
  }
}

function onUsernameBlur() {
  // 延迟关闭，让 mousedown 事件先触发
  blurTimer = setTimeout(() => {
    showDropdown.value = false
  }, 150)
}

function selectAccount(acc: AccountEntry) {
  username.value = acc.username
  password.value = acc.password
  selectedOperator.value = acc.operator
  showDropdown.value = false
  updateSliderPosition()
}

onMounted(async () => {
  await nextTick()
  updateSliderPosition()

  // 加载配置和账号列表
  try {
    const config: AppConfig = await invoke('load_config')
    if (config?.autostart) autostart.value = config.autostart

    // 加载已保存账号
    const accs: AccountEntry[] = await invoke('get_accounts')
    accounts.value = accs || []

    // 填充活跃账号
    if (config?.active_account && accs.length > 0) {
      const active = accs.find((a: AccountEntry) => a.username === config.active_account)
      if (active) {
        username.value = active.username
        password.value = active.password
        selectedOperator.value = active.operator
      } else if (accs[0]) {
        // 回退到第一个
        username.value = accs[0].username
        password.value = accs[0].password
        selectedOperator.value = accs[0].operator
      }
      updateSliderPosition()
    }
  } catch {}

  // --auto-login CLI 参数
  try {
    const argsStr = await invoke('get_cli_args') as string
    if (argsStr === '--auto-login') {
      if (username.value && password.value) handleLogin()
    }
  } catch {}

  // 获取真实自启状态
  try {
    const realAutostart = await invoke<boolean>('check_autostart')
    autostart.value = realAutostart
  } catch {}

  // 网络状态监听
  let unlisteners: UnlistenFn[] = []
  try {
    unlisteners.push(await listen('network-status', (event: EventPayload<NetworkStatus>) => {
      isOnline.value = event.payload?.is_online ?? false
    }))
    const status: NetworkStatus = await invoke('get_network_status')
    isOnline.value = status?.is_online ?? false
    ip.value = status?.ip ?? ''
  } catch {}

  // 断网恢复 → 自动重连
  try {
    unlisteners.push(await listen('network-restored', async () => {
      // 网络恢复时，如果当前未登录，自动尝试登录
      if (!isOnline.value && username.value && password.value && !isLoading.value) {
        // 等一小段时间让网络稳定
        await new Promise(r => setTimeout(r, 2000))
        handleLogin()
      }
    }))
  } catch {}

  // 保存 unlisten 函数以便组件卸载时清理
  eventUnlisteners = unlisteners
})

let eventUnlisteners: UnlistenFn[] = []

onUnmounted(() => {
  eventUnlisteners.forEach(fn => fn())
  eventUnlisteners = []
})

async function toggleAutostart() {
  const newState = !autostart.value
  try {
    if (newState) {
      await invoke('setup_autostart')
    } else {
      await invoke('remove_autostart')
    }
    autostart.value = newState
    invoke('save_config', { patch: { autostart: newState } }).catch(() => {})
  } catch {
    errorMsg.value = '设置开机自启失败'
    setTimeout(() => errorMsg.value = '', 3000)
  }
}

async function enableAutostart() {
  try {
    await invoke('setup_autostart')
    autostart.value = true
    await invoke('save_config', { patch: { autostart: true, autostart_prompted: true } })
  } catch {
    errorMsg.value = '设置开机自启失败'
    setTimeout(() => errorMsg.value = '', 3000)
  }
  showAutostartModal.value = false
  minimizeAfterModal()
}

async function dismissAutostartModal() {
  showAutostartModal.value = false
  await invoke('save_config', { patch: { autostart_prompted: true } }).catch(() => {})
  minimizeAfterModal()
}

async function minimizeAfterModal() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    setTimeout(() => getCurrentWindow().minimize(), 300)
  } catch {}
}

async function handleLogin() {
  if (!username.value.trim()) {
    errorMsg.value = '请输入账号'
    setTimeout(() => errorMsg.value = '', 3000)
    return
  }
  if (!password.value.trim()) {
    errorMsg.value = '请输入密码'
    setTimeout(() => errorMsg.value = '', 3000)
    return
  }

  isLoading.value = true
  isSuccess.value = false
  isError.value = false
  errorMsg.value = ''

  try {
    await invoke('login', {
      username: username.value.trim(),
      password: password.value.trim(),
      operator: selectedOperator.value,
    })

    isLoading.value = false

    // Tauri Result<String,String>: Ok 值直接返回，Err 走 catch
    // 能到这里说明登录成功
    isSuccess.value = true

      // 保存账号到 accounts 列表
      try {
        await invoke('save_account', {
          username: username.value.trim(),
          password: password.value.trim(),
          operator: selectedOperator.value,
        })
        // 刷新账号列表
        const accs: AccountEntry[] = await invoke('get_accounts')
        accounts.value = accs || []
      } catch {}

      // 保存其他配置
      try {
        await invoke('save_config', {
          patch: { autostart: autostart.value }
        })
      } catch {}

      // 获取 IP
      try {
        const ipResult: string = await invoke('get_ip')
        if (ipResult) ip.value = ipResult
      } catch {}

      // 首次登录且未开启自启 → 弹出确认弹窗
      if (!autostart.value) {
        try {
          const config: AppConfig = await invoke('load_config')
          if (!config?.autostart_prompted) {
            showAutostartModal.value = true
            // 弹窗期间不自动最小化，等用户选择后再最小化
            setTimeout(() => isSuccess.value = false, 2000)
            return
          }
        } catch {}
      }

      // 自动最小化窗口
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window')
        setTimeout(() => getCurrentWindow().minimize(), 600)
      } catch {}

      setTimeout(() => isSuccess.value = false, 2000)
  } catch (e: unknown) {
    isLoading.value = false
    isError.value = true
    errorMsg.value = String(e) || '连接服务器失败'
    // 显示友好错误页面
    showErrorPage.value = true
    setTimeout(() => { errorMsg.value = ''; isError.value = false }, 4000)
  }
}

function dismissErrorPage() {
  showErrorPage.value = false
}

function retryLogin() {
  showErrorPage.value = false
  handleLogin()
}

async function openWebLogin() {
  try {
    await invoke('open_url', { url: 'http://10.210.0.2/srun_portal_pc?ac_id=1&theme=pro' })
  } catch {
    window.open('http://10.210.0.2/srun_portal_pc?ac_id=1&theme=pro', '_blank')
  }
  showErrorPage.value = false
}
</script>

<style scoped>
.login-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 24px 28px;
  position: relative;
  z-index: 1;
}

.login-hero {
  text-align: center;
  margin-bottom: 28px;
}

.login-title {
  font-family: var(--font-display);
  font-size: 2rem;
  font-weight: 400;
  color: var(--text-primary);
  letter-spacing: 0.15em;
  margin-bottom: 12px;
}

.login-form {
  width: 100%;
  max-width: 380px;
}

.form-section {
  margin-bottom: 20px;
}

.form-label {
  display: block;
  font-size: 0.68rem;
  font-weight: 500;
  color: var(--text-hint);
  text-transform: uppercase;
  letter-spacing: 0.18em;
  margin-bottom: 8px;
  font-family: var(--font-body);
}

/* ── 账号下拉 ── */
.account-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 100;
  border-radius: var(--radius-md);
  margin-top: 4px;
  max-height: 160px;
  overflow-y: auto;
  padding: 4px;
}

.account-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s ease;
}

.account-item:hover {
  background: var(--surface-hover);
}

.account-item.active {
  background: var(--accent-glow);
}

.account-username {
  font-size: 0.84rem;
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.account-operator {
  font-size: 0.68rem;
  color: var(--text-hint);
  padding: 2px 8px;
  background: var(--surface-2);
  border-radius: 999px;
}

/* 下拉动画 */
.dropdown-enter-active {
  transition: all 0.2s cubic-bezier(0.32, 0.72, 0, 1);
}
.dropdown-leave-active {
  transition: all 0.15s ease;
}
.dropdown-enter-from, .dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}

.login-btn {
  width: 100%;
  margin-top: 8px;
}

.login-status-bar {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--divider);
}

.autostart-badge {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-hint);
  font-size: 0.7rem;
  cursor: pointer;
  transition: all 0.25s ease;
  font-family: inherit;
}

.autostart-badge:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.autostart-badge.enabled {
  border-color: var(--accent);
  background: var(--accent-glow);
  color: var(--accent);
}
.error-toast {
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border-radius: var(--radius-lg);
  font-size: 0.8rem;
  color: var(--error);
  white-space: nowrap;
  z-index: 10;
}

/* Modal transition */
.modal-fade-enter-active {
  transition: all 0.3s ease;
}
.modal-fade-leave-active {
  transition: all 0.2s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
.modal-fade-enter-from .modal-card {
  transform: scale(0.9) translateY(20px);
}

/* Error modal styles */
.error-modal {
  max-width: 400px;
}

.error-icon {
  background: rgba(192, 72, 72, 0.12);
}

.error-modal .modal-actions {
  flex-direction: column;
  gap: 10px;
}

.error-modal .modal-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.error-modal .modal-btn svg {
  flex-shrink: 0;
}
</style>
