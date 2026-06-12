<template>
  <div class="cleaner-view page-enter">
    <div class="cleaner-header anim-fade-up stagger-1">
      <span class="page-roman">V.</span>
      <h1 class="page-header-title">系统清理 <PageHelp>
        <p><strong>系统垃圾清理与软件卸载</strong></p>
        <p>「清理」：点击「开始扫描」检测临时文件、缓存、日志等垃圾。扫描完成后可选择「回收站删除」（可恢复）或「永久删除」。</p>
        <p>「大文件」：扫描磁盘中的大文件，可按大小排序后选择性删除。</p>
        <p>「卸载」：读取已安装软件列表，执行彻底卸载（含注册表残留、服务、进程清理）。卸载前自动备份注册表。</p>
        <ul>
          <li>系统关键路径（Windows、Program Files 等）受保护，不会被误删</li>
          <li>删除操作默认使用回收站，永久删除需二次确认</li>
        </ul>
      </PageHelp></h1>
      <p class="page-header-sub">SYSTEM CLEANER</p>
    </div>

    <div class="cleaner-tabs anim-fade-up stagger-2">
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

    <div class="cleaner-content anim-fade-up stagger-3">
      <!-- 常规清理 -->
      <div v-if="activeTab === 'clean'" class="tab-panel">
        <div class="panel-toolbar">
          <button class="btn-secondary" @click="loadRules" :disabled="scanning">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
              <path d="M21 12a9 9 0 1 1-9-9" stroke="currentColor" stroke-width="2"/>
              <path d="M21 3v9h-9" stroke="currentColor" stroke-width="2"/>
            </svg>
            刷新规则
          </button>
          <button class="btn-island" @click="startScan" :disabled="scanning">
            <span v-if="scanning">扫描中...</span>
            <span v-else>开始扫描</span>
          </button>
        </div>

        <div v-if="scanResult" class="scan-summary">
          <div class="summary-item">
            <span class="summary-label">可清理文件</span>
            <span class="summary-value">{{ scanResult.total_count }} 个</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">预计释放</span>
            <span class="summary-value accent">{{ formatSize(scanResult.total_size) }}</span>
          </div>
        </div>

        <div v-if="scanResult?.items.length" class="scan-results">
          <div class="result-header">
            <label class="select-all">
              <input type="checkbox" v-model="selectAll" @change="toggleSelectAll" />
              全选
            </label>
            <span class="result-count">已选 {{ selectedCount }} 项</span>
          </div>
          <div class="result-list">
            <div v-for="(item, idx) in scanResult.items" :key="idx" class="result-item">
              <label class="item-checkbox">
                <input type="checkbox" v-model="(item as any).selected" />
              </label>
              <div class="item-info">
                <span class="item-name">{{ item.name }}</span>
                <span class="item-path">{{ item.path }}</span>
              </div>
              <span class="item-size">{{ formatSize(item.size) }}</span>
            </div>
          </div>
          <div class="result-actions">
            <button class="btn-secondary" @click="cleanSelected('recycle')" :disabled="cleaning || selectedCount === 0">
              清理到回收站
            </button>
            <button class="btn-island" @click="cleanSelected('permanent')" :disabled="cleaning || selectedCount === 0">
              永久删除
            </button>
          </div>
        </div>

        <div v-if="cleanResult" class="clean-result" :class="cleanResult.failed_count > 0 ? 'warning' : 'success'">
          <p>已清理 {{ cleanResult.cleaned_count }} 个文件，释放 {{ formatSize(cleanResult.freed_bytes) }}</p>
          <p v-if="cleanResult.failed_count > 0">{{ cleanResult.failed_count }} 个文件清理失败</p>
        </div>
      </div>

      <!-- 大文件扫描 -->
      <div v-if="activeTab === 'large'" class="tab-panel">
        <div class="panel-toolbar">
          <div class="toolbar-inputs">
            <select v-model="largeFileRoot" class="select-input">
              <option value="C:\">C:\</option>
              <option value="D:\">D:\</option>
              <option value="E:\">E:\</option>
            </select>
            <select v-model="largeFileMin" class="select-input">
              <option :value="100 * 1024 * 1024">100 MB</option>
              <option :value="300 * 1024 * 1024">300 MB</option>
              <option :value="500 * 1024 * 1024">500 MB</option>
              <option :value="1024 * 1024 * 1024">1 GB</option>
            </select>
          </div>
          <button class="btn-island" @click="scanLargeFiles" :disabled="scanningLarge">
            <span v-if="scanningLarge">扫描中...</span>
            <span v-else>扫描大文件</span>
          </button>
        </div>

        <!-- 搜索框 -->
        <div v-if="largeFiles.length" class="panel-search">
          <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
            <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
            <path d="M21 21l-4.35-4.35" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
          <input v-model="largeFileSearchQuery" class="search-input" placeholder="搜索文件名或路径..." />
        </div>

        <div v-if="largeFiles.length" class="scan-results">
          <div class="result-header">
            <label class="select-all">
              <input type="checkbox" v-model="selectAllLarge" @change="toggleSelectAllLarge" />
              全选
            </label>
            <span class="result-count">共 {{ largeFiles.length }} 个文件</span>
          </div>
          <div class="result-list">
            <div v-for="(file, idx) in filteredLargeFiles" :key="idx" class="result-item">
              <label class="item-checkbox">
                <input type="checkbox" v-model="file.selected" />
              </label>
              <div class="item-info">
                <span class="item-name">{{ file.name }}</span>
                <span class="item-path">{{ file.path }}</span>
              </div>
              <button class="btn-icon-locate" @click="openFileLocation(file.path)" title="打开文件位置">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
              <span class="item-size accent">{{ formatSize(file.size) }}</span>
            </div>
          </div>
          <div class="result-actions">
            <button class="btn-secondary" @click="deleteLargeFiles('recycle')" :disabled="selectedLargeCount === 0">
              删除到回收站
            </button>
            <button class="btn-island" @click="deleteLargeFiles('permanent')" :disabled="selectedLargeCount === 0">
              永久删除
            </button>
          </div>
        </div>
      </div>

      <!-- 应用卸载 -->
      <div v-if="activeTab === 'uninstall'" class="tab-panel">
        <div class="panel-toolbar">
          <button class="btn-island" @click="scanApps" :disabled="scanningApps">
            <span v-if="scanningApps">扫描中...</span>
            <span v-else>扫描已安装应用</span>
          </button>
        </div>

        <!-- 搜索框 -->
        <div v-if="installedApps.length" class="panel-search">
          <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
            <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
            <path d="M21 21l-4.35-4.35" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
          <input v-model="appSearchQuery" class="search-input" placeholder="搜索应用名或发布者..." />
        </div>

        <div v-if="installedApps.length" class="scan-results">
          <div class="result-header">
            <span class="result-count">共 {{ installedApps.length }} 个应用</span>
          </div>
          <div class="result-list">
            <div v-for="(app, idx) in filteredApps" :key="idx" class="result-item app-item" @click="selectApp(app)">
              <div class="item-info">
                <span class="item-name">
                  {{ app.name }}
                  <span v-if="app.is_risky" class="badge badge-warning">风险</span>
                </span>
                <span class="item-path">{{ app.publisher }} · {{ app.version }}</span>
                <span class="item-path" v-if="app.install_path">{{ app.install_path }}</span>
              </div>
              <span class="item-size" v-if="app.size_bytes">{{ formatSize(app.size_bytes) }}</span>
            </div>
          </div>
        </div>

        <!-- 选中应用的操作 -->
        <div v-if="selectedApp" class="app-detail">
          <h3>{{ selectedApp.name }}</h3>
          <p class="app-meta">{{ selectedApp.publisher }} · {{ selectedApp.version }}</p>
          <p class="app-meta" v-if="selectedApp.install_path">安装位置：{{ selectedApp.install_path }}</p>
          <p class="app-meta" v-if="selectedApp.size_bytes">占用空间：{{ formatSize(selectedApp.size_bytes) }}</p>
          <p v-if="selectedApp.is_risky" class="app-risk">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="17" x2="12.01" y2="17" stroke="currentColor" stroke-width="2"/>
            </svg>
            {{ selectedApp.risk_reason }}
          </p>
          <div class="app-actions">
            <button class="btn-icon-locate" @click="openAppLocation(selectedApp.install_path)" title="打开安装位置" v-if="selectedApp.install_path">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              打开位置
            </button>
            <button class="btn-secondary" @click="standardUninstall" :disabled="uninstalling">
              标准卸载
            </button>
            <button class="btn-island" @click="forceUninstall" :disabled="uninstalling">
              强力卸载
            </button>
          </div>
          <div v-if="uninstallResult" class="clean-result" :class="uninstallResult.success ? 'success' : 'warning'">
            <p>{{ uninstallResult.message }}</p>
            <div v-if="uninstallResult.residue_items.length" class="residue-actions">
              <p>发现 {{ uninstallResult.residue_items.length }} 个残留项</p>
              <button class="btn-secondary" @click="cleanResidue" :disabled="cleaning">
                清理残留
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Toast -->
    <transition name="slide">
      <div v-if="toastMsg" class="toast-bar" :class="toastType">
        <span>{{ toastMsg }}</span>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
