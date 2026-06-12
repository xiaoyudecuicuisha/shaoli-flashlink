// Tauri 命令类型定义：所有 invoke 和 listen 调用的 TypeScript 接口

// ── 网络状态 ──
export interface NetworkStatus {
  is_online: boolean
  server_reachable: boolean
  ip: string
  public_internet: boolean
  campus_network: boolean
}

// ── 网络诊断类型 ──

export interface AdapterInfo {
  name: string
  mac_address: string
  ip_addresses: string[]
  is_up: boolean
  is_vpn: boolean
}

export interface AdapterDiagnostic {
  adapters: AdapterInfo[]
  active_adapter: string
  vpn_detected: boolean
  proxy_enabled: boolean
  proxy_address: string
}

export interface DnsResolutionResult {
  domain: string
  resolved_ips: string[]
  success: boolean
  duration_ms: number
}

export interface DnsDiagnostic {
  dns_servers: string[]
  resolution_tests: DnsResolutionResult[]
}

export interface LatencyResult {
  host: string
  host_desc: string
  latency_ms: number | null
  reachable: boolean
}

export interface SpeedResult {
  download_speed_mbps: number
  download_size_bytes: number
  duration_ms: number
  success: boolean
}

export interface PublicIpInfo {
  public_ip: string
  isp: string
  country: string
  city: string
  success: boolean
}

// ── 配置 ──

export interface AppConfig {
  theme: string
  autostart: boolean
  autostart_prompted: boolean
  active_account: string
}

// ── Hosts 状态 ──

export interface HostsStatus {
  fixed_count: number
  total: number
  sites: SiteInfo[]
}

export interface SiteInfo {
  domain: string
  name: string
  is_fixed: boolean
}

// ── 账号 ──

export interface AccountEntry {
  username: string
  password: string
  operator: string
}

// ── 白板 ──

export interface WhiteboardInfo {
  name: string
  board_type: string
  created_at: string
  updated_at: string
}

export interface WhiteboardData {
  name: string
  data: any
}

// ── 白板事件 ──

export interface WhiteboardEvent {
  name: string
  data?: any
}

// ── QQ 空间 ──

export interface QzoneUserInfo {
  uin: string
  nickname: string
  avatar_url: string
}

export interface Moment {
  content: string
  time: string
  images: string[]
  comments: Comment[]
}

export interface Comment {
  nickname: string
  content: string
  time: string
}

export interface FetchProgress {
  total: number
  current: number
  moments_count: number
  friends_count: number
  status: string
  is_running: boolean
}

export interface FetchOptions {
  fetch_all: boolean
  start_year: number
  start_month: number
  end_year: number
  end_month: number
  include_moments: boolean
  include_comments: boolean
  include_forwards: boolean
}

export interface LoginStatus {
  type: 'waiting' | 'scanned' | 'success' | 'failed'
  message?: string
  cookies?: Cookies
}

export interface Cookies {
  p_skey: string
  uin: string
  skey: string
  p_uin: string
  pt4_token: string
}

// ── 格式转换 ──

export interface ConvertResult {
  task_id: string
  output_path: string
}

export interface ConvertTaskStatus {
  id: string
  status: 'Pending' | 'Running' | 'Completed' | 'Cancelled' | { Failed: string }
  progress: number
  output_path?: string
  error?: string
}

export interface BatchConvertItem {
  input_path: string
  target_format: string
}

// ── 通用事件负载 ──

export interface EventPayload<T = any> {
  payload: T
}

// ── 清理工具 ──

export interface CleanRule {
  name: string
  path: string
  rule_type: string
  default_enabled: boolean
  category: string
  description: string
}

export interface ScanItem {
  name: string
  path: string
  size: number
  category: string
  is_dir: boolean
}

export interface ScanResult {
  items: ScanItem[]
  total_size: number
  total_count: number
}

export interface EstimateItem {
  name: string
  category: string
  estimated_bytes: number
  file_count: number
}

export interface CleanResult {
  cleaned_count: number
  freed_bytes: number
  failed_count: number
  errors: string[]
}

export interface LargeFile {
  path: string
  size: number
  name: string
}

export interface InstalledApp {
  name: string
  version: string
  publisher: string
  install_path: string
  uninstall_cmd: string
  quiet_cmd: string
  icon_path: string
  reg_key: string
  is_risky: boolean
  risk_reason: string
  size_bytes: number
}

export interface ResidueItem {
  path: string
  item_type: string
  size: number
}

export interface UninstallResult {
  success: boolean
  message: string
  residue_items: ResidueItem[]
}

// ── 网课助手 ──

export interface CourseProgress {
  videos_completed: number
  quizzes_answered: number
  quizzes_missed: number
  current_chapter: string
  status: string
}

// ── 桌面宠物 ──

export interface CustomPet {
  id: string
  name: string
  file_name: string
  file_type: string
}

export interface MoodMessages {
  happy: string[]
  sad: string[]
  busy: string[]
  sleep: string[]
}

export interface BubbleStyle {
  duration: number
  font_size: number
  bg_color: string
  text_color: string
}

export interface MenuItem {
  id: string
  label: string
  action: string
  icon: string
}

export interface PetConfig {
  enabled: boolean
  current_pet: string
  size: number
  default_pet_name: string
  custom_pets: CustomPet[]
  reaction_network: boolean
  reaction_course: boolean
  reaction_qzone: boolean
  reaction_convert: boolean
  click_messages: string[]
  shake_message: string
  mood_messages: MoodMessages
  click_animation: string
  bubble_style: BubbleStyle
  menu_items: MenuItem[]
}

export interface PetInfo {
  id: string
  name: string
  source: 'builtin' | 'custom'
}
