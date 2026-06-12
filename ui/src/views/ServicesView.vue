<template>
  <div class="services-view page-enter">
    <!-- 页面标题 -->
    <div class="page-header anim-fade-up stagger-1">
      <span class="page-roman">III.</span>
      <h1 class="page-header-title">校园服务 <PageHelp>
        <p><strong>文件转换与校园服务</strong></p>
        <p>「转换」标签页：拖入或选择文件（Word/Excel/PPT/RTF/TXT/CSV），选择目标格式，点击转换。支持批量处理。</p>
        <ul>
          <li>转换依赖本机安装的 Microsoft Office 或 WPS，未安装时会失败</li>
          <li>输出文件默认保存到源文件所在目录</li>
          <li>转换过程中可取消单个任务或全部取消</li>
        </ul>
        <p>「校园服务」和「校园站点」标签页提供常用校园平台快速入口。</p>
      </PageHelp></h1>
      <p class="page-header-sub">C A M P U S &nbsp; S E R V I C E S</p>
    </div>

    <!-- 3-Tab 胶囊导航 -->
    <div class="tab-bar anim-fade-up stagger-2">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="tab-btn"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        <span class="tab-icon">{{ tab.icon }}</span>
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <!-- Tab 内容区 -->
    <div class="tab-content">
      <!-- ═══ Tab 1: 转换 ═══ -->
      <transition name="tab-fade" mode="out-in">
        <div v-if="activeTab === 'convert'" key="convert" class="convert-grid">
          <div class="convert-main">
            <ConvertPanel />
          </div>
          <aside class="convert-info">
            <!-- 支持格式 -->
            <div class="info-card">
              <h4 class="info-title">支持格式</h4>
              <div class="format-grid">
                <div
                  v-for="fmt in supportedFormatList"
                  :key="fmt.ext"
                  class="format-cell"
                  :style="{ '--fmt-color': fmt.color }"
                >
                  <span class="format-cell-icon">{{ fmt.icon }}</span>
                  <span class="format-cell-label">{{ fmt.ext }}</span>
                </div>
              </div>
            </div>
            <!-- 使用提示 -->
            <div class="info-card">
              <h4 class="info-title">使用提示</h4>
              <ul class="tip-list">
                <li class="tip-item">
                  <span class="tip-dot"></span>
                  <span>需安装 Microsoft Office 或 WPS</span>
                </li>
                <li class="tip-item">
                  <span class="tip-dot"></span>
                  <span>输出文件保存到源文件同目录</span>
                </li>
                <li class="tip-item">
                  <span class="tip-dot"></span>
                  <span>支持批量处理，自动并发转换</span>
                </li>
                <li class="tip-item">
                  <span class="tip-dot"></span>
                  <span>转换中可随时取消单个或全部任务</span>
                </li>
              </ul>
            </div>
            <!-- 格式对照说明 -->
            <div class="info-card">
              <h4 class="info-title">格式对照</h4>
              <div class="convert-map">
                <div
                  v-for="pair in formatPairs"
                  :key="pair.from + pair.to"
                  class="convert-map-row"
                >
                  <span class="map-from">{{ pair.from }}</span>
                  <span class="map-arrow">→</span>
                  <span class="map-to" :style="{ color: pair.color }">{{ pair.to }}</span>
                </div>
              </div>
            </div>
          </aside>
        </div>

        <!-- ═══ Tab 2: 校园服务 ═══ -->
        <div v-else-if="activeTab === 'service'" key="service" class="service-pane">
          <BezelCard class="service-pane-card">
            <div class="pane-header">
              <div class="pane-title-row">
                <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--accent);">
                  <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>
                </svg>
                <span class="pane-title">校园服务</span>
                <span class="pane-count">{{ services.length }} 项</span>
              </div>
              <p class="pane-hint">点击任意服务，通过 QQ 联系作者咨询</p>
            </div>
            <div class="service-grid">
              <div
                v-for="svc in services"
                :key="svc.name"
                class="service-tag"
                @click="showQQToast(svc.name)"
              >
                <span class="service-tag-icon">{{ svc.icon }}</span>
                <span class="service-tag-name">{{ svc.name }}</span>
              </div>
            </div>
          </BezelCard>
        </div>

        <!-- ═══ Tab 3: 学校站点 ═══ -->
        <div v-else key="sites" class="site-pane">
          <BezelCard class="site-pane-card">
            <div class="pane-header">
              <div class="pane-title-row">
                <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--gold);">
                  <circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                </svg>
                <span class="pane-title">学校站点</span>
                <span class="pane-count">{{ sites.length }} 个</span>
              </div>
              <p class="pane-hint">常用校园站点 · 浏览器直接打开</p>
            </div>

            <!-- 常用站点：始终显示前 6 个 -->
            <div class="site-grid">
              <div
                v-for="site in sites.slice(0, 6)"
                :key="site.domain"
                class="site-item"
                @click="openSite(site.domain)"
              >
                <span class="site-name">{{ site.name }}</span>
                <span class="site-domain">{{ site.domain }}</span>
              </div>
            </div>

            <button
              v-if="sites.length > 6"
              class="sites-toggle-btn"
              @click="sitesExpanded = !sitesExpanded"
            >
              <span>{{ sitesExpanded ? '收起' : `展开全部 ${sites.length - 6} 个站点` }}</span>
              <svg class="sites-toggle-arrow" :class="{ expanded: sitesExpanded }" width="14" height="14" viewBox="0 0 24 24" fill="none">
                <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>

            <transition name="collapse">
              <div v-show="sitesExpanded && sites.length > 6" class="site-grid site-grid--more">
                <div
                  v-for="site in sites.slice(6)"
                  :key="site.domain"
                  class="site-item"
                  @click="openSite(site.domain)"
                >
                  <span class="site-name">{{ site.name }}</span>
                  <span class="site-domain">{{ site.domain }}</span>
                </div>
              </div>
            </transition>
          </BezelCard>
        </div>
      </transition>
    </div>

    <!-- QQ Toast -->
    <transition name="toast">
      <div v-if="toastVisible" class="toast" @click="dismissToast">
        <span class="toast-msg">
          {{ toastText }}<br>
          <span class="toast-qq">QQ：2867332502</span>
        </span>
        <button class="toast-btn" :class="{ copied: toastCopied }" @click.stop="copyQQ">
          {{ toastCopied ? '✓ 已复制' : '复制QQ' }}
        </button>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
