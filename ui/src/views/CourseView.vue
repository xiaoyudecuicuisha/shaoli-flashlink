<script setup lang="ts">
// 网课助手：超星平台窗口管理、题库导入/匹配、刷课进度展示

import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import BezelCard from '@/components/BezelCard.vue'
import BtnIsland from '@/components/BtnIsland.vue'
import PageHelp from '@/components/PageHelp.vue'
import type { CourseProgress } from '@/types'

// ============ 状态 ============
const activeTab = ref<'course' | 'qbank'>('course')
const running = ref(false)
const speed = ref(2.0)
const opening = ref(false)
const importing = ref(false)

const progress = ref<CourseProgress>({
  videos_completed: 0,
  quizzes_answered: 0,
  quizzes_missed: 0,
  current_chapter: '',
  status: 'idle',
})

const qbankInfo = ref<{ count: number; source: string; loaded: number } | null>(null)

// 运行日志
const logs = ref<string[]>([])

// Toast
const toastMsg = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string, type: 'success' | 'error' = 'success') {
  toastMsg.value = msg
  toastType.value = type
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
}

// ============ 事件监听 ============
let unlisten: (() => void) | null = null

onMounted(async () => {
  // 监听进度事件
  unlisten = await listen<CourseProgress>('course-progress', (event) => {
    progress.value = event.payload
    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false })
    logs.value.push(`[${time}] 进度更新: 视频 ${event.payload.videos_completed}, 答题 ${event.payload.quizzes_answered}`)
    if (logs.value.length > 100) logs.value.shift()
  })

  // 加载题库信息
  await loadQbankInfo()
})

onUnmounted(() => {
  if (unlisten) unlisten()
  if (toastTimer) clearTimeout(toastTimer)
})

// ============ 刷课操作 ============
async function openCourseWindow() {
  opening.value = true
  try {
    await invoke('course_open_window')
    addLog('网课窗口已打开，请在窗口中登录超星学习通')
    showToast('网课窗口已打开')
  } catch (e: unknown) {
    showToast('打开窗口失败: ' + String(e), 'error')
  } finally {
    opening.value = false
  }
}

async function startCourse() {
  running.value = true
  progress.value.status = 'running'
  logs.value = []
  addLog('刷课助手已启动')

  // 通过 eval 触发 WebView 中的 start
  // 由于注入脚本暴露了 window.__COURSE_HELPER__，
  // 我们通过 Rust 端向 WebView 发送 eval 指令
  try {
    await invoke('course_start')
  } catch {
    // course_start 可能不存在，直接通过日志提示
    addLog('请切换到网课窗口，脚本将自动开始')
  }
}

async function stopCourse() {
  running.value = false
  progress.value.status = 'stopped'
  addLog('刷课助手已停止')
  try {
    await invoke('course_stop')
  } catch {
    // 静默处理
  }
}

async function closeCourseWindow() {
  try {
    await invoke('course_close_window')
    running.value = false
    progress.value.status = 'idle'
    addLog('网课窗口已关闭')
    showToast('窗口已关闭')
  } catch (e: unknown) {
    showToast('关闭窗口失败: ' + String(e), 'error')
  }
}

function setSpeed(s: number) {
  speed.value = s
  addLog(`倍速设置为 ${s}x`)
  invoke('course_set_speed', { speed: s }).catch(() => {})
}

function addLog(msg: string) {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false })
  logs.value.push(`[${time}] ${msg}`)
  if (logs.value.length > 100) logs.value.shift()
}

// ============ 题库操作 ============
async function loadQbankInfo() {
  try {
    qbankInfo.value = await invoke('course_get_qbank_info')
  } catch {
    qbankInfo.value = null
  }
}

async function importQbank() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: '题库文件',
        extensions: ['txt', 'json', 'xlsx', 'xls', 'docx'],
      }],
    })

    if (!selected) return

    importing.value = true
    const total = await invoke<number>('course_import_qbank', { path: selected })
    showToast(`导入成功，当前共 ${total} 条题目`)
    addLog(`题库导入: ${total} 条`)
    await loadQbankInfo()
  } catch (e: unknown) {
    showToast('导入失败: ' + String(e), 'error')
  } finally {
    importing.value = false
  }
}

