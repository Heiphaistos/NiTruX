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
  <svg :width="width" :height="height" class="nx-sparkline">
    <polyline v-if="points" :points="points" fill="none" stroke="var(--nx-accent-primary)" stroke-width="2" />
  </svg>
</template>

<style scoped>
.nx-sparkline { display: block; }
</style>