// 系统清理：规则扫描、大文件查找、软件卸载、空文件夹清理

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { confirm } from '@tauri-apps/plugin-dialog'
import PageHelp from '@/components/PageHelp.vue'
import type {
  CleanRule, ScanItem, ScanResult, CleanResult,
  LargeFile, InstalledApp, UninstallResult
} from '@/types'

interface LargeFileItem extends LargeFile {
  selected: boolean
}

interface ScanItemWithSelect extends ScanItem {
  selected: boolean
}

const activeTab = ref<'clean' | 'large' | 'uninstall'>('clean')
const tabs = [
  { id: 'clean' as const, label: '常规清理', icon: '<path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>' },
  { id: 'large' as const, label: '大文件', icon: '<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/><path d="M11 8v6"/><path d="M8 11h6"/>' },
  { id: 'uninstall' as const, label: '应用卸载', icon: '<path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/><path d="M15 9l-6 6"/><path d="M9 9l6 6"/>' }
]

// 常规清理
const scanning = ref(false)
const cleaning = ref(false)
const scanResult = ref<ScanResult | null>(null)
const cleanResult = ref<CleanResult | null>(null)
const selectAll = ref(false)

const selectedCount = computed(() => scanResult.value?.items.filter(i => (i as ScanItemWithSelect).selected).length ?? 0)

