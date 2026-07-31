<script setup lang="ts">
defineProps<{ modelValue: string; options: { value: string; label: string }[] }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <select
    class="nx-select"
    :value="modelValue"
    @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
  </select>
</template>

<style scoped>
.nx-select {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  background: var(--nx-style-bg);
  color: var(--nx-text-primary);
  font-family: var(--nx-style-font-family);
  padding: 8px 10px;
  font-size: 13px;
}
</style>
