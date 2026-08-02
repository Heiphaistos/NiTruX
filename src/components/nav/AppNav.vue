<!-- src/components/nav/AppNav.vue -->
<script setup lang="ts">
import { type Component } from "vue";
import {
  LayoutDashboard, Stethoscope, Download, Package, HardDrive, Files,
  RefreshCw, Cpu, Wrench, Wifi, Shield, FileText, ScrollText, Settings,
  Palette, Zap, Thermometer, Gauge, BarChart3, Circle,
  Trash2, Sparkles, Save, ShieldCheck, PackageSearch,
  Globe, Radio, Bluetooth, FileCode,
  PieChart, Database, Server,
  Layers,
  Monitor, Activity, List, Users, History,
  Terminal, SquareTerminal,
} from "lucide-vue-next";
import { navigationCategories } from "@/navigation/categories";

withDefaults(defineProps<{ modelValue: string; variant?: "list" | "horizontal" | "icons" }>(), {
  variant: "list",
});
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
  "trash-2": Trash2,
  sparkles: Sparkles,
  save: Save,
  "shield-check": ShieldCheck,
  "package-search": PackageSearch,
  globe: Globe,
  radio: Radio,
  bluetooth: Bluetooth,
  "file-code": FileCode,
  "pie-chart": PieChart,
  database: Database,
  server: Server,
  layers: Layers,
  monitor: Monitor,
  activity: Activity,
  list: List,
  users: Users,
  history: History,
  terminal: Terminal,
  "square-terminal": SquareTerminal,
};

function getIcon(name: string): Component {
  return iconMap[name] ?? Circle;
}
</script>

<template>
  <nav class="nx-app-nav" :class="`nx-app-nav--${variant}`">
    <div v-for="category in navigationCategories" :key="category.id" class="nx-app-nav__category">
      <div v-if="variant === 'list'" class="nx-app-nav__title">{{ category.title }}</div>
      <button
        v-for="page in category.pages"
        :key="page.id"
        class="nx-app-nav__item"
        :class="{ active: modelValue === page.id }"
        :title="variant === 'icons' ? page.label : undefined"
        @click="$emit('update:modelValue', page.id)"
      >
        <component :is="getIcon(page.icon)" :size="16" class="nx-app-nav__icon" />
        <span v-if="variant !== 'icons'">{{ page.label }}</span>
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
  min-width: 0;
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
.nx-app-nav--list .nx-app-nav__item span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.nx-app-nav__item:hover { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-app-nav__item.active { background: var(--nx-style-bg); color: var(--nx-text-primary); font-weight: 600; }
.nx-app-nav__item.active .nx-app-nav__icon { opacity: 1; }

/* horizontal/icons: flattened single-row nav for headers, docks, and
   command bars -- the categorized vertical list (.nx-app-nav__title +
   stacked items) doesn't fit those contexts, so category grouping
   collapses and everything scrolls horizontally instead of overflowing
   the container's fixed height. */
.nx-app-nav--horizontal,
.nx-app-nav--icons {
  flex-direction: row;
  align-items: center;
  gap: 4px;
  padding: 0;
  overflow-x: auto;
  overflow-y: hidden;
  flex-wrap: nowrap;
}
.nx-app-nav--horizontal .nx-app-nav__category,
.nx-app-nav--icons .nx-app-nav__category {
  display: contents;
}
.nx-app-nav--horizontal .nx-app-nav__item,
.nx-app-nav--icons .nx-app-nav__item {
  width: auto;
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
