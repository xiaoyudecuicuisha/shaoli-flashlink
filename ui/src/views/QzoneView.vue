<template>
  <div class="qzone-view page-enter">
    <!-- 页面标题 -->
    <div class="page-header anim-fade-up stagger-1">
      <span class="page-roman">VII.</span>
      <h1 class="page-header-title">空间回忆 <PageHelp>
        <p><strong>QQ 空间历史说说导出</strong></p>
        <p>使用手机 QQ 扫描二维码登录，点击「开始获取」，系统自动抓取历史说说（文字、图片链接、评论）。</p>
        <ul>
          <li>可选择导出目录，默认保存到桌面「QQ空间」文件夹</li>
          <li>支持导出为 Excel 表格或 HTML 网页</li>
          <li>获取过程中可随时终止，已获取的数据会保留</li>
          <li>所有数据仅本地保存，不上传到任何服务器</li>
        </ul>
      </PageHelp></h1>
      <p class="page-header-sub">Q Z O N E &nbsp; M E M O R I E S</p>
    </div>

    <!-- 未登录状态：显示二维码 -->
    <div v-if="!isLoggedIn" class="login-section anim-fade-up stagger-2">
      <BezelCard class="qr-card">
        <div class="qr-header">
          <h3>QQ 空间登录</h3>
          <p>使用手机 QQ 扫码登录，获取你的空间历史</p>
        </div>
        
        <div class="qr-wrapper">
          <div v-if="qrLoading" class="qr-loading">
            <div class="spinner"></div>
            <span>获取二维码中...</span>
          </div>
          <img v-else-if="qrCode" :src="qrCode" alt="登录二维码" class="qr-image" />
          <div v-else class="qr-error">
            <span>{{ errorMessage }}</span>
            <BtnIsland @click="getQrCode">刷新二维码</BtnIsland>
          </div>
        </div>
        
        <div v-if="qrCode" class="login-actions">
          <p class="login-hint">请使用手机 QQ 扫描二维码</p>
          <div class="login-buttons">
            <BtnIsland @click="startPolling" :loading="polling">
              {{ polling ? '等待确认...' : '我已扫码' }}
            </BtnIsland>
            <button class="btn-secondary" @click="refreshQrCode" :disabled="qrLoading">
              刷新二维码
            </button>
          </div>
          <p v-if="polling" class="login-tip">二维码有效期约为 5 分钟，过期请点击刷新</p>
        </div>
        
        <div v-if="loginError" class="login-error">
          <svg class="icon-svg-sm" viewBox="0 0 24 24" style="color: var(--error);">
            <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span>{{ loginError }}</span>
          <button class="btn-ghost" @click="refreshQrCode">重新获取</button>
        </div>
      </BezelCard>
    </div>
    
    <!-- 已登录状态：显示用户信息和获取选项 -->
    <div v-else-if="!isFetching && moments.length === 0" class="fetch-section anim-fade-up stagger-2">
      <BezelCard class="user-card">
        <div class="user-info">
          <img :src="userInfo.avatar_url" alt="头像" class="user-avatar" />
          <div class="user-details">
            <h3>{{ userInfo.nickname }}</h3>
            <p>QQ: {{ userInfo.uin }}</p>
          </div>
          <button class="btn-ghost" @click="logout">退出登录</button>
        </div>
      </BezelCard>
      
      <BezelCard class="options-card">
        <h3 class="section-title">获取范围</h3>
        <div class="option-group">
          <label class="radio-label">
            <input type="radio" v-model="fetchOptions.fetch_all" :value="true" />
            <span>全部历史说说</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="fetchOptions.fetch_all" :value="false" />
            <span>按时间范围</span>
          </label>
        </div>
        
        <div v-if="!fetchOptions.fetch_all" class="time-range">
          <div class="time-input">
            <label>开始时间</label>
            <div class="year-month">
              <input v-model.number="fetchOptions.start_year" type="number" placeholder="年" min="2005" max="2026" />
              <span>年</span>
              <input v-model.number="fetchOptions.start_month" type="number" placeholder="月" min="1" max="12" />
              <span>月</span>
            </div>
          </div>
          <div class="time-input">
            <label>结束时间</label>
            <div class="year-month">
              <input v-model.number="fetchOptions.end_year" type="number" placeholder="年" min="2005" max="2026" />
              <span>年</span>
              <input v-model.number="fetchOptions.end_month" type="number" placeholder="月" min="1" max="12" />
              <span>月</span>
            </div>
          </div>
        </div>
        
        <h3 class="section-title">获取内容</h3>
        <div class="checkbox-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="fetchOptions.include_moments" />
            <span>说说</span>
          </label>
          <label class="checkbox-label">
            <input type="checkbox" v-model="fetchOptions.include_comments" />
            <span>评论</span>
          </label>
          <label class="checkbox-label">
            <input type="checkbox" v-model="fetchOptions.include_forwards" />
            <span>转发</span>
          </label>
        </div>
        
        <h3 class="section-title">输出目录</h3>
        <div class="output-dir-row">
          <input :value="outputDir" type="text" readonly class="input-field output-dir-input" placeholder="选择输出目录..." />
          <button class="btn-secondary btn-dir" @click="selectOutputDir">选择目录</button>
        </div>
        
        <BtnIsland class="fetch-btn" @click="startFetch">
          开始获取
        </BtnIsland>
      </BezelCard>
    </div>
    
    <!-- 获取中状态：显示进度 -->
    <div v-else-if="isFetching" class="progress-section anim-fade-up stagger-2">
      <BezelCard class="progress-card">
        <div class="progress-header">
          <h3>正在获取空间数据</h3>
          <span class="badge badge-neutral">{{ progress.status }}</span>
        </div>
        
        <div class="progress-bar-wrapper">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
          </div>
          <span class="progress-text">{{ progress.current }} / {{ progress.total }}</span>
        </div>
        
        <div class="progress-stats">
          <div class="stat-card">
            <span class="stat-value">{{ progress.moments_count }}</span>
            <span class="stat-label">条说说</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{{ progress.friends_count }}</span>
            <span class="stat-label">个好友</span>
          </div>
        </div>
        
        <div class="progress-actions">
          <button class="btn-stop" @click="stopFetch" :disabled="stopRequested">
            {{ stopRequested ? '正在终止...' : '终止获取' }}
          </button>
        </div>
      </BezelCard>
    </div>
    
    <!-- 浏览模式：显示说说列表 -->
    <div v-else class="browse-section anim-fade-up stagger-2">
      <div class="browse-header">
        <div class="browse-title">
          <h3>空间回忆</h3>
          <span class="total-count">共 {{ moments.length }} 条说说</span>
        </div>
        <div class="browse-actions">
          <div class="search-box">
            <input v-model="searchQuery" type="text" placeholder="搜索说说..." class="input-field" />
          </div>
          <button class="btn-secondary" @click="exportExcel">导出 Excel</button>
          <button class="btn-secondary" @click="exportHtml">导出 HTML</button>
          <button class="btn-secondary" @click="downloadImages">下载图片</button>
          <button class="btn-ghost" @click="resetFetch">重新获取</button>
        </div>
      </div>
      
      <div class="moments-list">
        <div v-for="(moment, index) in filteredMoments" :key="index" class="moment-card" @click="showDetail(moment)">
          <div class="moment-header">
            <span class="moment-time">{{ moment.time }}</span>
          </div>
          <div class="moment-content">
            <p>{{ formatContent(moment.content) }}</p>
          </div>
          <div v-if="moment.images.length > 0" class="moment-images">
            <img v-for="(img, i) in moment.images.slice(0, 3)" :key="i" :src="img" alt="图片" class="moment-image" />
            <span v-if="moment.images.length > 3" class="more-images">+{{ moment.images.length - 3 }}</span>
          </div>
          <div class="moment-footer">
            <span v-if="moment.comments.length > 0" class="comment-count">
              {{ moment.comments.length }} 条评论
            </span>
          </div>
        </div>
        
        <div v-if="filteredMoments.length === 0" class="empty-state">
          <p>{{ searchQuery ? '没有找到匹配的说说' : '暂无说说数据' }}</p>
        </div>
      </div>
      
      <!-- 分页 -->
      <div v-if="totalPages > 1" class="pagination">
        <button class="btn-ghost" @click="currentPage--" :disabled="currentPage <= 1">上一页</button>
        <span class="page-info">{{ currentPage }} / {{ totalPages }}</span>
        <button class="btn-ghost" @click="currentPage++" :disabled="currentPage >= totalPages">下一页</button>
      </div>
    </div>
    
    <!-- 说说详情弹窗 -->
    <transition name="modal-fade">
      <div v-if="selectedMoment" class="modal-overlay" @click.self="selectedMoment = null">
        <div class="modal-card detail-modal">
          <div class="modal-header">
            <h3>说说详情</h3>
            <button class="btn-ghost" @click="selectedMoment = null">
              <svg class="icon-svg-sm" viewBox="0 0 24 24">
                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="detail-time">{{ selectedMoment.time }}</div>
            <div class="detail-content">{{ formatContent(selectedMoment.content) }}</div>
            
            <div v-if="selectedMoment.images.length > 0" class="detail-images">
              <img v-for="(img, i) in selectedMoment.images" :key="i" :src="img" alt="图片" class="detail-image" />
            </div>
            
            <div v-if="selectedMoment.comments.length > 0" class="detail-comments">
              <h4>评论 ({{ selectedMoment.comments.length }})</h4>
              <div v-for="(comment, i) in selectedMoment.comments" :key="i" class="comment-item">
                <div class="comment-header">
                  <span class="comment-nickname">{{ comment.nickname }}</span>
                  <span class="comment-time">{{ comment.time }}</span>
                </div>
                <div class="comment-content">{{ comment.content }}</div>
              </div>
            </div>
          </div>
        </div>
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
// QQ 空间历史动态：QR 登录、获取进度、数据展示、多格式导出

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import BezelCard from '@/components/BezelCard.vue'
import BtnIsland from '@/components/BtnIsland.vue'
import PageHelp from '@/components/PageHelp.vue'
import type { FetchProgress, LoginStatus, QzoneUserInfo, Moment } from '@/types'

