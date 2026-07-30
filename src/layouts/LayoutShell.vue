<script setup lang="ts">
import { computed } from "vue";
import { useLayoutStore } from "@/stores/layoutStore";
import SidebarClassicLayout from "./SidebarClassicLayout.vue";
import WidgetsGridLayout from "./WidgetsGridLayout.vue";
import CommandFirstLayout from "./CommandFirstLayout.vue";
import CompactSidebarLayout from "./CompactSidebarLayout.vue";
import TopNavLayout from "./TopNavLayout.vue";
import MasterDetailLayout from "./MasterDetailLayout.vue";
import BentoLayout from "./BentoLayout.vue";
import FloatingDockLayout from "./FloatingDockLayout.vue";
import type { LayoutId } from "@/types/layout";

const layoutStore = useLayoutStore();

const componentMap: Record<LayoutId, unknown> = {
  "sidebar-classic": SidebarClassicLayout,
  "widgets-grid": WidgetsGridLayout,
  "command-first": CommandFirstLayout,
  "compact-sidebar": CompactSidebarLayout,
  "top-nav": TopNavLayout,
  "master-detail": MasterDetailLayout,
  "bento": BentoLayout,
  "floating-dock": FloatingDockLayout,
};

// Fallback guards against an invalid `current` value (e.g. hand-edited
// localStorage, or a future release renaming/removing a layout id):
// componentMap[...] would otherwise resolve to undefined and
// <component :is="undefined"> silently renders nothing.
const activeComponent = computed(() => componentMap[layoutStore.current] ?? SidebarClassicLayout);
</script>

<template>
  <component :is="activeComponent">
    <template #nav><slot name="nav" /></template>
    <slot />
  </component>
</template>
