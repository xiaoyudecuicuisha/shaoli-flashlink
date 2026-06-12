<template>
  <div class="pet-view page-enter">
    <!-- 页面标题 -->
    <div class="page-header anim-fade-left stagger-1">
      <span class="page-roman">VIII.</span>
      <h1 class="page-header-title">宠物 <PageHelp>
        <p><strong>桌面悬浮宠物</strong></p>
        <p>开启后会在桌面显示一只可互动的宠物伙伴。宠物会根据系统状态做出反应：网络断开会难过，恢复会开心，长时间无操作会打瞌睡。</p>
        <ul>
          <li>可上传自定义 GIF 图片（支持 GIF/APNG/PNG，最大 10MB）</li>
          <li>可调整大小（小/中/大）和自定义名称</li>
          <li>右键宠物可「摸一摸」「摇一摇」互动</li>
          <li>宠物窗口始终置顶，点击穿透不影响其他操作</li>
        </ul>
        <p><strong>互动自定义</strong></p>
        <ul>
          <li>自定义点击宠物时显示的随机消息（最多 20 条）</li>
          <li>自定义摇一摇消息和各心情状态下的消息</li>
          <li>选择点击动画：弹跳、摇晃、旋转、闪烁或随机</li>
          <li>调节气泡样式：显示时长、字号、背景色、文字色</li>
          <li>编辑右键菜单项：可增删、排序，最多 8 项</li>
          <li>修改后需点击「保存互动设置」生效</li>
        </ul>
      </PageHelp></h1>
      <p class="page-header-sub">P E T</p>
    </div>

    <div class="pet-scroll">
      <!-- 当前宠物预览 -->
      <BezelCard class="pet-card anim-fade-up stagger-2">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--gold);">
              <circle cx="12" cy="5" r="3" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M12 8c-4 0-7 2-7 5h14c0-3-3-5-7-5z" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M5 13l1 8h12l1-8" fill="none" stroke="currentColor" stroke-width="1.5"/>
            </svg>
            <span class="card-title">当前宠物</span>
          </div>
        </div>

        <div class="current-pet-preview">
          <div class="preview-frame">
            <img
              v-if="currentPetImage"
              :src="currentPetImage"
              class="preview-gif"
              alt="当前宠物"
            />
            <div v-else class="preview-placeholder">
              <svg viewBox="0 0 24 24" fill="none" width="48" height="48">
                <circle cx="12" cy="5" r="3" stroke="var(--text-muted)" stroke-width="1.5"/>
                <path d="M12 8c-4 0-7 2-7 5h14c0-3-3-5-7-5z" stroke="var(--text-muted)" stroke-width="1.5"/>
                <path d="M5 13l1 8h12l1-8" stroke="var(--text-muted)" stroke-width="1.5"/>
              </svg>
            </div>
          </div>
          <div class="preview-info">
            <span class="preview-name">{{ currentPetName }}</span>
            <span class="preview-desc">{{ config?.enabled ? '正在桌面上陪伴你' : '未开启桌面宠物' }}</span>
            <button
              class="btn-action btn-toggle-pet"
              :class="{ active: config?.enabled }"
              @click="togglePetWindow"
            >
              {{ config?.enabled ? '关闭宠物' : '开启宠物' }}
            </button>
          </div>
        </div>
      </BezelCard>

      <!-- 领养区域 -->
      <BezelCard class="pet-card adopt-card anim-fade-up stagger-3">
        <div class="adopt-banner">
          <div class="adopt-icon">🐾</div>
          <h3 class="adopt-title">领养你的专属宠物</h3>
          <p class="adopt-desc">上传一张 GIF 图片，让它成为你的桌面伙伴</p>
          <button class="btn-adopt" @click="uploadPet">
            <svg class="icon-svg-sm" viewBox="0 0 24 24" fill="none">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <polyline points="17 8 12 3 7 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              <line x1="12" y1="3" x2="12" y2="15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <span>上传 GIF / APNG / PNG</span>
          </button>
          <p class="adopt-hint">支持 GIF、APNG、PNG 格式，最大 10MB</p>
        </div>
      </BezelCard>

      <!-- 我的宠物 -->
      <BezelCard class="pet-card anim-fade-up stagger-4">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--accent);">
              <rect x="3" y="3" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <rect x="14" y="3" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <rect x="3" y="14" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <rect x="14" y="14" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
            </svg>
            <span class="card-title">我的宠物</span>
          </div>
        </div>

        <!-- 宠物命名（常驻 UI：直接给当前宠物改名字） -->
        <div class="pet-rename-bar">
          <div class="pet-rename-bar-label">
            <span class="pet-rename-bar-title">宠物名称</span>
            <span class="pet-rename-bar-hint">给当前选中的宠物起个名字</span>
          </div>
          <div class="pet-rename-bar-input">
            <input
              v-model="renameInput"
              type="text"
              class="pet-rename-input"
              :placeholder="renameInputPlaceholder"
              maxlength="20"
              @keyup.enter="saveCurrentPetName"
            />
            <button
              class="pet-rename-save"
              :disabled="!canSaveRename"
              @click="saveCurrentPetName"
            >
              保存
            </button>
          </div>
        </div>

        <div class="pet-grid">
          <!-- 默认宠物 -->
          <div
            class="pet-item"
            :class="{ selected: config?.current_pet === 'default' }"
            @click="selectPet('default')"
          >
            <div class="pet-item-preview">
              <img src="/pet/assets/default.gif" class="pet-item-gif" alt="默认" />
            </div>
            <span class="pet-item-name">{{ config?.default_pet_name || '默认小伙伴' }}</span>
            <span class="pet-item-tag builtin">内置</span>
          </div>

          <!-- 自定义宠物 -->
          <div
            v-for="pet in config?.custom_pets || []"
            :key="pet.id"
            class="pet-item"
            :class="{ selected: config?.current_pet === pet.id }"
            @click="selectPet(pet.id)"
          >
            <div class="pet-item-preview">
              <img
                v-if="customPetImages[pet.id]"
                :src="customPetImages[pet.id]"
                class="pet-item-gif"
                :alt="pet.name"
              />
              <div v-else class="pet-item-loading">加载中...</div>
            </div>
            <span class="pet-item-name" @dblclick.stop="renamePet(pet)">{{ pet.name }}</span>
            <span class="pet-item-tag custom">自定义</span>
            <button class="btn-rename-pet" @click.stop="renamePet(pet)" title="重命名这只宠物">
              <svg viewBox="0 0 14 14" width="12" height="12" fill="none">
                <path d="M9.5 1.5l3 3-7 7H2.5v-3l7-7z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
                <path d="M8 3l3 3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
              </svg>
              <span>改名</span>
            </button>
            <button class="btn-delete-pet" @click.stop="confirmDeletePet(pet)" title="删除">
              <svg viewBox="0 0 14 14" width="12" height="12">
                <line x1="4" y1="4" x2="10" y2="10" stroke="currentColor" stroke-width="1.5"/>
                <line x1="10" y1="4" x2="4" y2="10" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </button>
          </div>
        </div>
      </BezelCard>

      <!-- 宠物大小 -->
      <BezelCard class="pet-card anim-fade-up stagger-5">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--success);">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" fill="none" stroke="currentColor" stroke-width="1.5"/>
            </svg>
            <span class="card-title">宠物大小</span>
          </div>
        </div>

        <div class="size-options">
          <button
            v-for="opt in sizeOptions"
            :key="opt.value"
            class="size-btn"
            :class="{ active: config?.size === opt.value }"
            @click="changeSize(opt.value)"
          >
            <span class="size-label">{{ opt.label }}</span>
            <span class="size-desc">{{ opt.desc }}</span>
          </button>
        </div>
      </BezelCard>

      <!-- 联动设置 -->
      <BezelCard class="pet-card anim-fade-up stagger-6">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--warning);">
              <path d="M22 12h-4l-3 9L9 3l-3 9H2" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            <span class="card-title">联动设置</span>
          </div>
          <span class="card-hint">宠物会对这些事件做出反应</span>
        </div>

        <div class="reaction-list">
          <div class="reaction-item" v-for="r in reactionOptions" :key="r.key">
            <div class="reaction-info">
              <span class="reaction-icon">{{ r.icon }}</span>
              <span class="reaction-label">{{ r.label }}</span>
            </div>
            <button
              class="toggle-switch"
              :class="{ active: config?.[r.key] }"
              @click="toggleReaction(r.key)"
            >
              <span class="toggle-knob"></span>
            </button>
          </div>
        </div>
      </BezelCard>

      <!-- 互动自定义 -->
      <BezelCard class="pet-card anim-fade-up stagger-7">
        <div class="card-header">
          <div class="card-title-row">
            <svg class="icon-svg" viewBox="0 0 24 24" style="color: var(--accent);">
              <path d="M12 20h9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <path d="M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4L16.5 3.5z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
            </svg>
            <span class="card-title">互动自定义</span>
          </div>
          <span class="card-hint">自定义点击消息、动画、气泡样式和右键菜单</span>
        </div>

        <!-- 点击消息 -->
        <div class="custom-section">
          <div class="custom-section-title">点击消息</div>
          <div class="custom-section-hint">点击宠物时随机显示的文字</div>
          <div class="tag-list">
            <span v-for="(msg, i) in localClickMessages" :key="i" class="tag-item">
              {{ msg }}
              <button class="tag-remove" @click="removeClickMessage(i)">&times;</button>
            </span>
            <span v-if="!localClickMessages.length" class="tag-empty">暂无消息</span>
          </div>
          <div class="tag-add-row">
            <input
              v-model="newClickMessage"
              type="text"
              class="custom-input"
              placeholder="输入新消息..."
              maxlength="20"
              @keyup.enter="addClickMessage"
            />
            <button class="btn-add-tag" :disabled="!newClickMessage.trim()" @click="addClickMessage">添加</button>
          </div>
        </div>

        <!-- 摇一摇消息 -->
        <div class="custom-section">
          <div class="custom-section-title">摇一摇消息</div>
          <input
            v-model="localShakeMessage"
            type="text"
            class="custom-input full"
            placeholder="摇一摇时显示的文字"
            maxlength="20"
          />
        </div>

        <!-- 心情消息 -->
        <div class="custom-section">
          <div class="custom-section-title">心情消息</div>
          <div class="custom-section-hint">不同心情状态下随机显示的文字</div>
          <div class="mood-tabs">
            <button
              v-for="m in moodTabOptions"
              :key="m.key"
              class="mood-tab"
              :class="{ active: activeMoodTab === m.key }"
              @click="activeMoodTab = m.key"
            >{{ m.icon }} {{ m.label }}</button>
          </div>
          <div class="tag-list">
            <span v-for="(msg, i) in localMoodMessages[activeMoodTab]" :key="i" class="tag-item">
              {{ msg }}
              <button class="tag-remove" @click="removeMoodMessage(i)">&times;</button>
            </span>
            <span v-if="!localMoodMessages[activeMoodTab]?.length" class="tag-empty">暂无消息</span>
          </div>
          <div class="tag-add-row">
            <input
              v-model="newMoodMessage"
              type="text"
              class="custom-input"
              placeholder="输入新消息..."
              maxlength="20"
              @keyup.enter="addMoodMessage"
            />
            <button class="btn-add-tag" :disabled="!newMoodMessage.trim()" @click="addMoodMessage">添加</button>
          </div>
        </div>

        <!-- 点击动画 -->
        <div class="custom-section">
          <div class="custom-section-title">点击动画</div>
          <div class="anim-options">
            <button
              v-for="opt in animOptions"
              :key="opt.value"
              class="anim-btn"
              :class="{ active: localClickAnimation === opt.value }"
              @click="localClickAnimation = opt.value"
            >
              <span class="anim-icon">{{ opt.icon }}</span>
              <span class="anim-label">{{ opt.label }}</span>
            </button>
          </div>
        </div>

        <!-- 气泡样式 -->
        <div class="custom-section">
          <div class="custom-section-title">气泡样式</div>
          <div class="bubble-style-grid">
            <div class="style-field">
              <label>显示时长</label>
              <div class="style-slider-row">
                <input type="range" v-model.number="localBubbleDuration" min="1000" max="8000" step="500" class="style-slider" />
                <span class="style-value">{{ (localBubbleDuration / 1000).toFixed(1) }}s</span>
              </div>
            </div>
            <div class="style-field">
              <label>字号</label>
              <div class="style-slider-row">
                <input type="range" v-model.number="localBubbleFontSize" min="10" max="20" step="1" class="style-slider" />
                <span class="style-value">{{ localBubbleFontSize }}px</span>
              </div>
            </div>
            <div class="style-field">
              <label>背景色</label>
              <input type="color" v-model="localBubbleBgColor" class="style-color" />
            </div>
            <div class="style-field">
              <label>文字色</label>
              <input type="color" v-model="localBubbleTextColor" class="style-color" />
            </div>
          </div>
        </div>

        <!-- 右键菜单 -->
        <div class="custom-section">
          <div class="custom-section-title">右键菜单</div>
          <div class="custom-section-hint">拖拽排序，最多 8 项</div>
          <div class="menu-editor">
            <div v-for="(item, i) in localMenuItems" :key="item.id" class="menu-edit-item">
              <div class="menu-edit-arrows">
                <button class="menu-arrow-btn" :disabled="i === 0" @click="moveMenuItem(i, -1)" title="上移">↑</button>
                <button class="menu-arrow-btn" :disabled="i === localMenuItems.length - 1" @click="moveMenuItem(i, 1)" title="下移">↓</button>
              </div>
              <input v-model="item.icon" type="text" class="menu-edit-icon" placeholder="图标" maxlength="2" />
              <input v-model="item.label" type="text" class="menu-edit-label" placeholder="菜单文字" maxlength="12" />
              <select v-model="item.action" class="menu-edit-action">
                <option value="interact">互动</option>
                <option value="shake">摇一摇</option>
                <option value="settings">设置</option>
                <option value="close">关闭</option>
              </select>
              <button class="menu-edit-remove" @click="removeMenuItem(i)" :disabled="localMenuItems.length <= 1">
                <svg viewBox="0 0 14 14" width="12" height="12"><line x1="4" y1="4" x2="10" y2="10" stroke="currentColor" stroke-width="1.5"/><line x1="10" y1="4" x2="4" y2="10" stroke="currentColor" stroke-width="1.5"/></svg>
              </button>
            </div>
          </div>
          <button class="btn-add-menu" :disabled="localMenuItems.length >= 8" @click="addMenuItem">
            + 添加菜单项
          </button>
        </div>

        <!-- 保存按钮 -->
        <div class="custom-actions">
          <button class="btn-save-custom" :disabled="!hasCustomChanges" @click="saveCustomization">
            保存互动设置
          </button>
          <button class="btn-reset-custom" @click="resetCustomization">
            恢复默认
          </button>
        </div>
      </BezelCard>
    </div>

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
          <p class="modal-desc">确定要删除「{{ deleteTarget?.name }}」吗？<br>此操作不可撤销。</p>
          <div class="modal-actions">
            <button class="modal-btn danger" @click="deletePet">删除</button>
            <button class="modal-btn secondary" @click="showDeleteModal = false">取消</button>
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
// 桌面宠物设置：宠物选择/上传/改名、大小调节、联动开关、互动自定义