// 服务页三 Tab：格式转换 / 校园服务 / 校园站点

import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import BezelCard from '@/components/BezelCard.vue'
import ConvertPanel from '@/components/ConvertPanel.vue'
import PageHelp from '@/components/PageHelp.vue'

interface SiteItem {
  name: string
  domain: string
}

// ═══ Tab 定义 ═══
type TabKey = 'convert' | 'service' | 'sites'
const activeTab = ref<TabKey>('convert')

const tabs = [
  { key: 'convert' as TabKey, label: '转换', icon: '🔄' },
  { key: 'service' as TabKey, label: '校园服务', icon: '☁️' },
  { key: 'sites' as TabKey, label: '校园站点', icon: '🌐' },
]

// ═══ 转换 Tab 信息卡数据 ═══
const supportedFormatList = [
  { ext: 'PDF',  icon: '📄', color: '#E74C3C' },
  { ext: 'DOCX', icon: '📝', color: '#2B579A' },
  { ext: 'XLSX', icon: '📊', color: '#217346' },
  { ext: 'PPTX', icon: '📽️', color: '#D04423' },
  { ext: 'RTF',  icon: '📄', color: '#8B5CF6' },
  { ext: 'TXT',  icon: '📃', color: '#6B7280' },
  { ext: 'CSV',  icon: '📊', color: '#10B981' },
]

const formatPairs = [
  { from: 'Word',  to: 'PDF',  color: '#E74C3C' },
  { from: 'Excel', to: 'PDF',  color: '#E74C3C' },
  { from: 'PPT',   to: 'PDF',  color: '#E74C3C' },
  { from: 'RTF',   to: 'DOCX', color: '#2B579A' },
  { from: 'TXT',   to: 'PDF',  color: '#E74C3C' },
]

