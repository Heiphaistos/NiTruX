<script setup lang="ts">
import { onMounted, ref, type Component } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import LayoutShell from "@/layouts/LayoutShell.vue";
import DashboardPage from "@/pages/DashboardPage.vue";
import HardwarePage from "@/pages/HardwarePage.vue";
import DriversPage from "@/pages/DriversPage.vue";
import LogsPage from "@/pages/LogsPage.vue";
import ThemeEditorPage from "@/pages/ThemeEditorPage.vue";
import PackagesPage from "@/pages/PackagesPage.vue";
import DisksPage from "@/pages/DisksPage.vue";
import NetworkPage from "@/pages/NetworkPage.vue";
import SecurityPage from "@/pages/SecurityPage.vue";

const themeStore = useThemeStore();
onMounted(() => themeStore.setTheme(themeStore.active));

type PageId =
  | "dashboard"
  | "hardware"
  | "drivers"
  | "logs"
  | "theme-editor"
  | "packages"
  | "disks"
  | "network"
  | "security";
const currentPage = ref<PageId>("dashboard");
const pages: Record<PageId, Component> = {
  dashboard: DashboardPage,
  hardware: HardwarePage,
  drivers: DriversPage,
  logs: LogsPage,
  "theme-editor": ThemeEditorPage,
  packages: PackagesPage,
  disks: DisksPage,
  network: NetworkPage,
  security: SecurityPage,
};
</script>

<template>
  <LayoutShell>
    <template #nav>
      <nav class="app-nav">
        <button
          :class="{ active: currentPage === 'dashboard' }"
          @click="currentPage = 'dashboard'"
        >
          Dashboard
        </button>
        <button
          :class="{ active: currentPage === 'hardware' }"
          @click="currentPage = 'hardware'"
        >
          Matériel
        </button>
        <button
          :class="{ active: currentPage === 'drivers' }"
          @click="currentPage = 'drivers'"
        >
          Pilotes
        </button>
        <button
          :class="{ active: currentPage === 'logs' }"
          @click="currentPage = 'logs'"
        >
          Journaux
        </button>
        <button
          :class="{ active: currentPage === 'theme-editor' }"
          @click="currentPage = 'theme-editor'"
        >
          Apparence
        </button>
        <button
          :class="{ active: currentPage === 'packages' }"
          @click="currentPage = 'packages'"
        >
          Paquets
        </button>
        <button
          :class="{ active: currentPage === 'disks' }"
          @click="currentPage = 'disks'"
        >
          Disques
        </button>
        <button
          :class="{ active: currentPage === 'network' }"
          @click="currentPage = 'network'"
        >
          Réseau
        </button>
        <button
          :class="{ active: currentPage === 'security' }"
          @click="currentPage = 'security'"
        >
          Sécurité
        </button>
      </nav>
    </template>
    <component :is="pages[currentPage]" />
  </LayoutShell>
</template>

<style scoped>
.app-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 8px;
}
.app-nav button {
  text-align: left;
  padding: 8px 10px;
  border: none;
  background: transparent;
  color: var(--nx-text-secondary);
  border-radius: 6px;
  cursor: pointer;
  font: inherit;
}
.app-nav button:hover {
  background: var(--nx-bg-elevated);
  color: var(--nx-text-primary);
}
.app-nav button.active {
  background: var(--nx-bg-elevated);
  color: var(--nx-text-primary);
  font-weight: 600;
}
</style>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  background-color: var(--nx-bg-base);
  color: var(--nx-text-primary);
}
</style>
