<!-- src/components/nav/AppNav.vue -->
<script setup lang="ts">
import { type Component } from "vue";
import {
  LayoutDashboard, Stethoscope, Download, Package, HardDrive, Files,
  RefreshCw, Cpu, Wrench, Wifi, Shield, FileText, ScrollText, Settings,
  Palette, Zap, Thermometer, Gauge, BarChart3, Circle,
} from "lucide-vue-next";
import { navigationCategories } from "@/navigation/categories";

defineProps<{ modelValue: string }>();
defineEmits<{ "update:modelValue": [string] }>();

// Maps every icon name used in `categories.ts` to its lucide component.
// An id with no entry here falls back to `Circle` (Step 2's second test) --
// this can only happen if a future `categories.ts` entry's icon name is
// misspelled or not yet added to this map, never a crash.
const iconMap: Record<string, Component> = {
  "layout-dashboard": LayoutDashboard,
  stethoscope: Stethoscope,
  download: Download,
  package: Package,
  "hard-drive": HardDrive,
  files: Files,
  "refresh-cw": RefreshCw,
  cpu: Cpu,
  wrench: Wrench,
  wifi: Wifi,
  shield: Shield,
  "file-text": FileText,
  "scroll-text": ScrollText,
  settings: Settings,
  palette: Palette,
  zap: Zap,
  thermometer: Thermometer,
  gauge: Gauge,
  "bar-chart-3": BarChart3,
};

function getIcon(name: string): Component {
  return iconMap[name] ?? Circle;
}
</script>

<template>
  <nav class="nx-app-nav">
    <div v-for="category in navigationCategories" :key="category.id" class="nx-app-nav__category">
      <div class="nx-app-nav__title">{{ category.title }}</div>
      <button
        v-for="page in category.pages"
        :key="page.id"
        class="nx-app-nav__item"
        :class="{ active: modelValue === page.id }"
        @click="$emit('update:modelValue', page.id)"
      >
        <component :is="getIcon(page.icon)" :size="16" class="nx-app-nav__icon" />
        <span>{{ page.label }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.nx-app-nav { display: flex; flex-direction: column; gap: 2px; padding: 12px 8px; font-family: var(--nx-style-font-family); }
.nx-app-nav__category { margin-bottom: 10px; }
.nx-app-nav__title {
  padding: 6px 10px 4px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--nx-text-secondary);
  opacity: 0.7;
  font-weight: 700;
}
.nx-app-nav__item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  border: none;
  background: transparent;
  color: var(--nx-text-secondary);
  border-radius: var(--nx-style-radius);
  cursor: pointer;
  font: inherit;
  font-size: 13px;
}
.nx-app-nav__icon { flex-shrink: 0; opacity: 0.8; }
.nx-app-nav__item:hover { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-app-nav__item.active { background: var(--nx-style-bg); color: var(--nx-text-primary); font-weight: 600; }
.nx-app-nav__item.active .nx-app-nav__icon { opacity: 1; }
</style>
