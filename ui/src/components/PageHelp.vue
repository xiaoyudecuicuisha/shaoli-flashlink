<template>
  <span class="help-wrapper" ref="wrapperRef">
    <button
      class="help-trigger"
      :class="{ active: open }"
      @click.stop="toggle"
      :title="open ? '关闭帮助' : '查看帮助'"
    >?</button>
    <Teleport to="body">
      <Transition name="help-popover">
        <div
          v-if="open"
          class="help-popover"
          ref="panelRef"
          :style="panelStyle"
          @click.stop
        >
          <div class="help-popover-content">
            <slot />
          </div>
        </div>
      </Transition>
    </Teleport>
  </span>
</template>

<script setup lang="ts">
// 页面帮助弹窗：珊瑚色 ? 图标，点击展开帮助面板，Teleport 到 body

import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue'

const props = defineProps<{
  title?: string
}>()

const open = ref(false)
const wrapperRef = ref<HTMLElement | null>(null)
const panelRef = ref<HTMLElement | null>(null)

const panelStyle = ref<Record<string, string>>({})

function updatePosition() {
  if (!wrapperRef.value) return
  const rect = wrapperRef.value.getBoundingClientRect()
  const top = rect.bottom + 8
  const left = Math.max(12, Math.min(rect.left, window.innerWidth - 360))
  panelStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
  }
}

function toggle() {
  open.value = !open.value
  if (open.value) {
    nextTick(updatePosition)
  }
}

function close() {
  open.value = false
}

function handleDocClick(e: MouseEvent) {
  if (!open.value) return
  const target = e.target as Node
  if (panelRef.value?.contains(target)) return
  if (wrapperRef.value?.contains(target)) return
  close()
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && open.value) {
    close()
  }
}

function handleScroll() {
  if (open.value) {
    updatePosition()
  }
}

onMounted(() => {
  document.addEventListener('click', handleDocClick, true)
  document.addEventListener('keydown', handleKeydown)
  window.addEventListener('scroll', handleScroll, true)
  window.addEventListener('resize', handleScroll)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleDocClick, true)
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('scroll', handleScroll, true)
  window.removeEventListener('resize', handleScroll)
})
</script>

<style scoped>
.help-wrapper {
  display: inline-flex;
  align-items: center;
}

.help-trigger {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 1px solid var(--accent);
  background: var(--accent-glow);
  color: var(--accent);
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  line-height: 1;
}

.help-trigger:hover,
.help-trigger.active {
  background: var(--accent);
  color: #fff;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(237, 111, 92, 0.2);
}
</style>

<style>
/* Global styles — not scoped, so they apply to the teleported popover */
.help-popover {
  position: fixed;
  z-index: 9990;
  max-width: 340px;
  min-width: 260px;
  background: var(--surface-1);
  border: 1px solid var(--border);
  border-radius: 14px;
  box-shadow: var(--shadow-md);
  padding: 18px 20px;
  pointer-events: auto;
}

.help-popover-content p {
  font-size: 13px;
  line-height: 1.65;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.help-popover-content p:last-child {
  margin-bottom: 0;
}

.help-popover-content strong {
  color: var(--text-primary);
  font-weight: 600;
}

.help-popover-content ul {
  margin: 6px 0 10px 0;
  padding-left: 16px;
  list-style: disc;
}

.help-popover-content li {
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--text-secondary);
  margin-bottom: 5px;
}

.help-popover-content li:last-child {
  margin-bottom: 0;
}

/* Popover transition */
.help-popover-enter-active {
  transition: opacity 0.2s ease, transform 0.2s cubic-bezier(0.32, 0.72, 0, 1);
}
.help-popover-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.help-popover-enter-from {
  opacity: 0;
  transform: translateY(-6px) scale(0.97);
}
.help-popover-leave-to {
  opacity: 0;
  transform: translateY(-3px) scale(0.99);
}
</style>