import { ref, reactive, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import BezelCard from '@/components/BezelCard.vue'
import PageHelp from '@/components/PageHelp.vue'
import type { PetConfig, CustomPet, MenuItem, MoodMessages } from '@/types'

const config = ref<PetConfig | null>(null)
const customPetImages = reactive<Record<string, string>>({})
const showDeleteModal = ref(false)
const deleteTarget = ref<CustomPet | null>(null)
const toastMsg = ref('')
const toastType = ref<'success' | 'error' | 'info'>('info')
let toastTimer: ReturnType<typeof setTimeout> | null = null

// ── 命名栏（常驻 UI） ──
const renameInput = ref('')
const renameInputPlaceholder = computed(() => currentPetName.value || '未加载')

const sizeOptions = [
  { value: 120, label: '小巧', desc: '120px' },
  { value: 180, label: '标准', desc: '180px' },
  { value: 240, label: '大号', desc: '240px' }
]

const reactionOptions: { key: keyof PetConfig; icon: string; label: string }[] = [
  { key: 'reaction_network', icon: '🌐', label: '网络状态提醒' },
  { key: 'reaction_course', icon: '📚', label: '网课进度提醒' },
  { key: 'reaction_qzone', icon: '💬', label: 'QQ 空间获取提醒' },
  { key: 'reaction_convert', icon: '📄', label: '文档转换提醒' }
]

// ── 互动自定义本地状态 ──
const localClickMessages = ref<string[]>([])
const newClickMessage = ref('')
const localShakeMessage = ref('')
const localMoodMessages = ref<MoodMessages>({ happy: [], sad: [], busy: [], sleep: [] })
const activeMoodTab = ref<'happy' | 'sad' | 'busy' | 'sleep'>('happy')
const newMoodMessage = ref('')
const localClickAnimation = ref('random')
const localBubbleDuration = ref(3000)
const localBubbleFontSize = ref(12)
const localBubbleBgColor = ref('#1e1e1e')
const localBubbleTextColor = ref('#ffffff')
const localMenuItems = ref<MenuItem[]>([])

const moodTabOptions = [
  { key: 'happy' as const, icon: '😊', label: '开心' },
  { key: 'sad' as const, icon: '😢', label: '难过' },
  { key: 'busy' as const, icon: '⏳', label: '忙碌' },
  { key: 'sleep' as const, icon: '💤', label: '瞌睡' }
]

const animOptions = [
  { value: 'random', icon: '🎲', label: '随机' },
  { value: 'bounce', icon: '⬆', label: '弹跳' },
  { value: 'shake', icon: '↔', label: '摇晃' },
  { value: 'spin', icon: '🔄', label: '旋转' },
  { value: 'flash', icon: '⚡', label: '闪烁' }
]

// 将 config 中的自定义字段同步到本地状态
function syncLocalFromConfig() {
  if (!config.value) return
  localClickMessages.value = [...(config.value.click_messages || [])]
  localShakeMessage.value = config.value.shake_message || '别摇啦~'
  localMoodMessages.value = {
    happy: [...(config.value.mood_messages?.happy || [])],
    sad: [...(config.value.mood_messages?.sad || [])],
    busy: [...(config.value.mood_messages?.busy || [])],
    sleep: [...(config.value.mood_messages?.sleep || [])]
  } as MoodMessages
  localClickAnimation.value = config.value.click_animation || 'random'
  localBubbleDuration.value = config.value.bubble_style?.duration || 3000
  localBubbleFontSize.value = config.value.bubble_style?.font_size || 12
  localBubbleBgColor.value = config.value.bubble_style?.bg_color || '#1e1e1e'
  localBubbleTextColor.value = config.value.bubble_style?.text_color || '#ffffff'
  localMenuItems.value = (config.value.menu_items || []).map(m => ({ ...m }))
}

function addClickMessage() {
  const v = newClickMessage.value.trim()
  if (v && localClickMessages.value.length < 20) {
    localClickMessages.value.push(v)
    newClickMessage.value = ''
  }
}

function removeClickMessage(i: number) {
  localClickMessages.value.splice(i, 1)
}

function addMoodMessage() {
  const v = newMoodMessage.value.trim()
  const pool = localMoodMessages.value[activeMoodTab.value]
  if (v && pool.length < 10) {
    pool.push(v)
    newMoodMessage.value = ''
  }
}

function removeMoodMessage(i: number) {
  localMoodMessages.value[activeMoodTab.value].splice(i, 1)
}

let menuItemCounter = 0

function addMenuItem() {
  if (localMenuItems.value.length >= 8) return
  const id = 'custom_' + (++menuItemCounter) + '_' + Date.now()
  localMenuItems.value.push({ id, label: '新菜单', action: 'interact', icon: '' })
}

function removeMenuItem(i: number) {
  if (localMenuItems.value.length <= 1) return
  localMenuItems.value.splice(i, 1)
}

function moveMenuItem(i: number, dir: number) {
  const j = i + dir
  if (j < 0 || j >= localMenuItems.value.length) return
  const arr = localMenuItems.value
  const tmp = arr[i]
  arr[i] = arr[j]
  arr[j] = tmp
  // 触发响应式更新
  localMenuItems.value = [...arr]
}

const hasCustomChanges = computed(() => {
  if (!config.value) return false
  return (
    JSON.stringify(localClickMessages.value) !== JSON.stringify(config.value.click_messages || []) ||
    localShakeMessage.value !== (config.value.shake_message || '别摇啦~') ||
    JSON.stringify(localMoodMessages.value) !== JSON.stringify(config.value.mood_messages || {}) ||
    localClickAnimation.value !== (config.value.click_animation || 'random') ||
    localBubbleDuration.value !== (config.value.bubble_style?.duration || 3000) ||
    localBubbleFontSize.value !== (config.value.bubble_style?.font_size || 12) ||
    localBubbleBgColor.value !== (config.value.bubble_style?.bg_color || '#1e1e1e') ||
    localBubbleTextColor.value !== (config.value.bubble_style?.text_color || '#ffffff') ||
    JSON.stringify(localMenuItems.value) !== JSON.stringify(config.value.menu_items || [])
  )
})

async function saveCustomization() {
  if (!config.value) return
  const updated = { ...config.value }
  updated.click_messages = [...localClickMessages.value]
  updated.shake_message = localShakeMessage.value
  updated.mood_messages = {
    happy: [...localMoodMessages.value.happy],
    sad: [...localMoodMessages.value.sad],
    busy: [...localMoodMessages.value.busy],
    sleep: [...localMoodMessages.value.sleep]
  }
  updated.click_animation = localClickAnimation.value
  updated.bubble_style = {
    duration: localBubbleDuration.value,
    font_size: localBubbleFontSize.value,
    bg_color: localBubbleBgColor.value,
    text_color: localBubbleTextColor.value
  }
  updated.menu_items = localMenuItems.value.map(m => ({ ...m }))
  try {
    await invoke('pet_save_config', { patch: updated })
    config.value = updated
    const { emit } = await import('@tauri-apps/api/event')
    await emit('pet-config-updated')
    showToast('互动设置已保存', 'success')
  } catch (e) {
    showToast('保存失败: ' + String(e), 'error')
  }
}

async function resetCustomization() {
  if (!config.value) return
  const defaults = {
    click_messages: ['喵~', '汪!', '嘿嘿', '(害羞)', '❤️', '✨', '你好呀~', '(开心)'],
    shake_message: '别摇啦~',
    mood_messages: { happy: ['开心~', '✨', '❤️', '嘿嘿'], sad: ['呜呜...', '(难过)', '网络断开了...'], busy: ['忙碌中...', '稍等~'], sleep: ['困了...', '💤', 'zzZ'] },
    click_animation: 'random',
    bubble_style: { duration: 3000, font_size: 12, bg_color: '#1e1e1e', text_color: '#ffffff' },
    menu_items: [
      { id: 'interact', label: '摸摸我', action: 'interact', icon: '' },
      { id: 'shake', label: '摇一摇', action: 'shake', icon: '' },
      { id: 'settings', label: '打开设置', action: 'settings', icon: '' },
      { id: 'close', label: '关闭宠物', action: 'close', icon: '' }
    ]
  }
  const updated = { ...config.value, ...defaults }
  try {
    await invoke('pet_save_config', { patch: updated })
    config.value = updated
    syncLocalFromConfig()
    const { emit } = await import('@tauri-apps/api/event')
    await emit('pet-config-updated')
    showToast('已恢复默认', 'success')
  } catch (e) {
    showToast('重置失败: ' + String(e), 'error')
  }
}

const currentPetName = computed(() => {
  if (!config.value) return '未加载'
  if (config.value.current_pet === 'default') {
    return config.value.default_pet_name || '默认小伙伴'
  }
  const pet = config.value.custom_pets.find(p => p.id === config.value!.current_pet)
  return pet?.name || '未知宠物'
})

const canSaveRename = computed(() => {
  const v = renameInput.value.trim()
  if (!v) return false
  if (!config.value) return false
  if (v === currentPetName.value) return false
  return v.length > 0 && v.length <= 20
})

// 切换当前宠物时，同步输入框
watch(() => config.value?.current_pet, () => {
  renameInput.value = ''
})

async function saveCurrentPetName() {
  if (!canSaveRename.value || !config.value) return
  const newName = renameInput.value.trim()
  const petId = config.value.current_pet
  try {
    await invoke('pet_set_name', { petId, newName })
    await loadConfig()
    renameInput.value = ''
    showToast(`已将名字改为「${newName}」`, 'success')
  } catch (e) {
    showToast('改名失败: ' + String(e), 'error')
  }
}

const currentPetImage = computed(() => {
  if (!config.value) return null
  if (config.value.current_pet === 'default') return '/pet/assets/default.gif'
  return customPetImages[config.value.current_pet] || null
})

function showToast(msg: string, type: 'success' | 'error' | 'info' = 'error') {
  toastMsg.value = msg
  toastType.value = type
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = '' }, 3000)
}

