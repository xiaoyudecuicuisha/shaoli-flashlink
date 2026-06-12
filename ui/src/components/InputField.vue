<template>
  <div class="input-field-wrapper">
    <label v-if="label" class="input-label">{{ label }}</label>
    <div class="input-container">
      <input
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        class="input-field"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        @keyup.enter="$emit('enter')"
      />
      <span v-if="suffix" class="input-suffix">{{ suffix }}</span>
    </div>
    <span v-if="hint" class="input-hint">{{ hint }}</span>
  </div>
</template>

<script setup lang="ts">
// 统一输入框：标签 + 后缀 + 提示信息

defineProps<{
  modelValue: string
  type?: string
  label?: string
  placeholder?: string
  suffix?: string
  hint?: string
  disabled?: boolean
}>()

defineEmits(['update:modelValue', 'enter'])
</script>

<style scoped>
.input-field-wrapper {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.input-label {
  font-size: 0.72rem;
  font-weight: 500;
  color: var(--text-hint);
  text-transform: uppercase;
  letter-spacing: 0.15em;
  font-family: var(--font-body);
}

.input-container {
  position: relative;
  display: flex;
  align-items: center;
}

.input-container .input-field {
  flex: 1;
}

.input-suffix {
  position: absolute;
  right: 14px;
  font-size: 0.78rem;
  color: var(--text-hint);
  pointer-events: none;
}

.input-hint {
  font-size: 0.72rem;
  color: var(--text-muted);
}
</style>
