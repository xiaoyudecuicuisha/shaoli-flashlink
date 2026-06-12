<script setup lang="ts">
// 格式转换面板：多文件拖放、批量转换、进度跟踪、取消操作

import { ref, reactive, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import BtnIsland from '@/components/BtnIsland.vue'
import type { ConvertTaskStatus } from '@/types'

// ============ 类型定义 ============
interface FileItem {
  id: string
  name: string
  path: string
  icon: string
  sourceExt: string
  targetFormat: string
  status: 'pending' | 'converting' | 'completed' | 'cancelled' | 'error'
  progress: number
  taskId?: string
  outputPath?: string
  errorMsg?: string
}

interface ConvertResult {
  task_id: string
  output_path: string
}

// ============ 状态 ============
const fileList = ref<FileItem[]>([])
const supportedFormatsMap = reactive<Record<string, string[]>>({})
const selectedTargetFormat = ref('PDF')

// 轮询定时器
const pollTimers = ref<Map<string, number>>(new Map())

// 计算属性
const isConverting = computed(() =>
  fileList.value.some(f => f.status === 'converting')
)

const hasPending = computed(() =>
  fileList.value.some(f => f.status === 'pending')
)

const hasCompleted = computed(() =>
  fileList.value.some(f => f.status === 'completed')
)

// 计算所有待处理文件都支持的格式交集
const commonFormats = computed(() => {
  const pendingFiles = fileList.value.filter(f => f.status === 'pending')

  if (pendingFiles.length === 0) {
    return ['PDF', 'DOCX', 'XLSX', 'PPTX', 'TXT']
  }

  const allFormats = pendingFiles.map(f => {
    const formats = supportedFormatsMap[f.sourceExt] || []
    return formats.map(s => s.toUpperCase())
  })

  if (allFormats.length === 0) return []

  let result = allFormats[0]
  for (let i = 1; i < allFormats.length; i++) {
    result = result.filter(fmt => allFormats[i].includes(fmt))
  }

  return result
})

// 格式元信息
const FORMAT_META: Record<string, { label: string; icon: string; color: string }> = {
  PDF:  { label: 'PDF',  icon: '📄', color: '#E74C3C' },
  DOCX: { label: 'Word', icon: '📝', color: '#2B579A' },
  DOC:  { label: 'Word', icon: '📝', color: '#2B579A' },
  XLSX: { label: 'Excel', icon: '📊', color: '#217346' },
  XLS:  { label: 'Excel', icon: '📊', color: '#217346' },
  PPTX: { label: 'PPT',  icon: '📽️', color: '#D04423' },
  PPT:  { label: 'PPT',  icon: '📽️', color: '#D04423' },
  RTF:  { label: 'RTF',  icon: '📄', color: '#8B5CF6' },
  TXT:  { label: 'TXT',  icon: '📃', color: '#6B7280' },
  CSV:  { label: 'CSV',  icon: '📊', color: '#10B981' },
}

// ============ 工具函数 ============

function getFormatIcon(ext: string): string {
  return FORMAT_META[ext.toUpperCase()]?.icon ?? '📁'
}

function getFormatLabel(ext: string): string {
  return FORMAT_META[ext.toUpperCase()]?.label ?? ext
}

function getFormatColor(ext: string): string {
  return FORMAT_META[ext.toUpperCase()]?.color ?? '#666'
}

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// ============ 文件选择 ============

async function pickFiles() {
  try {
    const result = await open({
      multiple: true,
      filters: [{
        name: '文档文件',
        extensions: ['doc', 'docx', 'rtf', 'txt', 'xls', 'xlsx', 'csv', 'ppt', 'pptx']
      }]
    })

    if (result) {
      const paths = Array.isArray(result) ? result : [result]
      for (const path of paths) {
        await addFile(path)
      }
    }
  } catch (e) {
    // 用户取消
  }
}

async function addFile(path: string) {
  const ext = path.split('.').pop()?.toUpperCase() || ''
  const fileName = path.split(/[/\\]/).pop() || path

  // 检查是否已存在
  if (fileList.value.some(f => f.path === path)) {
    return
  }

  // 获取支持的格式
  let formats: string[] = []
  try {
    formats = await invoke<string[]>('get_supported_formats', { inputPath: path })
    supportedFormatsMap[ext] = formats
  } catch (e: unknown) {
    showToast('获取支持格式失败: ' + String(e))
  }

  // 默认目标格式
  const defaultTarget = formats.includes('PDF') ? 'PDF' : (formats[0] || '')

  fileList.value.push({
    id: generateId(),
    name: fileName,
    path,
    icon: getFormatIcon(ext),
    sourceExt: ext,
    targetFormat: defaultTarget,
    status: 'pending',
    progress: 0,
  })
}

// ============ 拖拽 ============

const dragging = ref(false)
let dragCounter = 0

function onDragEnter(e: DragEvent) {
  e.preventDefault()
  dragCounter++
  dragging.value = true
}

function onDragLeave(e: DragEvent) {
  e.preventDefault()
  dragCounter--
  if (dragCounter <= 0) {
    dragging.value = false
    dragCounter = 0
  }
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
}

async function onDrop(e: DragEvent) {
  e.preventDefault()
  dragging.value = false
  dragCounter = 0

  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    for (let i = 0; i < files.length; i++) {
      const filePath = (files[i] as any).path
      if (filePath) {
        await addFile(filePath)
      }
    }
  }
}

