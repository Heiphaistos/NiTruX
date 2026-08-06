<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ values: number[]; width?: number; height?: number }>();

const width = computed(() => props.width ?? 240);
const height = computed(() => props.height ?? 48);

const points = computed(() => {
  if (props.values.length === 0) return "";
  const min = Math.min(...props.values);
  const max = Math.max(...props.values);
  const range = max - min || 1;
  const stepX = props.values.length > 1 ? width.value / (props.values.length - 1) : 0;
  return props.values
    .map((v, i) => {
      const x = i * stepX;
      const y = height.value - ((v - min) / range) * height.value;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});
</script>

<template>
  <svg
    :viewBox="`0 0 ${width} ${height}`"
    :height="height"
    preserveAspectRatio="none"
    class="nx-sparkline"
    :style="{ maxWidth: `${width}px` }"
  >
    <polyline v-if="points" :points="points" fill="none" stroke="var(--nx-accent-primary)" stroke-width="2" />
  </svg>
</template>

<style scoped>
/* width:100% (capped at the instance's own `width` prop via the inline
   max-width) lets the chart shrink to fit a narrower card/window instead
   of overflowing it, while the fixed `height` attribute keeps its vertical
   size stable -- a fixed `width` SVG attribute, as this used to render,
   never shrinks below its intrinsic pixel size regardless of container
   width, which this app's window can be resized well below (no minWidth
   set in tauri.conf.json). `preserveAspectRatio="none"` lets the polyline,
   plotted in the viewBox's fixed internal coordinate space, stretch to
   whatever width the SVG actually ends up rendering at.
*/
.nx-sparkline { display: block; width: 100%; }
</style>
