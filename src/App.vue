<script setup lang="ts">
import { onMounted, ref, type Component } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import LayoutShell from "@/layouts/LayoutShell.vue";
import AppNav from "@/components/nav/AppNav.vue";
import DashboardPage from "@/pages/DashboardPage.vue";
import DiagnosticPage from "@/pages/DiagnosticPage.vue";
import DriversPage from "@/pages/DriversPage.vue";
import LogsPage from "@/pages/LogsPage.vue";
import ThemeEditorPage from "@/pages/ThemeEditorPage.vue";
import PackagesPage from "@/pages/PackagesPage.vue";
import DisksPage from "@/pages/DisksPage.vue";
import FileToolsPage from "@/pages/FileToolsPage.vue";
import NetworkPage from "@/pages/NetworkPage.vue";
import FirewallPage from "@/pages/FirewallPage.vue";
import TroubleshootPage from "@/pages/TroubleshootPage.vue";
import SettingsPreferencesPage from "@/pages/SettingsPreferencesPage.vue";
import QuickInstallPage from "@/pages/QuickInstallPage.vue";
import UpdatesPage from "@/pages/UpdatesPage.vue";
import ReportGeneratorPlaceholder from "@/pages/ReportGeneratorPlaceholder.vue";

const themeStore = useThemeStore();
onMounted(() => themeStore.setTheme(themeStore.active));

const currentPage = ref<string>("dashboard");

// Every id here must match a `NavPage.id` in `src/navigation/categories.ts`
// exactly -- AppNav renders nav items purely from that data file, so a
// mismatch here means a nav item that silently does nothing when clicked
// (falls back to the dashboard per the `?? pages.dashboard` guard below,
// not a crash, but still a real bug if it ever happens for an id that
// should have a real page).
const pages: Record<string, Component> = {
  dashboard: DashboardPage,
  diagnostic: DiagnosticPage,
  "quick-install": QuickInstallPage,
  "package-manager": PackagesPage,
  disks: DisksPage,
  "file-tools": FileToolsPage,
  updates: UpdatesPage,
  drivers: DriversPage,
  troubleshoot: TroubleshootPage,
  "network-overview": NetworkPage,
  firewall: FirewallPage,
  "report-generator": ReportGeneratorPlaceholder,
  logs: LogsPage,
  "settings-preferences": SettingsPreferencesPage,
  "settings-appearance": ThemeEditorPage,
};
</script>

<template>
  <LayoutShell>
    <template #nav>
      <AppNav v-model="currentPage" />
    </template>
    <component :is="pages[currentPage] ?? pages.dashboard" />
  </LayoutShell>
</template>

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