async function loadConfig() {
  try {
    const cfg: PetConfig = await invoke('pet_get_config')
    config.value = cfg
    syncLocalFromConfig()
    // 加载自定义宠物图片
    for (const pet of cfg.custom_pets) {
      if (!customPetImages[pet.id]) {
        try {
          const dataUrl: string = await invoke('pet_read_file', { pet_id: pet.id })
          customPetImages[pet.id] = dataUrl
        } catch {}
      }
    }
  } catch (e) {
    showToast('加载配置失败: ' + String(e))
  }
}

async function selectPet(petId: string) {
  if (!config.value || config.value.current_pet === petId) return
  config.value.current_pet = petId
  try {
    await invoke('pet_save_config', { patch: config.value })
    // 通知悬浮窗更新
    const { emit } = await import('@tauri-apps/api/event')
    await emit('pet-config-updated')
    showToast('已切换宠物', 'success')
  } catch (e) {
    showToast('切换失败: ' + String(e))
  }
}

async function changeSize(size: number) {
  if (!config.value || config.value.size === size) return
  config.value.size = size
  try {
    await invoke('pet_save_config', { patch: config.value })
    const { emit } = await import('@tauri-apps/api/event')
    await emit('pet-config-updated')
    showToast('已调整大小', 'success')
  } catch (e) {
    showToast('调整失败: ' + String(e))
  }
}

