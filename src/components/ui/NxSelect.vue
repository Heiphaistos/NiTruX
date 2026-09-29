<script setup lang="ts">
// See NxInput.vue's identical note: a bare <select> with no visible,
// associated label has no accessible name at all for assistive
// technology (worse than NxInput -- there isn't even a placeholder
// fallback). `ariaLabel` is optional and backward-compatible.
defineProps<{ modelValue: string; options: { value: string; label: string }[]; ariaLabel?: string }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <select
    class="nx-select"
    :value="modelValue"
    :aria-label="ariaLabel"
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
  padding: 8px 30px 8px 10px;
  font-size: 13px;
  /* WebKitGTK paints a native <select> with the GTK theme and ignores
     `background`: on a dark NiTruX theme the control came out white with
     the theme's light text on it, i.e. unreadable. Dropping the native
     appearance lets the theme colors apply; the arrow is redrawn below. */
  appearance: none;
  -webkit-appearance: none;
  background-image:
    linear-gradient(45deg, transparent 50%, var(--nx-text-secondary) 50%),
    linear-gradient(135deg, var(--nx-text-secondary) 50%, transparent 50%);
  background-position: calc(100% - 15px) 55%, calc(100% - 10px) 55%;
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
  cursor: pointer;
}
.nx-select option {
  background: var(--nx-bg-elevated);
  color: var(--nx-text-primary);
}
</style>
