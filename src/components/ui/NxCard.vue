<script setup lang="ts">
// `danger` styles the card AND, by default, marks it as an ARIA alert
// (announced to screen readers the moment it appears) -- correct for the
// overwhelming majority of usages, which are `v-if`-gated one-line error
// messages. A `danger` card that wraps interactive controls (inputs,
// selects, buttons) is a static section, not a transient alert, and
// `role="alert"` is explicitly discouraged on regions containing
// focusable content per WAI-ARIA authoring practices -- set `staticDanger`
// on that rare case to keep the red styling without the alert semantics.
withDefaults(defineProps<{ danger?: boolean; staticDanger?: boolean }>(), { danger: false, staticDanger: false });
</script>

<template>
  <div
    class="nx-card"
    :class="{ 'nx-card--danger': danger || staticDanger }"
    :role="danger ? 'alert' : undefined"
    :aria-live="danger ? 'assertive' : undefined"
  >
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
