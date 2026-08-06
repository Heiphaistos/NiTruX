<script setup lang="ts">
// `placeholder` alone is not an accessible name (WCAG 3.3.2): it vanishes
// once the field has a value, and assistive technology does not reliably
// expose it the way a real label/aria-label is. `ariaLabel` is optional
// and backward-compatible -- existing call sites without it keep working
// exactly as before, unlabeled -- callers should pass it wherever the
// field's purpose isn't otherwise conveyed by a visible, associated label.
defineProps<{ modelValue: string; placeholder?: string; ariaLabel?: string }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <input
    class="nx-input"
    :value="modelValue"
    :placeholder="placeholder"
    :aria-label="ariaLabel"
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