async function loadRules() {
  try {
    const rules = await invoke<CleanRule[]>('cleaner_get_rules')
    showToast(`已加载 ${rules.length} 条规则`, 'success')
  } catch (e) {
    showToast(String(e), 'error')
  }
}

async function startScan() {
  scanning.value = true
  cleanResult.value = null
  try {
    const result = await invoke<ScanResult>('cleaner_scan', { categories: ['Windows', '第三方', '国产软件', '开发工具', 'AI工具'] })
    scanResult.value = {
      ...result,
      items: result.items.map(i => ({ ...i, selected: true }))
    }
    showToast(`扫描完成，发现 ${result.total_count} 个可清理项`, 'success')
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    scanning.value = false
  }
}

function toggleSelectAll() {
  if (!scanResult.value) return
  scanResult.value.items.forEach(i => (i as ScanItemWithSelect).selected = selectAll.value)
}

async function cleanSelected(mode: string) {
  if (!scanResult.value) return
  cleaning.value = true
  try {
    const paths = scanResult.value.items
      .filter(i => (i as ScanItemWithSelect).selected)
      .map(i => i.path)
    const result = await invoke<CleanResult>('cleaner_clean', { paths, mode })
    cleanResult.value = result
    showToast(`已清理 ${result.cleaned_count} 个文件，释放 ${formatSize(result.freed_bytes)}`, 'success')
    scanResult.value = null
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    cleaning.value = false
  }
}