// ============ 文件操作 ============

function removeFile(fileId: string) {
  // 取消轮询
  const timer = pollTimers.value.get(fileId)
  if (timer) {
    clearInterval(timer)
    pollTimers.value.delete(fileId)
  }

  fileList.value = fileList.value.filter(f => f.id !== fileId)
}

function clearCompleted() {
  const completedIds = fileList.value
    .filter(f => f.status === 'completed' || f.status === 'cancelled')
    .map(f => f.id)

  for (const id of completedIds) {
    const timer = pollTimers.value.get(id)
    if (timer) {
      clearInterval(timer)
      pollTimers.value.delete(id)
    }
  }

  fileList.value = fileList.value.filter(f =>
    f.status !== 'completed' && f.status !== 'cancelled'
  )
}

function setAllTargetFormat(format: string) {
  selectedTargetFormat.value = format
  fileList.value.forEach(f => {
    if (f.status === 'pending') {
      // 只对支持该目标格式的文件进行设置
      const supportedFormats = supportedFormatsMap[f.sourceExt] || []
      const supportedUpper = supportedFormats.map(s => s.toUpperCase())
      if (supportedUpper.includes(format)) {
        f.targetFormat = format
      }
    }
  })
}

// ============ 转换操作 ============

async function startFileConvert(file: FileItem) {
  if (file.status !== 'pending') return

  file.status = 'converting'
  file.progress = 0
  file.errorMsg = undefined

  try {
    const result = await invoke<ConvertResult>('convert_document', {
      inputPath: file.path,
      targetFormat: file.targetFormat,
    })

    file.taskId = result.task_id
    file.outputPath = result.output_path

    // 开始轮询状态
    startPolling(file)
  } catch (e: unknown) {
    file.status = 'error'
    file.errorMsg = String(e)
  }
}

function startPolling(file: FileItem) {
  if (!file.taskId) return

  const timer = window.setInterval(async () => {
    try {
      const status = await invoke<ConvertTaskStatus>('get_convert_status', { taskId: file.taskId })

      file.progress = status.progress || 0

      if (status.status === 'Completed') {
        file.status = 'completed'
        file.outputPath = status.output_path
        stopPolling(file.id)
      } else if (status.status === 'Cancelled') {
        file.status = 'cancelled'
        stopPolling(file.id)
      } else if (typeof status.status === 'object' && status.status?.Failed) {
        file.status = 'error'
        file.errorMsg = status.status.Failed
        stopPolling(file.id)
      }
    } catch (e) {
      // 查询失败，停止轮询
      stopPolling(file.id)
    }
  }, 500)

  pollTimers.value.set(file.id, timer)
}

function stopPolling(fileId: string) {
  const timer = pollTimers.value.get(fileId)
  if (timer) {
    clearInterval(timer)
    pollTimers.value.delete(fileId)
  }
}

