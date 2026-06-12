<template>
  <div class="whiteboard-view page-enter">
    <!-- 页面标题 -->
    <div class="page-header anim-fade-left stagger-1">
      <div class="header-row">
        <div>
          <span class="page-roman">IV.</span>
          <h1 class="page-header-title">白板 <PageHelp>
            <p><strong>本地白板画布</strong></p>
            <p>支持三种模板：思维导图、流程图、自由画布。创建后可在画板上自由绘制、添加文字和图形。</p>
            <ul>
              <li>白板数据保存在本地，不会上传</li>
              <li>支持创建多个白板，每个白板独立保存</li>
              <li>画板内容自动保存，关闭后再次打开可继续编辑</li>
              <li>可在页面顶部设置白板存储目录（默认：桌面/白板）</li>
            </ul>
          </PageHelp></h1>
          <p class="page-header-sub">W H I T E B O A R D</p>
        </div>
        <div class="header-actions">
          <div class="dir-row">
            <input :value="boardDir" type="text" readonly class="dir-input" placeholder="选择存储目录..." />
            <button class="btn-dir" @click="selectBoardDir">更改</button>
          </div>
          <button class="btn-create" @click="showCreateModal = true">
            <svg class="icon-svg-sm" viewBox="0 0 24 24">
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            <span>新建白板</span>
          </button>
        </div>
      </div>
    </div>

    <div class="whiteboard-scroll">
      <!-- 空状态 -->
      <div v-if="boards.length === 0 && !loading" class="empty-state anim-fade-up stagger-2">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none">
          <rect x="3" y="3" width="18" height="18" rx="2" stroke="var(--text-muted)" stroke-width="1.5"/>
          <path d="M8 12h8M12 8v8" stroke="var(--text-muted)" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <p class="empty-text">还没有白板</p>
        <p class="empty-hint">点击下方任一模板快速开始</p>
        <div class="empty-templates">
          <button class="template-chip" @click="newWithType('mindmap')">
            <svg class="icon-svg-sm" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.5"/>
              <path d="M12 9V3M12 15v6M9 12H3M15 12h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            思维导图
          </button>
          <button class="template-chip" @click="newWithType('flowchart')">
            <svg class="icon-svg-sm" viewBox="0 0 24 24">
              <rect x="3" y="3" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.5"/>
              <rect x="14" y="14" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.5"/>
              <path d="M10 6.5h4M6.5 10v4M17.5 10v4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            流程图
          </button>
          <button class="template-chip" @click="newWithType('free')">
            <svg class="icon-svg-sm" viewBox="0 0 24 24">
              <path d="M3 17c3-3 6 2 9-1s3-8 9-8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            自由画
          </button>
        </div>
      </div>

      <!-- 白板列表 -->
      <div v-else class="board-grid">
        <BezelCard
          v-for="(board, index) in boards"
          :key="board.name"
          class="board-card anim-fade-up"
          :class="'stagger-' + Math.min(index + 2, 6)"
          hoverable
        >
          <div class="board-header">
            <div class="board-icon">
              <svg v-if="board.board_type === 'mindmap'" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="3" stroke="var(--accent)" stroke-width="1.5"/>
                <path d="M12 9V3M12 15v6M9 12H3M15 12h6" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
              <svg v-else-if="board.board_type === 'flowchart'" viewBox="0 0 24 24" fill="none">
                <rect x="3" y="3" width="7" height="7" rx="1" stroke="var(--gold)" stroke-width="1.5"/>
                <rect x="14" y="14" width="7" height="7" rx="1" stroke="var(--gold)" stroke-width="1.5"/>
                <path d="M10 6.5h4M6.5 10v4M17.5 10v4" stroke="var(--gold)" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none">
                <path d="M3 17c3-3 6 2 9-1s3-8 9-8" stroke="var(--success)" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </div>
            <div class="board-info">
              <span class="board-name">{{ board.name }}</span>
              <span class="board-type">{{ boardTypeLabel(board.board_type) }}</span>
            </div>
            <span v-if="isBoardOpen(board.name)" class="board-status">
              <span class="status-dot online"></span>
              编辑中
            </span>
          </div>

          <div class="board-meta">
            <span class="board-time">更新于 {{ formatTime(board.updated_at) }}</span>
          </div>

          <div class="board-actions">
            <button class="btn-action btn-open" @click="openBoard(board.name)">
              {{ isBoardOpen(board.name) ? '切换到窗口' : '打开' }}
            </button>
            <button 
              v-if="isBoardOpen(board.name)" 
              class="btn-action btn-close-board" 
              @click="closeBoard(board.name)"
            >
              关闭窗口
            </button>
            <button class="btn-action btn-delete" @click="confirmDelete(board.name)">删除</button>
          </div>
        </BezelCard>
      </div>
    </div>

    <!-- 新建白板弹窗 -->
    <transition name="modal-fade">
      <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
        <BezelCard class="modal-card">
          <div class="modal-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
              <rect x="3" y="3" width="18" height="18" rx="2" stroke="var(--accent)" stroke-width="1.5"/>
              <path d="M12 8v8M8 12h8" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <h3 class="modal-title">新建白板</h3>
          <div class="modal-input-wrapper">
            <input
              v-model="newBoardName"
              type="text"
              class="modal-input"
              placeholder="输入白板名称"
              @keyup.enter="createBoard"
              ref="nameInput"
            />
          </div>
          <div class="modal-actions">
            <button class="modal-btn primary" @click="createBoard" :disabled="!newBoardName.trim()">创建</button>
            <button class="modal-btn secondary" @click="showCreateModal = false">取消</button>
          </div>
        </BezelCard>
      </div>
    </transition>

    <!-- 删除确认弹窗 -->
    <transition name="modal-fade">
      <div v-if="showDeleteModal" class="modal-overlay" @click.self="showDeleteModal = false">
        <BezelCard class="modal-card">
          <div class="modal-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" stroke="var(--error)" stroke-width="1.5"/>
              <line x1="12" y1="8" x2="12" y2="12" stroke="var(--error)" stroke-width="1.5" stroke-linecap="round"/>
              <line x1="12" y1="16" x2="12.01" y2="16" stroke="var(--error)" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <h3 class="modal-title">确认删除</h3>
          <p class="modal-desc">确定要删除「{{ deleteTarget }}」吗？<br>此操作不可撤销。</p>
          <div class="modal-actions">
            <button class="modal-btn danger" @click="deleteBoard">删除</button>
            <button class="modal-btn secondary" @click="showDeleteModal = false">取消</button>
          </div>
        </BezelCard>
      </div>
    </transition>

    <!-- 关闭确认弹窗 -->
    <transition name="modal-fade">
      <div v-if="showCloseModal" class="modal-overlay" @click.self="showCloseModal = false">
        <BezelCard class="modal-card">
          <div class="modal-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" stroke="var(--accent)" stroke-width="1.5"/>
              <line x1="12" y1="8" x2="12" y2="12" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round"/>
              <line x1="12" y1="16" x2="12.01" y2="16" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <h3 class="modal-title">确认关闭</h3>
          <p class="modal-desc">白板已自动保存。<br>确定要关闭「{{ closeTarget }}」吗？</p>
          <div class="modal-actions">
            <button class="modal-btn primary" @click="confirmClose">关闭</button>
            <button class="modal-btn secondary" @click="showCloseModal = false">取消</button>
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
// 白板管理：创建/打开/删除白板、窗口管理、目录配置、事件监听

