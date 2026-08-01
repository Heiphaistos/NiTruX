<script setup lang="ts">
import { computed, onMounted, ref, type Component } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import { useLayoutStore } from "@/stores/layoutStore";
import type { LayoutId } from "@/types/layout";
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
import ReportGeneratorPage from "@/pages/ReportGeneratorPage.vue";
import TemperaturesPage from "@/pages/TemperaturesPage.vue";
import BenchmarkPage from "@/pages/BenchmarkPage.vue";
import PerfHistoryPage from "@/pages/PerfHistoryPage.vue";
import OptimizationsPage from "@/pages/OptimizationsPage.vue";
import AntivirusPage from "@/pages/AntivirusPage.vue";
import CleanerPage from "@/pages/CleanerPage.vue";
import DependenciesPage from "@/pages/DependenciesPage.vue";
import BackupPage from "@/pages/BackupPage.vue";
import UninstallerPage from "@/pages/UninstallerPage.vue";
import WiFiAnalyzerPage from "@/pages/WiFiAnalyzerPage.vue";
import DnsSwitcherPage from "@/pages/DnsSwitcherPage.vue";
import BluetoothPage from "@/pages/BluetoothPage.vue";
import ScriptsPage from "@/pages/ScriptsPage.vue";
import DiskVisualizerPage from "@/pages/DiskVisualizerPage.vue";
import DataRecoveryPage from "@/pages/DataRecoveryPage.vue";
import BootManagerPage from "@/pages/BootManagerPage.vue";
import RestorePointsPage from "@/pages/RestorePointsPage.vue";
import InstallProfilesPage from "@/pages/InstallProfilesPage.vue";

const themeStore = useThemeStore();
onMounted(() => themeStore.setTheme(themeStore.active));

const layoutStore = useLayoutStore();

// The categorized vertical list AppNav renders by default only fits
// layouts with a tall, wide nav area (a real sidebar). Layouts with a
// short header, a floating dock, or a compact command bar need a
// flattened single-row nav instead, or the full list overflows their
// container -- see AppNav's `variant` prop.
const NAV_VARIANT_BY_LAYOUT: Record<LayoutId, "list" | "horizontal" | "icons"> = {
  "sidebar-classic": "list",
  "widgets-grid": "horizontal",
  "command-first": "horizontal",
  "compact-sidebar": "list",
  "top-nav": "horizontal",
  "master-detail": "list",
  bento: "horizontal",
  "floating-dock": "icons",
};
const navVariant = computed(() => NAV_VARIANT_BY_LAYOUT[layoutStore.current]);

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
  "report-generator": ReportGeneratorPage,
  logs: LogsPage,
  "settings-preferences": SettingsPreferencesPage,
  "settings-appearance": ThemeEditorPage,
  optimizations: OptimizationsPage,
  temperatures: TemperaturesPage,
  benchmark: BenchmarkPage,
  "perf-history": PerfHistoryPage,
  uninstaller: UninstallerPage,
  cleaner: CleanerPage,
  backup: BackupPage,
  antivirus: AntivirusPage,
  dependencies: DependenciesPage,
  "wifi-analyzer": WiFiAnalyzerPage,
  "dns-switcher": DnsSwitcherPage,
  bluetooth: BluetoothPage,
  scripts: ScriptsPage,
  "disk-visualizer": DiskVisualizerPage,
  "data-recovery": DataRecoveryPage,
  "boot-manager": BootManagerPage,
  "restore-points": RestorePointsPage,
  "install-profiles": InstallProfilesPage,
};
</script>

<template>
  <LayoutShell>
    <template #nav>
      <AppNav v-model="currentPage" :variant="navVariant" />
    </template>
    <component :is="pages[currentPage] ?? pages.dashboard" @navigate="currentPage = $event" />
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
