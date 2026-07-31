<script setup lang="ts">
withDefaults(
  defineProps<{ variant?: "default" | "danger" | "ghost"; disabled?: boolean }>(),
  { variant: "default", disabled: false },
);
defineEmits<{ click: [MouseEvent] }>();
</script>

<template>
  <button
    class="nx-button"
    :class="`nx-button--${variant}`"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<style scoped>
.nx-button {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  font-family: var(--nx-style-font-family);
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
}
.nx-button:disabled { opacity: 0.5; cursor: not-allowed; }
.nx-button--default { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-button--danger { background: var(--nx-accent-danger); color: white; border-color: var(--nx-accent-danger); }
.nx-button--ghost { background: transparent; color: var(--nx-text-secondary); border-color: transparent; }
</style>
