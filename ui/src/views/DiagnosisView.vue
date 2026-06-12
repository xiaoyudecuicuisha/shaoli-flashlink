<template>
  <div class="diagnosis-view page-enter">
    <div class="page-header anim-fade-up stagger-1">
      <span class="page-roman">II.</span>
      <h1 class="page-header-title">网络诊断 <PageHelp>
        <p><strong>网络健康全面检测</strong></p>
        <p>进入页面自动运行诊断，点击「重新诊断」可刷新全部检测项。</p>
        <ul>
          <li>校园网络/公网连通：绿色=正常，红色=异常</li>
          <li>延迟：数值越小越好，&lt;100ms 优秀，100-300ms 一般，&gt;300ms 较差</li>
          <li>下载速度：通过 CDN 节点测试，单位 Mbps（8Mbps ≈ 1MB/s）</li>
          <li>DNS：显示当前 DNS 服务器及域名解析是否正常</li>
          <li>安全检测：检查 VPN/系统代理是否开启（可能影响校园网连接）</li>
        </ul>
        <p>「站点修复」可一键注入校园域名到 hosts 文件，解决教务系统等网站 DNS 污染导致的无法访问。需要管理员权限。</p>
      </PageHelp></h1>
      <p class="page-header-sub">N E T W O R K &nbsp; D I A G N O S T I C S</p>
    </div>

    <div class="diagnosis-scroll">
      <!-- 总览摘要 + 操作 -->
      <BezelCard class="overview-card anim-fade-up stagger-2">
        <div class="overview-grid">
          <div class="overview-item" v-for="(item, index) in summaryItems" :key="index">
            <span class="overview-icon" :class="item.status">
              <svg v-if="item.status === 'success'" width="18" height="18" viewBox="0 0 24 24" fill="none">
                <path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              <svg v-else-if="item.status === 'error'" width="18" height="18" viewBox="0 0 24 24" fill="none">
                <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
                <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
              </svg>
            </span>
            <div class="overview-text">
              <span class="overview-label">{{ item.label }}</span>
              <span class="overview-value" :class="item.status">{{ item.value }}</span>
            </div>
          </div>
        </div>
        <div class="overview-action">
          <BtnIsland @click="runDiagnosis" :loading="isChecking">
            {{ isChecking ? '诊断中...' : '重新诊断' }}
          </BtnIsland>
        </div>
      </BezelCard>

      <!-- 站点修复 -->
      <BezelCard class="fix-card anim-fade-up stagger-3">
        <div class="fix-header">
          <div class="fix-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
            <span class="fix-title">站点修复</span>
            <span class="badge" :class="hostsFixed ? 'badge-success' : 'badge-neutral'">
              {{ hostsFixed ? '已注入' : '未注入' }}
            </span>
          </div>
          <span class="fix-count" v-if="hostsFixed">已修复 {{ hostsCount }} 条域名</span>
        </div>
        <div class="fix-actions">
          <BtnIsland @click="fixHosts" :loading="hostsLoading">
            {{ hostsLoading ? '处理中...' : '一键修复' }}
          </BtnIsland>
          <button class="btn-secondary" @click="restoreHosts" :disabled="hostsLoading">恢复原样</button>
        </div>
        <div v-if="hostsFixed" class="hosts-expand">
          <button class="expand-toggle" @click="showHostsList = !showHostsList">
            <span>{{ showHostsList ? '收起列表' : '查看修复站点' }}</span>
            <svg class="icon-svg-sm" :class="{ rotated: showHostsList }" viewBox="0 0 24 24">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </button>
          <transition name="slide">
            <div v-if="showHostsList" class="hosts-list">
              <span class="host-chip" v-for="site in hostsList" :key="site">{{ site }}</span>
            </div>
          </transition>
        </div>
      </BezelCard>

      <!-- 双列布局：基础信息 + 公网出口 -->
      <div class="two-col anim-fade-up stagger-4">
        <BezelCard class="info-card">
          <div class="card-title">
            <svg class="icon-svg" viewBox="0 0 24 24">
              <rect x="2" y="2" width="20" height="8" rx="2" stroke="currentColor" stroke-width="1.5" fill="none"/>
              <rect x="2" y="14" width="20" height="8" rx="2" stroke="currentColor" stroke-width="1.5" fill="none"/>
              <circle cx="6" cy="6" r="1" fill="currentColor"/>
              <circle cx="6" cy="18" r="1" fill="currentColor"/>
            </svg>
            <span>基础信息</span>
          </div>
          <div class="info-rows">
            <div class="info-row" v-for="(item, i) in basicItems" :key="i">
              <span class="info-label">{{ item.label }}</span>
              <span class="info-value" :class="item.status">{{ item.value }}</span>
            </div>
          </div>
          <div v-if="adapterInfo" class="adapter-section">
            <div class="info-subtitle">网络适配器</div>
            <div class="adapter-list">
              <div class="adapter-row" v-for="adapter in adapterInfo.adapters" :key="adapter.name">
                <div class="adapter-main">
                  <span class="adapter-dot" :class="{ active: adapter.is_up, vpn: adapter.is_vpn }"></span>
                  <span class="adapter-name">{{ adapter.name }}</span>
                  <span class="adapter-tag" v-if="adapter.is_vpn">VPN</span>
                </div>
                <div class="adapter-ips" v-if="adapter.ip_addresses.length">
                  <span class="adapter-ip" v-for="ip in adapter.ip_addresses" :key="ip">{{ ip }}</span>
                </div>
              </div>
            </div>
          </div>
        </BezelCard>

        <BezelCard class="info-card">
          <div class="card-title">
            <svg class="icon-svg" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" fill="none"/>
              <line x1="2" y1="12" x2="22" y2="12" stroke="currentColor" stroke-width="1.5"/>
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
            <span>公网出口</span>
          </div>
          <div class="info-rows">
            <div class="info-row" v-for="(item, i) in publicItems" :key="i">
              <span class="info-label">{{ item.label }}</span>
              <span class="info-value" :class="item.status">{{ item.value }}</span>
            </div>
          </div>
        </BezelCard>
      </div>

      <!-- 延迟测试 -->
      <BezelCard class="latency-card anim-fade-up stagger-5">
        <div class="card-title">
          <svg class="icon-svg" viewBox="0 0 24 24">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" stroke="currentColor" stroke-width="1.5" fill="none"/>
          </svg>
          <span>延迟测试</span>
          <span class="section-badge" :class="latencySummary.status">{{ latencySummary.text }}</span>
        </div>
        <div class="latency-bars">
          <div class="latency-bar-row" v-for="r in latencyResults" :key="r.host">
            <span class="latency-label">{{ r.host_desc }}</span>
            <div class="latency-bar-wrap">
              <div class="latency-bar" :style="latencyBarStyle(r)">
                <span class="latency-ms">{{ r.reachable ? (r.latency_ms ?? '--') + ' ms' : '不可达' }}</span>
              </div>
            </div>
          </div>
        </div>
      </BezelCard>

      <!-- 双列布局：测速 + DNS -->
      <div class="two-col anim-fade-up stagger-6">
        <BezelCard class="info-card">
          <div class="card-title">
            <svg class="icon-svg" viewBox="0 0 24 24">
              <polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
            <span>下载测速</span>
            <span class="section-badge" :class="speedSummary.status">{{ speedSummary.text }}</span>
          </div>
          <div class="speed-display" v-if="speedResult">
            <div class="speed-number">{{ speedResult.download_speed_mbps }}</div>
            <div class="speed-unit">Mbps</div>
            <div class="speed-meta" v-if="speedResult.success">
              下载 {{ (speedResult.download_size_bytes / 1024).toFixed(0) }} KB / {{ (speedResult.duration_ms / 1000).toFixed(1) }}s
            </div>
          </div>
          <div class="speed-display" v-else>
            <div class="speed-number pending">--</div>
            <div class="speed-unit">Mbps</div>
          </div>
        </BezelCard>

        <BezelCard class="info-card">
          <div class="card-title">
            <svg class="icon-svg" viewBox="0 0 24 24">
              <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" stroke="currentColor" stroke-width="1.5" fill="none"/>
              <polyline points="22,6 12,13 2,6" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
            <span>DNS 诊断</span>
          </div>
          <div class="dns-section">
            <div class="info-subtitle">DNS 服务器</div>
            <div class="tag-list">
              <span class="tag" v-for="s in dnsServers" :key="s">{{ s }}</span>
            </div>
          </div>
          <div class="dns-section" v-if="dnsResults.length">
            <div class="info-subtitle">域名解析</div>
            <div class="info-rows compact">
              <div class="info-row" v-for="r in dnsResults" :key="r.domain">
                <span class="info-label">{{ r.domain }}</span>
                <span class="info-value" :class="r.success ? 'success' : 'error'">
                  {{ r.success ? r.resolved_ips.join(', ') : '解析失败' }}
                  <span class="info-extra">({{ r.duration_ms }}ms)</span>
                </span>
              </div>
            </div>
          </div>
        </BezelCard>
      </div>

      <!-- 安全检测 -->
      <BezelCard class="security-card anim-fade-up stagger-7">
        <div class="card-title">
          <svg class="icon-svg" style="color: var(--gold);" viewBox="0 0 24 24">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          </svg>
          <span>安全检测</span>
        </div>
        <div class="security-grid">
          <div class="security-item">
            <span class="security-label">VPN 状态</span>
            <span class="security-value" :class="securityItems.vpn.status">
              {{ securityItems.vpn.value }}
            </span>
            <span class="security-detail" v-if="securityItems.vpn.detail">{{ securityItems.vpn.detail }}</span>
          </div>
          <div class="security-item">
            <span class="security-label">系统代理</span>
            <span class="security-value" :class="securityItems.proxy.status">
              {{ securityItems.proxy.value }}
            </span>
            <span class="security-detail" v-if="securityItems.proxy.detail">{{ securityItems.proxy.detail }}</span>
          </div>
        </div>
      </BezelCard>
    </div>
  </div>
