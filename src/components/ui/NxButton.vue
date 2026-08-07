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
/* Black, not white: computed WCAG contrast ratios (relative luminance
   formula) of white text against every built-in theme's accentDanger show
   only 2 of 13 themes clearing the 4.5:1 AA minimum for this button's 13px
   text (e.g. Catppuccin Mocha 2.32:1, Tokyo Night 2.65:1 -- both far below
   even the 3:1 large-text threshold). Black clears 12 of 13 (only Adwaita
   falls just short, 4.35:1) -- the better fixed default across this app's
   actual theme set, not a hypothetical worst case. */
.nx-button--danger { background: var(--nx-accent-danger); color: black; border-color: var(--nx-accent-danger); }
.nx-button--ghost { background: transparent; color: var(--nx-text-secondary); border-color: transparent; }
</style>