async function deleteQbank() {
  try {
    await invoke('course_delete_qbank')
    qbankInfo.value = null
    showToast('题库已清空')
    addLog('题库已删除')
  } catch (e: unknown) {
    showToast('删除失败: ' + String(e), 'error')
  }
}

// ============ 工具 ============
const statusText = () => {
  const map: Record<string, string> = {
    idle: '空闲',
    running: '运行中',
    stopped: '已停止',
    completed: '已完成',
  }
  return map[progress.value.status] || progress.value.status
}

const speedOptions = [1, 1.25, 1.5, 2, 2.5, 3]

const tabs = [
  { id: 'course' as const, label: '刷课', icon: '<polygon points="5 3 19 12 5 21 5 3"/>' },
  { id: 'qbank' as const, label: '题库', icon: '<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>' },
]
</script>

<template>
  <div class="course-view page-enter">
    <div class="course-header anim-fade-up stagger-1">
      <span class="page-roman">VI.</span>
      <h1 class="page-header-title">网课助手 <PageHelp>
        <p><strong>超星学习通自动刷课</strong></p>
        <p>点击「打开网课窗口」弹出独立的课程浏览器窗口，在其中登录超星账号并进入课程页面。脚本会自动播放视频，并在章节测试中自动答题。</p>
        <ul>
          <li>需要先在弹出的课程窗口中登录超星账号</li>
          <li>在「题库」标签页导入题目文件（TXT/JSON/XLSX/DOCX），系统会自动匹配答案，题库越大准确率越高</li>
          <li>课程进度实时同步显示在主窗口</li>
          <li>建议在网速良好的环境下使用</li>
        </ul>
      </PageHelp></h1>
      <p class="page-header-sub">COURSE ASSISTANT</p>
    </div>

    <div class="course-tabs anim-fade-up stagger-2">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-btn"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <svg class="tab-icon" viewBox="0 0 24 24" v-html="tab.icon"></svg>
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="course-content anim-fade-up stagger-3">
      <!-- ========== 刷课 Tab ========== -->
      <div v-if="activeTab === 'course'" class="tab-panel">
        <!-- 平台卡片 -->
        <BezelCard>
          <div class="platform-card">
            <div class="platform-info">
              <h3 class="platform-name">超星学习通</h3>
              <p class="platform-desc">视频自动播放 / 章节测试自动答题</p>
            </div>
            <div class="platform-actions">
              <BtnIsland :loading="opening" @click="openCourseWindow">
                打开网课窗口
              </BtnIsland>
              <button class="btn-secondary" @click="closeCourseWindow">
                关闭窗口
              </button>
            </div>
          </div>
        </BezelCard>

        <!-- 控制面板 -->
        <BezelCard>
          <div class="control-panel">
            <div class="control-row">
              <div class="status-badge" :class="progress.status">
                <span class="status-dot"></span>
                <span>{{ statusText() }}</span>
              </div>
              <div class="speed-control">
                <span class="speed-label">倍速</span>
                <div class="speed-options">
                  <button
                    v-for="s in speedOptions"
                    :key="s"
                    class="speed-btn"
                    :class="{ active: speed === s }"
                    @click="setSpeed(s)"
                  >{{ s }}x</button>
                </div>
              </div>
            </div>
            <div class="control-actions">
              <BtnIsland
                :disabled="running"
                @click="startCourse"
              >
                开始刷课
              </BtnIsland>
              <button
                class="btn-secondary"
                :disabled="!running"
                @click="stopCourse"
              >
                暂停
              </button>
            </div>
          </div>
        </BezelCard>

        <!-- 进度统计 -->
        <div class="progress-stats">
          <div class="stat-item">
            <span class="stat-value">{{ progress.videos_completed }}</span>
            <span class="stat-label">视频完成</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ progress.quizzes_answered }}</span>
            <span class="stat-label">答题命中</span>
          </div>
          <div class="stat-item">
            <span class="stat-value warn">{{ progress.quizzes_missed }}</span>
            <span class="stat-label">答题跳过</span>
          </div>
          <div class="stat-item">
            <span class="stat-value chapter">{{ progress.current_chapter || '-' }}</span>
            <span class="stat-label">当前章节</span>
          </div>
        </div>

        <!-- 运行日志 -->
        <BezelCard>
          <div class="log-panel">
            <div class="log-header">
              <span class="log-title">运行日志</span>
              <button class="btn-text" @click="logs = []">清空</button>
            </div>
            <div class="log-list" ref="logListRef">
              <div v-for="(log, idx) in logs" :key="idx" class="log-entry">
                {{ log }}
              </div>
              <div v-if="logs.length === 0" class="log-empty">
                暂无日志，点击"打开网课窗口"开始
              </div>
            </div>
          </div>
        </BezelCard>

        <p class="disclaimer anim-fade-up stagger-4">
          本功能仅供学习研究使用，请遵守相关平台的服务条款
        </p>
      </div>

      <!-- ========== 题库 Tab ========== -->
      <div v-if="activeTab === 'qbank'" class="tab-panel">
        <BezelCard>
          <div class="qbank-import">
            <h3 class="section-title">导入题库</h3>
            <p class="section-desc">支持 .txt .json .xlsx .docx 格式，每次导入自动去重追加</p>
            <BtnIsland :loading="importing" @click="importQbank">
              选择题库文件
            </BtnIsland>
          </div>
        </BezelCard>

        <BezelCard>
          <div class="qbank-info">
            <h3 class="section-title">当前题库</h3>
            <template v-if="qbankInfo && qbankInfo.count > 0">
              <div class="info-row">
                <span class="info-label">题目数量</span>
                <span class="info-value">{{ qbankInfo.count }} 条</span>
              </div>
              <div class="info-row">
                <span class="info-label">文件来源</span>
                <span class="info-value">{{ qbankInfo.source }}</span>
              </div>
              <div class="qbank-actions">
                <button class="btn-secondary danger" @click="deleteQbank">
                  清空题库
                </button>
              </div>
            </template>
            <p v-else class="empty-hint">暂无题库，请先导入文件</p>
          </div>
        </BezelCard>

        <BezelCard>
          <div class="qbank-help">
            <h3 class="section-title">题库格式说明</h3>
            <div class="help-item">
              <span class="help-format">.txt</span>
              <span class="help-desc">每行一题，格式: 题目#答案 或 题目(制表符)答案</span>
            </div>
            <div class="help-item">
              <span class="help-format">.json</span>
              <span class="help-desc">JSON 数组，每项 {"question": "...", "answer": "..."}</span>
            </div>
            <div class="help-item">
              <span class="help-format">.xlsx</span>
              <span class="help-desc">Excel 表格，A列题目，B列答案</span>
            </div>
            <div class="help-item">
              <span class="help-format">.docx</span>
              <span class="help-desc">Word 文档，每两段一组（题目 + 答案）</span>
            </div>
          </div>
        </BezelCard>
      </div>
    </div>

    <!-- Toast -->
    <transition name="slide">
      <div v-if="toastMsg" class="toast-msg-bar" :class="toastType">
        <svg v-if="toastType === 'error'" width="14" height="14" viewBox="0 0 24 24" fill="none" style="flex-shrink:0; color: var(--error);">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="none"/>
          <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
          <line x1="12" y1="16" x2="12.01" y2="16" stroke="currentColor" stroke-width="2"/>
        </svg>
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" style="flex-shrink:0; color: var(--success);">
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" stroke="currentColor" stroke-width="2" fill="none"/>
          <polyline points="22 4 12 14.01 9 11.01" stroke="currentColor" stroke-width="2" fill="none"/>
        </svg>
        <span>{{ toastMsg }}</span>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.course-view {
  padding: 24px 28px;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.course-header {
  margin-bottom: 24px;
}

.course-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-secondary);
  font-size: 0.85rem;
  font-weight: 500;
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.tab-btn:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.tab-btn.active {
  background: var(--accent);
  color: white;
}