</template>

<script setup lang="ts">
// 网络诊断：连通性、公网 IP、延迟、测速、DNS、VPN/代理、Hosts 修复

import { ref, reactive, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import BezelCard from '@/components/BezelCard.vue'
import BtnIsland from '@/components/BtnIsland.vue'
import PageHelp from '@/components/PageHelp.vue'
import type {
  NetworkStatus,
  AdapterDiagnostic,
  DnsDiagnostic,
  LatencyResult,
  SpeedResult,
  PublicIpInfo,
} from '@/types'

interface DiagItem {
  label: string
  value: string
  status: 'success' | 'error' | 'pending'
}

// ── 总览摘要 ──
const isChecking = ref(false)
const summaryItems = ref<DiagItem[]>([
  { label: '校园网络', value: '检测中...', status: 'pending' },
  { label: '公网连通', value: '检测中...', status: 'pending' },
  { label: '本机 IP', value: '检测中...', status: 'pending' },
])

// ── 基础信息 ──
const basicItems = ref<DiagItem[]>([])
const adapterInfo = ref<AdapterDiagnostic | null>(null)

// ── 公网出口 ──
const publicItems = ref<DiagItem[]>([])

// ── 延迟测试 ──
const latencyResults = ref<LatencyResult[]>([])
const latencySummary = computed(() => {
  const reachable = latencyResults.value.filter(r => r.reachable)
  if (reachable.length === 0) return { text: '无连接', status: 'error' }
  const avg = reachable.reduce((s, r) => s + (r.latency_ms ?? 0), 0) / reachable.length
  return { text: `平均 ${avg.toFixed(0)} ms`, status: avg < 100 ? 'success' : (avg < 300 ? 'pending' : 'error') }
})

function latencyBarStyle(r: LatencyResult) {
  if (!r.reachable || r.latency_ms == null) {
    return { width: '100%', background: 'rgba(192, 72, 72, .15)' }
  }
  const pct = Math.min((r.latency_ms / 1000) * 100, 100)
  const color = r.latency_ms < 100 ? 'var(--gold)' : r.latency_ms < 300 ? 'var(--warning)' : 'var(--error)'
  return { width: `${pct}%`, background: color }
}

// ── 测速 ──
const speedResult = ref<SpeedResult | null>(null)
const speedSummary = computed(() => {
  if (!speedResult.value) return { text: '等待测试', status: 'pending' }
  if (!speedResult.value.success) return { text: '测速不可用（网络受限）', status: 'error' }
  return { text: `${speedResult.value.download_speed_mbps} Mbps`, status: 'success' }
})

// ── DNS 诊断 ──
const dnsServers = ref<string[]>([])
const dnsResults = ref<{ domain: string; resolved_ips: string[]; success: boolean; duration_ms: number }[]>([])

// ── 安全检测 ──
const securityItems = reactive({
  vpn: { value: '检测中...', status: 'pending' as string, detail: '' },
  proxy: { value: '检测中...', status: 'pending' as string, detail: '' },
})

// ── 站点修复 ──
const hostsFixed = ref(false)
const hostsCount = ref(0)
const hostsLoading = ref(false)
const showHostsList = ref(false)
const hostsList = ref<string[]>([
  'jwxt.zsit.edu.cn', 'jwc.zsit.edu.cn', 'jw.zsit.edu.cn',
  'www.zsit.edu.cn', 'zsit.edu.cn', 'cas.zsit.edu.cn',
  'oa.zsit.edu.cn', 'office.zsit.edu.cn', 'lib.zsit.edu.cn',
  'dns.zsit.edu.cn', 'jcc.zsit.edu.cn', 'pay.zsit.edu.cn',
  'kyc.zsit.edu.cn', 'app.zsit.edu.cn', 'old.zsit.edu.cn',
  'apply.zsit.edu.cn', 'zsxx.zsit.edu.cn', 'cjyw.zsit.edu.cn',
  'xgzx.zsit.edu.cn', 'ztb.zsit.edu.cn', 'rsw.zsit.edu.cn',
  'bwc.zsit.edu.cn', 'cwb.zsit.edu.cn', 'zcgl.zsit.edu.cn',
  'lm.zsit.edu.cn', 'txy.zsit.edu.cn', 'xdgc.zsit.edu.cn',
  'ffy.zsit.edu.cn', 'jgfy.zsit.edu.cn', 'yyjk.zsit.edu.cn',
  'yywx.zsit.edu.cn', 'jjgl.zsit.edu.cn', 'mks.zsit.edu.cn',
  'ggjc.zsit.edu.cn', 'cypxy.zsit.edu.cn',
])

// ── 运行诊断 ──
async function runDiagnosis() {
  isChecking.value = true
  summaryItems.value = [
    { label: '校园网络', value: '检测中...', status: 'pending' },
    { label: '公网连通', value: '检测中...', status: 'pending' },
    { label: '本机 IP', value: '检测中...', status: 'pending' },
  ]

  function invokeWithTimeout<T>(cmd: string, timeoutMs: number = 15000): Promise<T> {
    return Promise.race([
      invoke<T>(cmd),
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error(`诊断超时: ${cmd}`)), timeoutMs)
      ),
    ])
  }

  const results = await Promise.allSettled([
    invokeWithTimeout<NetworkStatus>('get_network_status'),
    invokeWithTimeout<AdapterDiagnostic>('get_adapter_info'),
    invokeWithTimeout<PublicIpInfo>('get_public_ip'),
    invokeWithTimeout<LatencyResult[]>('get_latency'),
    invokeWithTimeout<SpeedResult>('get_speed', 30000),
    invokeWithTimeout<DnsDiagnostic>('get_dns_diagnostic'),
  ])

  // 总览摘要
  const [netStatus] = results
  if (netStatus.status === 'fulfilled') {
    const ns = netStatus.value as NetworkStatus
    summaryItems.value[0] = {
      label: '校园网络',
      value: ns.campus_network ? '已连接' : '未连接',
      status: ns.campus_network ? 'success' : 'error',
    }
    summaryItems.value[1] = {
      label: '公网连通',
      value: ns.public_internet ? '已连接' : '未连接',
      status: ns.public_internet ? 'success' : 'error',
    }
    summaryItems.value[2] = {
      label: '本机 IP',
      value: ns.ip || '未获取',
      status: ns.ip ? 'success' : 'error',
    }
    basicItems.value = [
      { label: '本机 IP', value: ns.ip || '未获取', status: ns.ip ? 'success' : 'error' },
      { label: '校园网连通', value: ns.campus_network ? '是' : '否', status: ns.campus_network ? 'success' : 'error' },
      { label: '公网连通', value: ns.public_internet ? '是' : '否', status: ns.public_internet ? 'success' : 'error' },
    ]
  } else {
    summaryItems.value.forEach(i => { i.value = '获取失败'; i.status = 'error' })
  }

  // 适配器 + 安全检测
  const [, adapterRes] = results
  if (adapterRes.status === 'fulfilled') {
    adapterInfo.value = adapterRes.value as AdapterDiagnostic
    securityItems.vpn = {
      value: adapterInfo.value.vpn_detected ? '检测到 VPN' : '未检测到 VPN',
      status: adapterInfo.value.vpn_detected ? 'error' : 'success',
      detail: adapterInfo.value.vpn_detected
        ? adapterInfo.value.adapters.filter(a => a.is_vpn).map(a => a.name).join(', ')
        : '',
    }
    securityItems.proxy = {
      value: adapterInfo.value.proxy_enabled ? '已开启' : '未开启',
      status: adapterInfo.value.proxy_enabled ? 'error' : 'success',
      detail: adapterInfo.value.proxy_enabled ? adapterInfo.value.proxy_address : '',
    }
  }

  // 公网出口
  const [, , pubIpRes] = results
  if (pubIpRes.status === 'fulfilled') {
    const info = pubIpRes.value as PublicIpInfo
    publicItems.value = [
      { label: '公网 IP', value: info.public_ip, status: info.success ? 'success' : 'error' },
      { label: 'ISP', value: info.isp, status: 'success' },
      { label: '位置', value: `${info.country} ${info.city}`, status: 'success' },
    ]
  }

  // 延迟测试
  const [, , , latencyRes] = results
  if (latencyRes.status === 'fulfilled') {
    latencyResults.value = latencyRes.value as LatencyResult[]
  }

  // 测速
  const [, , , , speedRes] = results
  if (speedRes.status === 'fulfilled') {
    speedResult.value = speedRes.value as SpeedResult
  }

  // DNS 诊断
  const [, , , , , dnsRes] = results
  if (dnsRes.status === 'fulfilled') {
    const diag = dnsRes.value as DnsDiagnostic
    dnsServers.value = diag.dns_servers
    dnsResults.value = diag.resolution_tests
  }

  isChecking.value = false
}

