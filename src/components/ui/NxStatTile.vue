<script setup lang="ts">
defineProps<{ label: string; value: string; status?: "success" | "warning" | "danger" }>();
</script>

<template>
  <div class="nx-stat-tile">
    <div class="nx-stat-tile__label-row">
      <span v-if="status" class="nx-stat-tile__dot" :class="`nx-stat-tile__dot--${status}`" />
      <span class="nx-stat-tile__label">{{ label }}</span>
    </div>
    <div class="nx-stat-tile__value">{{ value }}</div>
  </div>
</template>

<style scoped>
.nx-stat-tile { font-family: var(--nx-style-font-family); }
.nx-stat-tile__label-row { display: flex; align-items: center; gap: 6px; margin-bottom: 4px; }
.nx-stat-tile__label { font-size: 12px; color: var(--nx-text-secondary); }
.nx-stat-tile__value { font-size: 22px; font-weight: 700; color: var(--nx-text-primary); }
.nx-stat-tile__dot { width: 7px; height: 7px; border-radius: 50%; }
.nx-stat-tile__dot--success { background: var(--nx-accent-success); box-shadow: 0 0 6px var(--nx-accent-success); }
.nx-stat-tile__dot--warning { background: var(--nx-accent-warning); box-shadow: 0 0 6px var(--nx-accent-warning); }
.nx-stat-tile__dot--danger { background: var(--nx-accent-danger); box-shadow: 0 0 6px var(--nx-accent-danger); }
</style>
