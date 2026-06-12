<template>
  <div class="bezel" :class="{ hoverable }">
    <div class="bezel-inner">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
// Double-Bezel 嵌套卡片组件：外层壳 + 内层核，支持悬停效果

defineProps<{
  hoverable?: boolean
}>()
</script>

<style scoped>
.bezel.hoverable:hover {
  transform: translateY(-2px);
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.1),
    0 4px 12px rgba(0, 0, 0, 0.06);
}

:root.light .bezel.hoverable:hover {
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.06),
    0 4px 12px rgba(0, 0, 0, 0.03);
}
</style>
