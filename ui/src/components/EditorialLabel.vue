<template>
  <span class="editorial-label" :class="{ large }">
    <span class="label-line"></span>
    <slot />
  </span>
</template>

<script setup lang="ts">
// Atelier 风格小节标签：左侧重音线 + 大写文字

defineProps<{
  large?: boolean
}>()
</script>

<style scoped>
.editorial-label {
  font-family: var(--font-body);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--accent);
  display: inline-flex;
  align-items: center;
  gap: 12px;
}

.editorial-label.large {
  font-size: 12px;
  letter-spacing: 0.18em;
}

.label-line {
  width: 18px;
  height: 1px;
  background: var(--accent);
  display: inline-block;
}

.editorial-label.large .label-line {
  width: 24px;
}
</style>