async function batchConvert() {
  const pendingFiles = fileList.value.filter(f => f.status === 'pending')

  // 批量转换
  try {
    const items = pendingFiles.map(f => ({
      input_path: f.path,
      target_format: f.targetFormat,
    }))

    const results = await invoke<ConvertResult[]>('batch_convert', { files: items })

    // 更新文件状态
    for (let i = 0; i < pendingFiles.length; i++) {
      const file = pendingFiles[i]
      const result = results[i]

      file.status = 'converting'
      file.taskId = result.task_id
      file.outputPath = result.output_path

      // 开始轮询
      startPolling(file)
    }
  } catch (e: unknown) {
    // 批量转换失败
    showToast('批量转换失败: ' + String(e))
  }
}

async function cancelFile(fileId: string) {
  const file = fileList.value.find(f => f.id === fileId)
  if (!file || !file.taskId) return

  try {
    await invoke('cancel_convert', { taskId: file.taskId })
    file.status = 'cancelled'
    stopPolling(fileId)
  } catch (e: unknown) {
    showToast('取消失败: ' + String(e))
  }
}

async function cancelAll() {
  try {
    await invoke('cancel_all_convert')

    // 更新所有文件状态
    fileList.value.forEach(f => {
      if (f.status === 'converting' || f.status === 'pending') {
        f.status = 'cancelled'
      }
    })

    // 清除所有轮询
    pollTimers.value.forEach((timer) => clearInterval(timer))
    pollTimers.value.clear()
  } catch (e: unknown) {
    showToast('取消所有失败: ' + String(e))
  }
}

// ============ 打开文件/目录 ============

const toastMsg = ref('')
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string) {
  toastMsg.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
}

async function openFile(filePath: string) {
  try {
    const { open: shellOpen } = await import('@tauri-apps/plugin-shell')
    await shellOpen(filePath)
  } catch (e) {
    showToast('打开文件失败：' + String(e))
  }
}

async function openOutputDir(filePath: string) {
  try {
    const dir = filePath.replace(/[/\\][^/\\]+$/, '')
    const { open: shellOpen } = await import('@tauri-apps/plugin-shell')
    await shellOpen(dir)
  } catch (e) {
    showToast('打开目录失败：' + String(e))
  }
}

// ============ 生命周期 ============

onUnmounted(() => {
  // 清除所有轮询
  pollTimers.value.forEach((timer) => clearInterval(timer))
  pollTimers.value.clear()
})
</script>