async function toggleReaction(key: keyof PetConfig) {
  if (!config.value) return
  const c = { ...config.value }
  const current = c[key]
  if (typeof current !== 'boolean') return
  const updated = { ...c, [key]: !current }
  try {
    await invoke('pet_save_config', { patch: updated })
    config.value = updated
  } catch {
    // 保持原值
  }
}

async function togglePetWindow() {
  if (!config.value) return
  try {
    if (config.value.enabled) {
      await invoke('pet_close_window')
      config.value.enabled = false
    } else {
      await invoke('pet_open_window')
      config.value.enabled = true
    }
    await invoke('pet_save_config', { patch: config.value })
    // 同步更新 AppConfig
    await invoke('save_config', { patch: { pet_enabled: config.value.enabled } })
    showToast(config.value.enabled ? '宠物已开启' : '宠物已关闭', 'success')
  } catch (e) {
    showToast('操作失败: ' + String(e))
  }
}

async function renamePet(pet: CustomPet) {
  const newName = prompt('给宠物改个名字:', pet.name)
  if (!newName || newName === pet.name) return
  try {
    await invoke('pet_set_name', { petId: pet.id, newName })
    await loadConfig()
    showToast(`已改名为「${newName}」`, 'success')
  } catch (e) {
    showToast('改名失败: ' + String(e))
  }
}