import { ref, onMounted, onUnmounted, onActivated } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import BezelCard from '@/components/BezelCard.vue'
import PageHelp from '@/components/PageHelp.vue'
import type { EventPayload, WhiteboardEvent } from '@/types'

interface WhiteboardInfo {
  name: string
  created_at: string
  updated_at: string
  board_type: string
}

const boards = ref<WhiteboardInfo[]>([])
const loading = ref(true)
const boardDir = ref('')
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const showCloseModal = ref(false)
const newBoardName = ref('')
const deleteTarget = ref('')
const closeTarget = ref('')
const openBoards = ref<string[]>([])
const toastMsg = ref('')
const toastType = ref<'success' | 'error' | 'info'>('info')
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string, type: 'success' | 'error' | 'info' = 'error') {
  toastMsg.value = msg
  toastType.value = type
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
}

// 白板类型标签
function boardTypeLabel(type: string): string {
  switch (type) {
    case 'mindmap': return '思维导图'
    case 'flowchart': return '流程图'
    default: return '自由画'
  }
}

// 格式化时间
function formatTime(timeStr: string): string {
  try {
    const date = new Date(timeStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    if (diff < 60000) return '刚刚'
    if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
    
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    const hours = String(date.getHours()).padStart(2, '0')
    const minutes = String(date.getMinutes()).padStart(2, '0')
    
    if (year === now.getFullYear()) {
      return `${month}-${day} ${hours}:${minutes}`
    }
    return `${year}-${month}-${day}`
  } catch {
    return timeStr
  }
}

// 检查白板是否已打开
function isBoardOpen(name: string): boolean {
  return openBoards.value.includes(name)
}

// 加载白板列表
async function loadBoards() {
  try {
    boards.value = await invoke('list_whiteboards')
  } catch (e) {
    showToast('加载白板列表失败: ' + String(e))
  }
  loading.value = false
}

// 加载白板存储目录
async function loadBoardDir() {
  try {
    boardDir.value = await invoke<string>('whiteboard_get_dir')
  } catch (e) {
    console.error('获取白板目录失败:', e)
  }
}

// 选择白板存储目录
async function selectBoardDir() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      defaultPath: boardDir.value,
      title: '选择白板存储目录',
    })
    if (selected && typeof selected === 'string') {
      await invoke('whiteboard_set_dir', { dir: selected })
      boardDir.value = selected
      await loadBoards()
      showToast('白板目录已更新', 'success')
    }
  } catch (e) {
    showToast('选择目录失败: ' + String(e))
  }
}