<template>
  <div class="convert-panel">
    <!-- 空状态：拖拽区 — 跟随父级高度自适应 -->
    <div
      v-if="fileList.length === 0"
      class="drop-zone"
      :class="{ dragging }"
      @dragenter="onDragEnter"
      @dragleave="onDragLeave"
      @dragover="onDragOver"
      @drop="onDrop"
      @click="pickFiles"
    >
      <div class="drop-icon">
        <svg width="48" height="48" viewBox="0 0 56 56" fill="none">
          <rect x="8" y="6" width="40" height="44" rx="4" stroke="currentColor" stroke-width="2" stroke-dasharray="4 3" opacity="0.4"/>
          <path d="M28 20v16M20 28l8-8 8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <p class="drop-text">拖拽文档到这里，或点击选择文件</p>
      <p class="drop-hint">支持多选 · Word · Excel · PPT · RTF · TXT · CSV</p>
    </div>

    <!-- 文件列表 -->
    <div v-else class="file-list-container">
      <!-- 批量操作栏 -->
      <div class="batch-actions">
        <div class="batch-left">
          <button class="btn-ghost" @click="pickFiles">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
            添加文件
          </button>
          <button class="btn-ghost" @click="clearCompleted" :disabled="!hasCompleted">
            清除完成
          </button>
        </div>
        <div class="batch-right">
          <span class="file-count">{{ fileList.length }} 个文件</span>
        </div>
      </div>

      <!-- 快捷格式选择 -->
      <div class="quick-format">
        <span class="quick-format-label">全部转换为：</span>
        <div class="quick-format-chips">
          <button
            v-for="fmt in commonFormats"
            :key="fmt"
            class="format-chip"
            :class="{ active: selectedTargetFormat === fmt }"
            :style="{ '--chip-color': getFormatColor(fmt) }"
            @click="setAllTargetFormat(fmt)"
          >
            {{ getFormatLabel(fmt) }}
          </button>
        </div>
      </div>

      <!-- 文件列表 -->
      <div class="file-list">
        <div
          v-for="file in fileList"
          :key="file.id"
          class="file-item"
          :class="file.status"
        >
          <span class="file-icon">{{ file.icon }}</span>

          <div class="file-info">
            <span class="file-name">{{ file.name }}</span>
            <span class="file-meta">
              <span class="file-ext">{{ file.sourceExt }}</span>
              <span class="file-arrow">→</span>
              <span class="file-target" :style="{ color: getFormatColor(file.targetFormat) }">
                {{ file.targetFormat }}
              </span>
            </span>
          </div>

          <div class="file-status">
            <template v-if="file.status === 'pending'">
              <select
                v-model="file.targetFormat"
                class="format-select"
                :style="{ borderColor: getFormatColor(file.targetFormat) }"
              >
                <option
                  v-for="fmt in (supportedFormatsMap[file.sourceExt] || [])"
                  :key="fmt"
                  :value="fmt.toUpperCase()"
                >
                  {{ fmt.toUpperCase() }}
                </option>
              </select>
            </template>

            <template v-else-if="file.status === 'converting'">
              <div class="progress-wrapper">
                <div class="progress-bar-mini">
                  <div class="progress-fill" :style="{ width: file.progress + '%' }"></div>
                </div>
                <span class="progress-text">{{ Math.round(file.progress) }}%</span>
              </div>
            </template>

            <template v-else-if="file.status === 'completed'">
              <span class="status-badge success">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
                完成
              </span>
            </template>

            <template v-else-if="file.status === 'cancelled'">
              <span class="status-badge neutral">已取消</span>
            </template>

            <template v-else-if="file.status === 'error'">
              <span class="status-badge error" :title="file.errorMsg">
                失败
              </span>
            </template>
          </div>

          <div class="file-actions">
            <template v-if="file.status === 'pending'">
              <button class="btn-icon" @click="startFileConvert(file)" title="开始转换">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <polygon points="5 3 19 12 5 21 5 3" fill="currentColor"/>
                </svg>
              </button>
            </template>

            <template v-else-if="file.status === 'converting'">
              <button class="btn-icon cancel" @click="cancelFile(file.id)" title="取消">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor"/>
                </svg>
              </button>
            </template>

            <template v-else-if="file.status === 'completed'">
              <button class="btn-icon success" @click="openFile(file.outputPath!)" title="打开文件">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  <polyline points="15 3 21 3 21 9" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  <line x1="10" y1="14" x2="21" y2="3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
              <button class="btn-icon" @click="openOutputDir(file.outputPath!)" title="打开目录">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
            </template>

            <button class="btn-icon remove" @click="removeFile(file.id)" title="移除">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- 底部操作栏 -->
      <div class="bottom-actions">
        <BtnIsland
          v-if="hasPending"
          @click="batchConvert"
          :loading="isConverting"
          :disabled="!hasPending"
        >
          {{ isConverting ? '转换中...' : '全部转换' }}
        </BtnIsland>

        <button
          v-if="isConverting"
          class="btn-secondary cancel-all"
          @click="cancelAll"
        >
          全部取消
        </button>
      </div>
    </div>

    <!-- 面板内 Toast（不依赖全局定位） -->
    <transition name="slide">
      <div v-if="toastMsg" class="panel-toast">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" style="flex-shrink:0; color: var(--error);">
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <span>{{ toastMsg }}</span>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.convert-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  position: relative;
}

/* ---- 拖拽区：跟随父级高度自适应，不再写死兜底 ---- */
.drop-zone {
  width: 100%;
  flex: 1;
  min-height: 0;            /* 允许父级高度控制，窗口小时自然收缩 */
  border: 2px dashed var(--border);
  border-radius: var(--radius-2xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1);
  padding: 40px 24px;
  background: var(--surface-1);
}