// 大文件扫描
const largeFileRoot = ref('C:\\')
const largeFileMin = ref(300 * 1024 * 1024)
const scanningLarge = ref(false)
const largeFiles = ref<LargeFileItem[]>([])
const selectAllLarge = ref(false)

const selectedLargeCount = computed(() => largeFiles.value.filter(f => f.selected).length)

async function scanLargeFiles() {
  scanningLarge.value = true
  try {
    const files = await invoke<LargeFile[]>('large_file_scan', {
      root: largeFileRoot.value,
      minBytes: largeFileMin.value,
      limit: 500,
      skipOptional: true,
      excludes: []
    })
    largeFiles.value = files.map(f => ({ ...f, selected: false }))
    largeFileSearchQuery.value = ''
    showToast(`扫描完成，发现 ${files.length} 个大文件`, 'success')
  } catch (e: any) {
    showToast(String(e), 'error')
  } finally {
    scanningLarge.value = false
  }
}

function toggleSelectAllLarge() {
  largeFiles.value.forEach(f => f.selected = selectAllLarge.value)
}

async function deleteLargeFiles(mode: string) {
  const paths = largeFiles.value.filter(f => f.selected).map(f => f.path)
  if (!paths.length) return

  // 永久删除需要二次确认
  if (mode === 'permanent') {
    const selectedSize = largeFiles.value.filter(f => f.selected).reduce((sum, f) => sum + f.size, 0)
    const confirmed = await confirm(
      `即将永久删除 ${paths.length} 个文件（${formatSize(selectedSize)}），此操作不可撤销。\n\n确定要继续吗？`,
      { title: '确认永久删除', kind: 'warning' }
    )
    if (!confirmed) return
  }

  try {
    const result = await invoke<CleanResult>('large_file_delete', { paths, mode })
    showToast(`已删除 ${result.cleaned_count} 个文件，释放 ${formatSize(result.freed_bytes)}`, 'success')
    largeFiles.value = largeFiles.value.filter(f => !f.selected)
  } catch (e) {
    showToast(String(e), 'error')
  }
}

// 应用卸载
const scanningApps = ref(false)
const installedApps = ref<InstalledApp[]>([])
const selectedApp = ref<InstalledApp | null>(null)
const uninstalling = ref(false)
const uninstallResult = ref<UninstallResult | null>(null)
const appSearchQuery = ref('')
const largeFileSearchQuery = ref('')

const filteredApps = computed(() => {
  if (!appSearchQuery.value) return installedApps.value
  const q = appSearchQuery.value.toLowerCase()
  return installedApps.value.filter(a =>
    a.name.toLowerCase().includes(q) || a.publisher.toLowerCase().includes(q)
  )
})

const filteredLargeFiles = computed(() => {
  if (!largeFileSearchQuery.value) return largeFiles.value
  const q = largeFileSearchQuery.value.toLowerCase()
  return largeFiles.value.filter(f => f.name.toLowerCase().includes(q) || f.path.toLowerCase().includes(q))
})

async function scanApps() {
  scanningApps.value = true
  try {
    installedApps.value = await invoke<InstalledApp[]>('uninstall_scan')
    showToast(`发现 ${installedApps.value.length} 个已安装应用`, 'success')
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    scanningApps.value = false
  }
}

function selectApp(app: InstalledApp) {
  selectedApp.value = app
  uninstallResult.value = null
}

async function standardUninstall() {
  if (!selectedApp.value) return
  uninstalling.value = true
  try {
    uninstallResult.value = await invoke<UninstallResult>('uninstall_standard', {
      appName: selectedApp.value.name,
      installPath: selectedApp.value.install_path,
      regKey: selectedApp.value.reg_key,
    })
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    uninstalling.value = false
  }
}

