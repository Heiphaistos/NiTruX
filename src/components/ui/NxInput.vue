<script setup lang="ts">
defineProps<{ modelValue: string; placeholder?: string }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <input
    class="nx-input"
    :value="modelValue"
    :placeholder="placeholder"
    @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>

<style scoped>
.nx-input {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  background: var(--nx-style-bg);
  color: var(--nx-text-primary);
  font-family: var(--nx-style-font-family);
  padding: 8px 10px;
  font-size: 13px;
  width: 100%;
  box-sizing: border-box;
}
</style>