async function uploadPet() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: '图片文件',
        extensions: ['gif', 'apng', 'png']
      }]
    })

    if (!selected) return

    // 弹出命名
    const name = prompt('给你的宠物起个名字吧:') || '我的宠物'

    const pet: CustomPet = await invoke('pet_upload', {
      sourcePath: selected,
      name: name
    })

    // 加载预览
    try {
      const dataUrl: string = await invoke('pet_read_file', { pet_id: pet.id })
      customPetImages[pet.id] = dataUrl
    } catch {}

    await loadConfig()
    showToast(`「${name}」领养成功!`, 'success')
  } catch (e) {
    showToast('上传失败: ' + String(e))
  }
}

function confirmDeletePet(pet: CustomPet) {
  deleteTarget.value = pet
  showDeleteModal.value = true
}

async function deletePet() {
  if (!deleteTarget.value) return
  try {
    await invoke('pet_delete_custom', { petId: deleteTarget.value.id })
    delete customPetImages[deleteTarget.value.id]
    showDeleteModal.value = false
    deleteTarget.value = null
    await loadConfig()
    showToast('已删除', 'success')
  } catch (e) {
    showToast('删除失败: ' + String(e))
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.pet-view {
  padding: 0 28px 40px;
  overflow-y: auto;
  height: 100%;
}

.pet-scroll {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-bottom: 20px;
}

.page-header {
  padding: 28px 0 8px;
}

.page-header-title {
  font-family: "Instrument Serif", "Noto Serif SC", serif;
  font-size: 28px;
  font-weight: 400;
  color: var(--text-primary);
}

.page-header-sub {
  font-size: 11px;
  letter-spacing: 4px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* ── 卡片 ── */
.pet-card { padding: 20px; }

.card-header {
  margin-bottom: 16px;
}

.card-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.card-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
  display: block;
}

/* ── 当前宠物预览 ── */
.current-pet-preview {
  display: flex;
  align-items: center;
  gap: 20px;
}

.preview-frame {
  width: 120px;
  height: 120px;
  border-radius: 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}

.preview-gif {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.preview-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.4;
}

.preview-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.preview-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.preview-desc {
  font-size: 13px;
  color: var(--text-muted);
}

.btn-toggle-pet {
  margin-top: 4px;
  padding: 6px 16px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  align-self: flex-start;
}

.btn-toggle-pet:hover {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.btn-toggle-pet.active {
  background: var(--error);
  border-color: var(--error);
  color: #fff;
}

/* ── 领养区域 ── */
.adopt-card { padding: 0; overflow: hidden; }

.adopt-banner {
  background: linear-gradient(135deg, var(--accent-bg), var(--surface));
  padding: 28px 24px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.adopt-icon {
  font-size: 36px;
  line-height: 1;
}

.adopt-title {
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary);
}

.adopt-desc {
  font-size: 13px;
  color: var(--text-muted);
}

.btn-adopt {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  margin-top: 4px;
}

.btn-adopt:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.adopt-hint {
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.7;
}

/* ── 宠物命名（常驻 UI 栏） ── */
.pet-rename-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  margin-bottom: 14px;
  border-radius: 10px;
  background: var(--surface-2);
  border: 1px solid var(--border-subtle);
}

.pet-rename-bar-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.pet-rename-bar-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.pet-rename-bar-hint {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.7;
}

.pet-rename-bar-input {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.pet-rename-input {
  width: 160px;
  height: 32px;
  padding: 0 12px;
  font-size: 13px;
  color: var(--text-primary);
  background: var(--surface-3);
  border: 1px solid var(--border);
  border-radius: 8px;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.pet-rename-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.pet-rename-input::placeholder {
  color: var(--text-secondary);
  opacity: 0.5;
}

.pet-rename-save {
  height: 32px;
  padding: 0 16px;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  background: var(--accent);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s, transform 0.15s, opacity 0.15s;
  white-space: nowrap;
}

.pet-rename-save:hover:not(:disabled) {
  background: var(--accent-hover);
  transform: translateY(-1px);
}

.pet-rename-save:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ── 宠物网格 ── */
.pet-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 12px;
}

.pet-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 8px;
  border-radius: 12px;
  border: 1.5px solid var(--border);
  background: var(--surface);
  cursor: pointer;
  transition: all 0.2s;
}

.pet-item:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
}

.pet-item.selected {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.pet-item-preview {
  width: 64px;
  height: 64px;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
}

.pet-item-gif {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.pet-item-loading {
  font-size: 11px;
  color: var(--text-muted);
}

.pet-item-name {
  font-size: 12px;
  color: var(--text-primary);
  text-align: center;
  line-height: 1.3;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pet-item-tag {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
}

.pet-item-tag.builtin {
  background: var(--accent-bg);
  color: var(--accent);
}

.pet-item-tag.custom {
  background: rgba(233, 185, 74, 0.15);
  color: var(--gold);
}

.btn-delete-pet {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: none;
  background: var(--error);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.pet-item:hover .btn-delete-pet {
  opacity: 1;
}

.btn-rename-pet {
  position: absolute;
  top: 4px;
  left: 4px;
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px 7px 2px 5px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 10px;
  font-weight: 500;
  cursor: pointer;
  opacity: 1;
  transition: opacity 0.2s, transform 0.15s;
  box-shadow: var(--shadow-sm);
}

.btn-rename-pet:hover {
  transform: scale(1.05);
}

/* ── 大小选择 ── */
.size-options {
  display: flex;
  gap: 10px;
}

.size-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 12px 8px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: var(--surface);
  cursor: pointer;
  transition: all 0.2s;
}

.size-btn:hover {
  border-color: var(--accent);
}

.size-btn.active {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.size-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.size-desc {
  font-size: 11px;
  color: var(--text-muted);
}

/* ── 联动设置 ── */
.reaction-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.reaction-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
}

.reaction-item:last-child {
  border-bottom: none;
}

.reaction-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.reaction-icon {
  font-size: 18px;
  width: 28px;
  text-align: center;
}

.reaction-label {
  font-size: 13px;
  color: var(--text-primary);
}

/* ── 开关 ── */
.toggle-switch {
  position: relative;
  width: 40px;
  height: 22px;
  border-radius: 11px;
  border: none;
  background: var(--border);
  cursor: pointer;
  transition: background 0.2s;
  padding: 0;
}

.toggle-switch.active {
  background: var(--accent);
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--text-primary);
  transition: transform 0.2s;
}

.toggle-switch.active .toggle-knob {
  transform: translateX(18px);
}

/* ── 弹窗 ── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(21, 20, 15, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-card {
  padding: 28px;
  max-width: 360px;
  width: 90%;
  text-align: center;
}

.modal-icon { margin-bottom: 12px; }
.modal-title { font-size: 17px; font-weight: 600; color: var(--text-primary); margin-bottom: 8px; }
.modal-desc { font-size: 13px; color: var(--text-muted); margin-bottom: 20px; line-height: 1.6; }
.modal-actions { display: flex; gap: 10px; justify-content: center; }

.modal-btn {
  padding: 8px 20px;
  border-radius: 8px;
  border: 1px solid var(--border);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.modal-btn.primary { background: var(--accent); color: #fff; border-color: var(--accent); }
.modal-btn.danger { background: var(--error); color: #fff; border-color: var(--error); }
.modal-btn.secondary { background: var(--surface); color: var(--text-primary); }

/* ── Toast ── */
.toast-bar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 18px;
  border-radius: 8px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 2000;
  box-shadow: var(--shadow-lg);
}

.toast-bar.success { background: var(--success); color: #fff; }
.toast-bar.error { background: var(--error); color: #fff; }
.toast-bar.info { background: var(--surface); color: var(--text-primary); border: 1px solid var(--border); }

/* ── 动画 ── */
.modal-fade-enter-active { transition: opacity 0.2s ease; }
.modal-fade-leave-active { transition: opacity 0.15s ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }

.slide-enter-active { transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1); }
.slide-leave-active { transition: all 0.2s ease; }
.slide-enter-from { opacity: 0; transform: translateX(-50%) translateY(16px); }
.slide-leave-to { opacity: 0; transform: translateX(-50%) translateY(16px); }

/* ── 互动自定义 ── */
.custom-section {
  padding: 14px 0;
  border-bottom: 1px solid var(--border);
}

.custom-section:last-of-type {
  border-bottom: none;
}

.custom-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.custom-section-hint {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
  min-height: 28px;
}

.tag-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 12px;
  background: var(--accent-bg);
  color: var(--accent);
  font-size: 12px;
  border: 1px solid var(--border);
}

.tag-remove {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  padding: 0;
  margin-left: 2px;
}

.tag-remove:hover { color: var(--error); }

.tag-empty {
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.6;
  padding: 4px 0;
}

.tag-add-row {
  display: flex;
  gap: 6px;
}

.custom-input {
  height: 30px;
  padding: 0 10px;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--surface-3);
  border: 1px solid var(--border);
  border-radius: 6px;
  outline: none;
  transition: border-color 0.15s;
  flex: 1;
  min-width: 0;
}

.custom-input.full { width: 100%; }

.custom-input:focus { border-color: var(--accent); }

.btn-add-tag {
  height: 30px;
  padding: 0 12px;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  background: var(--accent);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s;
}

.btn-add-tag:hover:not(:disabled) { background: var(--accent-hover); }
.btn-add-tag:disabled { opacity: 0.4; cursor: not-allowed; }

/* 心情标签页 */
.mood-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.mood-tab {
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s;
}

.mood-tab:hover { border-color: var(--accent); }

.mood-tab.active {
  background: var(--accent-bg);
  border-color: var(--accent);
  color: var(--accent);
}

/* 动画选择 */
.anim-options {
  display: flex;
  gap: 8px;
}

.anim-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 10px 4px;
  border-radius: 8px;
  border: 1.5px solid var(--border);
  background: var(--surface);
  cursor: pointer;
  transition: all 0.15s;
}