// ═══ 校园服务 ═══
const services = [
  { name: '笔记本电脑清灰', icon: '💨' },
  { name: '加装硬盘、内存条', icon: '🔧' },
  { name: '换硅脂、换液金', icon: '🧊' },
  { name: '台式主机组装安装', icon: '🖥️' },
  { name: '其他电脑问题（远程解决）', icon: '🌐' },
  { name: '校园上门取送', icon: '🚚' },
]

const serviceMessages: Record<string, string> = {
  '笔记本电脑清灰': '需要笔记本清灰服务？添加 QQ 咨询 →',
  '加装硬盘、内存条': '需要升级硬件？添加 QQ 咨询 →',
  '换硅脂、换液金': '需要更换散热材料？添加 QQ 咨询 →',
  '台式主机组装安装': '需要组装台式机？添加 QQ 咨询 →',
  '其他电脑问题（远程解决）': '电脑有问题？远程帮你搞定 →',
  '校园上门取送': '校园上门服务，添加 QQ 预约 →',
}

// ═══ 学校站点默认列表（34个）═══
const DEFAULT_SITES: SiteItem[] = [
  { name: '教务系统', domain: 'jwxt.zsit.edu.cn' },
  { name: '教务处', domain: 'jwc.zsit.edu.cn' },
  { name: '教务', domain: 'jw.zsit.edu.cn' },
  { name: '学校官网', domain: 'www.zsit.edu.cn' },
  { name: '统一认证(SSO)', domain: 'cas.zsit.edu.cn' },
  { name: '办公自动化', domain: 'oa.zsit.edu.cn' },
  { name: '办公系统', domain: 'office.zsit.edu.cn' },
  { name: '图书馆', domain: 'lib.zsit.edu.cn' },
  { name: 'DNS管理', domain: 'dns.zsit.edu.cn' },
  { name: '计财处', domain: 'jcc.zsit.edu.cn' },
  { name: '缴费系统', domain: 'pay.zsit.edu.cn' },
  { name: '科研处', domain: 'kyc.zsit.edu.cn' },
  { name: '应用平台', domain: 'app.zsit.edu.cn' },
  { name: '旧版网站', domain: 'old.zsit.edu.cn' },
  { name: '融合门户', domain: 'apply.zsit.edu.cn' },
  { name: '招生信息网', domain: 'zsxx.zsit.edu.cn' },
  { name: '就业信息网', domain: 'cjyw.zsit.edu.cn' },
  { name: '学生工作部', domain: 'xgzx.zsit.edu.cn' },
  { name: '招投标与采购中心', domain: 'ztb.zsit.edu.cn' },
  { name: '人事处', domain: 'rsw.zsit.edu.cn' },
  { name: '安全保卫部', domain: 'bwc.zsit.edu.cn' },
  { name: '计划财务处', domain: 'cwb.zsit.edu.cn' },
  { name: '资产与实验室管理处', domain: 'zcgl.zsit.edu.cn' },
  { name: '后勤管理处', domain: 'lm.zsit.edu.cn' },
  { name: '腾讯云互联网学院', domain: 'txy.zsit.edu.cn' },
  { name: '人工智能学院', domain: 'xdgc.zsit.edu.cn' },
  { name: '纺织服装学院', domain: 'ffy.zsit.edu.cn' },
  { name: '建筑环境学院', domain: 'jgfy.zsit.edu.cn' },
  { name: '医药健康学院', domain: 'yyjk.zsit.edu.cn' },
  { name: '语言文化学院', domain: 'yywx.zsit.edu.cn' },
  { name: '经管学院', domain: 'jjgl.zsit.edu.cn' },
  { name: '马克思主义学院', domain: 'mks.zsit.edu.cn' },
  { name: '公共教育学院', domain: 'ggjc.zsit.edu.cn' },
  { name: '蔡元培学院', domain: 'cypxy.zsit.edu.cn' },
]
const sitesExpanded = ref(false)

// 学校站点
const sites = ref<SiteItem[]>([])