// ── 站点修复 ──
async function loadHostsStatus() {
  try {
    const status = await invoke<any>('get_hosts_status')
    hostsFixed.value = (status?.fixed_count ?? 0) > 0
    hostsCount.value = status?.fixed_count ?? 0
  } catch {}
}

async function fixHosts() {
  hostsLoading.value = true
  try {
    await invoke('fix_hosts')
    hostsFixed.value = true
    const status = await invoke<any>('get_hosts_status')
    hostsCount.value = status?.fixed_count ?? 0
  } catch (e) {
    console.error('修复失败:', e)
  }
  hostsLoading.value = false
}

async function restoreHosts() {
  hostsLoading.value = true
  try {
    await invoke('restore_hosts')
    hostsFixed.value = false
    hostsCount.value = 0
  } catch (e) {
    console.error('恢复失败:', e)
  }
  hostsLoading.value = false
}

onMounted(() => {
  runDiagnosis()
  loadHostsStatus()
})
</script>

<style scoped>
.diagnosis-view {
  height: 100%;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

.page-header {
  margin-bottom: 20px;
  flex-shrink: 0;
}

.diagnosis-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 4px;
}

/* ── 总览卡片 ── */
.overview-card {
  padding: 18px 22px;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.overview-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.overview-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.overview-icon.success { background: rgba(110, 116, 72, .12); color: var(--success); }
.overview-icon.error { background: rgba(192, 72, 72, .12); color: var(--error); }
.overview-icon.pending { background: rgba(122, 114, 100, .12); color: var(--text-hint); }

.overview-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.overview-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.overview-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}