// 加载已打开的白板
async function loadOpenBoards() {
  try {
    openBoards.value = await invoke('get_open_whiteboards')
  } catch (e) {
    showToast('获取已打开白板失败: ' + String(e))
  }
}

// 创建白板
async function createBoard() {
  const name = newBoardName.value.trim()
  if (!name) return

  try {
    // 保存空白板数据
    await invoke('save_whiteboard', { name, data: { children: [] } })

    // 打开白板窗口
    await invoke('open_whiteboard_window', { name })

    // 刷新列表
    await loadBoards()
    await loadOpenBoards()

    // 关闭弹窗
    showCreateModal.value = false
    newBoardName.value = ''
  } catch (e) {
    showToast('创建白板失败: ' + String(e))
  }
}

// 空态模板快速创建
function newWithType(type: 'mindmap' | 'flowchart' | 'free') {
  const labels: Record<string, string> = {
    mindmap: '思维导图',
    flowchart: '流程图',
    free: '自由画',
  }
  const suffix = labels[type] || '白板'
  newBoardName.value = `${suffix} ${new Date().toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })}`
  showCreateModal.value = true
}

// 打开白板
async function openBoard(name: string) {
  try {
    await invoke('open_whiteboard_window', { name })
    await loadOpenBoards()
  } catch (e) {
    showToast('打开白板失败: ' + String(e))
  }
}

// 确认删除
function confirmDelete(name: string) {
  deleteTarget.value = name
  showDeleteModal.value = true
}

// 删除白板（如果窗口还开着，先关闭）
async function deleteBoard() {
  try {
    // 如果该白板有打开的窗口，先关闭它
    if (isBoardOpen(deleteTarget.value)) {
      try {
        await invoke('close_whiteboard_window', { name: deleteTarget.value })
      } catch {}
    }
    await invoke('delete_whiteboard', { name: deleteTarget.value })
    await loadBoards()
    await loadOpenBoards()
    showDeleteModal.value = false
  } catch (e) {
    showToast('删除白板失败: ' + String(e))
  }
}

// 关闭白板窗口
async function closeBoard(name: string) {
  closeTarget.value = name
  showCloseModal.value = true
}

// 确认关闭白板窗口
async function confirmClose() {
  try {
    await invoke('close_whiteboard_window', { name: closeTarget.value })
    await loadOpenBoards()
    showCloseModal.value = false
  } catch (e) {
    showToast('关闭白板窗口失败: ' + String(e))
  }
}

// 监听白板窗口关闭事件
let boardUnlisteners: UnlistenFn[] = []

onMounted(async () => {
  await loadBoardDir()
  await loadBoards()
  await loadOpenBoards()
  
  // 监听窗口变化
  boardUnlisteners.push(await listen('whiteboard-window-changed', () => {
    loadOpenBoards()
  }))
  
  // 监听白板数据变化（从白板窗口同步）
  boardUnlisteners.push(await listen('whiteboard-data-changed', (_event: EventPayload<WhiteboardEvent>) => {
    loadBoards()
  }))
  
  // 监听白板窗口关闭事件
  boardUnlisteners.push(await listen('whiteboard-closing', (event: EventPayload<WhiteboardEvent>) => {
    const name = event.payload?.name
    if (name) {
      // 刷新已打开白板列表
      loadOpenBoards()
      // 刷新白板列表（可能有数据更新）
      loadBoards()
    }
  }))
})

