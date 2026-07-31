<script setup lang="ts">
withDefaults(defineProps<{ status?: "success" | "warning" | "danger" | "info" }>(), { status: "info" });
</script>

<template>
  <span class="nx-badge" :class="`nx-badge--${status}`">
    <slot />
  </span>
</template>

<style scoped>
.nx-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--nx-style-font-family);
}
.nx-badge--success { background: color-mix(in srgb, var(--nx-accent-success) 18%, transparent); color: var(--nx-accent-success); }
.nx-badge--warning { background: color-mix(in srgb, var(--nx-accent-warning) 18%, transparent); color: var(--nx-accent-warning); }
.nx-badge--danger { background: color-mix(in srgb, var(--nx-accent-danger) 18%, transparent); color: var(--nx-accent-danger); }
.nx-badge--info { background: color-mix(in srgb, var(--nx-accent-primary) 18%, transparent); color: var(--nx-accent-primary); }
</style>