// 状态定义
const isLoggedIn = ref(false)
const isFetching = ref(false)
const stopRequested = ref(false)
const qrLoading = ref(false)
const qrCode = ref('')
const polling = ref(false)
const loginError = ref('')
const errorMessage = ref('')
const searchQuery = ref('')
const currentPage = ref(1)
const pageSize = 20
const selectedMoment = ref<any>(null)

// Toast 状态
const toastMsg = ref('')
const toastType = ref<'success' | 'error' | 'info'>('info')
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string, type: 'success' | 'error' | 'info' = 'info') {
  toastMsg.value = msg
  toastType.value = type
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
}

// 用户信息
const userInfo = ref({
  uin: '',
  nickname: '',
  avatar_url: ''
})

// 获取选项
const fetchOptions = ref({
  fetch_all: true,
  start_year: 2010,
  start_month: 1,
  end_year: 2026,
  end_month: 6,
  include_moments: true,
  include_comments: true,
  include_forwards: true
})

// 输出目录
const outputDir = ref('')

// 进度
const progress = ref({
  total: 0,
  current: 0,
  moments_count: 0,
  friends_count: 0,
  status: '',
  is_running: false
})

// 说说列表
const moments = ref<any[]>([])

// 进度百分比
const progressPercent = computed(() => {
  if (progress.value.total === 0) return 0
  return Math.round((progress.value.current / progress.value.total) * 100)
})

