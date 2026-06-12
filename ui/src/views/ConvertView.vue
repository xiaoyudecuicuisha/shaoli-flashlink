<template>
  <div class="convert-view page-enter">
    <div class="page-header anim-fade-up stagger-1">
      <h1 class="page-header-title">格式转换</h1>
      <p class="page-header-sub">C O N V E R T &nbsp; D O C U M E N T S</p>
    </div>
    <div class="convert-body anim-fade-up stagger-2">
      <ConvertPanel />
    </div>
  </div>
</template>

<script setup lang="ts">
// 格式转换页兼容外壳：直接渲染 ConvertPanel 组件

import ConvertPanel from '@/components/ConvertPanel.vue'
</script>

<style scoped>
.convert-view {
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

.convert-body {
  flex: 1;
  min-height: 0;
}
</style>