// Toast
const toastVisible = ref(false)
const toastText = ref('')
const toastCopied = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showQQToast(serviceName: string) {
  if (toastTimer) clearTimeout(toastTimer)
  toastCopied.value = false
  toastText.value = serviceMessages[serviceName] || `需要${serviceName}？添加 QQ 咨询 →`
  toastVisible.value = true
  toastTimer = setTimeout(() => { toastVisible.value = false }, 4000)
}

function dismissToast() {
  if (toastTimer) clearTimeout(toastTimer)
  toastVisible.value = false
}

async function copyQQ() {
  try {
    await navigator.clipboard.writeText('2867332502')
    toastCopied.value = true
    setTimeout(() => { toastCopied.value = false }, 2000)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = '2867332502'
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    toastCopied.value = true
    setTimeout(() => { toastCopied.value = false }, 2000)
  }
}

async function loadSites() {
  try {
    const status = await invoke<any>('get_hosts_status')
    sites.value = (status.sites && status.sites.length >= 10) ? status.sites : DEFAULT_SITES
  } catch {
    sites.value = DEFAULT_SITES
  }
}

async function openSite(domain: string) {
  try {
    await invoke('open_url', { url: `http://${domain}` })
  } catch {
    window.open(`http://${domain}`, '_blank')
  }
}

onMounted(() => {
  loadSites()
})
</script>

<style scoped>
.services-view {
  height: 100%;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

.page-header {
  margin-bottom: 14px;
  flex-shrink: 0;
}

/* ══════════════════════════════════════
   Tab 胶囊导航
   ══════════════════════════════════════ */
.tab-bar {
  display: flex;
  gap: 6px;
  padding: 4px;
  background: var(--surface-2);
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  margin-bottom: 14px;
  flex-shrink: 0;
}

.tab-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 9px 14px;
  border-radius: 9px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.84rem;
  font-weight: 500;
  transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1);
  position: relative;
}

.tab-btn:hover {
  color: var(--text-primary);
  background: var(--surface-hover);
}

.tab-btn.active {
  background: var(--surface-1);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
}

.tab-icon {
  font-size: 0.95rem;
  line-height: 1;
}

/* ══════════════════════════════════════
   Tab 内容区
   ══════════════════════════════════════ */
.tab-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

/* Tab 内的子页签内容：穿透 <transition> 包裹层（结构是 .tab-content > <transition> > .xxx-pane） */
.tab-content > * > div {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* ══════════════════════════════════════
   Tab 1: 转换 — 双列 3:2 布局
   ══════════════════════════════════════ */
.convert-grid {
  display: grid;
  grid-template-columns: 3fr 2fr;   /* 3:2 双列 */
  gap: 16px;
  height: 100%;
  min-height: 0;
  align-items: stretch;
}

.convert-main {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;          /* 显式声明，避免嵌套 <transition> 时 grid stretch 失效 */
  overflow: hidden;      /* 防止内容溢出遮挡右侧信息卡 */
  background: transparent;
  border-radius: var(--radius-2xl);
}

.convert-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
  overflow-y: auto;
  padding-right: 2px;
}

.info-card {
  background: var(--surface-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  padding: 16px 18px;
  flex-shrink: 0;
}

.info-title {
  font-size: 0.78rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: 12px;
}

/* 支持格式网格 */
.format-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 6px;
}

.format-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px 4px;
  background: var(--surface-2);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  transition: all 0.2s ease;
}

.format-cell:hover {
  border-color: var(--fmt-color);
  background: color-mix(in srgb, var(--fmt-color) 8%, transparent);
}

.format-cell-icon {
  font-size: 1.1rem;
  line-height: 1;
}

.format-cell-label {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

/* 使用提示 */
.tip-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tip-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 0.78rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

.tip-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent);
  margin-top: 7px;
  flex-shrink: 0;
}

/* 格式对照 */
.convert-map {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.convert-map-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  background: var(--surface-2);
  border-radius: var(--radius-md);
  font-size: 0.78rem;
  font-family: var(--font-mono);
}

.map-from {
  color: var(--text-secondary);
  font-weight: 500;
  flex: 1;
}

.map-arrow {
  color: var(--text-muted);
  font-size: 0.7rem;
}