// 过滤后的说说
const filteredMoments = computed(() => {
  let result = moments.value
  
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(m => m.content.toLowerCase().includes(query))
  }
  
  const start = (currentPage.value - 1) * pageSize
  const end = start + pageSize
  return result.slice(start, end)
})

// 总页数
const totalPages = computed(() => {
  let result = moments.value
  
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(m => m.content.toLowerCase().includes(query))
  }
  
  return Math.ceil(result.length / pageSize)
})

// 轮询定时器
let pollTimer: number | null = null

// 获取二维码
async function getQrCode() {
  qrLoading.value = true
  loginError.value = ''
  errorMessage.value = ''
  
  try {
    const result = await invoke<string>('qzone_get_qr_code')
    qrCode.value = result
  } catch (e: unknown) {
    errorMessage.value = String(e) || '获取二维码失败'
  } finally {
    qrLoading.value = false
  }
}

// 刷新二维码（清除轮询状态，重新获取）
async function refreshQrCode() {
  // 清除轮询定时器
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
  polling.value = false
  loginError.value = ''
  
  // 重新获取二维码
  await getQrCode()
}

// 开始轮询登录状态
async function startPolling() {
  polling.value = true
  loginError.value = ''
  
  // 从进度状态中提取 qrsig（后端保存在 status 中）
  const progressData = await invoke<FetchProgress>('qzone_get_progress')
  const qrsigMatch = progressData.status.match(/^qrsig:(.+)$/)
  const qrsig = qrsigMatch ? qrsigMatch[1] : ''
  
  if (!qrsig) {
    loginError.value = '获取登录凭证失败，请重新获取二维码'
    polling.value = false
    return
  }
  
  pollTimer = window.setInterval(async () => {
    try {
      const status = await invoke<LoginStatus>('qzone_poll_login', { qrsig })
      
      if (status.type === 'success') {
        // 登录成功
        clearInterval(pollTimer!)
        polling.value = false
        isLoggedIn.value = true
        
        // 获取用户信息
        await getUserInfo()
      } else if (status.type === 'failed') {
        // 登录失败
        clearInterval(pollTimer!)
        polling.value = false
        loginError.value = status.message ?? ''
      }
      // waiting 或 scanned 继续轮询
    } catch (e: unknown) {
      clearInterval(pollTimer!)
      polling.value = false
      loginError.value = String(e) || '登录失败'
    }
  }, 2000)
}