.overview-value.success { color: var(--success); }
.overview-value.error { color: var(--error); }
.overview-value.pending { color: var(--text-hint); }

.overview-action {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}

/* ── 站点修复 ── */
.fix-card {
  padding: 16px 20px;
}

.fix-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.fix-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.fix-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.fix-count {
  font-size: 12px;
  color: var(--text-tertiary);
}

.badge {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 10px;
}

.badge-success { background: rgba(110, 116, 72, .12); color: var(--success); }
.badge-neutral { background: rgba(122, 114, 100, .12); color: var(--text-hint); }

.fix-actions {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: center;
}

.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  background: rgba(247, 241, 222, .05);
  border: 1px solid rgba(247, 241, 222, .1);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all .15s;
}

.btn-secondary:hover {
  background: rgba(247, 241, 222, .1);
  color: var(--text-primary);
}

.btn-secondary:disabled {
  opacity: .4;
  cursor: not-allowed;
}

.hosts-expand {
  margin-top: 12px;
}

.expand-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 6px;
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 12px;
  cursor: pointer;
  transition: color .15s;
}

.expand-toggle:hover { color: var(--text-primary); }

.icon-svg-sm {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  transition: transform .25s ease;
}

.icon-svg-sm.rotated { transform: rotate(180deg); }