async function forceUninstall() {
  if (!selectedApp.value) return

  // 强力卸载需要二次确认
  const confirmed = await confirm(
    `即将对「${selectedApp.value.name}」执行强力卸载，这将：\n\n` +
    `• 强制终止相关进程\n` +
    `• 停止并删除相关服务\n` +
    `• 执行卸载程序\n` +
    `• 清理残留文件和注册表\n\n` +
    `此操作不可撤销，确定要继续吗？`,
    { title: '确认强力卸载', kind: 'warning' }
  )
  if (!confirmed) return

  uninstalling.value = true
  try {
    uninstallResult.value = await invoke<UninstallResult>('uninstall_force', {
      appName: selectedApp.value.name,
      installPath: selectedApp.value.install_path,
      regKey: selectedApp.value.reg_key,
    })
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    uninstalling.value = false
  }
}

async function cleanResidue() {
  if (!uninstallResult.value) return
  cleaning.value = true
  try {
    const result = await invoke<CleanResult>('uninstall_clean_residue', {
      items: uninstallResult.value.residue_items
    })
    showToast(`已清理 ${result.cleaned_count} 个残留项`, 'success')
    uninstallResult.value = null
    selectedApp.value = null
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    cleaning.value = false
  }
}

// 工具函数
function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
}

async function openFileLocation(filePath: string) {
  try {
    const { open: shellOpen } = await import('@tauri-apps/plugin-shell')
    const dir = filePath.replace(/[/\\][^/\\]+$/, '')
    await shellOpen(dir)
  } catch {
    showToast('无法打开文件位置，请确认文件存在', 'error')
  }
}

async function openAppLocation(installPath: string) {
  try {
    const { open: shellOpen } = await import('@tauri-apps/plugin-shell')
    const dir = installPath.replace(/[/\\][^/\\]+$/, '') || installPath
    await shellOpen(dir)
  } catch {
    showToast('无法打开安装位置', 'error')
  }
}

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
</script>

<style scoped>
.cleaner-view {
  padding: 24px 28px;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.cleaner-header {
  margin-bottom: 16px;
}

.cleaner-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
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

.cleaner-content {
  flex: 1;
  overflow: hidden;
}

.tab-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.panel-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.panel-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.search-icon {
  flex-shrink: 0;
  color: var(--text-hint);
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-size: 0.85rem;
}

.search-input::placeholder {
  color: var(--text-hint);
}

.toolbar-inputs {
  display: flex;
  gap: 8px;
}

.select-input {
  padding: 8px 12px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.85rem;
}

.scan-summary {
  display: flex;
  gap: 24px;
  padding: 16px 20px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.summary-label {
  font-size: 0.75rem;
  color: var(--text-hint);
}

.summary-value {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--text-primary);
}

.summary-value.accent {
  color: var(--accent);
}

.scan-results {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow: hidden;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.select-all {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--text-secondary);
  cursor: pointer;
}

.result-count {
  font-size: 0.8rem;
  color: var(--text-hint);
}

.result-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-right: 8px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  transition: all 0.2s ease;
}

.result-item:hover {
  border-color: var(--accent);
}

.app-item {
  cursor: pointer;
}

.item-checkbox {
  flex-shrink: 0;
}

.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-path {
  font-size: 0.75rem;
  color: var(--text-hint);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-size {
  flex-shrink: 0;
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.item-size.accent {
  color: var(--accent);
}

.result-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--divider);
}

.app-detail {
  padding: 20px;
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
}

.app-detail h3 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.app-meta {
  font-size: 0.8rem;
  color: var(--text-hint);
  margin-bottom: 4px;
}

.app-risk {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: var(--warning);
  margin-bottom: 12px;
}

.app-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.btn-icon-locate {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 0.82rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon-locate:hover {
  background: var(--accent-glow);
  border-color: var(--accent);
  color: var(--accent);
}

.residue-actions {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.clean-result {
  padding: 12px 16px;
  border-radius: var(--radius-md);
  font-size: 0.85rem;
}

.clean-result.success {
  background: rgba(110, 116, 72, 0.15);
  color: var(--success);
}

.clean-result.warning {
  background: rgba(192, 152, 72, 0.12);
  color: var(--warning);
}

.toast-bar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: var(--radius-lg);
  font-size: 0.85rem;
  z-index: 9999;
}

.toast-bar.success {
  background: var(--success);
  color: white;
}

.toast-bar.error {
  background: var(--error);
  color: white;
}
</style>