// 获取用户信息
async function getUserInfo() {
  try {
    const info = await invoke<QzoneUserInfo>('qzone_get_user_info')
    userInfo.value = info
  } catch (e: unknown) {
    showToast('获取用户信息失败: ' + String(e))
  }
}

// 开始获取数据
async function startFetch() {
  isFetching.value = true
  stopRequested.value = false
  
  try {
    await invoke('qzone_start_fetch', { options: fetchOptions.value })
    
    // 开始轮询进度
    const progressTimer = window.setInterval(async () => {
      try {
        const p = await invoke<FetchProgress>('qzone_get_progress')
        progress.value = p
        
        if (!p.is_running) {
          clearInterval(progressTimer)
          // 获取完成，加载说说列表
          await loadMoments()
          isFetching.value = false
        }
      } catch (e) {
        showToast('获取进度失败: ' + String(e))
      }
    }, 500)
  } catch (e: unknown) {
    isFetching.value = false
    showToast('获取失败: ' + String(e))
  }
}

// 终止获取
async function stopFetch() {
  stopRequested.value = true
  try {
    await invoke('qzone_stop_fetch')
  } catch (e: unknown) {
    showToast('终止失败: ' + String(e))
    stopRequested.value = false
  }
}

// 选择输出目录
async function selectOutputDir() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({
      directory: true,
      title: '选择数据输出目录',
      defaultPath: outputDir.value
    })
    if (path) {
      outputDir.value = path as string
    }
  } catch (e: unknown) {
    showToast('选择目录失败: ' + String(e))
  }
}

// 加载说说列表
async function loadMoments() {
  try {
    const result = await invoke<Moment[]>('qzone_get_moments', { page: 0, pageSize: 10000 })
    moments.value = result
  } catch (e: unknown) {
    showToast('加载说说失败: ' + String(e))
  }
}

