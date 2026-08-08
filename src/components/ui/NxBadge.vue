<script setup lang="ts">
// `live`: this badge is a one-off announcement of an action's outcome
// (e.g. "Sauvegarde créée", a `v-if`-gated success/failure result), not
// part of an ongoing state display or a table cell -- opt in explicitly
// rather than defaulting every badge to a live region, since most
// NxBadge usages in this app are static labels (a table's source column,
// a device's current status in a v-for loop) that would just add noise
// for screen reader users if announced on every render.
withDefaults(defineProps<{ status?: "success" | "warning" | "danger" | "info"; live?: boolean }>(), {
  status: "info",
  live: false,
});
</script>

<template>
  <span
    class="nx-badge"
    :class="`nx-badge--${status}`"
    :role="live ? 'status' : undefined"
    :aria-live="live ? 'polite' : undefined"
  >
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