.hosts-list {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.host-chip {
  font-size: 11px;
  padding: 3px 8px;
  background: rgba(247,241,222,.04);
  border: 1px solid rgba(247,241,222,.06);
  border-radius: 6px;
  color: var(--text-secondary);
  font-family: 'SF Mono', 'Fira Code', monospace;
}

/* ── 双列布局 ── */
.two-col {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.info-card {
  padding: 16px 20px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.icon-svg {
  width: 18px;
  height: 18px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.section-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: auto;
}

.section-badge.success { background: rgba(110, 116, 72, .12); color: var(--success); }
.section-badge.error { background: rgba(192, 72, 72, .12); color: var(--error); }
.section-badge.pending { background: rgba(122, 114, 100, .12); color: var(--text-hint); }

/* ── 信息行 ── */
.info-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-rows.compact {
  gap: 6px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-label {
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.info-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-align: right;
}

.info-value.success { color: var(--success); }
.info-value.error { color: var(--error); }
.info-value.pending { color: var(--text-hint); }

.info-extra {
  font-weight: 400;
  color: var(--text-tertiary);
  font-size: 11px;
}

.info-subtitle {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
  margin-top: 14px;
}

/* ── 适配器 ── */
.adapter-section {
  margin-top: 14px;
  border-top: 1px solid rgba(247,241,222,.06);
  padding-top: 14px;
}

.adapter-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.adapter-row {
  padding: 8px 10px;
  background: rgba(247,241,222,.03);
  border-radius: 8px;
  border: 1px solid rgba(247,241,222,.05);
}

.adapter-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

.adapter-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(247,241,222,.2);
  flex-shrink: 0;
}

.adapter-dot.active { background: var(--success); }
.adapter-dot.vpn { background: var(--warning); }

.adapter-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.adapter-tag {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(192, 152, 72, .15);
  color: var(--warning);
}

.adapter-ips {
  margin-top: 4px;
  padding-left: 14px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.adapter-ip {
  font-size: 10px;
  color: var(--text-tertiary);
  font-family: 'SF Mono', 'Fira Code', monospace;
  background: rgba(247,241,222,.04);
  padding: 1px 5px;
  border-radius: 3px;
}

/* ── 延迟测试 ── */
.latency-card {
  padding: 16px 20px;
}

.latency-bars {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.latency-bar-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.latency-label {
  font-size: 12px;
  color: var(--text-secondary);
  width: 70px;
  flex-shrink: 0;
  text-align: right;
}

.latency-bar-wrap {
  flex: 1;
  height: 22px;
  background: rgba(247,241,222,.04);
  border-radius: 6px;
  overflow: hidden;
}

.latency-bar {
  height: 100%;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding-right: 8px;
  min-width: 60px;
  transition: width .4s ease;
}

.latency-ms {
  font-size: 11px;
  font-weight: 700;
  color: #fff;
  white-space: nowrap;
}

/* ── 测速 ── */
.speed-display {
  text-align: center;
  padding: 8px 0;
}

.speed-number {
  font-size: 36px;
  font-weight: 800;
  color: var(--gold);
  line-height: 1;
}

.speed-number.pending { color: var(--text-tertiary); }

.speed-unit {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.speed-meta {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 6px;
}

/* ── DNS ── */
.dns-section {
  margin-top: 4px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag {
  font-size: 11px;
  padding: 3px 8px;
  background: rgba(247,241,222,.05);
  border: 1px solid rgba(247,241,222,.08);
  border-radius: 6px;
  color: var(--text-primary);
  font-family: 'SF Mono', 'Fira Code', monospace;
}

/* ── 安全检测 ── */
.security-card {
  padding: 16px 20px;
}

.security-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.security-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  background: rgba(247,241,222,.03);
  border-radius: 10px;
  border: 1px solid rgba(247,241,222,.05);
}

.security-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.security-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.security-value.success { color: var(--success); }
.security-value.error { color: var(--error); }
.security-value.pending { color: var(--text-hint); }

.security-detail {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* ── 过渡动画 ── */
.slide-enter-active,
.slide-leave-active {
  transition: all .25s ease;
  overflow: hidden;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  max-height: 0;
}

.slide-enter-to,
.slide-leave-from {
  opacity: 1;
  max-height: 400px;
}
</style>