onUnmounted(() => {
  boardUnlisteners.forEach(fn => fn())
  boardUnlisteners = []
})

// 每次页面激活时刷新列表（处理外部文件变动）
onActivated(() => {
  loadBoards()
  loadOpenBoards()
})
</script>

<style scoped>
.whiteboard-view {
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

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.btn-create {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 0.78rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--accent);
  color: white;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.btn-create:hover {
  background: var(--accent-hover);
  transform: translateY(-1px);
}

.btn-create:active {
  transform: scale(0.97);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.dir-input {
  width: 180px;
  height: 32px;
  padding: 0 10px;
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--surface-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: default;
  font-family: var(--font-mono);
}

.btn-dir {
  height: 32px;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.btn-dir:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.whiteboard-scroll {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 16px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
}

.empty-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-text {
  font-size: 1rem;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 0.85rem;
  color: var(--text-hint);
  margin-bottom: 20px;
}

.empty-templates {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: center;
}

.template-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 999px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 0.8rem;
  font-weight: 500;
  transition: all 0.2s ease;
  cursor: pointer;
}

.template-chip:hover {
  background: var(--surface-hover);
  border-color: var(--accent);
  color: var(--text-primary);
  transform: translateY(-1px);
}

.template-chip:active {
  transform: scale(0.97);
}

/* 白板网格 */
.board-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.board-card {
  padding: 18px;
  border-radius: var(--radius-lg);
}

.board-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.board-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.board-icon svg {
  width: 20px;
  height: 20px;
}

.board-info {
  flex: 1;
  min-width: 0;
}

.board-name {
  display: block;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.board-type {
  font-size: 0.7rem;
  color: var(--text-hint);
}

.board-status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.7rem;
  color: var(--success);
  flex-shrink: 0;
}

.board-meta {
  margin-bottom: 12px;
}

.board-time {
  font-size: 0.75rem;
  color: var(--text-hint);
}

.board-actions {
  display: flex;
  gap: 8px;
}

.btn-action {
  flex: 1;
  padding: 8px 12px;
  font-size: 0.78rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.btn-action:active {
  transform: scale(0.97);
}

.btn-open {
  background: var(--accent);
  color: white;
}

.btn-open:hover {
  background: var(--accent-hover);
  transform: translateY(-1px);
}

.btn-delete {
  background: var(--surface-2);
  color: var(--text-secondary);
}

.btn-delete:hover {
  background: var(--error);
  color: white;
}

.btn-close-board {
  background: var(--surface-3);
  color: var(--text-secondary);
}

.btn-close-board:hover {
  background: var(--warning);
  color: white;
}

/* 弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(21, 20, 15, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-card {
  max-width: 340px;
  width: 90%;
  padding: 32px 28px;
  border-radius: 16px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.modal-icon {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  font-family: var(--font-display);
}

.modal-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0;
}

.modal-input-wrapper {
  width: 100%;
  margin: 8px 0;
}

.modal-input {
  width: 100%;
  padding: 12px 16px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.9rem;
  font-family: var(--font-body);
  transition: border-color 0.2s ease;
}

.modal-input:focus {
  outline: none;
  border-color: var(--accent);
}

.modal-input::placeholder {
  color: var(--text-hint);
}

.modal-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
  margin-top: 8px;
}

.modal-btn {
  width: 100%;
  padding: 12px 20px;
  border-radius: 10px;
  border: none;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
  font-family: inherit;
}

.modal-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-btn.primary {
  background: var(--accent);
  color: white;
}

.modal-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 16px color-mix(in srgb, var(--accent) 30%, transparent);
}

.modal-btn.danger {
  background: var(--error);
  color: white;
}

.modal-btn.danger:hover {
  background: color-mix(in srgb, var(--error) 90%, white);
  transform: translateY(-1px);
}

.modal-btn.secondary {
  background: var(--surface-2);
  color: var(--text-secondary);
}

.modal-btn.secondary:hover {
  background: var(--surface-hover);
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
</style>
