<script setup lang="ts">
withDefaults(defineProps<{ danger?: boolean }>(), { danger: false });
</script>

<template>
  <div class="nx-card" :class="{ 'nx-card--danger': danger }">
    <slot />
  </div>
</template>

<style scoped>
.nx-card {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  box-shadow: var(--nx-style-shadow);
  backdrop-filter: blur(var(--nx-style-blur));
  -webkit-backdrop-filter: blur(var(--nx-style-blur));
  background: var(--nx-style-bg);
  font-family: var(--nx-style-font-family);
  color: var(--nx-text-primary);
  padding: 16px;
}
.nx-card--danger {
  border-color: color-mix(in srgb, var(--nx-accent-danger) 50%, transparent);
}
</style>
