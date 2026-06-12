<template>
  <button
    class="btn-island"
    :class="{ loading, success, error }"
    :disabled="disabled || loading"
    @click="$emit('click')"
  >
    <template v-if="loading">
      <span class="btn-loading">
        <span class="loading-dot"></span>
        <span class="loading-dot"></span>
        <span class="loading-dot"></span>
      </span>
    </template>
    <template v-else-if="success">
      <span class="btn-status">✓ 成功</span>
    </template>
    <template v-else-if="error">
      <span class="btn-status">失败，点击重试</span>
    </template>
    <template v-else>
      <slot />
      <span v-if="showIcon" class="btn-icon">
        <svg viewBox="0 0 24 24">
          <line x1="5" y1="12" x2="19" y2="12"/>
          <polyline points="12 5 19 12 12 19"/>
        </svg>
      </span>
    </template>
  </button>
</template>

<script setup lang="ts">
// 按钮组件：全圆角胶囊样式，支持加载/成功/错误/禁用状态

defineProps<{
  loading?: boolean
  success?: boolean
  error?: boolean
  disabled?: boolean
  showIcon?: boolean
}>()

defineEmits(['click'])
</script>

<style scoped>
.btn-island.loading {
  background: var(--surface-hover);
  color: var(--text-secondary);
}

.btn-island.success {
  background: var(--success);
}

.btn-island.error {
  background: var(--error);
}

.btn-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
}

.loading-dot {
  width: 5px;
  height: 5px;
  background: var(--text-secondary);
  border-radius: 50%;
  animation: loadingPulse 1.2s cubic-bezier(0.32, 0.72, 0, 1) infinite;
}

.loading-dot:nth-child(2) { animation-delay: 0.15s; }
.loading-dot:nth-child(3) { animation-delay: 0.3s; }

.btn-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
</style>