// 导出 Excel
async function exportExcel() {
  try {
    if (outputDir.value) {
      const path = `${outputDir.value}/${userInfo.value.uin}_说说列表.xlsx`
      await invoke('qzone_export_excel', { path })
      showToast('导出成功: ' + path, 'success')
      return
    }
    
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      defaultPath: `${userInfo.value.uin}_说说列表.xlsx`,
      filters: [{ name: 'Excel', extensions: ['xlsx'] }]
    })
    
    if (path) {
      await invoke('qzone_export_excel', { path })
      showToast('导出成功: ' + path, 'success')
    }
  } catch (e: unknown) {
    showToast('导出失败: ' + (e || '未知错误'), 'error')
  }
}

// 导出 HTML
async function exportHtml() {
  try {
    if (outputDir.value) {
      const path = `${outputDir.value}/${userInfo.value.uin}_说说网页版.html`
      await invoke('qzone_export_html', { path })
      showToast('导出成功: ' + path, 'success')
      return
    }
    
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      defaultPath: `${userInfo.value.uin}_说说网页版.html`,
      filters: [{ name: 'HTML', extensions: ['html'] }]
    })
    
    if (path) {
      await invoke('qzone_export_html', { path })
      showToast('导出成功: ' + path, 'success')
    }
  } catch (e: unknown) {
    showToast('导出失败: ' + (e || '未知错误'), 'error')
  }
}

// 下载图片
async function downloadImages() {
  try {
    if (outputDir.value) {
      const path = `${outputDir.value}/图片`
      const count = await invoke<number>('qzone_download_images', { path })
      showToast(`下载完成，共 ${count} 张图片`, 'success')
      return
    }
    
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({
      directory: true,
      title: '选择图片保存目录'
    })
    
    if (path) {
      const count = await invoke<number>('qzone_download_images', { path })
      showToast(`下载完成，共 ${count} 张图片`, 'success')
    }
  } catch (e: unknown) {
    showToast('下载失败: ' + (e || '未知错误'), 'error')
  }
}

// 退出登录
function logout() {
  isLoggedIn.value = false
  userInfo.value = { uin: '', nickname: '', avatar_url: '' }
  moments.value = []
  qrCode.value = ''
}

// 重新获取
function resetFetch() {
  moments.value = []
  progress.value = {
    total: 0,
    current: 0,
    moments_count: 0,
    friends_count: 0,
    status: '',
    is_running: false
  }
}

// 显示详情
function showDetail(moment: any) {
  selectedMoment.value = moment
}

// 格式化内容（去掉昵称前缀）
function formatContent(content: string) {
  const parts = content.split('：')
  if (parts.length > 1) {
    return parts.slice(1).join('：').trim()
  }
  return content
}

// 组件挂载时获取二维码
onMounted(async () => {
  getQrCode()
  // 初始化默认输出目录
  try {
    outputDir.value = await invoke<string>('qzone_get_default_output_dir')
  } catch {
    // 忽略错误，用户可手动选择
  }
})

// 组件卸载时清理定时器
onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
  }
})
</script>

<style scoped>
.qzone-view {
  height: 100%;
  padding: 24px 28px;
  overflow-y: auto;
  position: relative;
  z-index: 1;
}

.page-header {
  margin-bottom: 20px;
}

/* 登录区域 */
.login-section {
  max-width: 480px;
  margin: 0 auto;
}

.qr-card {
  padding: 32px;
}

.qr-header {
  text-align: center;
  margin-bottom: 24px;
}

.qr-header h3 {
  margin: 0 0 8px 0;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--text-primary);
}

.qr-header p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.qr-wrapper {
  width: 280px;
  height: 280px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-subtle);
  margin: 0 auto 24px;
}

.qr-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-secondary);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-subtle);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.qr-image {
  width: 250px;
  height: 250px;
  border-radius: var(--radius-md);
}

.qr-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-secondary);
}

.login-actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.login-buttons {
  display: flex;
  gap: 12px;
}