.map-to {
  font-weight: 600;
  flex: 1;
  text-align: right;
}

/* ══════════════════════════════════════
   Tab 2: 校园服务 — 双列网格
   ══════════════════════════════════════ */
.service-pane,
.site-pane {
  height: 100%;
  overflow: hidden;
}

.service-pane-card,
.site-pane-card {
  height: 100%;
  padding: 22px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.pane-header {
  margin-bottom: 16px;
  flex-shrink: 0;
}

.pane-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.pane-title {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--text-primary);
}

.pane-count {
  margin-left: auto;
  font-size: 0.72rem;
  color: var(--text-hint);
  font-weight: 400;
  font-family: var(--font-mono);
}

.pane-hint {
  font-size: 0.74rem;
  color: var(--text-hint);
  margin: 0;
}

.service-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);  /* 2 列 */
  gap: 8px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  align-content: start;
}

.service-tag {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-radius: var(--radius-md);
  background: var(--surface-2);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
  position: relative;
  overflow: hidden;
}

.service-tag::before {
  content: "";
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 0;
  background: var(--accent);
  border-radius: 0 2px 2px 0;
  transition: height 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.service-tag:hover {
  background: var(--surface-hover);
  border-color: var(--border);
}

.service-tag:hover::before {
  height: 60%;
}

.service-tag:active {
  transform: scale(0.98);
  background: var(--surface-active);
}

.service-tag-icon {
  font-size: 1.15rem;
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-1);
  border-radius: var(--radius-sm);
}

.service-tag-name {
  font-size: 0.84rem;
  font-weight: 500;
  color: var(--text-primary);
  flex: 1;
  min-width: 0;
}

/* ══════════════════════════════════════
   Tab 3: 学校站点
   ══════════════════════════════════════ */
.site-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  align-content: start;
}

.site-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  background: var(--surface-2);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
}

.site-item:hover {
  background: var(--surface-hover);
  border-color: var(--gold);
  transform: translateY(-1px);
}

.site-name {
  font-size: 0.82rem;
  font-weight: 500;
  color: var(--text-primary);
}

.site-domain {
  font-size: 0.68rem;
  color: var(--text-hint);
  font-family: var(--font-mono);
}

.sites-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  width: 100%;
  padding: 8px 0;
  margin-top: 8px;
  font-size: 0.76rem;
  color: var(--accent-soft);
  background: none;
  border: 1px dashed var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.sites-toggle-btn:hover {
  background: var(--surface-hover);
  color: var(--accent);
}

.sites-toggle-arrow {
  transition: transform 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.sites-toggle-arrow.expanded {
  transform: rotate(180deg);
}

.site-grid--more {
  margin-top: 8px;
}

/* ══════════════════════════════════════
   Tab 切换动画
   ══════════════════════════════════════ */
.tab-fade-enter-active,
.tab-fade-leave-active {
  transition: all 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.tab-fade-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.tab-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* collapse transition */
.collapse-enter-active {
  transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1);
  overflow: hidden;
}
.collapse-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}
.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
}
.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 600px;
}

/* ══════════════════════════════════════
   Toast
   ══════════════════════════════════════ */
.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  background: var(--surface-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  animation: toastIn 0.35s cubic-bezier(0.32, 0.72, 0, 1) both;
}

.toast-msg {
  flex: 1;
  font-size: 0.84rem;
  line-height: 1.4;
  color: var(--text-primary);
}

.toast-qq {
  font-family: var(--font-mono);
  font-weight: 500;
  color: var(--accent-soft);
}

.toast-btn {
  padding: 6px 14px;
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  border-radius: 999px;
  background: var(--accent);
  color: white;
  transition: all 0.2s ease;
  flex-shrink: 0;
  white-space: nowrap;
}

.toast-btn:hover {
  background: var(--accent-hover);
}

.toast-btn.copied {
  background: var(--success);
}

@keyframes toastIn {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(16px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0) scale(1);
  }
}

.toast-enter-active {
  transition: all 0.35s cubic-bezier(0.32, 0.72, 0, 1);
}
.toast-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 1, 1);
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(16px) scale(0.95);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(8px) scale(0.97);
}
</style>