.tab-icon {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
  fill: none;
}

.course-content {
  flex: 1;
  overflow: hidden;
}

.tab-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
  padding-right: 4px;
}

/* ---- 平台卡片 ---- */
.platform-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.platform-name {
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.platform-desc {
  font-size: 0.8rem;
  color: var(--text-hint);
}

.platform-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}

/* ---- 控制面板 ---- */
.control-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.control-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.status-badge .status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-hint);
}

.status-badge.running .status-dot {
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.status-badge.completed .status-dot {
  background: var(--accent);
}

.speed-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.speed-label {
  font-size: 0.8rem;
  color: var(--text-hint);
}

.speed-options {
  display: flex;
  gap: 4px;
}

.speed-btn {
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-secondary);
  font-size: 0.75rem;
  transition: all 0.2s ease;
}

.speed-btn:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.speed-btn.active {
  background: var(--accent);
  color: white;
}

.control-actions {
  display: flex;
  gap: 12px;
}

/* ---- 进度统计 ---- */
.progress-stats {
  display: flex;
  gap: 16px;
}

.stat-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 14px 10px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
}

.stat-value {
  font-size: 1.3rem;
  font-weight: 700;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.stat-value.warn {
  color: var(--warning);
}

.stat-value.chapter {
  font-size: 0.85rem;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 140px;
}

.stat-label {
  font-size: 0.72rem;
  color: var(--text-hint);
}

/* ---- 日志 ---- */
.log-panel {
  display: flex;
  flex-direction: column;
  max-height: 200px;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.log-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.btn-text {
  font-size: 0.75rem;
  color: var(--text-hint);
  transition: color 0.2s;
}

.btn-text:hover {
  color: var(--text-primary);
}

.log-list {
  flex: 1;
  overflow-y: auto;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.log-entry {
  padding: 3px 0;
  line-height: 1.5;
}

.log-empty {
  color: var(--text-hint);
  font-style: italic;
  padding: 12px 0;
}

/* ---- 免责声明 ---- */
.disclaimer {
  font-size: 0.72rem;
  color: var(--text-muted);
  text-align: center;
  padding: 8px 0;
  flex-shrink: 0;
}

/* ---- 题库 Tab ---- */
.qbank-import,
.qbank-info,
.qbank-help {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
}

.section-desc {
  font-size: 0.8rem;
  color: var(--text-hint);
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid var(--divider);
}

.info-label {
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.info-value {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-primary);
}

.qbank-actions {
  margin-top: 8px;
}

.btn-secondary.danger {
  color: var(--error);
  border-color: var(--error);
}

.btn-secondary.danger:hover {
  background: var(--error);
  color: white;
}

.empty-hint {
  font-size: 0.85rem;
  color: var(--text-hint);
  font-style: italic;
}

.help-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 0;
}

.help-format {
  flex-shrink: 0;
  padding: 2px 10px;
  background: var(--surface-2);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 0.78rem;
  font-weight: 600;
  color: var(--accent);
  min-width: 48px;
  text-align: center;
}

.help-desc {
  font-size: 0.82rem;
  color: var(--text-secondary);
}

/* ---- Toast ---- */
.toast-msg-bar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  font-size: 0.82rem;
  color: var(--text-primary);
  white-space: nowrap;
  z-index: 9999;
  box-shadow: var(--shadow-lg);
}

.toast-msg-bar.error {
  border-color: var(--error);
}

.toast-msg-bar.success {
  border-color: var(--success);
}

/* ---- 通用按钮 ---- */
.btn-secondary {
  padding: 8px 18px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-secondary);
  font-size: 0.82rem;
  font-weight: 500;
  border: 1px solid var(--border-subtle);
  transition: all 0.2s ease;
}

.btn-secondary:hover:not(:disabled) {
  background: var(--surface-hover);
  color: var(--text-primary);
  border-color: var(--border);
}

.btn-secondary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ---- 动画 ---- */
.slide-enter-active {
  transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1);
}

.slide-leave-active {
  transition: all 0.2s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}
</style>