.login-hint {
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.login-tip {
  color: var(--text-hint);
  font-size: 0.82rem;
  text-align: center;
}

.login-error {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 16px;
  color: var(--error);
}

/* 获取区域 */
.fetch-section {
  max-width: 600px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.user-card {
  padding: 24px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
}

.user-details {
  flex: 1;
}

.user-details h3 {
  margin: 0 0 4px 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.user-details p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.options-card {
  padding: 24px;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--text-primary);
}

.option-group {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.radio-label, .checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.time-range {
  display: flex;
  gap: 24px;
  margin-bottom: 16px;
  padding: 16px;
  background: var(--surface-2);
  border-radius: var(--radius-md);
}

.time-input {
  flex: 1;
}

.time-input label {
  display: block;
  margin-bottom: 8px;
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.year-month {
  display: flex;
  align-items: center;
  gap: 8px;
}

.year-month input {
  width: 80px;
  padding: 10px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--surface-1);
  color: var(--text-primary);
  font-size: 0.88rem;
}

.year-month span {
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.checkbox-group {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
}

.output-dir-row {
  display: flex;
  gap: 10px;
  margin-bottom: 24px;
}

.output-dir-input {
  flex: 1;
  padding: 10px 14px;
  font-size: 0.85rem;
}

.btn-dir {
  white-space: nowrap;
  flex-shrink: 0;
}

.fetch-btn {
  width: 100%;
}

/* 进度区域 */
.progress-section {
  max-width: 600px;
  margin: 0 auto;
}

.progress-card {
  padding: 32px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.progress-header h3 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.progress-bar-wrapper {
  margin-bottom: 24px;
}

.progress-text {
  display: block;
  text-align: center;
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.progress-stats {
  display: flex;
  gap: 16px;
  justify-content: center;
}

.progress-actions {
  display: flex;
  justify-content: center;
  margin-top: 24px;
}

.btn-stop {
  padding: 10px 28px;
  background: var(--error);
  color: #fff;
  border: none;
  border-radius: var(--radius-lg);
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-stop:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-stop:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 浏览区域 */
.browse-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.browse-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.browse-title {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.browse-title h3 {
  margin: 0;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--text-primary);
}

.total-count {
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.browse-actions {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}

.search-box {
  position: relative;
}

.search-box .input-field {
  width: 200px;
  padding: 8px 14px;
  font-size: 0.85rem;
}

/* 说说卡片 */
.moments-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.moment-card {
  padding: 20px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.moment-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.moment-header {
  margin-bottom: 12px;
}

.moment-time {
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.moment-content p {
  margin: 0;
  color: var(--text-primary);
  font-size: 0.92rem;
  line-height: 1.6;
}

.moment-images {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  align-items: center;
}

.moment-image {
  width: 80px;
  height: 80px;
  object-fit: cover;
  border-radius: var(--radius-md);
}

.more-images {
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.moment-footer {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--divider);
}

.comment-count {
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}

/* 分页 */
.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  padding: 20px 0;
}

.page-info {
  color: var(--text-secondary);
  font-size: 0.88rem;
}

/* 弹窗 */
.detail-modal {
  max-width: 700px;
  width: 90%;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--divider);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
}

.detail-time {
  color: var(--text-secondary);
  font-size: 0.88rem;
  margin-bottom: 16px;
}

.detail-content {
  color: var(--text-primary);
  font-size: 0.95rem;
  line-height: 1.7;
  margin-bottom: 20px;
}

.detail-images {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}

.detail-image {
  width: 100%;
  height: 150px;
  object-fit: cover;
  border-radius: var(--radius-md);
  cursor: pointer;
}

.detail-comments h4 {
  margin: 0 0 16px 0;
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
}

.comment-item {
  padding: 12px;
  background: var(--surface-2);
  border-radius: var(--radius-md);
  margin-bottom: 12px;
}

.comment-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.comment-nickname {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.88rem;
}

.comment-time {
  color: var(--text-secondary);
  font-size: 0.78rem;
}

.comment-content {
  color: var(--text-primary);
  font-size: 0.88rem;
  line-height: 1.5;
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

/* ---- Toast 提示 ---- */
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
</style>