.anim-btn:hover { border-color: var(--accent); }

.anim-btn.active {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.anim-icon { font-size: 16px; }

.anim-label {
  font-size: 11px;
  color: var(--text-primary);
}

/* 气泡样式 */
.bubble-style-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.style-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.style-field label {
  font-size: 11px;
  color: var(--text-secondary);
}

.style-slider-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.style-slider {
  flex: 1;
  accent-color: var(--accent);
  height: 4px;
}

.style-value {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 36px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.style-color {
  width: 36px;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  padding: 2px;
  background: var(--surface);
}

/* 菜单编辑器 */
.menu-editor {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 8px;
}

.menu-edit-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 6px;
  background: var(--surface-2);
  border: 1px solid var(--border-subtle);
}

.menu-edit-arrows {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.menu-arrow-btn {
  width: 18px;
  height: 14px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  line-height: 1;
  transition: all 0.15s;
}

.menu-arrow-btn:hover:not(:disabled) { background: var(--accent-bg); color: var(--accent); }
.menu-arrow-btn:disabled { opacity: 0.25; cursor: not-allowed; }

.menu-edit-icon {
  width: 32px;
  height: 26px;
  text-align: center;
  font-size: 13px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface-3);
  color: var(--text-primary);
  outline: none;
}

.menu-edit-icon:focus { border-color: var(--accent); }

.menu-edit-label {
  flex: 1;
  height: 26px;
  padding: 0 8px;
  font-size: 12px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface-3);
  color: var(--text-primary);
  outline: none;
  min-width: 0;
}

.menu-edit-label:focus { border-color: var(--accent); }

.menu-edit-action {
  height: 26px;
  padding: 0 6px;
  font-size: 11px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface-3);
  color: var(--text-primary);
  outline: none;
  cursor: pointer;
}

.menu-edit-remove {
  width: 22px;
  height: 22px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.menu-edit-remove:hover { background: var(--error); color: #fff; }
.menu-edit-remove:disabled { opacity: 0.3; cursor: not-allowed; }

.btn-add-menu {
  width: 100%;
  padding: 6px;
  font-size: 12px;
  border: 1px dashed var(--border);
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-add-menu:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.btn-add-menu:disabled { opacity: 0.4; cursor: not-allowed; }

/* 保存/重置 */
.custom-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}

.btn-save-custom {
  flex: 1;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  background: var(--accent);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-save-custom:hover:not(:disabled) { background: var(--accent-hover); transform: translateY(-1px); }
.btn-save-custom:disabled { opacity: 0.4; cursor: not-allowed; }

.btn-reset-custom {
  padding: 8px 16px;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-reset-custom:hover { border-color: var(--error); color: var(--error); }
</style>