.drop-zone:hover,
.drop-zone.dragging {
  border-color: var(--accent);
  background: var(--accent-glow);
}

.drop-zone.dragging {
  transform: scale(1.01);
}

.drop-icon {
  color: var(--text-hint);
  transition: color 0.3s;
}

.drop-zone:hover .drop-icon,
.drop-zone.dragging .drop-icon {
  color: var(--accent);
}

.drop-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin: 0;
  text-align: center;
}

.drop-hint {
  font-size: 12px;
  color: var(--text-hint);
  margin: 0;
  letter-spacing: 0.5px;
  text-align: center;
}

/* ---- 文件列表容器 ---- */
.file-list-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* ---- 批量操作栏 ---- */
.batch-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.batch-left {
  display: flex;
  gap: 8px;
}

.file-count {
  font-size: 0.82rem;
  color: var(--text-hint);
}

/* ---- 快捷格式选择 ---- */
.quick-format {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 12px;
  padding: 10px 14px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
}

.quick-format-label {
  font-size: 0.82rem;
  color: var(--text-hint);
  white-space: nowrap;
}

.quick-format-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.format-chip {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.78rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.format-chip:hover {
  border-color: var(--chip-color);
  color: var(--text-primary);
}

.format-chip.active {
  border-color: var(--chip-color);
  background: color-mix(in srgb, var(--chip-color) 10%, transparent);
  color: var(--text-primary);
}

/* ---- 文件列表 ---- */
.file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.file-item:hover {
  border-color: var(--accent);
}

.file-item.completed {
  border-color: var(--success);
  background: color-mix(in srgb, var(--success) 5%, transparent);
}

.file-item.error {
  border-color: var(--error);
  background: color-mix(in srgb, var(--error) 5%, transparent);
}

.file-icon {
  font-size: 22px;
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.file-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 0.72rem;
}

.file-ext {
  color: var(--text-hint);
}

.file-arrow {
  color: var(--text-muted);
}

.file-target {
  font-weight: 600;
}

/* ---- 格式选择 ---- */
.format-select {
  padding: 5px 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-primary);
  font-size: 0.76rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.format-select:focus {
  outline: none;
  border-color: var(--accent);
}

/* ---- 进度条 ---- */
.progress-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 90px;
}

.progress-bar-mini {
  flex: 1;
  height: 4px;
  background: var(--surface-2);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 0.72rem;
  color: var(--text-hint);
  font-family: var(--font-mono);
  min-width: 32px;
  text-align: right;
}

/* ---- 状态徽章 ---- */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 0.7rem;
  font-weight: 500;
}

.status-badge.success {
  background: color-mix(in srgb, var(--success) 12%, transparent);
  color: var(--success);
}

.status-badge.neutral {
  background: var(--surface-2);
  color: var(--text-hint);
}

.status-badge.error {
  background: color-mix(in srgb, var(--error) 12%, transparent);
  color: var(--error);
}

/* ---- 操作按钮 ---- */
.file-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.btn-icon {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--text-hint);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.btn-icon.success:hover {
  background: color-mix(in srgb, var(--success) 10%, transparent);
  color: var(--success);
}

.btn-icon.cancel:hover {
  background: color-mix(in srgb, var(--warning) 10%, transparent);
  color: var(--warning);
}

.btn-icon.remove:hover {
  background: color-mix(in srgb, var(--error) 10%, transparent);
  color: var(--error);
}

/* ---- 底部操作栏 ---- */
.bottom-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  padding-top: 12px;
  flex-shrink: 0;
}

.cancel-all {
  background: transparent;
  border: 1px solid var(--error);
  color: var(--error);
}

.cancel-all:hover {
  background: color-mix(in srgb, var(--error) 10%, transparent);
}

/* ---- 面板内 Toast（右下角，不与全局冲突） ---- */
.panel-toast {
  position: absolute;
  bottom: 12px;
  right: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--surface-1);
  border: 1px solid var(--error);
  border-radius: var(--radius-lg);
  font-size: 0.8rem;
  color: var(--text-primary);
  white-space: nowrap;
  z-index: 100;
  box-shadow: var(--shadow-md);
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.slide-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.slide-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
