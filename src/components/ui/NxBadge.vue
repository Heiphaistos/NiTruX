<script setup lang="ts">
import { computed } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import { mixHex, pickAccessibleTextColor } from "@/lib/accessibleColor";

// `live`: this badge is a one-off announcement of an action's outcome
// (e.g. "Sauvegarde créée", a `v-if`-gated success/failure result), not
// part of an ongoing state display or a table cell -- opt in explicitly
// rather than defaulting every badge to a live region, since most
// NxBadge usages in this app are static labels (a table's source column,
// a device's current status in a v-for loop) that would just add noise
// for screen reader users if announced on every render.
const props = withDefaults(defineProps<{ status?: "success" | "warning" | "danger" | "info"; live?: boolean }>(), {
  status: "info",
  live: false,
});

// Pinia is always active in production (main.ts sets it up before mounting
// App), but plenty of pre-existing unit tests mount a page containing an
// NxBadge without needing any store themselves, so they never bothered to.
// Falling back to the original CSS-only coloring (still declared below)
// when no store is available keeps every one of those tests working
// unchanged, rather than forcing Pinia setup into ~20 unrelated spec files
// for a dependency they don't otherwise need.
let themeStore: ReturnType<typeof useThemeStore> | null = null;
try {
  themeStore = useThemeStore();
} catch {
  themeStore = null;
}

// The background used to be `color-mix(accent 18%, transparent)`, which
// shows whatever sits behind the badge -- 12 style dispositions, some
// gradient/blurred, made the actually-rendered background impossible to
// compute (and therefore impossible to guarantee AA contrast against).
// Mixing onto the theme's own bgElevated instead makes it a known, opaque
// color per theme, closing that gap; measured across all 13 built-in
// themes, 31 of 52 (theme, status) pairs failed WCAG AA (4.5:1) with the
// raw accent as text -- some badly (e.g. Adwaita's success badge at
// 1.98:1). pickAccessibleTextColor derives a same-hue shade (lightened on
// a dark background, darkened on a light one) that clears AA instead,
// computed per theme rather than hardcoded, so it self-corrects for any
// custom/imported theme too, not just the 13 built-ins this was measured
// against.
const colorKey = computed<"accentSuccess" | "accentWarning" | "accentDanger" | "accentPrimary">(() => {
  if (props.status === "success") return "accentSuccess";
  if (props.status === "warning") return "accentWarning";
  if (props.status === "danger") return "accentDanger";
  return "accentPrimary";
});
const background = computed(() => {
  if (!themeStore) return undefined;
  return mixHex(themeStore.active.colors[colorKey.value], 18, themeStore.active.colors.bgElevated);
});
const textColor = computed(() => {
  if (!themeStore || !background.value) return undefined;
  return pickAccessibleTextColor(themeStore.active.colors[colorKey.value], background.value);
});
</script>

<template>
  <span
    class="nx-badge"
    :class="`nx-badge--${status}`"
    :style="background ? { background, color: textColor } : undefined"
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
/* Fallback for the rare case Pinia isn't active (see script) -- normal
   app usage always has the inline style above, computed per theme. */
.nx-badge--success { background: color-mix(in srgb, var(--nx-accent-success) 18%, transparent); color: var(--nx-accent-success); }
.nx-badge--warning { background: color-mix(in srgb, var(--nx-accent-warning) 18%, transparent); color: var(--nx-accent-warning); }
.nx-badge--danger { background: color-mix(in srgb, var(--nx-accent-danger) 18%, transparent); color: var(--nx-accent-danger); }
.nx-badge--info { background: color-mix(in srgb, var(--nx-accent-primary) 18%, transparent); color: var(--nx-accent-primary); }
</style>
