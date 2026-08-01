<!-- src/components/ui/NxQuickActionTile.vue -->
<script setup lang="ts">
import { type Component } from "vue";

defineProps<{ icon: Component; label: string; gradient: string }>();
defineEmits<{ click: [MouseEvent] }>();
</script>

<template>
  <button class="nx-quick-action" :style="{ background: gradient }" @click="$emit('click', $event)">
    <component :is="icon" :size="22" class="nx-quick-action__icon" />
    <span class="nx-quick-action__label">{{ label }}</span>
  </button>
</template>

<style scoped>
.nx-quick-action {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
  padding: 16px;
  border: none;
  border-radius: var(--nx-style-radius);
  color: white;
  cursor: pointer;
  font: inherit;
  min-width: 140px;
  text-align: left;
}
.nx-quick-action__icon { opacity: 0.95; }
.nx-quick-action__label { font-size: 13px; font-weight: 600; }
</style>
